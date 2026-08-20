use crate::AppState;
use crate::routes::health_check;
use axum::{Router, routing::get};

pub fn create_router(app_state: AppState) -> Router {
    Router::new()
        .route("/health_check", get(health_check))
        .with_state(app_state)
}
