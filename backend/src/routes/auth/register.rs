use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Serialize;

use crate::data::{Database, RegisterOutcome};
use crate::types::NewUser;

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
