use log::info;
use tokio::signal;
use tokio::sync::mpsc;

use rust_home::clients::get_clients;
use rust_home::{api, work_handler, work_handler::WorkHandler, WorkMessage};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let (shelly_client, firestore_client) = get_clients();

    let (sender, receiver) = mpsc::channel::<WorkMessage>(32);

    let work_handler = WorkHandler::new(firestore_client, shelly_client, receiver);

    let poll_sender = sender.clone();
    let api_sender = sender.clone();

    tokio::spawn(async move { work_handler.start().await });
    tokio::spawn(async move { work_handler::poll(poll_sender, 10).await });
    tokio::spawn(async {
        info!("Adding shutdown handler");
        shutdown_signal().await
    });

    api::start(api_sender).await
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Signal received, starting graceful shutdown");
}
