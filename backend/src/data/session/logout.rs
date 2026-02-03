use anyhow::{anyhow, Result};
use deadpool_sqlite::rusqlite::params;

use super::super::Database;

pub enum LogoutOutcome {
    Success,
    NotLoggedIn,
    InternalServerError,
}

impl Database {
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
