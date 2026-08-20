use makrust::AppState;
use makrust::database::get_pool;
use makrust::routes::create_router;

#[tokio::main]
async fn main() {
    let pool = get_pool().await;
    let app = create_router(AppState {
        database: pool.clone(),
    });
    // let app = create_router();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    println!("Server started successfully at 0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}
