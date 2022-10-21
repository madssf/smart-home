use std::str::FromStr;

use chrono::{NaiveDateTime, NaiveTime, Utc, Weekday};
use sqlx::types::ipnetwork::IpNetwork;
use testcontainers::clients::Cli;
use uuid::Uuid;

use configuration::DatabaseTestConfig;
use rust_home::db::{plugs, rooms, schedules, temp_actions, temperature_logs};
use rust_home::domain::{ActionType, Plug, PriceLevel, Schedule, TempAction, TemperatureLog};

mod configuration;

fn plug(room_id: &Uuid) -> Plug {
    Plug::new("test_plug", "127.0.0.1", "username", "password", room_id)
        .expect("Could not create plug")
}

fn schedule(room_ids: Vec<Uuid>) -> Schedule {
    Schedule::new(
        &PriceLevel::NORMAL,
        vec![Weekday::Mon, Weekday::Tue],
        vec![(
            NaiveTime::from_hms(12, 00, 00),
            NaiveTime::from_hms(13, 00, 00),
        )],
        20.0,
        room_ids,
    )
    .expect("Could not create schedule")
}

fn temp_action(room_ids: Vec<Uuid>) -> TempAction {
    TempAction::new(
        &NaiveDateTime::from_timestamp(1666291743, 0),
        "ON",
        room_ids,
    )
    .expect("Failed to create temp_action")
}

fn temperature_log(room_id: Uuid) -> TemperatureLog {
    TemperatureLog {
        room_id,
        time: Utc::now().naive_utc(),
        temp: 20.0,
    }
}

#[tokio::test]
async fn can_create_room() {
    let docker = Cli::default();

    let test_config = DatabaseTestConfig::new(&docker).await;
    let rooms_client = rooms::RoomsClient::new(test_config.db_config);
    rooms_client
        .create_room("test_room")
        .await
        .expect("Could not insert plug");

    let result = rooms_client.get_rooms().await.expect("Can't get rooms");

    let result_room = result[0].clone();

    assert_eq!(result_room.name, "test_room")
}

#[tokio::test]
async fn can_insert_plug() {
    let docker = Cli::default();

    let test_config = DatabaseTestConfig::new(&docker).await;
    let rooms_client = rooms::RoomsClient::new(test_config.db_config.clone());
    rooms_client
        .create_room("test_room")
        .await
        .expect("Could not insert plug");
    let rooms = rooms_client.get_rooms().await.expect("Can't get rooms");
    let room_id = rooms[0].clone().id;

    let plugs_client = plugs::PlugsClient::new(test_config.db_config.clone());

    let new_plug = plug(&room_id);

    plugs_client
        .create_plug(new_plug.clone())
        .await
        .expect("Could not insert plug");

    let result = plugs_client.get_plugs().await.expect("Can't get plugs");

    let result_plug = result[0].clone();

    assert_eq!(result_plug, new_plug);
}

#[tokio::test]
async fn can_update_plug() {
    let docker = Cli::default();

    let test_config = DatabaseTestConfig::new(&docker).await;
    let rooms_client = rooms::RoomsClient::new(test_config.db_config.clone());
    rooms_client
        .create_room("test_room")
        .await
        .expect("Could not insert plug");
    let rooms = rooms_client.get_rooms().await.expect("Can't get rooms");
    let room_id = rooms[0].clone().id;

    let plugs_client = plugs::PlugsClient::new(test_config.db_config.clone());
    let new_plug = plug(&room_id);

    plugs_client
        .create_plug(new_plug)
        .await
        .expect("Could not insert plug");

    let stored = plugs_client.get_plugs().await.expect("Can't get plugs");
    let stored_plug = stored[0].clone();

    let updated_plug = Plug {
        id: stored_plug.id,
        name: "new_name".to_string(),
        ip: IpNetwork::from_str("127.0.0.2").expect("Could not create IP"),
        username: "new_uname".to_string(),
        password: "new_pass".to_string(),
        room_id,
    };

    plugs_client
        .update_plug(updated_plug.clone())
        .await
        .expect("Can't update plug");

    let result = plugs_client.get_plugs().await.expect("Can't get plugs");
    let result_plug = result[0].clone();

    assert_eq!(result_plug, updated_plug);
}

