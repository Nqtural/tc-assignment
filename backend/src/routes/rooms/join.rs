use axum::{
    extract::{Path, State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use tower_cookies::Cookies;
use serde::Serialize;

use crate::data::{Database, JoinRoomOutcome};

#[derive(Serialize)]
pub enum JoinRoomStatus {
    Success,
    AlreadyMember,
    InvalidCode,
    InvalidCredentials,
    InternalServerError,
}

pub async fn join(
    State(db): State<Database>,
    cookies: Cookies,
    Path(code): Path<String>,
) -> impl IntoResponse {
    if let Some(session_uuid_cookie) = cookies.get("session_uuid") {

        match db.join_room(session_uuid_cookie.value().to_string(), code).await {
            Ok(JoinRoomOutcome::Success) => (StatusCode::OK, Json(JoinRoomStatus::Success)),
            Ok(JoinRoomOutcome::AlreadyMember) => (StatusCode::CONFLICT, Json(JoinRoomStatus::AlreadyMember)),
            Ok(JoinRoomOutcome::NotLoggedIn) => (StatusCode::UNAUTHORIZED, Json(JoinRoomStatus::InvalidCredentials)),
            Ok(JoinRoomOutcome::InvalidCode) => (StatusCode::NOT_FOUND, Json(JoinRoomStatus::InvalidCode)),
            Err(e) => {
                eprintln!("Join room error: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, Json(JoinRoomStatus::InternalServerError))
            }
        }
    } else {
        (StatusCode::UNAUTHORIZED, Json(JoinRoomStatus::InvalidCredentials))
    }
}
