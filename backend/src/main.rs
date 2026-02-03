use anyhow::Result;
use axum::{
    routing::get,
    routing::post,
    Router,
};
use tower_cookies::CookieManagerLayer;
use backend::{data::Database, routes};
use backend::cors::dev_cors;

#[tokio::main]
async fn main() -> Result<()> {
    let database = Database::new().await?;

    let app = Router::<Database>::new()
        .route("/health", get(routes::health::health))
        .route("/auth/is_logged_in", post(routes::auth::is_logged_in))
        .route("/auth/login", post(routes::auth::login))
        .route("/auth/logout", post(routes::auth::logout))
        .route("/auth/register", post(routes::auth::register))
        .with_state(database)
        .layer(CookieManagerLayer::new())
        // always dev mode for now
        .layer(dev_cors());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
