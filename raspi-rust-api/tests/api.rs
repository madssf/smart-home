use std::sync::Arc;

use actix_web::dev::Server;
use testcontainers::clients::Cli;
use tokio::sync::mpsc;

use rust_home::api::start;
use rust_home::db::plugs::PlugsClient;
use rust_home::db::rooms::RoomsClient;
use rust_home::db::schedules::SchedulesClient;
use rust_home::db::temp_actions::TempActionsClient;
use rust_home::db::temperature_logs::TemperatureLogsClient;
use rust_home::db::DbClients;
use rust_home::domain::WorkMessage;

use crate::configuration::DatabaseTestConfig;

mod configuration;

async fn spawn_api() -> Server {
    let docker = Cli::default();

    let (sender, _) = mpsc::channel::<WorkMessage>(32);
    let test_config = DatabaseTestConfig::new(&docker).await;

    let db_config = test_config.db_config;

    let rooms_client = Arc::new(RoomsClient::new(db_config.clone()));
    let plugs_client = Arc::new(PlugsClient::new(db_config.clone()));
    let schedules_client = Arc::new(SchedulesClient::new(db_config.clone()));
    let temp_actions_client = Arc::new(TempActionsClient::new(db_config.clone()));
    let temperature_logs_client = Arc::new(TemperatureLogsClient::new(db_config.clone()));

    start(
        sender.clone(),
        "127.0.0.1".to_string(),
        8080,
        DbClients {
            rooms: rooms_client,
            plugs: plugs_client,
            schedules: schedules_client,
            temp_actions: temp_actions_client,
            temperature_logs: temperature_logs_client,
        },
    )
    .await
    .expect("Failed to start api")
}

#[tokio::test]
async fn api() {
    tokio::spawn(spawn_api().await);

    let client = reqwest::Client::new();
    let result = client
        .get("http://127.0.0.1:8080/_/health")
        .send()
        .await
        .expect("failed to execute");

    assert!(result.status().is_success());
}
