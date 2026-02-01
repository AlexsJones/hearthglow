use crate::data::configuration::Configuration;
use crate::data::dbconnector::HGDBConnection;
use crate::data::dbconnector::SQLConnector;
use crate::server::types::CreatePersonResponse;
use crate::server::types::GetPersonRequest;
use crate::server::types::GetPersonResponse;
pub(crate) mod types;
use axum::extract::Path;
use axum::routing::patch;
use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use log::debug;
use serde::Deserialize;
use std::sync::Arc;
use types::{CreatePersonRequest, CreateStarChartRequest, UpdateStarChartRequest};

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
        .route("/stars", get(list_star_charts))
        .route("/stars", post(create_star_chart))
        .route("/stars/{id}", get(get_star_chart))
        .route("/stars/{id}", patch(update_star_chart))
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
async fn get_person(
    State(state): State<ServerConfig>,
    Path(name): Path<String>,
) -> Result<Json<GetPersonResponse>, (StatusCode, String)> {
    let person = state
        .database_connection
        .get_person(&name)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    match person {
        Some(p) => Ok(Json(p)),
        None => Err((StatusCode::NOT_FOUND, format!("No person named {}", &name))),
    }
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

async fn create_star_chart(
    State(state): State<ServerConfig>,
    Json(payload): Json<CreateStarChartRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    Ok(StatusCode::CREATED)
}

async fn update_star_chart(
    State(state): State<ServerConfig>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateStarChartRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    Ok(StatusCode::OK)
}

async fn get_star_chart(
    State(state): State<ServerConfig>,
    Path(id): Path<i32>,
) -> Result<String, (StatusCode, String)> {
    Ok(format!("Star Chart: {:?}", ""))
}

async fn list_star_charts(
    State(state): State<ServerConfig>,
) -> Result<String, (StatusCode, String)> {
    Ok(format!("Star Charts: {:?}", "star_charts"))
}
