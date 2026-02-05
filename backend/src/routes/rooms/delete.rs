use axum::{
    extract::{Path, State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use tower_cookies::Cookies;
use serde::Serialize;

use crate::data::{Database, DeleteRoomOutcome};

#[derive(Serialize)]
pub enum DeleteRoomStatus {
    Success,
    InternalServerError,
    NotLoggedIn,
    NotOwner,
}

pub async fn delete(
    State(db): State<Database>,
    cookies: Cookies,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    if let Some(session_uuid_cookie) = cookies.get("session_uuid") {
        match db.delete_room(session_uuid_cookie.value().to_string(), id).await {
            Ok(DeleteRoomOutcome::Success) => (StatusCode::OK, Json(DeleteRoomStatus::Success)),
            Ok(DeleteRoomOutcome::NotLoggedIn) => (StatusCode::UNAUTHORIZED, Json(DeleteRoomStatus::NotLoggedIn)),
            Ok(DeleteRoomOutcome::NotOwner) => (StatusCode::UNAUTHORIZED, Json(DeleteRoomStatus::NotOwner)),
            Err(e) => {
                eprintln!("Create room error: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, Json(DeleteRoomStatus::InternalServerError))
            }
        }
    } else {
        (StatusCode::UNAUTHORIZED, Json(DeleteRoomStatus::NotLoggedIn))
    }
}
