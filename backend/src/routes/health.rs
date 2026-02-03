use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct Health {
    status: &'static str,
}

pub const fn health() -> Json<Health> {
    Json(Health { status: "ok" })
}