#[tokio::test]
async fn can_delete_plug() {
    let docker = Cli::default();

    let test_config = DatabaseTestConfig::new(&docker).await;
    let rooms_client = rooms::RoomsClient::new(test_config.db_config.clone());
    rooms_client
        .create_room("test_room")
        .await
        .expect("Could not insert room");
    let rooms = rooms_client.get_rooms().await.expect("Can't get rooms");
    let room_id = rooms[0].clone().id;

    let plugs_client = plugs::PlugsClient::new(test_config.db_config.clone());

    let new_plug = plug(&room_id);

    plugs_client
        .create_plug(new_plug)
        .await
        .expect("Could not insert plug");

    let stored = plugs_client.get_plugs().await.expect("Can't get plugs");

    let id = stored[0].clone().id;

    plugs_client
        .delete_plug(&id)
        .await
        .expect("Failed to delete plug");

    let result = plugs_client.get_plugs().await.expect("Can't get plugs");
    assert_eq!(result.len(), 0)
}

#[tokio::test]
async fn can_insert_schedule() {
    let docker = Cli::default();

    let test_config = DatabaseTestConfig::new(&docker).await;
    let rooms_client = rooms::RoomsClient::new(test_config.db_config.clone());
    rooms_client
        .create_room("test_room")
        .await
        .expect("Could not insert room");
    let rooms = rooms_client.get_rooms().await.expect("Can't get rooms");
    let room_id = rooms[0].clone().id;

    let schedules_client = schedules::SchedulesClient::new(test_config.db_config.clone());

    let new_schedule = schedule(vec![room_id]);

    schedules_client
        .create_schedule(new_schedule.clone())
        .await
        .expect("Could not insert schedule");

    let result = schedules_client
        .get_schedules()
        .await
        .expect("Can't get schedules");

    let result_schedule = result[0].clone();

    assert_eq!(result_schedule, new_schedule);
}

#[tokio::test]
async fn can_update_schedule() {
    let docker = Cli::default();

    let test_config = DatabaseTestConfig::new(&docker).await;
    let rooms_client = rooms::RoomsClient::new(test_config.db_config.clone());
    rooms_client
        .create_room("test_room")
        .await
        .expect("Could not insert room");
    rooms_client
        .create_room("test_room_2")
        .await
        .expect("Could not insert room");

    let rooms = rooms_client.get_rooms().await.expect("Can't get rooms");
    let room_id_1 = rooms[0].clone().id;
    let room_id_2 = rooms[1].clone().id;

    let schedules_client = schedules::SchedulesClient::new(test_config.db_config.clone());

    let new_schedule = schedule(vec![room_id_1]);

    schedules_client
        .create_schedule(new_schedule.clone())
        .await
        .expect("Could not insert schedule");

    let stored = schedules_client
        .get_schedules()
        .await
        .expect("Can't get schedules");

    let stored_schedule = stored[0].clone();

    let update_expected = Schedule {
        id: stored_schedule.id,
        price_level: PriceLevel::CHEAP,
        days: vec![Weekday::Fri],
        time_windows: vec![(NaiveTime::from_hms(1, 0, 0), NaiveTime::from_hms(2, 0, 0))],
        temp: 2.0,
        room_ids: vec![room_id_2],
    };

    schedules_client
        .update_schedule(update_expected.clone())
        .await
        .expect("Could not update schedule");

    let stored = schedules_client
        .get_schedules()
        .await
        .expect("Can't get schedules");

    let stored_schedule = stored[0].clone();

    assert_eq!(stored_schedule, update_expected);
}

