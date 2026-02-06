use anyhow::{anyhow, Result};
use deadpool_sqlite::rusqlite::{OptionalExtension, params};

use super::super::Database;
use super::super::utils;
use crate::types::NewUser;

pub enum RegisterOutcome {
    Success,
    UserAlreadyExists,
}

impl Database {
    /// Registers a new user in the `users` table.
    ///
    /// If a user with the same email already exists, no insertion is performed and
    /// [`RegisterOutcome::UserAlreadyExists`] is returned.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - a database connection cannot be acquired from the pool
    /// - a database task fails to run or complete
    /// - the required SQL queries fail to execute
    /// - password hashing fails
    pub async fn register_user(&self, user: NewUser) -> Result<RegisterOutcome> {
        let conn = self.pool.get().await?;

        let email = user.email.clone();
        let existing_user: Option<i64> = conn
            .interact(move |conn| {
                conn.query_row(
                    "SELECT id FROM users WHERE email = ?1",
                    params![email],
                    |row| row.get::<_, i64>(0),
                )
                .optional()
            })
            .await
            .map_err(|e| anyhow!("{e}"))??;

        if existing_user.is_some() {
            return Ok(RegisterOutcome::UserAlreadyExists);
        }

        let password_hash = utils::hash_password(&user.password)?;

        conn.interact(move |conn| {
            conn.execute(
                "INSERT INTO users (email, name, surname, password_hash) VALUES (?1, ?2, ?3, ?4)",
                params![user.email, user.name, user.surname, password_hash],
            )
        })
        .await
        .map_err(|e| anyhow!("{e}"))??;

        Ok(RegisterOutcome::Success)
    }
}
