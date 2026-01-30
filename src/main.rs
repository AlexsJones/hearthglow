pub mod data;
pub(crate) mod entity;

use data::configuration::Configuration;
use data::dbconnector::{HGDBConnection, SQLConnector};
use log::{debug, info};
use std::env;

#[tokio::main]
async fn main() {
    unsafe {
        env::set_var("RUST_LOG", "debug");
    }
    env_logger::init();
    //
    let config = Configuration::load().unwrap();
    debug!("Loaded configuration: {:?}", config);
    // Check the database connection
    let mut db_connector = SQLConnector::new(&config.database.path);
    db_connector.connect().await.unwrap();
    db_connector.check().await.unwrap();
    // Has database been initialised ?
    if db_connector.is_initialized().await.unwrap() {
        info!("Database is initialized");
    } else {
        info!("Database is not initialized, performing initialization");
        db_connector.initialize(&config).await.unwrap();
    }

    db_connector.close().await.unwrap();
}
