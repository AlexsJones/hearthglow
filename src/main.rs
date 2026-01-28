pub mod data;
use data::configuration::Configuration;
use data::dbconnector::{HGDBConnection, SQLConnector};
use log::debug;
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

    db_connector.close().await.unwrap();
}
