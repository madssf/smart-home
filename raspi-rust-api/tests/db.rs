use std::collections::HashMap;
use std::ops::{Add, Sub};
use std::str::FromStr;
use std::sync::Arc;

use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime, Utc, Weekday};
use sqlx::types::ipnetwork::IpNetwork;
use sqlx::PgPool;
use testcontainers::clients::Cli;
use uuid::Uuid;

use configuration::DatabaseTestConfig;
use rust_home::db;
use rust_home::db::{plugs, rooms, schedules, temp_actions, temperature_logs};
use rust_home::domain::{
    Button, NotificationSettings, Plug, PriceInfo, PriceLevel, Room, Schedule, TempAction,
    TempActionType, TempSensor, TemperatureLog,
};

mod configuration;
mod setup;

fn plug(room_id: &Uuid) -> Plug {
    Plug::new("test_plug", "127.0.0.1", "username", "password", room_id)
        .expect("Could not create plug")
}

fn temp_action(room_ids: Vec<Uuid>) -> TempAction {
    TempAction::new(
        &NaiveDateTime::from_timestamp(1666291743, 0),
        &TempActionType::ON(Some(22.0)),
        room_ids,
    )
}

fn temperature_log(room_id: Uuid) -> TemperatureLog {
    TemperatureLog {
        room_id,
        time: Utc::now().naive_utc(),
        temp: 20.0,
    }
}

async fn create_room(pool: &PgPool) {
    rooms::create_room(pool, "test_room", &None)
        .await
        .expect("Could not insert room");
}

#[tokio::test]
async fn rooms() {
    let docker = Cli::default();

    let test_config = DatabaseTestConfig::new(&docker).await;
    let pool = Arc::new(test_config.db_config.pool);
    create_room(&pool).await;

    let result = rooms::get_rooms(&pool).await.expect("Can't get rooms");

    let result_room = result[0].clone();

    assert_eq!(result_room.name, "test_room");
    assert_eq!(result_room.min_temp, None);

    rooms::update_room(
        &pool,
        &Room {
            id: result_room.id,
            name: "test2".to_string(),
            min_temp: Some(20.0),
        },
    )
    .await
    .expect("Couldn't update room");

    let result = rooms::get_rooms(&pool).await.expect("Can't get rooms");
    let result_room = result[0].clone();
    assert_eq!(result_room.name, "test2");
    assert_eq!(result_room.min_temp, Some(20.0));
}

