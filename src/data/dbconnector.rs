use crate::data::configuration::Configuration;
use crate::server::types::*;
use sea_orm::{
    ActiveValue::Set, ColumnTrait, Database, DatabaseConnection, EntityTrait, QueryFilter,
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

        for member in config.family.values() {
            let person = crate::entity::people::ActiveModel {
                first_name: Set(member.first_name.clone()),
                last_name: Set(member.last_name.clone()),
                ..Default::default()
            };
            let _result = crate::entity::people::Entity::insert(person)
                .exec(self.database_connection.as_ref().unwrap())
                .await?;

            // setup relationships with the entity::helpers
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
        Ok(person.map(|p| GetPersonResponse {
            first_name: p.first_name,
            last_name: p.last_name,
        }))
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
}
