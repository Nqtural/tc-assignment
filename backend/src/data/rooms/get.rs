use anyhow::{anyhow, Result};
use deadpool_sqlite::rusqlite::{OptionalExtension, params};

use crate::types::Room;
use super::super::Database;

pub enum GetRoomOutcome {
    Success(Vec<Room>),
    NotLoggedIn,
}

impl Database {
    /// Attempts to get rooms current user is a member of using the logged in user's ID.
    pub async fn get_rooms(&self, session_uuid: String) -> Result<GetRoomOutcome> {
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
            return Ok(GetRoomOutcome::NotLoggedIn);
        };

        let rooms = conn
            .interact(move |conn| -> Result<Vec<Room>> {
                let mut stmt = conn.prepare(
                    "
                    SELECT r.name, r.description
                    FROM rooms r
                    JOIN room_members rm ON rm.room_id = r.id
                    WHERE rm.user_id = ?1
                    ",
                )?;

                let rooms_iter = stmt.query_map(params![user_id], |row| {
                    Ok(Room {
                        name: row.get(0)?,
                        description: row.get(1)?,
                    })
                })?;

                let mut rooms = Vec::new();
                for room in rooms_iter {
                    rooms.push(room?);
                }

                Ok(rooms)
            })
            .await
            .map_err(|e| anyhow!("{e}"))??;

        Ok(GetRoomOutcome::Success(rooms))
    }
}
