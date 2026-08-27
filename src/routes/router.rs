use crate::configuration::AuthSettings;
use crate::routes::{health_check, users};
use axum::{
    Router,
    extract::FromRef,
    routing::{get, post},
};
use sqlx::postgres::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub auth: AuthSettings,
}

// Lets handlers that only need the state's db_pool keep
// working even if more things are added to AppState.
impl FromRef<AppState> for PgPool {
    fn from_ref(state: &AppState) -> Self {
        state.db_pool.clone()
    }
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health-check", get(health_check))
        // this route is only to test the auth functionality, delete later
        .route("/users/me", get(users::me))
        .route("/users/{username}", get(users::user))
        .route("/users", post(users::create_user))
        .route("/sign-in", post(users::sign_in))
        .with_state(state)
}
