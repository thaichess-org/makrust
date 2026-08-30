use crate::configuration::{AuthSettings, EmailSettings};
use crate::routes::{email, health_check, users};
use axum::{
    Router,
    extract::FromRef,
    http::{HeaderValue, Method, header::CONTENT_TYPE},
    routing::{get, post},
};
use sqlx::postgres::PgPool;
use tower_http::cors::CorsLayer;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub auth: AuthSettings,
    pub email: EmailSettings,
}

// Lets handlers that only need the state's db_pool keep
// working even if more things are added to AppState.
impl FromRef<AppState> for PgPool {
    fn from_ref(state: &AppState) -> Self {
        state.db_pool.clone()
    }
}

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(
            state
                .auth
                .frontend_origin
                .parse::<HeaderValue>()
                .expect("AuthSettings.frontend_origin must be a valid header value"),
        )
        // lets the browser attach the session cookie to cross-origin requests
        .allow_credentials(true)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([CONTENT_TYPE]);

    Router::new()
        .route("/health-check", get(health_check))
        // * remove later, this just for testing.
        .route("/email/{recipient}", get(email::send))
        // * this route is only to test the auth functionality, delete later
        .route("/users/me", get(users::me))
        .route("/users/{username}", get(users::user))
        .route("/users", post(users::create_user))
        .route("/sign-in", post(users::sign_in))
        .route("/sign-out", post(users::sign_out))
        .layer(cors)
        .with_state(state)
}
