use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use deadpool_sqlite::rusqlite::{OptionalExtension, params};
use serde::{Deserialize, Serialize};
use tower_cookies::{Cookies, Cookie};
use crate::data::{Database, LoginOutcome, RegisterOutcome};
use crate::user::NewUser;

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
    State(db): State<Database>,
    cookies: Cookies,
    Json(user): Json<LoginRequest>,
) -> impl IntoResponse {
    match db.login_user(user.email, user.password).await {
        Ok(LoginOutcome::Success(session_uuid)) => {
            // set the cookie
            let c = Cookie::build(("session_uuid", session_uuid))
                .path("/")
                .http_only(true);
                // we don't run server/frontend over https yet
                //.secure(true); // only over HTTPS

            cookies.add(c.into());

            (StatusCode::OK, Json(LoginStatus::Success))
        }
        Ok(LoginOutcome::UserDoesNotExist) => (StatusCode::NOT_ACCEPTABLE, Json(LoginStatus::UserDoesNotExist)),
        Ok(LoginOutcome::InvalidCredentials) => (StatusCode::UNAUTHORIZED, Json(LoginStatus::InvalidCredentials)),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(LoginStatus::InternalServerError)),
    }
}

#[derive(Serialize)]
pub enum RegisterStatus {
    Success,
    UserAlreadyExists,
    InternalServerError,
}

pub async fn register(
    State(db): State<Database>,
    Json(user): Json<NewUser>,
) -> impl IntoResponse {
    match db.register_user(user).await {
        Ok(RegisterOutcome::Success) => (StatusCode::OK, Json(RegisterStatus::Success)),
        Ok(RegisterOutcome::UserAlreadyExists) => (StatusCode::CONFLICT, Json(RegisterStatus::UserAlreadyExists)),
        Err(e) => {
            eprintln!("Register error: {e}");
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
    State(db): State<Database>,
    cookies: Cookies,
) -> impl IntoResponse {
    let conn = match db.pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("Database db.pool error: failed to get connection: {e}");
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
enum LogoutStatus {
    Success,
    NotLoggedIn,
    InternalServerError,
}

pub async fn logout(
    State(db): State<Database>,
    cookies: Cookies,
) -> impl IntoResponse {
    let conn = match db.pool.get().await {
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
