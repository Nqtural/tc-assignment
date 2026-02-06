use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use tower_cookies::Cookies;
use serde::Serialize;

use crate::data::{Database, CreateRoomOutcome};
use crate::types::NewRoom;

#[derive(Serialize)]
pub enum CreateRoomStatus {
    Success,
    InvalidCredentials,
    InternalServerError,
}

pub async fn create(
    State(db): State<Database>,
    cookies: Cookies,
    Json(room): Json<NewRoom>,
) -> impl IntoResponse {
    if let Some(session_uuid_cookie) = cookies.get("session_uuid") {
        match db.create_room(session_uuid_cookie.value().to_string(), room).await {
            Ok(CreateRoomOutcome::Success) => (StatusCode::OK, Json(CreateRoomStatus::Success)),
            Ok(CreateRoomOutcome::NotLoggedIn) => (StatusCode::UNAUTHORIZED, Json(CreateRoomStatus::InvalidCredentials)),
            Err(e) => {
                eprintln!("Create room error: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, Json(CreateRoomStatus::InternalServerError))
            }
        }
    } else {
        (StatusCode::UNAUTHORIZED, Json(CreateRoomStatus::InvalidCredentials))
    }
}
