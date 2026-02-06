use axum::{
    extract::{Path, State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use tower_cookies::Cookies;
use serde::Serialize;

use crate::data::{Database, InvitationCodeOutcome};

#[derive(Serialize)]
pub enum InvitationCodeStatus {
    Success(String),
    InternalServerError,
    NotLoggedIn,
    NotOwner,
}

pub async fn invitation_code(
    State(db): State<Database>,
    cookies: Cookies,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    if let Some(session_uuid_cookie) = cookies.get("session_uuid") {
        match db.get_invitation_code(session_uuid_cookie.value().to_string(), id).await {
            Ok(InvitationCodeOutcome::Success(code)) => (StatusCode::OK, Json(InvitationCodeStatus::Success(code))),
            Ok(InvitationCodeOutcome::NotLoggedIn) => (StatusCode::UNAUTHORIZED, Json(InvitationCodeStatus::NotLoggedIn)),
            Ok(InvitationCodeOutcome::NotOwner) => (StatusCode::UNAUTHORIZED, Json(InvitationCodeStatus::NotOwner)),
            Err(e) => {
                eprintln!("Invitation code error: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, Json(InvitationCodeStatus::InternalServerError))
            }
        }
    } else {
        (StatusCode::UNAUTHORIZED, Json(InvitationCodeStatus::NotLoggedIn))
    }
}
