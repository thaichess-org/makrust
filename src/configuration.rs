use dotenv::dotenv;
use sqlx::postgres::{PgConnectOptions, PgSslMode};

#[derive(serde::Deserialize, Clone)]
pub struct ApplicationSettings {
    pub port: u16,
    pub host: String,
}

#[derive(serde::Deserialize, Clone)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
    // determine if we need the connection to be encrypted or not
    pub require_ssl: bool,
}

impl DatabaseSettings {
    // made this way to make it easier creating a new database per test
    pub fn without_db(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            // try en encrypted connection, fallbak to unencrypted if it fails
            PgSslMode::Prefer
        };

        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(&self.password)
            .port(self.port)
            .ssl_mode(ssl_mode)
    }

    pub fn with_db(&self) -> PgConnectOptions {
        self.without_db().database(&self.database_name)
    }
}

#[derive(serde::Deserialize, Clone)]
pub struct AuthSettings {
    pub session_ttl_days: i64,
    pub cookie_secure: bool,
    // helps set up CORS
    pub frontend_origin: String,
}

#[derive(serde::Deserialize, Clone)]
pub struct EmailSettings {
    pub base_url: String,
    pub sender: String,
    pub server_token: String,
}

#[derive(serde::Deserialize, Clone)]
pub struct Settings {
    pub application: ApplicationSettings,
    pub database: DatabaseSettings,
    pub auth: AuthSettings,
    pub email: EmailSettings,
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    dotenv().ok();

    let base_path = std::env::current_dir().expect("Failed to determine current directory");
    let configuration_directory = base_path.join("configuration");
    // get running environment, default to development
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "development".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT.");
    println!("App running in APP_ENVIRONMENT: {}", environment.as_str());
    let environment_filename = format!("{}.yaml", environment.as_str());

    // fallback to "token" in dev just to get the app running if laoding .env fails
    let email_server_token: String = match std::env::var("EMAIL_SERVER_TOKEN") {
        Ok(token) => token,
        Err(_) => match environment {
            Environment::Development => "token".into(),
            Environment::Production => {
                panic!("Email token must be set")
            }
        },
    };

    // load configuration settings from ./configuration
    let settings = config::Config::builder()
        .add_source(config::File::from(
            configuration_directory.join("base.yaml"),
        ))
        .add_source(config::File::from(
            configuration_directory.join(environment_filename),
        ))
        .add_source(
            // this will help to add the DO envs
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .set_override("email.server_token", email_server_token)?
        .build()?;

    settings.try_deserialize::<Settings>()
}

pub enum Environment {
    Development,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Development => "development",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(env: String) -> Result<Self, Self::Error> {
        match env.to_lowercase().as_str() {
            "development" => Ok(Self::Development),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. Use `development` or `production`.",
                other
            )),
        }
    }
}
