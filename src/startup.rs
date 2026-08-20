use crate::configuration::get_configuration;
use crate::database::get_pool;
use crate::routes::create_router;

pub struct Application {
    port: u16,
}

impl Application {
    pub async fn run() -> Result<Self, std::io::Error> {
        let configuration = get_configuration().expect("Failed to load configuration.");
        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let pool = get_pool(&configuration.database).await;
        let router = create_router(pool.clone());
        let listener = tokio::net::TcpListener::bind(&address).await.unwrap();
        let port = listener.local_addr().unwrap().port();
        axum::serve(listener, router).await.unwrap();

        Ok(Self { port })
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}
