use crate::data::configuration::Configuration;
use crate::data::dbconnector::HGDBConnection;
use crate::data::dbconnector::SQLConnector;
use axum::extract::Path;
use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use log::debug;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Clone)]
pub struct ServerConfig {
    pub database_connection: Arc<SQLConnector>,
}

pub async fn run(_config: Configuration, database_connection: SQLConnector, port: u16) {
    debug!("Starting server on port {}", port);

    let shared_db = Arc::new(database_connection);

    let app = Router::new()
        .route("/people", get(list_people))
        .route("/people/{first_name}", get(get_person))
        .route("/people", post(create_person))
        .with_state(ServerConfig {
            database_connection: shared_db.clone(),
        });

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .unwrap();

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();

    // NOTE: this only runs after the server stops.
    // Also: you can’t call close() on Arc<T> unless SQLConnector::close takes &self.
    // shared_db.close().await.unwrap();
}

#[derive(Debug, Deserialize)]
pub struct CreatePersonRequest {
    pub first_name: String,
    pub last_name: String,
}

async fn get_person(
    State(state): State<ServerConfig>,
    Path(first_name): Path<String>,
) -> Result<String, (StatusCode, String)> {
    let person = state
        .database_connection
        .get_person(first_name.as_str())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(format!("Person: {:?}", person))
}

async fn list_people(State(state): State<ServerConfig>) -> Result<String, (StatusCode, String)> {
    let people = state
        .database_connection
        .get_people()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(format!("People: {:?}", people))
}

async fn create_person(
    State(state): State<ServerConfig>,
    Json(payload): Json<CreatePersonRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .database_connection
        .as_ref()
        .create_person(&payload)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::CREATED)
}
