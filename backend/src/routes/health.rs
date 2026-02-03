use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct Health {
    status: &'static str,
}

// Needs async for axum to implement trait `Handler<_, _>`
#[allow(clippy::unused_async)]
pub async fn health() -> Json<Health> {
    Json(Health { status: "ok" })
}
