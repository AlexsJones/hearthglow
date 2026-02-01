use crate::data::configuration::Configuration;
use anyhow::Context;
use crate::server::types::*;
use sea_orm::{
    ActiveValue::Set, ActiveModelTrait, ColumnTrait, Database, DatabaseConnection, EntityTrait,
    QueryFilter,
};
pub(crate) trait HGDBConnection {
    async fn connect(&mut self) -> Result<(), anyhow::Error>;
    async fn check(&self) -> Result<(), anyhow::Error>;
    async fn close(&self) -> Result<(), anyhow::Error>;
    async fn is_initialized(&self) -> Result<bool, anyhow::Error>;
    async fn initialize(&self, config: &Configuration) -> Result<(), anyhow::Error>;
    async fn create_person(
        &self,
        person: &CreatePersonRequest,
    ) -> Result<CreatePersonResponse, anyhow::Error>;
    async fn get_people(&self) -> Result<Vec<String>, anyhow::Error>;
    async fn get_person(
        &self,
        first_name: &str,
    ) -> Result<Option<GetPersonResponse>, anyhow::Error>;
    async fn create_star_chart(
        &self,
        star_chart: &CreateStarChartRequest,
    ) -> Result<CreateStarChartResponse, anyhow::Error>;
    async fn get_star_chart(
        &self,
        star_chart_id: i32,
    ) -> Result<Option<GetStarChartResponse>, anyhow::Error>;
    async fn get_star_charts(&self) -> Result<Vec<GetStarChartResponse>, anyhow::Error>;
    async fn update_star_chart(
        &self,
        star_chart_id: i32,
        star_chart: &UpdateStarChartRequest,
    ) -> Result<UpdateStarChartResponse, anyhow::Error>;
    async fn delete_star_chart(&self, star_chart_id: i32) -> Result<(), anyhow::Error>;
    async fn delete_person(&self, person_id: i32) -> Result<(), anyhow::Error>;
    async fn get_all_people(&self) -> Result<Vec<PersonListItem>, anyhow::Error>;
    async fn increment_star_chart(
        &self,
        star_chart_id: i32,
        delta: i32,
    ) -> Result<UpdateStarChartResponse, anyhow::Error>;
}

// Implement the increment_star_chart method for SQLConnector
impl SQLConnector {
    // Helper used internally by tests or server handlers
    pub async fn increment_star_chart_internal(
        &self,
        star_chart_id: i32,
        delta: i32,
    ) -> Result<UpdateStarChartResponse, anyhow::Error> {
        use crate::entity::star_charts;

        let db = self.database_connection.as_ref().unwrap();
        let existing = star_charts::Entity::find_by_id(star_chart_id).one(db).await?;
        anyhow::ensure!(existing.is_some(), "star chart {} not found", star_chart_id);

    let existing_model = existing.unwrap();
    let mut am: star_charts::ActiveModel = existing_model.clone().into();
    let new_count = existing_model.star_count + delta;
    am.star_count = sea_orm::ActiveValue::Set(new_count);

        let res = am.update(db).await?;
        Ok(UpdateStarChartResponse { id: res.id })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    use std::time::{SystemTime, UNIX_EPOCH};

    fn make_tmp_dir() -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let mut p = std::env::temp_dir();
        p.push(format!("hearthglow_test_db_{}", now));
        std::fs::create_dir_all(&p).unwrap();
        p.to_string_lossy().to_string()
    }

    #[tokio::test]
    async fn test_person_crud_dbconnector() {
        let db_path = make_tmp_dir();
        let mut conn = SQLConnector::new(&db_path);
        conn.connect().await.unwrap();

        // create person
        let req = CreatePersonRequest {
            first_name: "Jane".into(),
            last_name: "Doe".into(),
        };

        let resp = conn.create_person(&req).await.unwrap();
        assert!(resp.id > 0);

        // get person by first name
        let p = conn.get_person("Jane").await.unwrap();
        assert!(p.is_some());
        let p = p.unwrap();
        assert_eq!(p.first_name, "Jane");
        assert_eq!(p.last_name, "Doe");

        // list people contains our created name
        let list = conn.get_people().await.unwrap();
        assert!(list.iter().any(|s| s.contains("Jane Doe")));
    }

