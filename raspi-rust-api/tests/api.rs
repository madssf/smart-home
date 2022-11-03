use std::sync::Arc;

use actix_web::dev::Server;
use testcontainers::clients::Cli;
use tokio::sync::{mpsc, RwLock};

use rust_home::api::start;
use rust_home::clients::shelly_client::ShellyClient;
use rust_home::clients::tibber_client::TibberClient;
use rust_home::domain::WorkMessage;
use rust_home::service::consumption_cache::ConsumptionCache;

use crate::configuration::DatabaseTestConfig;

mod configuration;

async fn spawn_api() -> Server {
    let docker = Cli::default();

    let (sender, _) = mpsc::channel::<WorkMessage>(32);
    let test_config = DatabaseTestConfig::new(&docker).await;
    let tibber_client = Arc::new(TibberClient::new("dummy_token".to_string()));

    start(
        sender.clone(),
        "127.0.0.1".to_string(),
        8080,
        tibber_client,
        Arc::new(ShellyClient::default()),
        Arc::new(RwLock::new(ConsumptionCache::default())),
        test_config.db_config.pool,
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
