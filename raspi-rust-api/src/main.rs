use log::info;
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
    tokio::spawn(async move { work_handler::poll(poll_sender, 1).await });
    tokio::spawn(async move {
        info!("Adding shutdown handler");
        tokio::signal::ctrl_c().await.unwrap();
        std::process::exit(0);
    });

    api::start(api_sender).await
}
