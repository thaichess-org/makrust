use crate::routes::{health_check, users};
use axum::{
    Router,
    routing::{get, post},
};
use sqlx::postgres::PgPool;

pub fn create_router(db_pool: PgPool) -> Router {
    Router::new()
        .route("/health_check", get(health_check))
        .route("/users/{username}", get(users::user))
        .route("/users", post(users::create_user))
        .with_state(db_pool)
}