#[tokio::test]
async fn can_insert_plug() {
    let docker = Cli::default();

    let test_config = DatabaseTestConfig::new(&docker).await;
    let pool = Arc::new(test_config.db_config.pool);

    create_room(&pool).await;

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

    create_room(&pool).await;

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
    create_room(&pool).await;

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

    create_room(&pool).await;

    rooms::create_room(&pool, "test_room_2", &None)
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
    create_room(&pool).await;

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
    create_room(&pool).await;

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

    create_room(&pool).await;

    rooms::create_room(&pool, "test_room_2", &None)
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
        action_type: TempActionType::ON(Some(23.0)),
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

    create_room(&pool).await;

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
        rooms::create_room(&pool, format!("room_{}", i).as_str(), &None)
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

    rooms::create_room(&pool, "dummy", &None)
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

#[tokio::test]
async fn prices() {
    let docker = Cli::default();

    let test_config = DatabaseTestConfig::new(&docker).await;
    let pool = Arc::new(test_config.db_config.pool);
    let date = NaiveDate::from_ymd(2020, 1, 1);
    let start_time =
        NaiveDateTime::new(date, NaiveTime::from_hms(0, 0, 0)).sub(Duration::seconds(1));
    let end_time =
        NaiveDateTime::new(date, NaiveTime::from_hms(23, 0, 0)).add(Duration::seconds(1));
    let mut some_prices: Vec<PriceInfo> = vec![];
    for i in 0..24 {
        some_prices.push({
            PriceInfo {
                amount: i as f64,
                currency: "NOK".to_string(),
                ext_price_level: PriceLevel::Normal,
                price_level: Some(PriceLevel::Cheap),
                starts_at: NaiveDateTime::new(date, NaiveTime::from_hms(i, 0, 0)),
            }
        })
    }

    db::prices::insert_prices(&pool, &some_prices)
        .await
        .expect("Failed to insert prices");
    let stored = db::prices::get_prices(&pool, &start_time, &end_time)
        .await
        .expect("Failed to get prices");
    assert_eq!(some_prices.len(), 24);
    assert_eq!(stored.len(), 24);
    assert_eq!(stored, some_prices);

    let mut new_prices: Vec<PriceInfo> = vec![];
    for i in 12..=23 {
        new_prices.push({
            PriceInfo {
                amount: i as f64 * 1.5,
                currency: "NOK".to_string(),
                ext_price_level: PriceLevel::VeryExpensive,
                price_level: Some(PriceLevel::VeryCheap),
                starts_at: NaiveDateTime::new(date, NaiveTime::from_hms(i, 0, 0)),
            }
        })
    }
    db::prices::insert_prices(&pool, &new_prices)
        .await
        .expect("Failed to insert prices");

    let stored = db::prices::get_prices(&pool, &start_time, &end_time)
        .await
        .expect("Failed to get prices");
    assert_eq!(&stored[0..12], &some_prices[0..12]);

    assert_eq!(&stored[12..24], &new_prices);

    let current = db::prices::get_price(&pool, &end_time.add(Duration::minutes(59)))
        .await
        .expect("failed to fetch price");
    assert_eq!(
        current,
        Some(PriceInfo {
            amount: 23.0 * 1.5,
            currency: "NOK".to_string(),
            ext_price_level: PriceLevel::VeryExpensive,
            price_level: Some(PriceLevel::VeryCheap),
            starts_at: NaiveDateTime::new(date, NaiveTime::from_hms(23, 0, 0)),
        })
    )
}

#[tokio::test]
async fn settings() {
    let docker = Cli::default();

    let test_config = DatabaseTestConfig::new(&docker).await;
    let pool = Arc::new(test_config.db_config.pool);

    let settings = db::notification_settings::get_notification_settings(pool.as_ref())
        .await
        .expect("Failed to get settings");
    assert_eq!(settings, None);

    db::notification_settings::upsert_notification_settings(
        pool.as_ref(),
        &NotificationSettings {
            id: Some(1),
            max_consumption: Some(3),
            max_consumption_timeout_minutes: 15,
            ntfy_topic: "test_topic".to_string(),
        },
    )
    .await
    .expect("Failed to upsert settings");

    let settings = db::notification_settings::get_notification_settings(pool.as_ref())
        .await
        .expect("Failed to get settings");
    assert_eq!(
        settings,
        Some(NotificationSettings {
            id: Some(1),
            max_consumption: Some(3),
            max_consumption_timeout_minutes: 15,
            ntfy_topic: "test_topic".to_string(),
        })
    );

    db::notification_settings::upsert_notification_settings(
        pool.as_ref(),
        &NotificationSettings {
            id: Some(1),
            max_consumption: Some(10),
            max_consumption_timeout_minutes: 20,
            ntfy_topic: "test_topic".to_string(),
        },
    )
    .await
    .expect("Failed to upsert settings");

    let settings = db::notification_settings::get_notification_settings(pool.as_ref())
        .await
        .expect("Failed to get settings");
    assert_eq!(
        settings,
        Some(NotificationSettings {
            id: Some(1),
            max_consumption: Some(10),
            max_consumption_timeout_minutes: 20,
            ntfy_topic: "test_topic".to_string(),
        })
    );
}

#[tokio::test]
async fn temp_sensors() {
    let docker = Cli::default();

    let test_config = DatabaseTestConfig::new(&docker).await;
    let pool = Arc::new(test_config.db_config.pool);

    let sensors = db::temp_sensors::get_temp_sensors(pool.as_ref())
        .await
        .expect("Failed to get sensors");
    assert_eq!(sensors.len(), 0);

    create_room(&pool).await;

    let rooms = rooms::get_rooms(&pool).await.expect("Can't get rooms");
    let room_id_1 = rooms[0].clone().id;

    let sensor_1 = TempSensor {
        id: "0x00158d0008072632".to_string(),
        room_id: room_id_1,
        battery_level: None,
    };

    db::temp_sensors::insert_temp_sensor(pool.as_ref(), &sensor_1)
        .await
        .expect("Failed to insert temp sensor");

    let sensors = db::temp_sensors::get_temp_sensors(&pool)
        .await
        .expect("Couldn't get sensors");
    assert_eq!(sensors.len(), 1);
    assert_eq!(sensors[0], sensor_1);
    let sensor = db::temp_sensors::get_temp_sensor(&pool, "0x00158d0008072632")
        .await
        .expect("Couldn't get sensor");
    assert_eq!(sensor, Some(sensor_1));
    db::temp_sensors::delete_temp_sensor(&pool, "0x00158d0008072632")
        .await
        .expect("Couldn't delete sensor");
    let sensors = db::temp_sensors::get_temp_sensors(&pool)
        .await
        .expect("Couldn't get sensors");
    assert_eq!(sensors.len(), 0);
}

#[tokio::test]
async fn buttons() {
    let docker = Cli::default();

    let test_config = DatabaseTestConfig::new(&docker).await;
    let pool = Arc::new(test_config.db_config.pool);

    let buttons = db::buttons::get_buttons(pool.as_ref())
        .await
        .expect("Failed to get buttons");
    assert_eq!(buttons.len(), 0);

    create_room(&pool).await;

    let rooms = rooms::get_rooms(&pool).await.expect("Can't get rooms");
    let room_id_1 = rooms[0].clone().id;

    plugs::create_plug(&pool, plug(&room_id_1))
        .await
        .expect("Failed to create plug");
    plugs::create_plug(
        &pool,
        Plug::new(
            "test_plug_2",
            "127.0.0.2",
            "username",
            "password",
            &room_id_1,
        )
        .expect("Failed to create plug"),
    )
    .await
    .expect("Failed to create plug");
    let plugs = plugs::get_plugs(&pool).await.expect("Failed to get plugs");
    let plug_id_1 = plugs[0].clone().id;
    let plug_id_2 = plugs[1].clone().id;

    let button_1 = Button {
        id: Uuid::new_v4(),
        name: "test button".to_string(),
        ip: IpNetwork::from_str("127.0.0.1").expect("Failed to create IP"),
        username: "aaa".to_string(),
        password: "aaa".to_string(),
        plug_ids: vec![plug_id_1],
    };

    db::buttons::create_button(pool.as_ref(), &button_1)
        .await
        .expect("Failed to insert button");

    let buttons = db::buttons::get_buttons(&pool)
        .await
        .expect("Couldn't get buttons");
    assert_eq!(buttons.len(), 1);
    assert_eq!(buttons[0], button_1);

    let updated_button = Button {
        id: button_1.id,
        name: "test 2".to_string(),
        ip: IpNetwork::from_str("127.0.1.24").expect("Failed to create IP"),
        username: "lol".to_string(),
        password: "lol".to_string(),
        plug_ids: vec![plug_id_1, plug_id_2],
    };

    db::buttons::update_button(&pool, &updated_button)
        .await
        .expect("Failed to update button");
    let buttons = db::buttons::get_buttons(&pool)
        .await
        .expect("Couldn't get buttons");
    assert_eq!(buttons.len(), 1);
    assert_eq!(buttons[0], updated_button);

    db::buttons::delete_button(&pool, &button_1.id)
        .await
        .expect("Couldn't delete button");
    let buttons = db::buttons::get_buttons(&pool)
        .await
        .expect("Couldn't get buttons");
    assert_eq!(buttons.len(), 0);
}
