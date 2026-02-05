use anyhow::{anyhow, Result};
use deadpool_sqlite::rusqlite::{OptionalExtension, params};

use super::super::Database;
use super::super::utils;

pub enum InvitationCodeOutcome {
    Success(String),
    NotLoggedIn,
    NotOwner,
}

impl Database {
    /// Attempts to get or create an invitation code for a room using the logged-in user's ID and a given room ID.
    pub async fn get_invitation_code(
        &self,
        session_uuid: String,
        room_id: i32,
    ) -> Result<InvitationCodeOutcome> {
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
            return Ok(InvitationCodeOutcome::NotLoggedIn);
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

        if !is_owner {
            return Ok(InvitationCodeOutcome::NotOwner);
        }

        if let Some(code) = conn
            .interact(move |conn| {
                conn.query_row(
                    "SELECT key FROM invitation_keys WHERE room_id = ?1",
                    params![room_id],
                    |row| row.get(0),
                )
                .optional()
            })
            .await
            .map_err(|e| anyhow!("{e}"))?? 
        {
            return Ok(InvitationCodeOutcome::Success(code));
        }

        let code: String = loop {
            let candidate = utils::generate_invitation_code();

            let candidate_clone = candidate.clone();
            let exists: bool = conn
                .interact(move |conn| {
                    conn.query_row(
                        "SELECT EXISTS(SELECT 1 FROM invitation_keys WHERE key = ?1)",
                        params![candidate_clone],
                        |row| row.get(0),
                    )
                })
                .await
                .map_err(|e| anyhow!("{e}"))??;

            if !exists {
                break candidate;
            }
        };

        let code_clone = code.clone();
        conn.interact(move |conn| {
            conn.execute(
                "INSERT INTO invitation_keys (room_id, key) VALUES (?1, ?2)",
                params![room_id, code_clone],
            )
        })
        .await
        .map_err(|e| anyhow!("{e}"))??;

        Ok(InvitationCodeOutcome::Success(code))
    }
}
