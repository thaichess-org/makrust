#[derive(serde::Deserialize, Clone)]
pub struct ApplicationSettings {
    pub port: u16,
    pub host: String,
    pub base_url: String,
}

#[derive(serde::Deserialize, Clone)]
pub struct Settings {
    pub application: ApplicationSettings,
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine current directory");
    let configuration_directory = base_path.join("configuration");

    // get running environment, default to development
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "development".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT.");
    println!("App running in APP_ENVIRONMENT: {}", environment.as_str());
    let environment_filename = format!("{}.yaml", environment.as_str());

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
