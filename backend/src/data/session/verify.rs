use anyhow::{anyhow, Result};
use deadpool_sqlite::rusqlite::{OptionalExtension, params};

use super::super::Database;

impl Database {
    /// Checks whether a session with the given UUID exists.
    ///
    /// Returns `true` if the session exists, `false` otherwise.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - a database connection cannot be acquired from the pool
    /// - a database task fails to run or complete
    /// - executing the SQL query fails
    pub async fn verify_session(&self, session_uuid: String) -> Result<bool> {
        let conn = self.pool.get().await?;

        let exists: Option<i64> = conn
            .interact(move |conn| {
                conn.query_row(
                    "SELECT user_id FROM sessions WHERE uuid = ?1",
                    params![session_uuid],
                    |row| row.get(0),
                )
                .optional()
            })
            .await
            .map_err(|e| anyhow!("{e}"))??;

        Ok(exists.is_some())
    }
}
