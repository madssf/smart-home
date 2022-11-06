use std::collections::HashMap;
use std::ops::Add;
use std::str::FromStr;
use std::sync::Arc;

use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime, Utc, Weekday};
use sqlx::types::ipnetwork::IpNetwork;
use testcontainers::clients::Cli;
use uuid::Uuid;

use configuration::DatabaseTestConfig;
use rust_home::db::{plugs, rooms, schedules, temp_actions, temperature_logs};
use rust_home::domain::{ActionType, Plug, PriceLevel, Room, Schedule, TempAction, TemperatureLog};

mod configuration;
mod setup;

fn plug(room_id: &Uuid) -> Plug {
    Plug::new("test_plug", "127.0.0.1", "username", "password", room_id)
        .expect("Could not create plug")
}

fn temp_action(room_ids: Vec<Uuid>) -> TempAction {
    TempAction::new(
        &NaiveDateTime::from_timestamp(1666291743, 0),
        &ActionType::ON,
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
async fn rooms() {
    let docker = Cli::default();

    let test_config = DatabaseTestConfig::new(&docker).await;
    let pool = Arc::new(test_config.db_config.pool);
    rooms::create_room(&pool, "test_room")
        .await
        .expect("Could not insert plug");

    let result = rooms::get_rooms(&pool).await.expect("Can't get rooms");

    let result_room = result[0].clone();

    assert_eq!(result_room.name, "test_room");

    rooms::update_room(
        &pool,
        &Room {
            id: result_room.id,
            name: "test2".to_string(),
        },
    )
    .await
    .expect("Couldn't update room");

    let result = rooms::get_rooms(&pool).await.expect("Can't get rooms");
    let result_room = result[0].clone();
    assert_eq!(result_room.name, "test2");
}

#[tokio::test]
async fn can_insert_plug() {
    let docker = Cli::default();

    let test_config = DatabaseTestConfig::new(&docker).await;
    let pool = Arc::new(test_config.db_config.pool);

    rooms::create_room(&pool, "test_room")
        .await
        .expect("Could not insert plug");
    let rooms = rooms::get_rooms(&pool).await.expect("Can't get rooms");
    let room_id = rooms[0].clone().id;

    let new_plug = plug(&room_id);

    plugs::create_plug(&pool, new_plug.clone())
        .await
        .expect("Could not insert plug");

    let result = plugs::get_plugs(&pool).await.expect("Can't get plugs");

    let result_plug = result[0].clone();

    assert_eq!(result_plug, new_plug);

    let should_fail = rooms::delete_room(&pool, &room_id).await;

    assert!(should_fail.is_err())
}

#[tokio::test]
async fn can_update_plug() {
    let docker = Cli::default();

    let test_config = DatabaseTestConfig::new(&docker).await;
    let pool = Arc::new(test_config.db_config.pool);

    rooms::create_room(&pool, "test_room")
        .await
        .expect("Could not insert plug");
    let rooms = rooms::get_rooms(&pool).await.expect("Can't get rooms");
    let room_id = rooms[0].clone().id;

    let new_plug = plug(&room_id);

    plugs::create_plug(&pool, new_plug)
        .await
        .expect("Could not insert plug");

    let stored = plugs::get_plugs(&pool).await.expect("Can't get plugs");
    let stored_plug = stored[0].clone();

    let updated_plug = Plug {
        id: stored_plug.id,
        name: "new_name".to_string(),
        ip: IpNetwork::from_str("127.0.0.2").expect("Could not create IP"),
        username: "new_uname".to_string(),
        password: "new_pass".to_string(),
        room_id,
    };

    plugs::update_plug(&pool, updated_plug.clone())
        .await
        .expect("Can't update plug");

    let result = plugs::get_plugs(&pool).await.expect("Can't get plugs");
    let result_plug = result[0].clone();

    assert_eq!(result_plug, updated_plug);
}

#[tokio::test]
async fn can_delete_plug() {
    let docker = Cli::default();

    let test_config = DatabaseTestConfig::new(&docker).await;
    let pool = Arc::new(test_config.db_config.pool);
    rooms::create_room(&pool, "test_room")
        .await
        .expect("Could not insert room");
    let rooms = rooms::get_rooms(&pool).await.expect("Can't get rooms");
    let room_id = rooms[0].clone().id;

    let new_plug = plug(&room_id);

    plugs::create_plug(&pool, new_plug)
        .await
        .expect("Could not insert plug");

    let stored = plugs::get_plugs(&pool).await.expect("Can't get plugs");

    let id = stored[0].clone().id;

    plugs::delete_plug(&pool, &id)
        .await
        .expect("Failed to delete plug");

    let result = plugs::get_plugs(&pool).await.expect("Can't get plugs");
    assert_eq!(result.len(), 0)
}

#[tokio::test]
async fn schedules() {
    let docker = Cli::default();

    let test_config = DatabaseTestConfig::new(&docker).await;
    let pool = Arc::new(test_config.db_config.pool);
    rooms::create_room(&pool, "test_room")
        .await
        .expect("Could not insert room");
    rooms::create_room(&pool, "test_room_2")
        .await
        .expect("Could not insert room");

    let rooms = rooms::get_rooms(&pool).await.expect("Can't get rooms");
    let room_id_2 = rooms[1].clone().id;

    let new_schedule = setup::schedule(vec![&rooms[0]]);

    schedules::create_schedule(&pool, new_schedule.clone())
        .await
        .expect("Could not insert schedule");

    let stored = schedules::get_schedules(&pool)
        .await
        .expect("Can't get schedules");

    let stored_schedule = stored[0].clone();

    let update_expected = Schedule {
        id: stored_schedule.id,
        temps: HashMap::from([
            (PriceLevel::VeryCheap, 18.0),
            (PriceLevel::VeryExpensive, 22.0),
        ]),
        days: vec![Weekday::Fri],
        time_windows: vec![(NaiveTime::from_hms(1, 0, 0), NaiveTime::from_hms(2, 0, 0))],
        room_ids: vec![room_id_2],
    };

    schedules::update_schedule(&pool, update_expected.clone())
        .await
        .expect("Could not update schedule");

    let stored = schedules::get_schedules(&pool)
        .await
        .expect("Can't get schedules");

    let stored_schedule = stored[0].clone();

    assert_eq!(stored_schedule, update_expected);

    let room_schedules = schedules::get_room_schedules(&pool, &room_id_2)
        .await
        .expect("Cant get room schedules");
    assert_eq!(room_schedules[0], stored_schedule);

    let current_active = schedules::get_matching_schedule(
        &pool,
        &room_id_2,
        &NaiveDateTime::new(
            NaiveDate::from_weekday_of_month(2020, 11, Weekday::Fri, 1),
            NaiveTime::from_hms(1, 30, 0),
        ),
    )
    .await
    .expect("Couldn't fetch schedule");
    assert_eq!(current_active, Some(update_expected));
    let current_active = schedules::get_matching_schedule(
        &pool,
        &room_id_2,
        &NaiveDateTime::new(
            NaiveDate::from_weekday_of_month(2020, 11, Weekday::Sat, 1),
            NaiveTime::from_hms(1, 30, 0),
        ),
    )
    .await
    .expect("Couldn't fetch schedule");
    assert_eq!(current_active, None)
}

#[tokio::test]
async fn can_delete_schedule() {
    let docker = Cli::default();

    let test_config = DatabaseTestConfig::new(&docker).await;
    let pool = Arc::new(test_config.db_config.pool);
    rooms::create_room(&pool, "test_room")
        .await
        .expect("Could not insert room");

    let rooms = rooms::get_rooms(&pool).await.expect("Can't get rooms");

    let new_schedule = setup::schedule(vec![&rooms[0]]);

    schedules::create_schedule(&pool, new_schedule.clone())
        .await
        .expect("Could not insert schedule");

    let stored = schedules::get_schedules(&pool)
        .await
        .expect("Can't get schedules");

    let stored_schedule = stored[0].clone();

    schedules::delete_schedule(&pool, &stored_schedule.id)
        .await
        .expect("Could not delete schedule");

    let stored = schedules::get_schedules(&pool)
        .await
        .expect("Can't get schedules");

    assert_eq!(stored.len(), 0);
}

#[tokio::test]
async fn schedules_constraints() {
    let docker = Cli::default();

    let test_config = DatabaseTestConfig::new(&docker).await;
    let pool = Arc::new(test_config.db_config.pool);
    rooms::create_room(&pool, "test_room")
        .await
        .expect("Could not insert room");

    let rooms = rooms::get_rooms(&pool).await.expect("Can't get rooms");

    let new_schedule = setup::schedule(vec![&rooms[0]]);

    schedules::create_schedule(&pool, new_schedule.clone())
        .await
        .expect("Could not insert schedule");

    let stored = schedules::get_schedules(&pool)
        .await
        .expect("Can't get schedules");

    assert_eq!(stored[0], new_schedule);

    let mut time_windows = stored[0].time_windows.clone();
    time_windows.push((
        time_windows[0].0,
        time_windows[0].1.add(Duration::minutes(1)),
    ));

    let duplicate_times = schedules::update_schedule(
        &pool,
        Schedule {
            id: stored[0].id,
            temps: stored[0].temps.clone(),
            days: stored[0].days.clone(),
            time_windows,
            room_ids: stored[0].room_ids.clone(),
        },
    )
    .await;

    assert!(duplicate_times.is_err());
}

#[tokio::test]
async fn temp_actions() {
    let docker = Cli::default();

    let test_config = DatabaseTestConfig::new(&docker).await;
    let pool = Arc::new(test_config.db_config.pool);
    rooms::create_room(&pool, "test_room")
        .await
        .expect("Could not insert room");
    rooms::create_room(&pool, "test_room_2")
        .await
        .expect("Could not insert room");

    let rooms = rooms::get_rooms(&pool).await.expect("Can't get rooms");
    let room_id_1 = rooms[0].clone().id;
    let room_id_2 = rooms[1].clone().id;

    let new_action = temp_action(vec![room_id_1]);

    temp_actions::create_temp_action(&pool, new_action)
        .await
        .expect("Failed to insert temp action");

    let stored = temp_actions::get_temp_actions(&pool)
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

    temp_actions::update_temp_action(&pool, updated_action.clone())
        .await
        .expect("Failed to update temp action");

    let after_update = temp_actions::get_temp_actions(&pool)
        .await
        .expect("Failed to get temp actions");

    let after_update_action = after_update[0].clone();

    assert_eq!(after_update_action, updated_action);

    temp_actions::delete_temp_action(&pool, &after_update_action.id)
        .await
        .expect("Failed to delete temp action");
    let after_delete = temp_actions::get_temp_actions(&pool)
        .await
        .expect("Failed to get temp actions");
    assert_eq!(after_delete.len(), 0)
}

#[tokio::test]
async fn temperature_logs() {
    let docker = Cli::default();

    let test_config = DatabaseTestConfig::new(&docker).await;
    let pool = Arc::new(test_config.db_config.pool);
    rooms::create_room(&pool, "test_room")
        .await
        .expect("Could not insert room");

    let rooms = rooms::get_rooms(&pool).await.expect("Can't get rooms");
    let room_id = rooms[0].clone().id;

    let log_entry = temperature_log(room_id);

    temperature_logs::create_temp_log(&pool, log_entry.clone())
        .await
        .expect("Failed to insert temp action");

    let duplicate = temperature_logs::create_temp_log(&pool, log_entry).await;

    assert!(duplicate.is_err());

    for _ in 0..1000 {
        temperature_logs::create_temp_log(&pool, temperature_log(room_id))
            .await
            .expect("Could not create temp_log")
    }

    let stored = temperature_logs::get_temp_logs(&pool)
        .await
        .expect("Failed to get temp actions");

    assert_eq!(stored.len(), 1001)
}

#[tokio::test]
async fn latest_temp_logs() {
    let docker = Cli::default();

    let test_config = DatabaseTestConfig::new(&docker).await;
    let pool = Arc::new(test_config.db_config.pool);

    for i in 1..5 {
        rooms::create_room(&pool, format!("room_{}", i).as_str())
            .await
            .expect("Could not insert room");
    }

    let rooms = rooms::get_rooms(&pool).await.expect("Can't get rooms");

    let time = NaiveDateTime::new(
        NaiveDate::from_ymd(2022, 1, 1),
        NaiveTime::from_hms(1, 0, 0),
    );
    for room in rooms.clone() {
        for i in 1..=100 {
            temperature_logs::create_temp_log(
                &pool,
                TemperatureLog {
                    room_id: room.id,
                    time: time + Duration::minutes(i),
                    temp: (i as f64 / 10.0),
                },
            )
            .await
            .expect("Failed to insert temperature log")
        }
    }

    let current_temps = temperature_logs::get_current_temps(&pool, &rooms)
        .await
        .expect("Couldn't get latest temps");
    assert_eq!(current_temps.len(), 4);
    for (_, room_temp) in current_temps {
        assert_eq!(room_temp.temp, 10.0)
    }

    rooms::create_room(&pool, "dummy")
        .await
        .expect("Cant create room");
    let new_rooms = rooms::get_rooms(&pool).await.expect("Cant get rooms");
    let new_room = new_rooms
        .iter()
        .find(|room| room.name == "dummy")
        .expect("Couldnt find room");

    let current_non_existing = temperature_logs::get_current_temps(&pool, &vec![new_room.clone()])
        .await
        .expect("Couldnt get temps");
    assert_eq!(current_non_existing.get(&new_room.id), None)
}
