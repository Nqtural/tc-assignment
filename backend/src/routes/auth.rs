use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use tower_cookies::{Cookies, Cookie};
use crate::data::{Database, LoginOutcome, LogoutOutcome, RegisterOutcome};
use crate::user::NewUser;

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

#[derive(Serialize)]
enum LoginStatus {
    Success,
    UserDoesNotExist,
    InvalidCredentials,
    InternalServerError,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

pub async fn login(
    State(db): State<Database>,
    cookies: Cookies,
    Json(user): Json<LoginRequest>,
) -> impl IntoResponse {
    match db.login_user(user.email, user.password).await {
        Ok(LoginOutcome::Success(session_uuid)) => {
            // set the cookie
            let c = Cookie::build(("session_uuid", session_uuid))
                .path("/")
                .http_only(true);
                // we don't run server/frontend over https yet
                //.secure(true); // only over HTTPS

            cookies.add(c.into());

            (StatusCode::OK, Json(LoginStatus::Success))
        }
        Ok(LoginOutcome::UserDoesNotExist) => (StatusCode::NOT_ACCEPTABLE, Json(LoginStatus::UserDoesNotExist)),
        Ok(LoginOutcome::InvalidCredentials) => (StatusCode::UNAUTHORIZED, Json(LoginStatus::InvalidCredentials)),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(LoginStatus::InternalServerError)),
    }
}

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

#[derive(Serialize)]
pub enum RegisterStatus {
    Success,
    UserAlreadyExists,
    InternalServerError,
}

pub async fn register(
    State(db): State<Database>,
    Json(user): Json<NewUser>,
) -> impl IntoResponse {
    match db.register_user(user).await {
        Ok(RegisterOutcome::Success) => (StatusCode::OK, Json(RegisterStatus::Success)),
        Ok(RegisterOutcome::UserAlreadyExists) => (StatusCode::CONFLICT, Json(RegisterStatus::UserAlreadyExists)),
        Err(e) => {
            eprintln!("Register error: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(RegisterStatus::InternalServerError))
        }
    }
}
