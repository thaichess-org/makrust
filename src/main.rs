// use makrust::AppState;
// use makrust::database::get_pool;
use makrust::routes::create_router;
// use std::sync::Arc;

#[tokio::main]
async fn main() {
    // let pool = get_pool().await;
    // let app = create_router(Arc::new(AppState {
    //     database: pool.clone(),
    // }));
    let app = create_router();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("Server started successfully at 127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}
