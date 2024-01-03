use std::sync::Arc;

use actix_web::dev::Server;
use testcontainers::clients::Cli;
use tokio::sync::{mpsc, RwLock};

use rust_home::api::start;
use rust_home::clients::shelly_client::ShellyClient;
use rust_home::clients::tibber_client::TibberClient;
use rust_home::domain::WorkMessage;
use rust_home::service::consumption_cache::ConsumptionCache;
use rust_home::service::notifications::NotificationMessage;

use crate::configuration::DatabaseTestConfig;

mod configuration;

const IP: &str = "127.0.0.1";
const PORT: u16 = 8081;

fn base_url() -> String {
    format!("http://{}:{}", IP, PORT)
}

fn url_for(path: &str) -> String {
    format!("{}{}", base_url(), path)
}

async fn spawn_api() -> Server {
    env_logger::init();

    let docker = Cli::default();

    let (work_tx, _) = mpsc::channel::<WorkMessage>(32);
    let (notification_tx, _) = mpsc::channel::<NotificationMessage>(32);
    let test_config = DatabaseTestConfig::new(&docker).await;
    let tibber_client = Arc::new(TibberClient::new("dummy_token".to_string()));

    start(
        work_tx.clone(),
        IP.to_string(),
        PORT,
        tibber_client,
        Arc::new(ShellyClient::default()),
        Arc::new(RwLock::new(ConsumptionCache::new(notification_tx.clone()))),
        Arc::new(test_config.db_config.pool),
    )
    .await
    .expect("Failed to start api")
}

async fn assert_api_ready() {
    let client = reqwest::Client::new();
    let result = client
        .get(url_for("/_/health"))
        .send()
        .await
        .expect("failed to execute");

    assert!(result.status().is_success());
}

#[tokio::test]
async fn given_api_started_then_return_200() {
    tokio::spawn(spawn_api().await);

    assert_api_ready().await;
}