    #[tokio::test]
    async fn test_star_chart_crud_dbconnector() {
        let db_path = make_tmp_dir();
        let mut conn = SQLConnector::new(&db_path);
        conn.connect().await.unwrap();

        // create a person to link the star chart to
        let person_req = CreatePersonRequest {
            first_name: "StarOwner".into(),
            last_name: "One".into(),
        };
        let person_resp = conn.create_person(&person_req).await.unwrap();

        let create_req = CreateStarChartRequest {
            name: "natal".into(),
            description: "initial".into(),
            person_id: person_resp.id,
            star_count: 3,
            star_total: 10,
        };

        let created = conn.create_star_chart(&create_req).await.unwrap();
        assert!(created.id > 0);

        // fetch list
        let charts = conn.get_star_charts().await.unwrap();
        assert!(charts.iter().any(|c| c.id == created.id));

        // get single
        let single = conn.get_star_chart(created.id).await.unwrap();
        assert!(single.is_some());
        let single = single.unwrap();
        assert_eq!(single.id, created.id);
        assert_eq!(single.name, "natal");

        // update
        let update_req = UpdateStarChartRequest {
            name: "natal_updated".into(),
            description: "updated-description".into(),
            star_count: None,
            star_total: None,
        };

        let updated = conn.update_star_chart(created.id, &update_req).await.unwrap();
        assert_eq!(updated.id, created.id);

        let single2 = conn.get_star_chart(created.id).await.unwrap().unwrap();
        assert_eq!(single2.name, "natal_updated");
        assert_eq!(single2.description, "updated-description");
    }

    #[tokio::test]
    async fn test_parent_child_dbconnector() {
        let db_path = make_tmp_dir();
        let mut conn = SQLConnector::new(&db_path);
        conn.connect().await.unwrap();

        // create parent
        let parent_req = CreatePersonRequest {
            first_name: "Parent".into(),
            last_name: "One".into(),
        };
        let parent_resp = conn.create_person(&parent_req).await.unwrap();

        // create child
        let child_req = CreatePersonRequest {
            first_name: "Child".into(),
            last_name: "Two".into(),
        };
        let child_resp = conn.create_person(&child_req).await.unwrap();

        // open direct DB connection to insert the parent-child link (helpers expects a &DatabaseConnection)
        let db = sea_orm::Database::connect(format!("sqlite://{}/db.sqlite?mode=rwc", db_path))
            .await
            .expect("connect to sqlite");

        // add link
        crate::entity::helpers::add_parent_child(&db, parent_resp.id, child_resp.id)
            .await
            .expect("add parent-child link");

        // ensure the link is present in the link table (sanity check)
        let links = crate::entity::person_parent::Entity::find()
            .all(&db)
            .await
            .expect("query links");
        // debug output
        dbg!(parent_resp.id, child_resp.id, &links);
        assert!(links.iter().any(|l| l.parent_id == parent_resp.id && l.child_id == child_resp.id));

        // sanity-check: directly query children using the same DB connection we used to insert the link
        let kids_direct = crate::entity::helpers::children_of(&db, parent_resp.id)
            .await
            .expect("direct children_of");
        dbg!(&kids_direct);
        assert!(!kids_direct.is_empty(), "direct children query returned empty");

        // fetch parent via connector and ensure child appears in children list
        let parent = conn.get_person("Parent").await.unwrap().unwrap();
        assert!(parent.children.iter().any(|c| c.first_name == "Child" && c.last_name == "Two"), "children: {:?}", parent.children);
    }

