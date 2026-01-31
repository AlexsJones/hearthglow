use crate::data::configuration::Configuration;
use crate::data::dbconnector::{HGDBConnection, SQLConnector};
use axum::{Router, routing::get};
use log::debug;
pub async fn run(config: Configuration, database_connection: SQLConnector, port: u16) {
    // Start the server
    debug!("Starting server on port {}", port);
    let app = Router::new().route("/", get(|| async { "Hello, World!" }));

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
    database_connection.close().await.unwrap();
}
