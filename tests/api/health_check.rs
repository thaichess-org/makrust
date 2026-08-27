use crate::startup::new_test_app;
use axum::http::StatusCode;

#[tokio::test]
async fn health_check() {
    let server = new_test_app().await;
    let response = server.get("/health-check").await;
    assert_eq!(response.status_code(), StatusCode::OK);
}