    #[tokio::test]
    async fn test_star_chart_tracking_dbconnector() {
        use crate::entity::{people, star_charts};

        let db_path = make_tmp_dir();
        let mut conn = SQLConnector::new(&db_path);
        conn.connect().await.unwrap();

        // create a person Bob
        let person_req = CreatePersonRequest {
            first_name: "Bob".into(),
            last_name: "Builder".into(),
        };
        let person_resp = conn.create_person(&person_req).await.unwrap();

        // create a star chart for Bob with initial 3/10 stars
        let create_req = CreateStarChartRequest {
            name: "cleaning".into(),
            description: "weekly chores".into(),
            person_id: person_resp.id,
            star_count: 3,
            star_total: 10,
        };

        let created = conn.create_star_chart(&create_req).await.unwrap();
        assert!(created.id > 0);

        // open a direct DB connection to update the star count (simulate adding stars)
        let db = sea_orm::Database::connect(format!("sqlite://{}/db.sqlite?mode=rwc", db_path))
            .await
            .expect("connect to sqlite");

        // fetch the star chart and increment star_count to 5
        let existing = star_charts::Entity::find_by_id(created.id)
            .one(&db)
            .await
            .expect("find chart");
        assert!(existing.is_some());
        let mut am: star_charts::ActiveModel = existing.unwrap().into();
        am.star_count = sea_orm::ActiveValue::Set(5);
        let updated = am.update(&db).await.expect("update chart");

        // fetch updated chart and associated person and assert values
        let fetched_chart = star_charts::Entity::find_by_id(updated.id)
            .one(&db)
            .await
            .expect("fetch chart");
        assert!(fetched_chart.is_some());
        let c = fetched_chart.unwrap();
        assert_eq!(c.chart_type, "cleaning");
        assert_eq!(c.star_count, 5);
        assert_eq!(c.star_total, 10);

        // fetch the person and assert name
        let p = people::Entity::find_by_id(c.person_id)
            .one(&db)
            .await
            .expect("fetch person");
        assert!(p.is_some());
        let p = p.unwrap();
        assert_eq!(p.first_name, "Bob");
    }
}
#[derive(Debug, Clone)]
pub struct SQLConnector {
    path: String,
    database_connection: Option<DatabaseConnection>,
}

impl SQLConnector {
    pub fn new(path: &str) -> Self {
        SQLConnector {
            path: path.to_string(),
            database_connection: None,
        }
    }
}

