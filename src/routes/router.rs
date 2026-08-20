use crate::routes::health_check;
use axum::{Router, routing::get};
use sqlx::postgres::PgPool;

pub fn create_router(db_pool: PgPool) -> Router {
    Router::new()
        .route("/health_check", get(health_check))
        .with_state(db_pool)
}
