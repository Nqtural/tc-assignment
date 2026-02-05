use anyhow::{anyhow, Result};
use deadpool_sqlite::rusqlite::{OptionalExtension, params};

use super::super::Database;

pub enum LeaveRoomOutcome {
    Success,
    NotLoggedIn,
    NotMember,
    OwnerCannotLeave,
}

impl Database {
    /// Attempts to leave a room using the logged in user's ID and a given room ID.
    pub async fn leave_room(
        &self,
        session_uuid: String,
        room_id: i32,
    ) -> Result<LeaveRoomOutcome> {
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
            return Ok(LeaveRoomOutcome::NotLoggedIn);
        };

        let is_owner: bool = conn
            .interact(move |conn| {
                conn.query_row(
                    "SELECT EXISTS(SELECT 1 FROM rooms WHERE id = ?1 AND owner = ?2)",
                    params![room_id, user_id],
                    |row| row.get(0),
                )
            })
            .await
            .map_err(|e| anyhow!("{e}"))??;

        if is_owner {
            return Ok(LeaveRoomOutcome::OwnerCannotLeave);
        }

        let affected = conn
            .interact(move |conn| {
                conn.execute(
                    "DELETE FROM room_members WHERE room_id = ?1 AND user_id = ?2",
                    params![room_id, user_id],
                )
            })
            .await
            .map_err(|e| anyhow!("{e}"))??;

        if affected == 0 {
            return Ok(LeaveRoomOutcome::NotMember);
        }

        Ok(LeaveRoomOutcome::Success)
    }
}
