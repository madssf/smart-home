use std::sync::Arc;

use log::info;
use tokio::sync::{mpsc, RwLock};

use rust_home::clients::{
    shelly_client::ShellyClient, tibber_client::TibberClient, tibber_subscriber::TibberSubscriber,
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
    let shelly_client = Arc::new(ShellyClient::default());
    let consumption_cache = Arc::new(RwLock::new(ConsumptionCache::default()));

    let (sender, receiver) = mpsc::channel::<WorkMessage>(10);

    let work_handler = WorkHandler::new(
        shelly_client.clone(),
        tibber_client.clone(),
        sender.clone(),
        receiver,
        Arc::new(db_config.pool.clone()),
    );

    let poll_sender = sender.clone();
    let api_sender = sender.clone();
    let subscriber_cache = consumption_cache.clone();

    tokio::spawn(async { work_handler.start().await });
    tokio::spawn(async { work_handler::poll(poll_sender, 2).await });
    if configuration.run_subscriber {
        tokio::spawn(async { TibberSubscriber::new(subscriber_cache).subscribe().await });
    }

    let server = api::start(
        api_sender,
        configuration.application_host,
        configuration.application_port,
        tibber_client,
        shelly_client,
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
