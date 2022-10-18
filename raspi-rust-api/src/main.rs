use log::info;
use tokio::sync::mpsc;

use rust_home::clients::get_clients;
use rust_home::db::DbConfig;
use rust_home::{
    api, configuration::get_configuration, work_handler, work_handler::WorkHandler, WorkMessage,
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration().expect("Failed to read configuration.");
    let db = DbConfig::new(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to database");
    env_logger::init();
    let (shelly_client, firestore_client) = get_clients();

    let (sender, receiver) = mpsc::channel::<WorkMessage>(32);

    let work_handler = WorkHandler::new(firestore_client, shelly_client, receiver);

    let poll_sender = sender.clone();
    let api_sender = sender.clone();

    tokio::spawn(async { work_handler.start().await });
    tokio::spawn(async { work_handler::poll(poll_sender, 10).await });
    tokio::spawn(async {
        info!("Adding shutdown handler");
        shutdown_signal().await
    });

    api::start(api_sender, configuration.application_port).await
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c().await.unwrap();
    info!("Signal received, starting graceful shutdown");
    std::process::exit(0);
}
