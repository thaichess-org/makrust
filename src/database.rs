use dotenv::dotenv;
use sqlx::migrate::MigrateError;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

const CONNECT_MAX_ATTEMPTS: u32 = 3;
const CONNECT_BASE_DELAY: Duration = Duration::from_millis(500);
const CONNECT_MAX_DELAY: Duration = Duration::from_secs(5);

pub async fn get_pool() -> PgPool {
    dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let mut attempt = 1;
    loop {
        match PgPoolOptions::new()
            .max_connections(10)
            // any http call timeout should be longer than this
            .acquire_timeout(std::time::Duration::from_secs(3))
            .connect(&db_url)
            .await
        {
            Ok(pool) => {
                println!("Connected to database");
                return pool;
            }
            Err(error) if attempt < CONNECT_MAX_ATTEMPTS => {
                let delay = CONNECT_BASE_DELAY
                    .saturating_mul(1 << (attempt - 1))
                    .min(CONNECT_MAX_DELAY);
                println!(
                    "Failed to connect to database (attempt {}/{}): {}. Retrying in {:?}...",
                    attempt, CONNECT_MAX_ATTEMPTS, error, delay
                );
                tokio::time::sleep(delay).await;
                attempt += 1;
            }
            Err(error) => {
                println!(
                    "Failed to connect to database after {} attempts: {}",
                    attempt, error
                );
                std::process::exit(1);
            }
        }
    }
}

/// Applies pending migrations. Intended to be run as a separate deploy step
/// (see `src/bin/migrate.rs`), not on every app boot, so that rolling out
/// multiple app replicas never races to apply the same migration.
pub async fn run_migrations(pool: &PgPool) -> Result<(), MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}
