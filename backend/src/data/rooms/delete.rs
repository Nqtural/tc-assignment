use anyhow::{anyhow, Result};
use deadpool_sqlite::rusqlite::{OptionalExtension, params};

use super::super::Database;

pub enum DeleteRoomOutcome {
    Success,
    NotLoggedIn,
    NotOwner,
}

impl Database {
    /// Attempts to delete a room using the logged in user's ID and a given room ID.
    pub async fn delete_room(
        &self,
        session_uuid: String,
        id: i32,
    ) -> Result<DeleteRoomOutcome> {
        let conn = self.pool.get().await?;

        let user_id: Option<i64> = conn
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

        let Some(user_id) = user_id else {
            return Ok(DeleteRoomOutcome::NotLoggedIn);
        };

        let affected = conn
            .interact(move |conn| {
                conn.execute(
                    "DELETE FROM rooms WHERE id = ?1 AND owner = ?2",
                    params![id, user_id],
                )
            })
            .await
            .map_err(|e| anyhow!("{e}"))??;

        if affected == 0 {
            return Ok(DeleteRoomOutcome::NotOwner);
        }

        Ok(DeleteRoomOutcome::Success)
    }
}
