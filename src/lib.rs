pub mod database;
pub mod routes;

use sqlx::postgres::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub database: PgPool,
}
