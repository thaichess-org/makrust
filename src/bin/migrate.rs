use makrust::database::{get_pool, run_migrations};

#[tokio::main]
async fn main() {
    let pool = get_pool().await;

    println!("Running migrations...");
    if let Err(error) = run_migrations(&pool).await {
        println!("Failed to run migrations: {}", error);
        std::process::exit(1);
    }
    println!("Migrations complete");
}
