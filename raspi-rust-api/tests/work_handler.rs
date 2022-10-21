use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use testcontainers::clients::Cli;
use tokio::sync::mpsc;

use rust_home::db::DbClients;
use rust_home::domain::{PriceLevel, WorkMessage};
use rust_home::prices::PriceInfo;
use rust_home::shelly_client::ShellyClient;
use rust_home::work_handler;

use crate::configuration::DatabaseTestConfig;

mod configuration;

#[tokio::test]
async fn starts() {
    let shelly_client = ShellyClient::default();
    let docker = Cli::default();
    let test_config = DatabaseTestConfig::new(&docker).await;
    let db_clients = DbClients::new(&test_config.db_config);
    let (sender, receiver) = mpsc::channel::<WorkMessage>(32);
    let handler =
        work_handler::WorkHandler::new(shelly_client, sender, receiver, db_clients.clone());

    let price_info = PriceInfo {
        amount: 0.0,
        currency: "NOK".to_string(),
        level: PriceLevel::CHEAP,
    };

    let now = NaiveDateTime::new(
        NaiveDate::from_ymd(2020, 1, 1),
        NaiveTime::from_hms(0, 0, 0),
    );

    let result = handler.main_handler(&price_info, &now).await;
    assert!(result.is_ok())
}

#[tokio::test]
async fn handles_temp_log() {
    let shelly_client = ShellyClient::default();
    let docker = Cli::default();
    let test_config = DatabaseTestConfig::new(&docker).await;
    let db_clients = DbClients::new(&test_config.db_config);
    let (sender, receiver) = mpsc::channel::<WorkMessage>(32);
    let handler =
        work_handler::WorkHandler::new(shelly_client, sender.clone(), receiver, db_clients.clone());

    let rooms_client = db_clients.rooms.clone();
    rooms_client
        .create_room("test")
        .await
        .expect("Failed to create room");
    let rooms = rooms_client.get_rooms().await.expect("Failed to get rooms");
    let room_id = rooms[0].id;
    handler
        .temp_handler(&room_id, &20.0, sender.clone())
        .await
        .expect("Temp handler failed");
    let temp_logs_client = db_clients.temperature_logs.clone();
    let temp_logs = temp_logs_client
        .get_temp_logs()
        .await
        .expect("Failed to get temp_logs");
    assert_eq!(temp_logs.len(), 1);
    assert_eq!(temp_logs[0].temp, 20.0);
    assert_eq!(temp_logs[0].room_id, room_id);
}
