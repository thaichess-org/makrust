use crate::domain::{NewUser, NewUserError};
use axum::Json;
use axum::extract::{Form, Path, State};
use axum::http::StatusCode;
use chrono::{DateTime, Utc};
use sqlx;
use sqlx::postgres::PgPool;

#[derive(Debug, serde::Serialize)]
pub struct UserRecord {
    username: String,
    display_name: Option<String>,
    bio: Option<String>,
    country_code: Option<String>,
    role: String,
    is_active: bool,
    created_at: DateTime<Utc>,
    last_seen_at: Option<DateTime<Utc>>,
}

#[derive(serde::Deserialize)]
pub(crate) struct SignUp {
    username: String,
    email: String,
    password: String,
}

#[derive(Debug, serde::Serialize)]
pub struct ErrorResponse {
    error: &'static str,
    message: &'static str,
}

pub async fn user(
    State(db_pool): State<PgPool>,
    Path(username): Path<String>,
) -> Result<Json<UserRecord>, StatusCode> {
    let user = sqlx::query_as!(
        UserRecord,
        r#"SELECT username, display_name, bio, country_code, role, is_active, created_at, last_seen_at
        FROM users WHERE username = $1"#,
        username
    )
    .fetch_one(&db_pool)
    .await;

    match user {
        Ok(user) => Ok(Json(user)),
        Err(sqlx::Error::RowNotFound) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn create_user(
    State(db_pool): State<PgPool>,
    Form(sign_up): Form<SignUp>,
) -> Result<Json<UserRecord>, (StatusCode, Json<ErrorResponse>)> {
    let new_user = match NewUser::new(sign_up.username, sign_up.email, sign_up.password) {
        Ok(new_user) => new_user,
        // TODO: look into using IntoResponse to simplify this or a helper function
        Err(NewUserError::UsernameParsingError) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "username",
                    message: "Please only use letters, numbers, and a max length of 50 characters.",
                }),
            ));
        }
        Err(NewUserError::EmailParsingError) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "email",
                    message: "Please use a valid email.",
                }),
            ));
        }
        Err(NewUserError::PasswordError) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "password",
                    message: "Password must be at least 8 characters long, and no longer than 64. No space allowed.",
                }),
            ));
        }
        // TODO: I'm eating this error here, better to log it later
        Err(NewUserError::HashingError(_)) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "server",
                    message: "Application error occurred. Please try again",
                }),
            ));
        }
    };

    let registered_user = sqlx::query_as!(
        UserRecord,
        r#"INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3)
        RETURNING username, display_name, bio, country_code, role, is_active, created_at, last_seen_at"#,
        new_user.username.as_ref(),
        new_user.email.as_ref(),
        new_user.password_hash,
    )
    .fetch_one(&db_pool)
    .await;

    match registered_user {
        Ok(user) => Ok(Json(user)),
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                error: "conflict",
                message: "Username or email is already taken.",
            }),
        )),
        // TODO: I'm eating this error here, better to log it later
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "server",
                message: "Application error occurred. Please try again",
            }),
        )),
    }
}
