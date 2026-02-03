use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Serialize;
use tower_cookies::Cookies;
use crate::data::Database;

#[derive(Serialize)]
enum LoggedInStatus {
    LoggedIn,
    LoggedOut,
}

pub async fn is_logged_in(
    State(db): State<Database>,
    cookies: Cookies,
) -> impl IntoResponse {
    let status = match cookies.get("session_uuid") {
        Some(c) => {
            if db.verify_session(c.value().to_string()).await.unwrap_or(false) {
                LoggedInStatus::LoggedIn
            } else {
                LoggedInStatus::LoggedOut
            }
        }
        None => LoggedInStatus::LoggedOut,
    };

    (StatusCode::OK, Json(status))
}
