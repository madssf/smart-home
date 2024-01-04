use std::net::SocketAddr;
use std::sync::Arc;

use testcontainers::clients::Cli;
use tokio::sync::{mpsc, oneshot, RwLock};
use tokio::task::{self, JoinHandle};

use rust_home::api::start;
use rust_home::clients::shelly_client::ShellyClient;
use rust_home::clients::tibber_client::TibberClient;
use rust_home::domain::WorkMessage;
use rust_home::service::consumption_cache::ConsumptionCache;
use rust_home::service::notifications::NotificationMessage;

use crate::configuration::DatabaseTestConfig;

mod configuration;

fn url_for(local_addr: &SocketAddr, path: &str) -> String {
    format!("http://{}{}", local_addr, path)
}
async fn spin_up_test_server() -> (SocketAddr, oneshot::Sender<()>, JoinHandle<()>) {
    env_logger::init();

    let docker = Cli::default();

    let (work_tx, _) = mpsc::channel::<WorkMessage>(32);
    let (notification_tx, _) = mpsc::channel::<NotificationMessage>(32);
    let test_config = DatabaseTestConfig::new(&docker).await;
    let tibber_client = Arc::new(TibberClient::new("dummy_token".to_string()));

    let server = start(
        work_tx.clone(),
        tibber_client,
        Arc::new(ShellyClient::default()),
        Arc::new(RwLock::new(ConsumptionCache::new(notification_tx.clone()))),
        Arc::new(test_config.db_config.pool),
    )
    .await;
    // Bind to port 0 to get a random available port
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();

    let local_addr = listener.local_addr().unwrap();

    let (tx, rx) = oneshot::channel();

    // Spawn the server into a background task
    let server_handle = task::spawn(async move {
        axum::serve(
            listener,
            server.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(async {
            rx.await.ok();
        })
        .await
        .unwrap();
    });

    // Return the server's address and the oneshot sender for shutting down
    (local_addr, tx, server_handle)
}

async fn assert_api_ready(local_addr: &SocketAddr) {
    let client = reqwest::Client::new();
    let result = client
        .get(url_for(local_addr, "/_/health"))
        .send()
        .await
        .expect("failed to execute");

    assert!(result.status().is_success());
}

#[tokio::test]
async fn given_api_started_then_return_200() {
    let (addr, shutdown_tx, server_handle) = spin_up_test_server().await;

    assert_api_ready(&addr).await;

    // Shut down the server
    shutdown_tx.send(()).unwrap();
    server_handle.await.unwrap();
}
