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

#[derive(Debug, Serialize)]
pub struct GetPersonResponse {
    pub first_name: String,
    pub last_name: String,
    pub children: Vec<GetPersonResponse>,
    pub star_charts: Vec<GetStarChartResponse>,
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
    pub star_count: i32,
    pub star_total: i32,
    // who this chart belongs to
    pub person_first_name: String,
    pub person_last_name: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStarChartRequest {
    pub name: String,
    pub description: String,
    pub star_count: Option<i32>,
    pub star_total: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct PersonListItem {
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
}

#[derive(Debug, Deserialize)]
pub struct IncrementStarChartRequest {
    pub delta: i32,
}

#[derive(Debug, Serialize)]
pub struct UpdateStarChartResponse {
    pub id: i32,
}

#[derive(Debug, Deserialize)]
pub struct CreateCalendarEventRequest {
    pub title: String,
    pub person_id: i32,
    pub start: String,
    pub end: String,
}

#[derive(Debug, Serialize)]
pub struct CreateCalendarEventResponse {
    pub id: i32,
}

#[derive(Debug, Serialize)]
pub struct CalendarEventResponse {
    pub id: i32,
    pub title: String,
    pub start: String,
    pub end: String,
    #[serde(rename = "resourceId")]
    pub resource_id: i32,
}

#[derive(Debug, Serialize)]
pub struct CalendarPersonResponse {
    pub id: i32,
    pub title: String,
    #[serde(rename = "eventBackgroundColor")]
    pub event_background_color: Option<String>,
    #[serde(rename = "eventTextColor")]
    pub event_text_color: Option<String>,
}
