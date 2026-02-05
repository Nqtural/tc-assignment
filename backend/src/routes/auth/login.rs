use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use tower_cookies::{Cookies, Cookie};

use crate::data::{Database, LoginOutcome};

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
