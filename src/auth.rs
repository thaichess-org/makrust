use crate::domain::{Session, is_last_seen_stale};
use crate::routes::AppState;
use axum::extract::FromRequestParts;
use axum::http::{StatusCode, request::Parts};
use axum_extra::extract::cookie::CookieJar;
use chrono::{DateTime, Utc};
use sqlx::types::Uuid;

pub const SESSION_COOKIE_NAME: &str = "session_id";
pub const IP_ADDRESS_HEADER: &str = "x-forwarded-for";

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub id: Uuid,
    pub username: String,
}

struct SessionRow {
    session_id: Uuid,
    user_id: Uuid,
    session_created_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    revoked_at: Option<DateTime<Utc>>,
    username: String,
    last_seen_at: Option<DateTime<Utc>>,
}

/// check header to see if request has the session_id
impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_headers(&parts.headers);

        // if session cookie is missing or malformed, return error
        let session_id = jar
            .get(SESSION_COOKIE_NAME)
            .and_then(|cookie| Uuid::parse_str(cookie.value()).ok())
            .ok_or(StatusCode::UNAUTHORIZED)?;

        let row = sqlx::query_as!(
            SessionRow,
            r#"SELECT s.id as session_id, s.user_id, s.created_at as session_created_at,
                      s.expires_at, s.revoked_at, u.username, u.last_seen_at
               FROM sessions s
               JOIN users u ON u.id = s.user_id
               WHERE s.id = $1"#,
            session_id
        )
        .fetch_optional(&state.db_pool)
        .await
        // TODO: I'm eating this error here, better to log it later
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

        let session = Session {
            id: row.session_id,
            user_id: row.user_id,
            created_at: row.session_created_at,
            expires_at: row.expires_at,
            revoked_at: row.revoked_at,
        };

        // check if it's expired or revoked
        session
            .validate(Utc::now())
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

        if is_last_seen_stale(row.last_seen_at) {
            // TODO: I'm ignoring the error for now, add to logs later
            let _ = sqlx::query!(
                "UPDATE users SET last_seen_at = now() WHERE id = $1",
                row.user_id
            )
            .execute(&state.db_pool)
            .await;
        }

        // ok, everything's good, user is authenticated
        Ok(AuthenticatedUser {
            id: row.user_id,
            username: row.username,
        })
    }
}
