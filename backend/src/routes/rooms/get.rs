use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use tower_cookies::Cookies;
use serde::Serialize;

use crate::data::{Database, GetRoomOutcome};
use crate::types::Room;

#[derive(Serialize)]
pub enum GetRoomStatus {
    Success(Vec<Room>),
    InvalidCredentials,
    InternalServerError,
}

pub async fn get(
    State(db): State<Database>,
    cookies: Cookies,
) -> impl IntoResponse {
    if let Some(session_uuid_cookie) = cookies.get("session_uuid") {

        match db.get_rooms(session_uuid_cookie.value().to_string()).await {
            Ok(GetRoomOutcome::Success(rooms)) => (StatusCode::OK, Json(GetRoomStatus::Success(rooms))),
            Ok(GetRoomOutcome::NotLoggedIn) => (StatusCode::UNAUTHORIZED, Json(GetRoomStatus::InvalidCredentials)),
            Err(e) => {
                eprintln!("Get rooms error: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, Json(GetRoomStatus::InternalServerError))
            }
        }
    } else {
        (StatusCode::UNAUTHORIZED, Json(GetRoomStatus::InvalidCredentials))
    }
}
