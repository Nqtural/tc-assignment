use anyhow::Result;
use axum::{
    routing::get,
    routing::post,
    Router,
};
use deadpool_sqlite::Pool;
use tower_cookies::CookieManagerLayer;
use backend::{data, routes};
use backend::cors::dev_cors;

#[tokio::main]
async fn main() -> Result<()> {
    let database = data::Database::new().await?;

    let app = Router::<Pool>::new()
        .route("/health", get(routes::health::health))
        .route("/auth/register", post(routes::auth::register))
        .route("/auth/is_logged_in", post(routes::auth::is_logged_in))
        .route("/auth/login", post(routes::auth::login))
        .route("/auth/logout", post(routes::auth::logout))
        .with_state(database.pool)
        .layer(CookieManagerLayer::new())
        // always dev mode for now
        .layer(dev_cors());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
