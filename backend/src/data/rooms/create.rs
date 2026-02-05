use anyhow::{anyhow, Result};
use deadpool_sqlite::rusqlite::{Error, OptionalExtension, params};

use crate::types::NewRoom;
use super::super::Database;

pub enum CreateRoomOutcome {
    Success,
    NotLoggedIn,
}

impl Database {
    /// Attempts to create a new room using the logged in user's ID and a given name and description.
    pub async fn create_room(
        &self,
        session_uuid: String,
        room_data: NewRoom,
    ) -> Result<CreateRoomOutcome> {
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
            return Ok(CreateRoomOutcome::NotLoggedIn);
        };

        conn.interact(move |conn| {
            // Insert room
            conn.execute(
                "INSERT INTO rooms (owner, name, description) VALUES (?1, ?2, ?3)",
                params![user_id, &room_data.name, &room_data.description],
            )?;

            // Get generated room id
            let room_id = conn.last_insert_rowid();

            // Insert owner as room member
            conn.execute(
                "INSERT INTO room_members (room_id, user_id) VALUES (?1, ?2)",
                params![room_id, user_id],
            )?;

            Ok::<_, Error>(())
        })
        .await
        .map_err(|e| anyhow!("{e}"))??;

        Ok(CreateRoomOutcome::Success)
    }
}
