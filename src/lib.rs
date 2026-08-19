pub mod database;
pub mod routes;

use sqlx::postgres::PgPool;

pub struct AppState {
    pub database: PgPool,
}
