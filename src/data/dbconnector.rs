use anyhow::Error;
use sea_orm::{Database, DatabaseConnection};

pub(crate) trait HGDBConnection {
    async fn connect(&mut self) -> Result<(), anyhow::Error>;
    async fn check(&self) -> Result<(), anyhow::Error>;
    async fn close(&self) -> Result<(), anyhow::Error>;
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
}
