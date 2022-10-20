use std::sync::Arc;

use log::info;
use tokio::sync::mpsc;

use rust_home::clients::get_clients;
use rust_home::db::plugs::PlugsClient;
use rust_home::db::schedules::SchedulesClient;
use rust_home::db::temp_actions::TempActionsClient;
use rust_home::db::DbConfig;
use rust_home::domain::WorkMessage;
use rust_home::{api, configuration::get_configuration, work_handler, work_handler::WorkHandler};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let configuration = get_configuration().expect("Failed to read configuration.");
    let db_config = DbConfig::new(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to database");

    let plugs_client = Arc::new(PlugsClient::new(db_config.clone()));
    let schedules_client = Arc::new(SchedulesClient::new(db_config.clone()));
    let temp_actions_client = Arc::new(TempActionsClient::new(db_config.clone()));
    let (shelly_client, firestore_client) = get_clients();

    let (sender, receiver) = mpsc::channel::<WorkMessage>(32);

    let work_handler = WorkHandler::new(
        firestore_client,
        shelly_client,
        receiver,
        plugs_client.clone(),
        schedules_client.clone(),
        temp_actions_client.clone(),
    );

    let poll_sender = sender.clone();
    let api_sender = sender.clone();

    tokio::spawn(async { work_handler.start().await });
    tokio::spawn(async { work_handler::poll(poll_sender, 10).await });
    tokio::spawn(async {
        info!("Adding shutdown handler");
        shutdown_signal().await
    });

    api::start(
        api_sender,
        configuration.application_port,
        plugs_client.clone(),
    )
    .await
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c().await.unwrap();
    info!("Signal received, starting graceful shutdown");
    std::process::exit(0);
}
