use crate::auth::{AuthenticatedUser, IP_ADDRESS_HEADER, SESSION_COOKIE_NAME};
use crate::domain::{NewUser, NewUserError, Password, PasswordError};
use crate::routes::AppState;
use axum::Json;
use axum::extract::{Form, Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use sqlx;
use sqlx::postgres::PgPool;
use sqlx::types::Uuid;
use sqlx::types::ipnetwork::IpNetwork;
use time::Duration as CookieDuration;

#[derive(Debug, serde::Serialize)]
pub struct UserRecord {
    #[serde(skip_serializing)]
    id: sqlx::types::Uuid,
    username: String,
    #[serde(skip_serializing)]
    password_hash: String,
    display_name: Option<String>,
    bio: Option<String>,
    country_code: Option<String>,
    role: String,
    is_active: bool,
    created_at: DateTime<Utc>,
    last_seen_at: Option<DateTime<Utc>>,
}

#[derive(serde::Deserialize)]
pub(crate) struct SignIn {
    username: String,
    password: String,
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
    let user = fetch_user(&db_pool, &username).await;

    match user {
        Ok(user) => Ok(Json(user)),
        Err(sqlx::Error::RowNotFound) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn me(
    State(db_pool): State<PgPool>,
    user: AuthenticatedUser,
) -> Result<Json<UserRecord>, StatusCode> {
    match fetch_user(&db_pool, &user.username).await {
        Ok(user_record) => Ok(Json(user_record)),
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
        Err(NewUserError::PasswordError(PasswordError::InvalidPassword)) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "password",
                    message: "Password must be at least 8 characters long, and no longer than 64. No space allowed.",
                }),
            ));
        }
        // TODO: I'm eating this error here, better to log it later
        Err(NewUserError::PasswordError(PasswordError::HashingError(_))) => {
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
        RETURNING id, username, password_hash, display_name, bio, country_code, role, is_active, created_at, last_seen_at"#,
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

struct NewSession {
    id: sqlx::types::Uuid,
    #[allow(dead_code)]
    expires_at: DateTime<Utc>,
}

pub async fn sign_in(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    jar: CookieJar,
    Form(sign_in): Form<SignIn>,
) -> Result<(CookieJar, Json<UserRecord>), StatusCode> {
    let db_pool = &app_state.db_pool;

    let user = match fetch_user(db_pool, &sign_in.username).await {
        Ok(user) => user,
        Err(sqlx::Error::RowNotFound) => return Err(StatusCode::UNAUTHORIZED),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    // check that passwords match, create user session if they do.
    match Password::verify(&sign_in.password, &user.password_hash) {
        Ok(_) => (),
        Err(PasswordError::InvalidPassword) => return Err(StatusCode::UNAUTHORIZED),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    }

    let user_agent = headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);

    let ip_address: Option<IpNetwork> = headers
        .get(IP_ADDRESS_HEADER)
        // `X-Forwarded-For` can be a comma-separated list of proxy hops;
        // the first entry is the original client.
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .and_then(|ip| ip.trim().parse().ok());

    let expires_at = Utc::now() + ChronoDuration::days(app_state.auth.session_ttl_days);

    // the session.id will be the user's session token
    let new_session = sqlx::query_as!(
        NewSession,
        r#"INSERT INTO sessions (user_id, expires_at, user_agent, ip_address)
           VALUES ($1, $2, $3, $4)
           RETURNING id, expires_at"#,
        user.id,
        expires_at,
        user_agent,
        ip_address,
    )
    .fetch_one(db_pool)
    .await;

    let new_session = match new_session {
        Ok(session) => session,
        // TODO: I'm eating this error here, better to log it later
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let cookie = Cookie::build((SESSION_COOKIE_NAME, new_session.id.to_string()))
        .http_only(true)
        .secure(app_state.auth.cookie_secure)
        // `Lax` sends the cookie on top-level navigations (e.g. clicking a
        // link to the site) but not on cross-site requests triggered by
        // other sites (e.g. an `<img>` tag on someone else's page), which
        // gives us reasonable CSRF protection without breaking normal use.
        .same_site(SameSite::Lax)
        // Send this cookie on every path on the site, not just `/sign-in`.
        .path("/")
        .max_age(CookieDuration::days(app_state.auth.session_ttl_days))
        .build();

    // Returning a tuple of (CookieJar, Json<UserRecord>) tells axum to
    // apply the cookie jar's Set-Cookie header(s) to the response, then
    // use the Json<UserRecord> as the response body.
    Ok((jar.add(cookie), Json(user)))
}

/// revokes user session and removes session cookie from headers
pub async fn sign_out(
    State(app_state): State<AppState>,
    jar: CookieJar,
) -> Result<(CookieJar, StatusCode), StatusCode> {
    let session_id = jar
        .get(SESSION_COOKIE_NAME)
        .and_then(|cookie| Uuid::parse_str(cookie.value()).ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    sqlx::query!(
        "UPDATE sessions SET revoked_at = now() WHERE id = $1",
        session_id
    )
    .execute(&app_state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let cleared = Cookie::build(SESSION_COOKIE_NAME).path("/").build();

    Ok((jar.remove(cleared), StatusCode::NO_CONTENT))
}

pub async fn fetch_user(db_pool: &PgPool, username: &String) -> Result<UserRecord, sqlx::Error> {
    sqlx::query_as!(
        UserRecord,
        r#"SELECT id, username, password_hash, display_name, bio, country_code, role, is_active, created_at, last_seen_at
        FROM users WHERE username = $1"#,
        username
    )
    .fetch_one(db_pool)
    .await
}