#[tokio::test]
async fn can_delete_schedule() {
    let docker = Cli::default();

    let test_config = DatabaseTestConfig::new(&docker).await;
    let rooms_client = rooms::RoomsClient::new(test_config.db_config.clone());
    rooms_client
        .create_room("test_room")
        .await
        .expect("Could not insert room");

    let rooms = rooms_client.get_rooms().await.expect("Can't get rooms");
    let room_id = rooms[0].clone().id;

    let schedules_client = schedules::SchedulesClient::new(test_config.db_config.clone());

    let new_schedule = schedule(vec![room_id]);

    schedules_client
        .create_schedule(new_schedule.clone())
        .await
        .expect("Could not insert schedule");

    let stored = schedules_client
        .get_schedules()
        .await
        .expect("Can't get schedules");

    let stored_schedule = stored[0].clone();

    schedules_client
        .delete_schedule(&stored_schedule.id)
        .await
        .expect("Could not delete schedule");

    let stored = schedules_client
        .get_schedules()
        .await
        .expect("Can't get schedules");

    assert_eq!(stored.len(), 0);
}

#[tokio::test]
async fn temp_actions() {
    let docker = Cli::default();

    let test_config = DatabaseTestConfig::new(&docker).await;
    let rooms_client = rooms::RoomsClient::new(test_config.db_config.clone());
    rooms_client
        .create_room("test_room")
        .await
        .expect("Could not insert room");
    rooms_client
        .create_room("test_room_2")
        .await
        .expect("Could not insert room");

    let rooms = rooms_client.get_rooms().await.expect("Can't get rooms");
    let room_id_1 = rooms[0].clone().id;
    let room_id_2 = rooms[1].clone().id;

    let client = temp_actions::TempActionsClient::new(test_config.db_config);

    let new_action = temp_action(vec![room_id_1]);

    client
        .create_temp_action(new_action)
        .await
        .expect("Failed to insert temp action");

    let stored = client
        .get_temp_actions()
        .await
        .expect("Failed to get temp actions");

    assert_eq!(stored.len(), 1);

    let stored_action = stored[0].clone();

    let updated_action = TempAction {
        id: stored_action.id,
        room_ids: vec![room_id_1, room_id_2],
        action_type: ActionType::ON,
        expires_at: stored_action.expires_at,
    };

    client
        .update_temp_action(updated_action.clone())
        .await
        .expect("Failed to update temp action");

    let after_update = client
        .get_temp_actions()
        .await
        .expect("Failed to get temp actions");

    let after_update_action = after_update[0].clone();

    assert_eq!(after_update_action, updated_action);

    client
        .delete_temp_action(&after_update_action.id)
        .await
        .expect("Failed to delete temp action");
    let after_delete = client
        .get_temp_actions()
        .await
        .expect("Failed to get temp actions");
    assert_eq!(after_delete.len(), 0)
}

#[tokio::test]
async fn temperature_logs() {
    let docker = Cli::default();

    let test_config = DatabaseTestConfig::new(&docker).await;
    let rooms_client = rooms::RoomsClient::new(test_config.db_config.clone());
    rooms_client
        .create_room("test_room")
        .await
        .expect("Could not insert room");

    let rooms = rooms_client.get_rooms().await.expect("Can't get rooms");
    let room_id = rooms[0].clone().id;

    let client = temperature_logs::TemperatureLogsClient::new(test_config.db_config);

    let log_entry = temperature_log(room_id);

    client
        .create_temp_log(log_entry.clone())
        .await
        .expect("Failed to insert temp action");

    let duplicate = client.create_temp_log(log_entry).await;

    assert!(duplicate.is_err());

    for _ in 0..1000 {
        client
            .create_temp_log(temperature_log(room_id))
            .await
            .expect("Could not create temp_log")
    }

    let stored = client
        .get_temp_logs()
        .await
        .expect("Failed to get temp actions");

    assert_eq!(stored.len(), 1001)
}
