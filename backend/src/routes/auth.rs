use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use bcrypt::{hash, DEFAULT_COST};
use deadpool_sqlite::Pool;
use deadpool_sqlite::rusqlite::{OptionalExtension, params};
use serde::{Deserialize, Serialize};
use tower_cookies::{Cookies, Cookie};
use uuid::Uuid;
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
enum LoggedInStatus {
    LoggedIn,
    LoggedOut,
    InternalServerError,
}

pub async fn is_logged_in(
    State(pool): State<Pool>,
    cookies: Cookies,
) -> impl IntoResponse {
    let conn = match pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("Database pool error: failed to get connection: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(LoggedInStatus::InternalServerError));
        }
    };

    let session_uuid = match cookies.get("session_uuid") {
        Some(c) => c.value().to_owned(),
        None => {
            eprintln!("debug: user does not have session_uuid cookie");
            return (StatusCode::OK, Json(LoggedInStatus::LoggedOut));
        }
    };

    let existing_session: Result<Option<i64>, _> = conn
        .interact(move |conn| {
            conn.query_row(
                "SELECT user_id FROM sessions WHERE uuid = ?1",
                params![session_uuid],
                |row| row.get::<_, i64>(0),
            )
                .optional()
        })
        .await
        .map_err(|e| eprintln!("Pool interact error (checking existing user): {e}"))
        .unwrap_or(Ok(None));

    match existing_session {
        Ok(Some(_)) => (StatusCode::OK, Json(LoggedInStatus::LoggedIn)),
        Ok(None) => (StatusCode::OK, Json(LoggedInStatus::LoggedOut)),
        Err(e) => {
            eprintln!("Database error checking existing user: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(LoggedInStatus::InternalServerError))
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
    cookies: Cookies,
    Json(user): Json<LoginRequest>,
) -> impl IntoResponse {
    let conn = match pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("Database pool error: failed to get connection: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(LoginStatus::InternalServerError));
        }
    };

    let existing_user: Result<Option<(i64, String)>, _> = conn
        .interact(move |conn| {
            conn.query_row(
                "SELECT id, password_hash FROM users WHERE email = ?1",
                params![user.email],
                |row| Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                )),
            )
                .optional()
        })
        .await
        .map_err(|e| eprintln!("Pool interact error (checking existing user): {e}"))
        .unwrap_or(Ok(None));

    match existing_user {
        Ok(Some((user_id, existing_hash))) => {
            match bcrypt::verify(&user.password, &existing_hash) {
                Ok(success) => {
                    if success {
                        // create a session ID
                        let session_uuid = Uuid::new_v4().to_string();

                        // store it in the database
                        let session_uuid_clone = session_uuid.clone();
                        if let Err(e) = conn.interact(move |conn| {
                            conn.execute(
                                "INSERT INTO sessions (uuid, user_id) VALUES (?1, ?2)",
                                params![session_uuid_clone, user_id],
                            )
                        }).await {
                            eprintln!("Failed to create session: {e}");
                            return (StatusCode::INTERNAL_SERVER_ERROR, Json(LoginStatus::InternalServerError));
                        }

                        // set the cookie
                        let c = Cookie::build(("session_uuid", session_uuid))
                            .path("/")
                            .http_only(true);
                            // we don't run server/frontend over https yet
                            //.secure(true); // only over HTTPS

                        cookies.add(c.into());

                        (StatusCode::OK, Json(LoginStatus::Success))
                    } else {
                        (StatusCode::UNAUTHORIZED, Json(LoginStatus::InvalidCredentials))
                    }
                }
                Err(e) => {
                    eprintln!("Failed to verify password: {e}");
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(LoginStatus::InternalServerError))
                }
            }
        }
        Ok(None) => {
            eprintln!("User does not exist");
            (StatusCode::NOT_ACCEPTABLE, Json(LoginStatus::UserDoesNotExist))
        }
        Err(e) => {
            eprintln!("Database error checking existing user: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(LoginStatus::InternalServerError))
        }
    }
}

#[derive(Serialize)]
enum LogoutStatus {
    Success,
    NotLoggedIn,
    InternalServerError,
}

pub async fn logout(
    State(pool): State<Pool>,
    cookies: Cookies,
) -> impl IntoResponse {
    let conn = match pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("Database pool error: failed to get connection: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(LogoutStatus::InternalServerError));
        }
    };

    let session_uuid = match cookies.get("session_uuid") {
        Some(c) => c.value().to_owned(),
        None => {
            eprintln!("debug: user does not have session_uuid cookie");
            return (StatusCode::OK, Json(LogoutStatus::NotLoggedIn));
        }
    };

    let session_uuid_clone = session_uuid.clone();
    let existing_session: Result<Option<i64>, _> = conn
        .interact(move |conn| {
            conn.query_row(
                "SELECT user_id FROM sessions WHERE uuid = ?1",
                params![session_uuid_clone],
                |row| row.get::<_, i64>(0),
            )
                .optional()
        })
        .await
        .map_err(|e| eprintln!("Pool interact error (checking existing user): {e}"))
        .unwrap_or(Ok(None));

    match existing_session {
        Ok(Some(_user_id)) => {
            let delete_result = conn
                .interact(move |conn| {
                    conn.execute(
                        "DELETE FROM sessions WHERE uuid = ?1",
                        params![session_uuid],
                    )
                })
            .await;

            match delete_result {
                Ok(Ok(_)) => (
                    StatusCode::OK,
                    Json(LogoutStatus::Success),
                ),

                Ok(Err(e)) => {
                    eprintln!("SQLite error deleting session: {e}");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(LogoutStatus::InternalServerError),
                    )
                }

                Err(e) => {
                    eprintln!("Interact error deleting session: {e}");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(LogoutStatus::InternalServerError),
                    )
                }
            }
        }

        Ok(None) => (
            StatusCode::OK,
            Json(LogoutStatus::NotLoggedIn),
        ),

        Err(e) => {
            eprintln!("Database error checking existing user: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(LogoutStatus::InternalServerError),
            )
        }
    }
}

fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
    hash(password, DEFAULT_COST)
}
