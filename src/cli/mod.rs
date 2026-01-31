use crate::data::configuration::Configuration;
use crate::data::dbconnector::{HGDBConnection, SQLConnector};
use crate::server;
use clap::Parser;
use log::{debug, info};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = true)]
    server: bool,
    #[arg(short, long, default_value_t = 8080)]
    port: u16,
}

pub async fn run() {
    let args = Args::parse();

    debug!("Running in server mode: {}", args.server);
    if !args.server {
        // Client code here
    }
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
    // Start the server
    server::run(config, db_connector, args.port).await;
}
