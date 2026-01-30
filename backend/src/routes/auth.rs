use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use bcrypt::{hash, DEFAULT_COST};
use deadpool_sqlite::Pool;
use deadpool_sqlite::rusqlite::{OptionalExtension, params};
use serde::{Deserialize, Serialize};
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
            println!("debug: hashing {}", &user.password);
            let password_hash = match hash_password(&user.password) {
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

#[derive(Serialize)]
enum LoginStatus {
    Success,
    UserDoesNotExist,
    InvalidCredentials,
    InternalServerError,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

pub async fn login(
    State(pool): State<Pool>,
    Json(user): Json<LoginRequest>,
) -> impl IntoResponse {
    let conn = match pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("Database pool error: failed to get connection: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(LoginStatus::InternalServerError));
        }
    };

    let existing_user_password_hash: Result<Option<String>, _> = conn
        .interact(move |conn| {
            conn.query_row(
                "SELECT password_hash FROM users WHERE email = ?1",
                params![user.email],
                |row| row.get::<_, String>(0),
            )
                .optional()
        })
        .await
        .map_err(|e| eprintln!("Pool interact error (checking existing user): {e}"))
        .unwrap_or(Ok(None));

    match existing_user_password_hash {
        Ok(Some(existing_hash)) => {
            match bcrypt::verify(&user.password, &existing_hash) {
                Ok(success) => {
                    if success {
                        (StatusCode::OK, Json(LoginStatus::Success))
                    } else {
                        eprintln!("{} does not verify with {}", &user.password, existing_hash);
                        (StatusCode::UNAUTHORIZED, Json(LoginStatus::InvalidCredentials))
                    }
                },
                Err(e) => {
                    eprintln!("failed to verify password: {e}");
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(LoginStatus::InternalServerError))
                }
            }
        }
        Ok(None) => {
            eprintln!("Password hashing failed: no password found for user");
            (StatusCode::NOT_FOUND, Json(LoginStatus::UserDoesNotExist))
        }
        Err(e) => {
            eprintln!("Database error checking existing user: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(LoginStatus::InternalServerError))
        }
    }
}

fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
    hash(password, DEFAULT_COST)
}
