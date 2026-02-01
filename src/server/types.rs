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
}

#[derive(Debug, Deserialize)]
pub struct CreateStarChartRequest {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStarChartRequest {
    pub name: String,
    pub description: String,
}
