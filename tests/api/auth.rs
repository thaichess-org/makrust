use crate::startup::new_test_app;
use axum::http::StatusCode;
use cookie::{Cookie, SameSite};

// signs a user up, then signs them in, returning the
// session_id cookie.
async fn sign_up_and_sign_in(server: &axum_test::TestServer, username: &str) -> Cookie<'static> {
    server
        .post("/users")
        .form(&serde_json::json!({
            "username": username,
            "email": format!("{username}@example.com"),
            "password": "password123",
        }))
        .await;

    let response = server
        .post("/sign-in")
        .form(&serde_json::json!({
            "username": username,
            "password": "password123",
        }))
        .await;

    response.cookie("session_id")
}

#[tokio::test]
async fn sign_in_sets_httponly_lax_session_cookie() {
    let app = new_test_app().await;
    sign_up_and_sign_in(&app.server, "thaiChessMaster").await;

    // should have the session cookie
    let response = app
        .server
        .post("/sign-in")
        .form(&serde_json::json!({
            "username": "thaiChessMaster",
            "password": "password123",
        }))
        .await;
    response.assert_contains_cookie("session_id");

    // Check the cookie was built with the security attributes we expect: HttpOnly (JavaScript can't read it) SameSite=Lax (limits cross-site sending).
    let cookie: Cookie = response.cookie("session_id");
    assert_eq!(cookie.http_only(), Some(true));
    assert_eq!(cookie.same_site(), Some(SameSite::Lax));

    // Confirm a matching row was actually inserted into the `sessions`
    // table, by querying the test's database pool directly.
    let session_count: i64 = sqlx::query_scalar!(
        r#"SELECT count(*) as "count!" FROM sessions s
           JOIN users u ON u.id = s.user_id
           WHERE u.username = 'thaiChessMaster'"#
    )
    .fetch_one(&app._db_pool)
    .await
    .expect("Failed to count sessions");
    assert!(session_count >= 1);
}

#[tokio::test]
async fn sign_in_with_wrong_password_is_unauthorized() {
    let app = new_test_app().await;
    app.server
        .post("/users")
        .form(&serde_json::json!({
            "username": "thaiChessMaster",
            "email": "thaichess@example.com",
            "password": "password123",
        }))
        .await;

    let response = app
        .server
        .post("/sign-in")
        .form(&serde_json::json!({
            "username": "thaiChessMaster",
            "password": "wrong-password",
        }))
        .expect_failure()
        .await;
    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn sign_in_with_unknown_username_is_unauthorized() {
    let app = new_test_app().await;

    let response = app
        .server
        .post("/sign-in")
        .form(&serde_json::json!({
            "username": "nobody",
            "password": "password123",
        }))
        .expect_failure()
        .await;
    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn me_succeeds_with_valid_session_cookie() {
    let app = new_test_app().await;
    sign_up_and_sign_in(&app.server, "thaiChessMaster").await;

    // Thanks to `.save_cookies()` on the test server, this request
    // automatically carries the session cookie set by the sign-in above,
    // exactly as a real browser would.
    let response = app.server.get("/users/me").await;
    assert_eq!(response.status_code(), StatusCode::OK);
    let body: serde_json::Value = response.json();
    assert_eq!(body["username"], "thaiChessMaster");
}

#[tokio::test]
async fn me_is_unauthorized_without_cookie() {
    let app = new_test_app().await;

    // No sign-in happened, so there's no session cookie at all.
    let response = app.server.get("/users/me").expect_failure().await;
    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn me_is_unauthorized_with_garbage_cookie() {
    let app = new_test_app().await;

    // A cookie value that isn't even a valid UUID should be rejected before
    // ever touching the database.
    let response = app
        .server
        .get("/users/me")
        .add_cookie(Cookie::new("session_id", "not-a-uuid"))
        .expect_failure()
        .await;
    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn me_is_unauthorized_with_expired_session() {
    let app = new_test_app().await;
    sign_up_and_sign_in(&app.server, "thaiChessMaster").await;

    // Force the session that sign-in just created into the past, so it
    // reads as expired without needing to actually wait 30 days.
    sqlx::query!(
        r#"UPDATE sessions SET expires_at = now() - interval '1 minute'
           WHERE user_id = (SELECT id FROM users WHERE username = 'thaiChessMaster')"#
    )
    .execute(&app._db_pool)
    .await
    .expect("Failed to expire session");

    let response = app.server.get("/users/me").expect_failure().await;
    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn me_is_unauthorized_with_revoked_session() {
    let app = new_test_app().await;
    sign_up_and_sign_in(&app.server, "thaiChessMaster").await;

    // Mark the session revoked directly in the database (this is the same
    // effect a future sign-out endpoint would have).
    sqlx::query!(
        r#"UPDATE sessions SET revoked_at = now()
           WHERE user_id = (SELECT id FROM users WHERE username = 'thaiChessMaster')"#
    )
    .execute(&app._db_pool)
    .await
    .expect("Failed to revoke session");

    let response = app.server.get("/users/me").expect_failure().await;
    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn stale_last_seen_at_is_refreshed_on_authenticated_request() {
    let app = new_test_app().await;
    sign_up_and_sign_in(&app.server, "thaiChessMaster").await;

    // Simulate a user who hasn't been seen in a while (older than the
    // 30 minute staleness time).
    sqlx::query!(
        r#"UPDATE users SET last_seen_at = now() - interval '30 minutes'
           WHERE username = 'thaiChessMaster'"#
    )
    .execute(&app._db_pool)
    .await
    .expect("Failed to set stale last_seen_at");

    // Any authenticated request should refresh it.
    let response = app.server.get("/users/me").await;
    assert_eq!(response.status_code(), StatusCode::OK);

    let last_seen_at: chrono::DateTime<chrono::Utc> = sqlx::query_scalar!(
        r#"SELECT last_seen_at as "last_seen_at!" FROM users WHERE username = 'thaiChessMaster'"#
    )
    .fetch_one(&app._db_pool)
    .await
    .expect("Failed to read last_seen_at");

    let age = chrono::Utc::now() - last_seen_at;
    assert!(age < chrono::Duration::minutes(1));
}

#[tokio::test]
async fn session_cookie_is_removed_after_signing_out() {
    let app = new_test_app().await;
    sign_up_and_sign_in(&app.server, "thaiChessMaster").await;

    let response = app.server.post("/sign-out").await;
    assert_eq!(response.status_code(), StatusCode::NO_CONTENT);

    // cookie should be removed
    let cookie: Cookie = response.cookie("session_id");
    assert_eq!(cookie.value(), "");

    // with the cookie removed, user is not authorized
    let response = app.server.get("/users/me").expect_failure().await;
    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn session_should_be_revoked_after_signing_out() {
    let app = new_test_app().await;
    let session_cookie = sign_up_and_sign_in(&app.server, "thaiChessMaster").await;
    let session_id: sqlx::types::Uuid = session_cookie
        .value()
        .parse()
        .expect("session cookie value should be a valid UUID");

    let response = app.server.post("/sign-out").await;
    assert_eq!(response.status_code(), StatusCode::NO_CONTENT);

    // cookie should be removed
    let cookie: Cookie = response.cookie("session_id");
    assert_eq!(cookie.value(), "");

    // check sessions.revoked_at now has a date/time value
    let revoked_at: Option<chrono::DateTime<chrono::Utc>> = sqlx::query_scalar!(
        r#"SELECT revoked_at FROM sessions WHERE id = $1"#,
        session_id
    )
    .fetch_one(&app._db_pool)
    .await
    .expect("Failed to fetch session revoked_at");
    assert!(revoked_at.is_some());
}
