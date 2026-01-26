use axum::extract::{State, Json};
use deadpool_sqlite::Pool;
use serde::Serialize;
use crate::user::NewUser;

#[derive(Serialize)]
pub enum RegisterStatus {
    Success,
    UserAlreadyExists,
    InvalidInput,
}

pub async fn register(
    State(_pool): State<Pool>,
    Json(_user): Json<NewUser>,
) -> Json<RegisterStatus> {
    Json(RegisterStatus::Success)
}
