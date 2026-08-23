use axum_test::TestServer;
use makrust::configuration::DatabaseSettings;
use makrust::database::{get_pool, run_migrations};
use makrust::routes::create_router;
use std::ops::Deref;
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::testcontainers::{ContainerAsync, ImageExt, runners::AsyncRunner};

pub struct TestApp {
    pub server: TestServer,
    // in case we need to run SQL directly from the test
    pub _db_pool: sqlx::Pool<sqlx::Postgres>,
    // Held only to keep the container alive for the lifetime of the test;
    // dropping it stops and removes the container.
    _container: ContainerAsync<Postgres>,
}

impl Deref for TestApp {
    type Target = TestServer;

    fn deref(&self) -> &Self::Target {
        &self.server
    }
}

pub async fn new_test_app() -> TestApp {
    let container = Postgres::default()
        .with_tag("18-alpine")
        .start()
        .await
        .expect("Failed to start postgres test container");

    let database_settings = DatabaseSettings {
        username: "postgres".to_string(),
        password: "postgres".to_string(),
        port: container
            .get_host_port_ipv4(5432)
            .await
            .expect("Failed to get test container port"),
        host: container
            .get_host()
            .await
            .expect("Failed to get test container host")
            .to_string(),
        database_name: "postgres".to_string(),
        require_ssl: false,
    };

    let pool: sqlx::Pool<sqlx::Postgres> = get_pool(&database_settings).await;
    run_migrations(&pool)
        .await
        .expect("Failed to run migrations on test database");

    let app = create_router(pool.clone());

    let server = TestServer::builder()
        // Preserve cookies across requests
        // for the session cookie to work.
        // .save_cookies()
        .expect_success_by_default()
        .build(app);

    TestApp {
        server,
        _db_pool: pool,
        _container: container,
    }
}
