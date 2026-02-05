use axum::{
    extract::{Path, State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use tower_cookies::Cookies;
use serde::Serialize;

use crate::data::{Database, LeaveRoomOutcome};

#[derive(Serialize)]
pub enum LeaveRoomStatus {
    Success,
    InternalServerError,
    NotLoggedIn,
    NotMember,
    OwnerCannotLeave,
}

pub async fn leave(
    State(db): State<Database>,
    cookies: Cookies,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    if let Some(session_uuid_cookie) = cookies.get("session_uuid") {
        match db.leave_room(session_uuid_cookie.value().to_string(), id).await {
            Ok(LeaveRoomOutcome::Success) => (StatusCode::OK, Json(LeaveRoomStatus::Success)),
            Ok(LeaveRoomOutcome::NotLoggedIn) => (StatusCode::UNAUTHORIZED, Json(LeaveRoomStatus::NotLoggedIn)),
            Ok(LeaveRoomOutcome::NotMember) => (StatusCode::BAD_REQUEST, Json(LeaveRoomStatus::NotMember)),
            Ok(LeaveRoomOutcome::OwnerCannotLeave) => (StatusCode::BAD_REQUEST, Json(LeaveRoomStatus::OwnerCannotLeave)),
            Err(e) => {
                eprintln!("Leave room error: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, Json(LeaveRoomStatus::InternalServerError))
            }
        }
    } else {
        (StatusCode::UNAUTHORIZED, Json(LeaveRoomStatus::NotLoggedIn))
    }
}
