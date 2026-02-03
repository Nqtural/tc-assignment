use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Serialize;
use tower_cookies::Cookies;
use crate::data::{Database, LogoutOutcome};

#[derive(Serialize)]
enum LogoutStatus {
    Success,
    NotLoggedIn,
    InternalServerError,
}

pub async fn logout(
    State(db): State<Database>,
    cookies: Cookies,
) -> impl IntoResponse {
    let status = match cookies.get("session_uuid") {
        Some(c) => db.logout_user(c.value().to_string()).await.unwrap_or(LogoutOutcome::InternalServerError),
        None => LogoutOutcome::NotLoggedIn,
    };

    match status {
        LogoutOutcome::Success => (StatusCode::OK, Json(LogoutStatus::Success)),
        LogoutOutcome::NotLoggedIn => (StatusCode::OK, Json(LogoutStatus::NotLoggedIn)),
        LogoutOutcome::InternalServerError => (StatusCode::INTERNAL_SERVER_ERROR, Json(LogoutStatus::InternalServerError)),
    }
}

