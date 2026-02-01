use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreatePersonRequest {
    pub first_name: String,
    pub last_name: String,
}

#[derive(Debug, Serialize)]
pub struct CreatePersonResponse {
    pub id: i32,
}

#[derive(Debug, Serialize)]
pub struct GetPersonRequest {
    pub first_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetPersonResponse {
    pub first_name: String,
    pub last_name: String,
    pub children: Vec<GetPersonResponse>,
}

#[derive(Debug, Deserialize)]
pub struct CreateStarChartRequest {
    pub name: String,
    pub description: String,
    pub person_id: i32,
    pub star_count: i32,
    pub star_total: i32,
}

#[derive(Debug, Serialize)]
pub struct CreateStarChartResponse {
    pub id: i32,
}

#[derive(Debug, Serialize)]
pub struct GetStarChartResponse {
    pub id: i32,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStarChartRequest {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Serialize)]
pub struct UpdateStarChartResponse {
    pub id: i32,
}
