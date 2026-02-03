use crate::data::configuration::Configuration;
use crate::server::types::*;
use anyhow::Context;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, Database, DatabaseConnection, EntityTrait,
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
            .register(crate::entity::calendar_events::Entity)
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

        for member in config.family.values() {
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
                            let exists = crate::entity::person_parent::Entity::find()
                                .filter(
                                    crate::entity::person_parent::Column::ParentId.eq(parent.id),
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
                        None => {}
                    }
                }
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
        let db = self.database_connection.as_ref().unwrap();
        if let Some(p) = person {
            let kids_models = match crate::entity::helpers::children_of(db, p.id).await {
                Ok(k) => k,
                Err(_) => Vec::new(),
            };

            let found_children = kids_models
                .into_iter()
                .map(|k| GetPersonResponse {
                    id: k.id,
                    first_name: k.first_name,
                    last_name: k.last_name,
                    children: Vec::new(),
                    star_charts: Vec::new(),
                })
                .collect();

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
                id: p.id,
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

        let _ = star_charts::Entity::delete_many()
            .filter(star_charts::Column::PersonId.eq(person_id))
            .exec(db)
            .await?;

        let _ = person_parent::Entity::delete_many()
            .filter(person_parent::Column::ParentId.eq(person_id))
            .exec(db)
            .await?;
        let _ = person_parent::Entity::delete_many()
            .filter(person_parent::Column::ChildId.eq(person_id))
            .exec(db)
            .await?;

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
        self.increment_star_chart_internal(star_chart_id, delta).await
    }
}

impl SQLConnector {
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

impl SQLConnector {
    pub async fn list_calendar_people(
        &self,
    ) -> Result<Vec<crate::server::types::CalendarPersonResponse>, anyhow::Error> {
        let db = self.database_connection.as_ref().unwrap();
        let people = crate::entity::people::Entity::find().all(db).await?;
        let palette = [
            "#ff8a65",
            "#ffd54f",
            "#81c784",
            "#64b5f6",
            "#ba68c8",
            "#4db6ac",
        ];
        let items = people
            .into_iter()
            .map(|p| {
                let color = palette[(p.id as usize) % palette.len()].to_string();
                crate::server::types::CalendarPersonResponse {
                    id: p.id,
                    title: format!("{} {}", p.first_name, p.last_name),
                    event_background_color: Some(color),
                    event_text_color: Some("#2b1a0f".to_string()),
                }
            })
            .collect();
        Ok(items)
    }

    pub async fn list_calendar_events(
        &self,
    ) -> Result<Vec<crate::server::types::CalendarEventResponse>, anyhow::Error> {
        let db = self.database_connection.as_ref().unwrap();
        let events = crate::entity::calendar_events::Entity::find().all(db).await?;
        Ok(events
            .into_iter()
            .map(|event| crate::server::types::CalendarEventResponse {
                id: event.id,
                title: event.title,
                start: event.start_time,
                end: event.end_time,
                resource_id: event.person_id,
            })
            .collect())
    }

    pub async fn create_calendar_event(
        &self,
        payload: &crate::server::types::CreateCalendarEventRequest,
    ) -> Result<crate::server::types::CreateCalendarEventResponse, anyhow::Error> {
        let db = self.database_connection.as_ref().unwrap();
        let event = crate::entity::calendar_events::ActiveModel {
            person_id: Set(payload.person_id),
            title: Set(payload.title.clone()),
            start_time: Set(payload.start.clone()),
            end_time: Set(payload.end.clone()),
            ..Default::default()
        };
        let result = event.insert(db).await?;
        Ok(crate::server::types::CreateCalendarEventResponse { id: result.id })
    }
}
