use log::info;
use serde_aux::field_attributes::deserialize_number_from_string;

use crate::observability::{get_app_environment, Environment};

#[derive(serde::Deserialize, Debug)]
pub struct Settings {
    pub database: DatabaseSettings,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub application_port: u16,
    pub application_host: String,
    pub run_subscriber: bool,
}
#[derive(serde::Deserialize, Debug)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let configuration_directory = base_path.join("configuration");

    let environment = get_app_environment();
    info!("Detected environment: {}", environment);

    let environment_filename = format!("{}.yaml", environment.to_string().to_lowercase());
    let settings = config::Config::builder()
        .add_source(config::File::from(
            configuration_directory.join("base.yaml"),
        ))
        .add_source(config::File::from(
            configuration_directory.join(&environment_filename),
        ));

    let settings = match environment {
        Environment::Dev => settings,
        Environment::Production => settings.add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        ),
    };
    let settings = settings.build()?;
    settings.try_deserialize::<Settings>()
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        )
    }
}
