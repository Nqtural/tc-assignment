use anyhow::Result;
use axum::{
    routing::{delete, get, post},
    Router,
};
use tower_cookies::CookieManagerLayer;
use std::env;

use backend::{data::Database, routes};
use backend::cors;

#[tokio::main]
async fn main() -> Result<()> {
    
    let mut dev = false;

    for arg in env::args().skip(1) {
        dev = matches!(arg.as_str(), "--dev");
    }

    let cors_layer = if dev {
        println!("WARN: running in development mode");
        cors::dev()
    } else {
        cors::prod()
    };

    let database = Database::new().await?;

    let app = Router::new()
        .route("/auth/is_logged_in", post(routes::auth::is_logged_in))
        .route("/auth/login", post(routes::auth::login))
        .route("/auth/logout", post(routes::auth::logout))
        .route("/auth/register", post(routes::auth::register))
        .route("/health", get(routes::health::health))
        .route("/rooms/create", post(routes::rooms::create))
        .route("/rooms/delete/{id}", delete(routes::rooms::delete))
        .route("/rooms/get", get(routes::rooms::get))
        .with_state(database)
        .layer(CookieManagerLayer::new())
        .layer(cors_layer);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
