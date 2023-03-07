use std::sync::Arc;

use log::info;
use tokio::sync::{mpsc, RwLock};

use rust_home::clients::mqtt::MqttClient;
use rust_home::clients::{
    shelly_client::ShellyClient, tibber_client::TibberClient, tibber_subscriber::TibberSubscriber,
};
use rust_home::db::DbConfig;
use rust_home::domain::WorkMessage;
use rust_home::service::consumption_cache::ConsumptionCache;
use rust_home::service::notifications::{NotificationHandler, NotificationMessage};
use rust_home::{
    api, configuration::get_configuration, cron_scheduler, env_var, work_handler::WorkHandler,
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let configuration = get_configuration().expect("Failed to read configuration.");
    let db_config = DbConfig::new(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to database");
    let pool = Arc::new(db_config.pool.clone());

    let tibber_client = Arc::new(TibberClient::new(env_var("TIBBER_API_TOKEN")));
    let shelly_client = Arc::new(ShellyClient::default());

    let (notification_tx, notification_rx) = mpsc::channel::<NotificationMessage>(10);
    let (work_message_tx, work_message_rx) = mpsc::channel::<WorkMessage>(10);

    let consumption_cache = Arc::new(RwLock::new(ConsumptionCache::new(notification_tx)));
    let work_handler = WorkHandler::new(
        shelly_client.clone(),
        tibber_client.clone(),
        work_message_tx.clone(),
        work_message_rx,
        pool.clone(),
    );
    tokio::spawn(async { work_handler.start().await });

    let notification_handler = NotificationHandler::new(notification_rx, pool.clone());
    tokio::spawn(async { notification_handler.start().await });

    if configuration.mqtt.run_mqtt {
        let mqtt_client = MqttClient::new(
            configuration.mqtt.host,
            configuration.mqtt.base_topic,
            pool.clone(),
            work_message_tx.clone(),
        );
        tokio::spawn(async move { mqtt_client.start().await });
    } else {
        info!("Not running MQTT, disabled in config")
    }

    let cron_tibber_client = tibber_client.clone();
    let cron_pool = pool.clone();
    tokio::spawn(async { cron_scheduler::start(cron_tibber_client, cron_pool).await });

    let subscriber_cache = consumption_cache.clone();
    if configuration.run_live_consumption_subscriber {
        tokio::spawn(async { TibberSubscriber::new(subscriber_cache).subscribe().await });
    } else {
        info!("Not running live consumption subscriber, disabled in config")
    }

    let server = api::start(
        work_message_tx,
        configuration.application_host,
        configuration.application_port,
        tibber_client,
        shelly_client,
        consumption_cache.clone(),
        pool,
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
