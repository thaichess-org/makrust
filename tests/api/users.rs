use crate::startup::new_test_app;
use axum::http::StatusCode;

#[tokio::test]
async fn new_user_is_created_and_found() {
    let server = new_test_app().await;
    let response = server
        .post("/users")
        .form(&serde_json::json!({
            "username": "thaichess",
            "email": "thaichess@thaichess.com",
            "password": "password123",
        }))
        .await;
    assert_eq!(response.status_code(), StatusCode::OK);

    let response = server.get("/users/thaichess").await;
    assert_eq!(response.status_code(), StatusCode::OK)
}

#[tokio::test]
async fn user_does_not_exist() {
    let server = new_test_app().await;
    let response = server.get("/users/thaichess").expect_failure().await;
    assert_eq!(response.status_code(), StatusCode::NOT_FOUND)
}

#[tokio::test]
async fn data_for_new_user_is_not_valid() {
    let server = new_test_app().await;

    // invalid username
    let response = server
        .post("/users")
        .form(&serde_json::json!({
            "username": "",
            "email": "thaichess@thaichess.org",
            "password": "password123",
        }))
        .expect_failure()
        .await;
    let body: serde_json::Value = response.json();
    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "username");

    // invalid email
    let response = server
        .post("/users")
        .form(&serde_json::json!({
            "username": "thaichess",
            "email": "@thaichess.org",
            "password": "password123",
        }))
        .expect_failure()
        .await;
    let body: serde_json::Value = response.json();
    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "email");

    // invalid password
    let response = server
        .post("/users")
        .form(&serde_json::json!({
            "username": "thaichess",
            "email": "thaichess@thaichess.org",
            "password": "short",
        }))
        .expect_failure()
        .await;
    let body: serde_json::Value = response.json();
    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "password");
}
