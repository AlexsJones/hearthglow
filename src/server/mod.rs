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
    routing::{get, post, delete},
    response::IntoResponse,
};
use log::debug;
use std::sync::Arc;
use types::{
    CreatePersonRequest,
    CreateStarChartRequest,
    UpdateStarChartRequest,
    IncrementStarChartRequest,
};

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
    .route("/admin/people", get(admin_list_people))
    .route("/admin/people/{id}", delete(admin_delete_person))
    .route("/admin/stars/{id}", delete(admin_delete_star))
        .route("/stars", get(get_star_charts))
    .route("/stars", post(create_star_chart))
        .route("/stars/{id}", get(get_star_chart))
    .route("/stars/{id}", patch(update_star_chart))
    .route("/stars/{id}/increment", post(increment_star_chart))
        .route("/app.js", get(serve_app_js))
    .route("/styles.css", get(serve_styles))
    .route("/", get(serve_index))
        .route("/initialize", post(initialize_db))
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

async fn serve_index() -> Result<impl IntoResponse, (StatusCode, String)> {
    match tokio::fs::read("frontend/dist/index.html").await {
        Ok(bytes) => Ok(([("content-type", "text/html; charset=utf-8")], bytes)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn serve_app_js() -> Result<impl IntoResponse, (StatusCode, String)> {
    match tokio::fs::read("frontend/dist/app.js").await {
        Ok(bytes) => Ok(([("content-type", "application/javascript; charset=utf-8")], bytes)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn serve_styles() -> Result<impl IntoResponse, (StatusCode, String)> {
    match tokio::fs::read("frontend/dist/styles.css").await {
        Ok(bytes) => Ok(([("content-type", "text/css; charset=utf-8")], bytes)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
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

async fn list_people(State(state): State<ServerConfig>) -> Result<Json<Vec<String>>, (StatusCode, String)> {
    let people = state
        .database_connection
        .get_people()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // people are returned as "First Last" strings; extract first names
    let first_names: Vec<String> = people
        .into_iter()
        .map(|full| full.split_whitespace().next().unwrap_or("").to_string())
        .collect();

    Ok(Json(first_names))
}

async fn create_person(
    State(state): State<ServerConfig>,
    Json(payload): Json<CreatePersonRequest>,
) -> Result<(StatusCode, Json<CreatePersonResponse>), (StatusCode, String)> {
    let resp = state
        .database_connection
        .as_ref()
        .create_person(&payload)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(resp)))
}

async fn create_star_chart(
    State(state): State<ServerConfig>,
    Json(payload): Json<CreateStarChartRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let _star_chart = state
        .database_connection
        .as_ref()
        .create_star_chart(&payload)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::CREATED)
}

async fn update_star_chart(
    State(state): State<ServerConfig>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateStarChartRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let _star_chart = state
        .database_connection
        .as_ref()
        .update_star_chart(id, &payload)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::OK)
}

async fn increment_star_chart(
    State(state): State<ServerConfig>,
    Path(id): Path<i32>,
    Json(payload): Json<IncrementStarChartRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .database_connection
        .as_ref()
        .increment_star_chart(id, payload.delta)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::OK)
}

async fn get_star_chart(
    State(state): State<ServerConfig>,
    Path(id): Path<i32>,
) -> Result<Json<crate::server::types::GetStarChartResponse>, (StatusCode, String)> {
    let star_chart = state
        .database_connection
        .as_ref()
        .get_star_chart(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    match star_chart {
        Some(s) => Ok(Json(s)),
        None => Err((
            StatusCode::NOT_FOUND,
            format!("No star chart with id {}", id),
        )),
    }
}

async fn get_star_charts(
    State(state): State<ServerConfig>,
) -> Result<Json<Vec<crate::server::types::GetStarChartResponse>>, (StatusCode, String)> {
    let charts = state
        .database_connection
        .as_ref()
        .get_star_charts()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(charts))
}

async fn admin_list_people(State(state): State<ServerConfig>) -> Result<Json<Vec<crate::server::types::PersonListItem>>, (StatusCode, String)> {
    let people = state
        .database_connection
        .as_ref()
        .get_all_people()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(people))
}

async fn admin_delete_person(
    State(state): State<ServerConfig>,
    Path(id): Path<i32>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .database_connection
        .as_ref()
        .delete_person(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::OK)
}

async fn admin_delete_star(
    State(state): State<ServerConfig>,
    Path(id): Path<i32>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .database_connection
        .as_ref()
        .delete_star_chart(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::OK)
}

async fn initialize_db(State(state): State<ServerConfig>) -> Result<StatusCode, (StatusCode, String)> {
    // Load configuration from file and initialize DB (convenience endpoint for development)
    let cfg = Configuration::load().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    state
        .database_connection
        .as_ref()
        .initialize(&cfg)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::OK)
}
