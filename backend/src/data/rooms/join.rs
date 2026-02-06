use anyhow::{anyhow, Result};
use deadpool_sqlite::rusqlite::{OptionalExtension, params};

use super::super::Database;

pub enum JoinRoomOutcome {
    Success,
    AlreadyMember,
    InvalidCode,
    NotLoggedIn,
}

impl Database {
    /// Attempts to join a room using the logged-in user's ID and a given room invitation code.
    pub async fn join_room(
        &self,
        session_uuid: String,
        code: String,
    ) -> Result<JoinRoomOutcome> {
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
            return Ok(JoinRoomOutcome::NotLoggedIn);
        };

        let room_id: Option<i32> = conn
            .interact(move |conn| {
                conn.query_row(
                    "SELECT room_id FROM invitation_codes WHERE code = ?1",
                    params![code],
                    |row| row.get(0),
                )
                .optional()
            })
            .await
            .map_err(|e| anyhow!("{e}"))??;

        let Some(room_id) = room_id else {
            return Ok(JoinRoomOutcome::InvalidCode);
        };

        let already_member: bool = conn
            .interact(move |conn| {
                conn.query_row(
                    "SELECT EXISTS(SELECT 1 FROM room_members WHERE room_id = ?1 AND user_id = ?2)",
                    params![room_id, user_id],
                    |row| row.get(0),
                )
            })
            .await
            .map_err(|e| anyhow!("{e}"))??;

        if already_member {
            return Ok(JoinRoomOutcome::AlreadyMember);
        }

        conn.interact(move |conn| {
            conn.execute(
                "INSERT INTO room_members (room_id, user_id) VALUES (?1, ?2)",
                params![room_id, user_id],
            )
        })
        .await
        .map_err(|e| anyhow!("{e}"))??;

        Ok(JoinRoomOutcome::Success)
    }
}
