use makrust::database::get_pool;
use makrust::routes::create_router;
use makrust::{AppState, configuration::get_configuration};

// TODO: clean up this function
#[tokio::main]
async fn main() {
    let configuration = get_configuration().expect("Failed to load configuration.");
    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let pool = get_pool().await;
    let app = create_router(AppState {
        database: pool.clone(),
    });
    let listener = tokio::net::TcpListener::bind(&address).await.unwrap();
    println!("Server started successfully at {}.", &address);
    axum::serve(listener, app).await.unwrap();
}
