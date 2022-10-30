use std::sync::Arc;

use log::info;
use tokio::sync::{mpsc, Mutex};

use rust_home::clients::{
    shelly_client::ShellyClient, tibber_client::TibberClient,
    tibber_subscriber::live_power_subscriber,
};
use rust_home::db::DbConfig;
use rust_home::domain::WorkMessage;
use rust_home::service::consumption_cache::ConsumptionCache;
use rust_home::{
    api, configuration::get_configuration, env_var, work_handler, work_handler::WorkHandler,
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let configuration = get_configuration().expect("Failed to read configuration.");
    let db_config = DbConfig::new(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to database");

    let tibber_client = Arc::new(TibberClient::new(env_var("TIBBER_API_TOKEN")));
    let shelly_client = ShellyClient::default();
    let consumption_cache = Arc::new(Mutex::new(ConsumptionCache::default()));

    let (sender, receiver) = mpsc::channel::<WorkMessage>(10);

    let work_handler = WorkHandler::new(
        shelly_client,
        tibber_client.clone(),
        sender.clone(),
        receiver,
        Arc::new(db_config.pool.clone()),
    );

    let poll_sender = sender.clone();
    let api_sender = sender.clone();
    let subscriber_cache = consumption_cache.clone();

    tokio::spawn(async { work_handler.start().await });
    tokio::spawn(async { work_handler::poll(poll_sender, 10).await });
    tokio::spawn(async { live_power_subscriber(subscriber_cache).await });

    let server = api::start(
        api_sender,
        configuration.application_host,
        configuration.application_port,
        tibber_client,
        consumption_cache.clone(),
        db_config.pool.clone(),
    )
    .await?;

    tokio::spawn(async { server.await });
    shutdown_signal().await;

    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c().await.unwrap();
    info!("Signal received, shutting down.");
    std::process::exit(0);
}
