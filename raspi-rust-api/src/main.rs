use std::sync::Arc;

use log::info;
use tokio::sync::mpsc;

use rust_home::db::plugs::PlugsClient;
use rust_home::db::rooms::RoomsClient;
use rust_home::db::schedules::SchedulesClient;
use rust_home::db::temp_actions::TempActionsClient;
use rust_home::db::temperature_logs::TemperatureLogsClient;
use rust_home::db::{DbClients, DbConfig};
use rust_home::domain::WorkMessage;
use rust_home::shelly_client::ShellyClient;
use rust_home::{api, configuration::get_configuration, work_handler, work_handler::WorkHandler};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let configuration = get_configuration().expect("Failed to read configuration.");
    let db_config = DbConfig::new(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to database");

    let rooms_client = Arc::new(RoomsClient::new(db_config.clone()));
    let plugs_client = Arc::new(PlugsClient::new(db_config.clone()));
    let schedules_client = Arc::new(SchedulesClient::new(db_config.clone()));
    let temp_actions_client = Arc::new(TempActionsClient::new(db_config.clone()));
    let temperature_logs_client = Arc::new(TemperatureLogsClient::new(db_config.clone()));
    let shelly_client = ShellyClient::default();

    let (sender, receiver) = mpsc::channel::<WorkMessage>(32);

    let work_handler = WorkHandler::new(
        shelly_client,
        receiver,
        plugs_client.clone(),
        schedules_client.clone(),
        temp_actions_client.clone(),
        temperature_logs_client.clone(),
    );

    let poll_sender = sender.clone();
    let api_sender = sender.clone();

    tokio::spawn(async { work_handler.start().await });
    tokio::spawn(async { work_handler::poll(poll_sender, 10).await });
    tokio::spawn(async {
        info!("Adding shutdown handler");
        shutdown_signal().await
    });

    let server = api::start(
        api_sender,
        "0.0.0.0".to_string(),
        configuration.application_port,
        DbClients {
            rooms: rooms_client.clone(),
            plugs: plugs_client.clone(),
            schedules: schedules_client.clone(),
            temp_actions: temp_actions_client.clone(),
            temperature_logs: temperature_logs_client.clone(),
        },
    );

    server.await?.await
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c().await.unwrap();
    info!("Signal received, starting graceful shutdown");
    std::process::exit(0);
}
