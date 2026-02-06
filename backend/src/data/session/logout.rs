use anyhow::{anyhow, Result};
use deadpool_sqlite::rusqlite::params;

use super::super::Database;

pub enum LogoutOutcome {
    Success,
    NotLoggedIn,
    InternalServerError,
}

impl Database {
    /// Logs out a user by deleting their session from the database.
    ///
    /// If a session with the given UUID exists, it is removed. Otherwise, no action is taken.
    ///
    /// # Returns
    ///
    /// - `Ok(LogoutOutcome::Success)` if the session was found and deleted.
    /// - `Ok(LogoutOutcome::NotLoggedIn)` if no session exists with the given UUID.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - a database connection cannot be acquired from the pool
    /// - a database task fails to run or complete
    /// - executing the SQL query fails
    pub async fn logout_user(&self, session_uuid: String) -> Result<LogoutOutcome> {
        let conn = self.pool.get().await?;

        let deleted = conn
            .interact(move |conn| {
                conn.execute(
                    "DELETE FROM sessions WHERE uuid = ?1",
                    params![session_uuid],
                )
            })
            .await
            .map_err(|e| anyhow!("{e}"))??;

        if deleted > 0 {
            Ok(LogoutOutcome::Success)
        } else {
            Ok(LogoutOutcome::NotLoggedIn)
        }
    }
}