impl HGDBConnection for SQLConnector {
    async fn connect(&mut self) -> Result<(), anyhow::Error> {
        let db =
            Database::connect(format!("sqlite://{}/db.sqlite?mode=rwc", self.path.clone())).await?;
        db.get_schema_builder()
            .register(crate::entity::people::Entity)
            .register(crate::entity::person_parent::Entity)
            .register(crate::entity::star_charts::Entity)
            .sync(&db)
            .await?;
        self.database_connection = Some(db);
        Ok(())
    }
    async fn check(&self) -> Result<(), anyhow::Error> {
        if let Some(ref db) = self.database_connection {
            db.ping().await?;
        }
        Ok(())
    }
    async fn close(&self) -> Result<(), anyhow::Error> {
        if let Some(ref db) = self.database_connection {
            let db = db.clone();
            db.close().await?;
        }
        Ok(())
    }
    async fn is_initialized(&self) -> Result<bool, anyhow::Error> {
        let person = crate::entity::people::Entity::find()
            .one(self.database_connection.as_ref().unwrap())
            .await?;
        Ok(person.is_some())
    }
    async fn initialize(&self, config: &Configuration) -> Result<(), anyhow::Error> {
        // Let's populate the people table with our configuration data
        // for each family member setup the database entity with the right relationships

        let db = self.database_connection.as_ref().unwrap();

        // Insert all people first
        for member in config.family.values() {
            let person = crate::entity::people::ActiveModel {
                first_name: Set(member.first_name.clone()),
                last_name: Set(member.last_name.clone()),
                ..Default::default()
            };
            let _result = crate::entity::people::Entity::insert(person)
                .exec(db)
                .await?;
        }

        // Then create parent-child links based on the configuration's `children` lists
        // We do a lookup by first_name for both parent and child. If a name is missing
        // we skip that link with a debug context.
        for member in config.family.values() {
            // find parent record
            if let Some(parent) = crate::entity::people::Entity::find()
                .filter(crate::entity::people::Column::FirstName.eq(member.first_name.clone()))
                .one(db)
                .await?
            {
                for child_name in &member.children {
                    match crate::entity::people::Entity::find()
                        .filter(crate::entity::people::Column::FirstName.eq(child_name.clone()))
                        .one(db)
                        .await?
                    {
                        Some(child) => {
                            // insert link if not already present
                            let exists = crate::entity::person_parent::Entity::find()
                                .filter(
                                    crate::entity::person_parent::Column::ParentId
                                        .eq(parent.id),
                                )
                                .filter(
                                    crate::entity::person_parent::Column::ChildId.eq(child.id),
                                )
                                .one(db)
                                .await?;
                            if exists.is_none() {
                                crate::entity::helpers::add_parent_child(db, parent.id, child.id)
                                    .await
                                    .context(format!(
                                        "failed to add parent-child link {} -> {}",
                                        parent.id, child.id
                                    ))?;
                            }
                        }
                        None => {
                            // child not found; skip
                        }
                    }
                }
            } else {
                // parent not found; skip
            }
        }

        Ok(())
    }
    async fn get_person(
        &self,
        first_name: &str,
    ) -> Result<Option<GetPersonResponse>, anyhow::Error> {
        let person: Option<crate::entity::people::Model> = crate::entity::people::Entity::find()
            .filter(crate::entity::people::Column::FirstName.eq(first_name))
            .one(self.database_connection.as_ref().unwrap())
            .await?;
        // If found, also populate direct children (shallow) using the helpers
        let db = self.database_connection.as_ref().unwrap();
        if let Some(p) = person {
            let kids_models = match crate::entity::helpers::children_of(db, p.id).await {
                Ok(k) => k,
                Err(_) => Vec::new(),
            };

            let found_children = kids_models
                .into_iter()
                .map(|k| GetPersonResponse {
                    first_name: k.first_name,
                    last_name: k.last_name,
                    children: Vec::new(),
                    star_charts: Vec::new(),
                })
                .collect();

            // fetch star charts for this person
            let charts = crate::entity::star_charts::Entity::find()
                .filter(crate::entity::star_charts::Column::PersonId.eq(p.id))
                .all(db)
                .await?;

            let found_charts = charts
                .into_iter()
                .map(|c| GetStarChartResponse {
                    id: c.id,
                    name: c.chart_type,
                    description: c.chart_key,
                    star_count: c.star_count,
                    star_total: c.star_total,
                    person_first_name: p.first_name.clone(),
                    person_last_name: p.last_name.clone(),
                })
                .collect();

            Ok(Some(GetPersonResponse {
                first_name: p.first_name,
                last_name: p.last_name,
                children: found_children,
                star_charts: found_charts,
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_people(&self) -> Result<Vec<String>, anyhow::Error> {
        let people = crate::entity::people::Entity::find()
            .all(self.database_connection.as_ref().unwrap())
            .await?;

        let names = people
            .into_iter()
            .map(|person| format!("{} {}", person.first_name, person.last_name))
            .collect();

        Ok(names)
    }

    async fn create_person(
        &self,
        person: &CreatePersonRequest,
    ) -> Result<CreatePersonResponse, anyhow::Error> {
        let person = person.to_owned();

        let new_person = crate::entity::people::ActiveModel {
            first_name: Set(person.first_name.clone()),
            last_name: Set(person.last_name.clone()),
            ..Default::default()
        };

        let result = crate::entity::people::Entity::insert(new_person)
            .exec(self.database_connection.as_ref().unwrap())
            .await?;

        Ok(CreatePersonResponse {
            id: result.last_insert_id,
        })
    }

    async fn create_star_chart(
        &self,
        star_chart: &CreateStarChartRequest,
    ) -> Result<CreateStarChartResponse, anyhow::Error> {
        let star_chart = star_chart.to_owned();
        // Use the provided person_id and ensure the person exists
        let db = self.database_connection.as_ref().unwrap();
        let person = crate::entity::people::Entity::find_by_id(star_chart.person_id)
            .one(db)
            .await?;
        anyhow::ensure!(person.is_some(), "person {} not found", star_chart.person_id);

        let now = chrono::Utc::now();
        let new_star_chart = crate::entity::star_charts::ActiveModel {
            person_id: Set(star_chart.person_id),
            chart_type: Set(star_chart.name.clone()),
            chart_key: Set(star_chart.description.clone()),
            star_total: Set(star_chart.star_total),
            star_count: Set(star_chart.star_count),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        let result = crate::entity::star_charts::Entity::insert(new_star_chart)
            .exec(db)
            .await?;

        Ok(CreateStarChartResponse {
            id: result.last_insert_id,
        })
    }

    async fn get_star_charts(&self) -> Result<Vec<GetStarChartResponse>, anyhow::Error> {
        let charts = crate::entity::star_charts::Entity::find()
            .all(self.database_connection.as_ref().unwrap())
            .await?;

        let db = self.database_connection.as_ref().unwrap();
        let mut results: Vec<GetStarChartResponse> = Vec::new();
        for c in charts.into_iter() {
            // fetch owner person
            let person = crate::entity::people::Entity::find_by_id(c.person_id).one(db).await?;
            let (pf, pl) = match person {
                Some(p) => (p.first_name, p.last_name),
                None => ("".to_string(), "".to_string()),
            };
            results.push(GetStarChartResponse {
                id: c.id,
                name: c.chart_type.clone(),
                description: c.chart_key.clone(),
                star_count: c.star_count,
                star_total: c.star_total,
                person_first_name: pf,
                person_last_name: pl,
            });
        }

        Ok(results)
    }

    async fn delete_star_chart(&self, star_chart_id: i32) -> Result<(), anyhow::Error> {
        use crate::entity::star_charts;
        let db = self.database_connection.as_ref().unwrap();
        let _res = star_charts::Entity::delete_by_id(star_chart_id).exec(db).await?;
        Ok(())
    }

    async fn delete_person(&self, person_id: i32) -> Result<(), anyhow::Error> {
        use crate::entity::{people, person_parent, star_charts};
        let db = self.database_connection.as_ref().unwrap();

        // remove star charts
        let _ = star_charts::Entity::delete_many()
            .filter(star_charts::Column::PersonId.eq(person_id))
            .exec(db)
            .await?;

        // remove parent/child links involving this person
        let _ = person_parent::Entity::delete_many()
            .filter(person_parent::Column::ParentId.eq(person_id))
            .exec(db)
            .await?;
        let _ = person_parent::Entity::delete_many()
            .filter(person_parent::Column::ChildId.eq(person_id))
            .exec(db)
            .await?;

        // finally delete the person
        let _ = people::Entity::delete_by_id(person_id).exec(db).await?;
        Ok(())
    }

    async fn get_all_people(&self) -> Result<Vec<PersonListItem>, anyhow::Error> {
        let db = self.database_connection.as_ref().unwrap();
        let people = crate::entity::people::Entity::find().all(db).await?;
        let results = people
            .into_iter()
            .map(|p| PersonListItem {
                id: p.id,
                first_name: p.first_name,
                last_name: p.last_name,
            })
            .collect();
        Ok(results)
    }

    async fn get_star_chart(
        &self,
        star_chart_id: i32,
    ) -> Result<Option<GetStarChartResponse>, anyhow::Error> {
        let chart = crate::entity::star_charts::Entity::find_by_id(star_chart_id)
            .one(self.database_connection.as_ref().unwrap())
            .await?;

        if let Some(c) = chart {
            let db = self.database_connection.as_ref().unwrap();
            let person = crate::entity::people::Entity::find_by_id(c.person_id).one(db).await?;
            let (pf, pl) = match person {
                Some(p) => (p.first_name, p.last_name),
                None => ("".to_string(), "".to_string()),
            };
            Ok(Some(GetStarChartResponse {
                id: c.id,
                name: c.chart_type,
                description: c.chart_key,
                star_count: c.star_count,
                star_total: c.star_total,
                person_first_name: pf,
                person_last_name: pl,
            }))
        } else {
            Ok(None)
        }
    }

    async fn update_star_chart(
        &self,
        star_chart_id: i32,
        star_chart: &UpdateStarChartRequest,
    ) -> Result<UpdateStarChartResponse, anyhow::Error> {
        use crate::entity::star_charts;

        let db = self.database_connection.as_ref().unwrap();

        let existing = star_charts::Entity::find_by_id(star_chart_id).one(db).await?;
        anyhow::ensure!(existing.is_some(), "star chart {} not found", star_chart_id);

        let mut am: star_charts::ActiveModel = existing.unwrap().into();
        am.chart_type = Set(star_chart.name.clone());
        am.chart_key = Set(star_chart.description.clone());
        if let Some(sc) = star_chart.star_count {
            am.star_count = Set(sc);
        }
        if let Some(st) = star_chart.star_total {
            am.star_total = Set(st);
        }

        let res = am.update(db).await?;

        Ok(UpdateStarChartResponse { id: res.id })
    }

    async fn increment_star_chart(
        &self,
        star_chart_id: i32,
        delta: i32,
    ) -> Result<UpdateStarChartResponse, anyhow::Error> {
        // Delegate to helper
        self.increment_star_chart_internal(star_chart_id, delta).await
    }
}
