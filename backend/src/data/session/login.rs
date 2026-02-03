use anyhow::{anyhow, Result};
use bcrypt::verify;
use deadpool_sqlite::rusqlite::{OptionalExtension, params};
use uuid::Uuid;

use super::super::Database;

pub enum LoginOutcome {
    Success(String),
    InvalidCredentials,
    UserDoesNotExist,
}

impl Database {
     pub async fn login_user(&self, email: String, password: String) -> Result<LoginOutcome> {
        let conn = self.pool.get().await?;

        let user: Option<(i64, String)> = conn
            .interact(move |conn| {
                conn.query_row(
                    "SELECT id, password_hash FROM users WHERE email = ?1",
                    params![email],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .optional()
            })
            .await
            .map_err(|e| anyhow!("{e}"))??;

        match user {
            Some((user_id, hash)) => {
                if verify(&password, &hash)? {
                    let session_uuid = Uuid::new_v4().to_string();

                    let session_uuid_clone = session_uuid.clone();
                    conn.interact(move |conn| {
                        conn.execute(
                            "INSERT INTO sessions (uuid, user_id) VALUES (?1, ?2)",
                            params![session_uuid_clone, user_id],
                        )
                    })
                    .await
                    .map_err(|e| anyhow!("{e}"))??;

                    Ok(LoginOutcome::Success(session_uuid))
                } else {
                    Ok(LoginOutcome::InvalidCredentials)
                }
            }
            None => Ok(LoginOutcome::UserDoesNotExist),
        }
    }
}
