use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use bcrypt::{hash, DEFAULT_COST};
use deadpool_sqlite::Pool;
use deadpool_sqlite::rusqlite::{OptionalExtension, params};
use serde::Serialize;
use crate::user::NewUser;

#[derive(Serialize)]
pub enum RegisterStatus {
    Success,
    UserAlreadyExists,
    InternalServerError,
}

pub async fn register(
    State(pool): State<Pool>,
    Json(user): Json<NewUser>,
) -> impl IntoResponse {
    let conn = match pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("Database pool error: failed to get connection: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(RegisterStatus::InternalServerError));
        }
    };

    let email = user.email.clone();
    let existing_user: Result<Option<i64>, _> = conn
        .interact(move |conn| {
            conn.query_row(
                "SELECT id FROM users WHERE email = ?1",
                params![email],
                |row| row.get(0),
            )
            .optional()
        })
        .await
        .map_err(|e| eprintln!("Pool interact error (checking existing user): {e}"))
        .unwrap_or(Ok(None));

    match existing_user {
        Ok(Some(_)) => {
            eprintln!("Register failed: user with email {} already exists", user.email);
            (StatusCode::CONFLICT, Json(RegisterStatus::UserAlreadyExists))
        }
        Ok(None) => {
            let password_hash = match hash(&user.password, DEFAULT_COST) {
                Ok(h) => h,
                Err(e) => {
                    eprintln!("Password hashing failed: {e}");
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(RegisterStatus::InternalServerError));
                }
            };

            let email = user.email.clone();
            let name = user.name.clone();
            let surname = user.surname.clone();

            let insert_result = conn
                .interact(move |conn| {
                    conn.execute(
                        "INSERT INTO users (email, name, surname, password_hash) VALUES (?1, ?2, ?3, ?4)",
                        params![email, name, surname, password_hash],
                    )
                })
                .await;

            match insert_result {
                Ok(Ok(_)) => (StatusCode::OK, Json(RegisterStatus::Success)),
                Ok(Err(e)) => {
                    eprintln!("Database error during insert: {e}");
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(RegisterStatus::InternalServerError))
                }
                Err(e) => {
                    eprintln!("Pool interact error during insert: {e}");
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(RegisterStatus::InternalServerError))
                }
            }
        }
        Err(e) => {
            eprintln!("Database error checking existing user: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(RegisterStatus::InternalServerError))
        }
    }
}
