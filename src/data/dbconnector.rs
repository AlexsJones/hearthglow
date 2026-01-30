use sea_orm::{ActiveValue::Set, Database, DatabaseConnection, EntityTrait};

pub(crate) trait HGDBConnection {
    async fn connect(&mut self) -> Result<(), anyhow::Error>;
    async fn check(&self) -> Result<(), anyhow::Error>;
    async fn close(&self) -> Result<(), anyhow::Error>;
    async fn is_initialized(&self) -> Result<bool, anyhow::Error>;
    async fn initialize(&self, config: &crate::Configuration) -> Result<(), anyhow::Error>;
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
            .register(crate::entity::people::Entity)
            .register(crate::entity::person_parent::Entity)
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
    async fn initialize(&self, config: &crate::Configuration) -> Result<(), anyhow::Error> {
        // Let's populate the people table with our configuration data
        // for each family member setup the database entity with the right relationships

        for (_name, member) in &config.family {
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
}
