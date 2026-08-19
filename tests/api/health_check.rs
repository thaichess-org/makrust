use axum::http::StatusCode;
use axum_test::TestServer;
use makrust::routes::create_router;

async fn new_test_app() -> TestServer {
    let app = create_router();
    TestServer::builder()
        // Preserve cookies across requests
        // for the session cookie to work.
        // .save_cookies()
        .expect_success_by_default()
        .build(app)
}

#[tokio::test]
async fn health_check() {
    let server = new_test_app().await;
    let response = server.get("/health_check").await;
    assert_eq!(response.status_code(), StatusCode::OK);
}
