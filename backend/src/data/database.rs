use anyhow::{Result, anyhow};
use deadpool_sqlite::{Config, Pool, Runtime};
use deadpool_sqlite::rusqlite::{self, params};

#[derive(Clone)]
pub struct Database {
    pub pool: Pool,
}

impl Database {
    pub async fn new() -> Result<Self> {
        let cfg = Config::new("db.sqlite3");
        let pool = cfg.create_pool(Runtime::Tokio1)?;

        {
            let conn = pool.get().await?;
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
                .map_err(|e| anyhow!("{e}"))??;
        }

        Ok(Self { pool })
    }
}
