use axum::{
    routing::get,
    routing::post,
    Router,
};
use deadpool_sqlite::{Config, Pool, Runtime};
use deadpool_sqlite::rusqlite::{self, params};
use tower_cookies::CookieManagerLayer;
use backend::routes;
use backend::user;
use backend::cors::dev_cors;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = Config::new("db.sqlite3");
    let pool = cfg.create_pool(Runtime::Tokio1).unwrap();

    {
        let conn = pool.get().await.unwrap();
        conn.interact(|conn| {
            conn.execute(
                "CREATE TABLE IF NOT EXISTS users (
                    id INTEGER PRIMARY KEY,
                    name TEXT NOT NULL,
                    surname TEXT NOT NULL,
                    password_hash TEXT NOT NULL,
                    email TEXT NOT NULL
                )",
                [],
            )?;

            conn.execute(
                "INSERT INTO users (id, name, surname, password_hash, email)
                VALUES (1, ?1, ?2, ?3, ?4)
                ON CONFLICT(id) DO UPDATE SET
                    name = excluded.name,
                    surname = excluded.surname,
                    password_hash = excluded.password_hash,
                    email = excluded.email",
                    params!["Admin", "Admin", "passwd_hash", "admin@example.com"],
            )?;
            
            conn.execute("DROP TABLE IF EXISTS sessions", [])?;
            conn.execute(
                "CREATE TABLE sessions (
                    uuid TEXT PRIMARY KEY,
                    user_id INTEGER NOT NULL
                )",
                [],
            )?;

            Ok::<_, rusqlite::Error>(())
        })
            .await
            .unwrap()?;
    }

    // debug: print users database table
    let _ = pool.get().await?.interact(|conn| {
        let mut stmt = conn.prepare("SELECT id, name, surname, password_hash, email FROM users")?;
        let users = stmt.query_map([], |row| {
            Ok(user::User {
                id: row.get(0)?,
                name: row.get(1)?,
                surname: row.get(2)?,
                password_hash: row.get(3)?,
                email: row.get(4)?,
            })
        })?;
        for user in users {
            println!("{:?}", user);
        }
        Ok::<_, rusqlite::Error>(())
    }).await?;

    let app = Router::<Pool>::new()
        .route("/health", get(routes::health::health))
        .route("/auth/register", post(routes::auth::register))
        .with_state(pool.clone())
        .route("/auth/is_logged_in", post(routes::auth::is_logged_in))
        .with_state(pool.clone())
        .route("/auth/login", post(routes::auth::login))
        .with_state(pool)
        .layer(CookieManagerLayer::new())
        // always dev mode for now
        .layer(dev_cors());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
