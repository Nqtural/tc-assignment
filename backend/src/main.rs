use anyhow::Result;
use axum::{
    routing::get,
    routing::post,
    Router,
};
use tower_cookies::CookieManagerLayer;
use std::env;

use backend::{data::Database, routes};
use backend::cors::dev_cors;

#[tokio::main]
async fn main() -> Result<()> {
    
    let mut dev = false;

    for arg in env::args().skip(1) {
        match arg.as_str() {
            "--dev" => dev = true,
            _ => {
                println!(" ");
            }
        }
    }

    if dev {
        println!("dev mode")
    }

    let database = Database::new().await?;

    let app = Router::new()
        .route("/auth/is_logged_in", post(routes::auth::is_logged_in))
        .route("/auth/login", post(routes::auth::login))
        .route("/auth/logout", post(routes::auth::logout))
        .route("/auth/register", post(routes::auth::register))
        .route("/health", get(routes::health::health))
        .route("/rooms/create", post(routes::rooms::create))
        .route("/rooms/get", get(routes::rooms::get))
        .with_state(database)
        .layer(CookieManagerLayer::new())
        .layer(dev_cors());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
