use std::ops::{Add, Sub};
use std::sync::Arc;

use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime, Utc, Weekday};
use testcontainers::clients::Cli;
use tokio::sync::mpsc;
use wiremock::{Mock, MockServer, ResponseTemplate};
use wiremock::matchers::any;

use rust_home::clients::shelly_client::ShellyClient;
use rust_home::clients::tibber_client::TibberClient;
use rust_home::db;
use rust_home::db::DbConfig;
use rust_home::domain::{
    ActionType, Button, Plug, PriceInfo, PriceLevel, Room, TempAction, TempActionType,
    TemperatureLog, WorkMessage,
};
use rust_home::work_handler::WorkHandler;

use crate::configuration::DatabaseTestConfig;

mod configuration;
mod setup;

async fn setup(
    db_config: &DbConfig,
    num_rooms: u32,
    shelly_port: Option<u16>,
) -> (WorkHandler, Vec<Room>) {
    let shelly_client = if let Some(shelly_port) = shelly_port {
        ShellyClient::new_with_port(shelly_port)
    } else {
        ShellyClient::default()
    };

    let (sender, receiver) = mpsc::channel::<WorkMessage>(32);
    let tibber_client = Arc::new(TibberClient::new("dummy_token".to_string()));
    let shelly_client = Arc::new(shelly_client);
    let handler = WorkHandler::new(
        shelly_client,
        tibber_client,
        sender.clone(),
        receiver,
        Arc::new(db_config.pool.clone()),
    );
    for i in 0..num_rooms {
        db::rooms::create_room(&db_config.pool, &format!("test_room_{}", i), &None)
            .await
            .expect("Failed to create room");
    }

    let rooms = db::rooms::get_rooms(&db_config.pool)
        .await
        .expect("Failed to get rooms");
    (handler, rooms)
}

#[tokio::test]
async fn starts() {
    let docker = Cli::default();
    let test_config = DatabaseTestConfig::new(&docker).await;
    let (handler, _rooms) = setup(&test_config.db_config, 0, None).await;

    let price_info = PriceInfo {
        amount: 0.0,
        currency: "NOK".to_string(),
        ext_price_level: PriceLevel::Cheap,
        price_level: None,
        starts_at: Utc::now().naive_local(),
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
    let docker = Cli::default();
    let test_config = DatabaseTestConfig::new(&docker).await;
    let (handler, rooms) = setup(&test_config.db_config, 1, None).await;

    let room_id = rooms[0].id;
    handler
        .temperature_handler(&room_id, &20.0)
        .await
        .expect("Temp handler failed");
    let temp_logs = db::temperature_logs::get_temp_logs(&test_config.db_config.pool)
        .await
        .expect("Failed to get temp_logs");
    assert_eq!(temp_logs.len(), 1);
    assert_eq!(temp_logs[0].temp, 20.0);
    assert_eq!(temp_logs[0].room_id, room_id);
}

#[tokio::test]
async fn temp_actions_work() {
    let docker = Cli::default();
    let test_config = DatabaseTestConfig::new(&docker).await;
    let mock_server = MockServer::start().await;

    let mock_ip = mock_server.address().ip().to_string();
    let mock_port = mock_server.address().port();

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    let (handler, rooms) = setup(&test_config.db_config, 2, Some(mock_port)).await;
    let now = NaiveDateTime::new(
        NaiveDate::from_weekday_of_month(2020, 1, Weekday::Mon, 1),
        NaiveTime::from_hms(1, 0, 0),
    );

    db::temperature_logs::create_temp_log(
        &test_config.db_config.pool,
        TemperatureLog {
            room_id: rooms[0].id,
            temp: 18.5,
            time: now.sub(Duration::minutes(30)),
        },
    )
    .await
    .expect("Failed to create temp log");

    let new_plug = Plug::new("test", &mock_ip, "admin", "password", &rooms[0].id, &true)
        .expect("Couldnt create plug");
    db::plugs::create_plug(&test_config.db_config.pool, &new_plug)
        .await
        .expect("Couldnt insert plug");

    let schedule = setup::schedule(vec![&rooms[0]]);

    db::schedules::create_schedule(&test_config.db_config.pool, schedule)
        .await
        .expect("Could insert schedule");

    handler
        .main_handler(
            &PriceInfo {
                ext_price_level: PriceLevel::Normal,
                amount: 20.0,
                currency: "USD".to_string(),
                starts_at: Utc::now().naive_local(),
                price_level: None,
            },
            &now,
        )
        .await
        .expect("Handler failed");

    let received_requests = mock_server.received_requests().await.unwrap();
    assert_eq!(received_requests.len(), 1);
    let query_param = received_requests[0].url.query().expect("Missing query");
    assert_eq!(query_param, "turn=on");
    mock_server.reset().await;

    db::temp_actions::create_temp_action(
        &test_config.db_config.pool,
        TempAction::new(
            &None,
            &now.add(Duration::hours(1)),
            &TempActionType::OFF,
            vec![rooms[0].id],
        ),
    )
    .await
    .expect("Failed to insert temp action");

    handler
        .main_handler(
            &PriceInfo {
                ext_price_level: PriceLevel::Normal,
                amount: 20.0,
                currency: "USD".to_string(),
                starts_at: Utc::now().naive_local(),
                price_level: None,
            },
            &now,
        )
        .await
        .expect("Handler failed");

    let received_requests = mock_server.received_requests().await.unwrap();
    assert_eq!(received_requests.len(), 1);
    let query_param = received_requests[0].url.query().expect("Missing query");
    assert_eq!(query_param, "turn=off");
}

#[tokio::test]
async fn temp_actions_with_start_time_work() {
    let docker = Cli::default();
    let test_config = DatabaseTestConfig::new(&docker).await;
    let mock_server = MockServer::start().await;

    let mock_ip = mock_server.address().ip().to_string();
    let mock_port = mock_server.address().port();

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    let (handler, rooms) = setup(&test_config.db_config, 2, Some(mock_port)).await;
    let now = NaiveDateTime::new(
        NaiveDate::from_weekday_of_month(2020, 1, Weekday::Mon, 1),
        NaiveTime::from_hms(1, 0, 0),
    );

    db::temperature_logs::create_temp_log(
        &test_config.db_config.pool,
        TemperatureLog {
            room_id: rooms[0].id,
            temp: 18.5,
            time: now.sub(Duration::minutes(30)),
        },
    )
    .await
    .expect("Failed to create temp log");

    let new_plug = Plug::new("test", &mock_ip, "admin", "password", &rooms[0].id, &true)
        .expect("Couldnt create plug");
    db::plugs::create_plug(&test_config.db_config.pool, &new_plug)
        .await
        .expect("Couldnt insert plug");

    let schedule = setup::schedule(vec![&rooms[0]]);

    db::schedules::create_schedule(&test_config.db_config.pool, schedule)
        .await
        .expect("Could insert schedule");

    db::temp_actions::create_temp_action(
        &test_config.db_config.pool,
        TempAction::new(
            &Some(now.add(Duration::minutes(2))),
            &now.add(Duration::hours(1)),
            &TempActionType::OFF,
            vec![rooms[0].id],
        ),
    )
    .await
    .expect("Failed to insert temp action");

    handler
        .main_handler(
            &PriceInfo {
                ext_price_level: PriceLevel::Normal,
                amount: 20.0,
                currency: "USD".to_string(),
                starts_at: Utc::now().naive_local(),
                price_level: None,
            },
            &now,
        )
        .await
        .expect("Handler failed");

    let received_requests = mock_server.received_requests().await.unwrap();
    assert_eq!(received_requests.len(), 1);
    let query_param = received_requests[0].url.query().expect("Missing query");
    assert_eq!(query_param, "turn=on");
    mock_server.reset().await;

    handler
        .main_handler(
            &PriceInfo {
                ext_price_level: PriceLevel::Normal,
                amount: 20.0,
                currency: "USD".to_string(),
                starts_at: Utc::now().naive_local(),
                price_level: None,
            },
            &now.add(Duration::minutes(5)),
        )
        .await
        .expect("Handler failed");

    let received_requests = mock_server.received_requests().await.unwrap();
    assert_eq!(received_requests.len(), 1);
    let query_param = received_requests[0].url.query().expect("Missing query");
    assert_eq!(query_param, "turn=off");
}

#[tokio::test]
async fn temp_actions_override_existing_schedule_temp() {
    let docker = Cli::default();
    let test_config = DatabaseTestConfig::new(&docker).await;
    let mock_server = MockServer::start().await;

    let mock_ip = mock_server.address().ip().to_string();
    let mock_port = mock_server.address().port();

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    let (handler, rooms) = setup(&test_config.db_config, 2, Some(mock_port)).await;
    let now = NaiveDateTime::new(
        NaiveDate::from_weekday_of_month(2020, 1, Weekday::Mon, 1),
        NaiveTime::from_hms(1, 0, 0),
    );

    db::temperature_logs::create_temp_log(
        &test_config.db_config.pool,
        TemperatureLog {
            room_id: rooms[0].id,
            temp: 21.0,
            time: now.sub(Duration::minutes(30)),
        },
    )
    .await
    .expect("Failed to create temp log");

    let new_plug = Plug::new("test", &mock_ip, "admin", "password", &rooms[0].id, &true)
        .expect("Couldnt create plug");
    db::plugs::create_plug(&test_config.db_config.pool, &new_plug)
        .await
        .expect("Couldnt insert plug");

    let schedule = setup::schedule(vec![&rooms[0]]);

    db::schedules::create_schedule(&test_config.db_config.pool, schedule)
        .await
        .expect("Could insert schedule");

    db::temp_actions::create_temp_action(
        &test_config.db_config.pool,
        TempAction::new(
            &None,
            &now.add(Duration::hours(1)),
            &TempActionType::ON(Some(24.0)),
            vec![rooms[0].id],
        ),
    )
    .await
    .expect("Failed to insert temp action");

    handler
        .main_handler(
            &PriceInfo {
                ext_price_level: PriceLevel::Normal,
                amount: 20.0,
                currency: "USD".to_string(),
                starts_at: Utc::now().naive_local(),
                price_level: None,
            },
            &now,
        )
        .await
        .expect("Handler failed");

    let received_requests = mock_server.received_requests().await.unwrap();
    assert_eq!(received_requests.len(), 1);
    let query_param = received_requests[0].url.query().expect("Missing query");
    assert_eq!(query_param, "turn=on");

    let actions = db::temp_actions::get_temp_actions(&test_config.db_config.pool)
        .await
        .expect("Failed to get temp actions");
    db::temp_actions::delete_temp_action(&test_config.db_config.pool, &actions[0].id)
        .await
        .expect("Failed to delete temp action");

    mock_server.reset().await;

    db::temp_actions::create_temp_action(
        &test_config.db_config.pool,
        TempAction::new(
            &None,
            &now.add(Duration::hours(1)),
            &TempActionType::ON(Some(14.0)),
            vec![rooms[0].id],
        ),
    )
    .await
    .expect("Failed to insert temp action");

    handler
        .main_handler(
            &PriceInfo {
                ext_price_level: PriceLevel::Normal,
                amount: 20.0,
                currency: "USD".to_string(),
                starts_at: Utc::now().naive_local(),
                price_level: None,
            },
            &now,
        )
        .await
        .expect("Handler failed");

    let received_requests = mock_server.received_requests().await.unwrap();
    assert_eq!(received_requests.len(), 1);
    let query_param = received_requests[0].url.query().expect("Missing query");
    assert_eq!(query_param, "turn=off");
}

#[tokio::test]
async fn min_temp_overrides_temp_action() {
    let docker = Cli::default();
    let test_config = DatabaseTestConfig::new(&docker).await;
    let mock_server = MockServer::start().await;

    let mock_ip = mock_server.address().ip().to_string();
    let mock_port = mock_server.address().port();

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    let (handler, rooms) = setup(&test_config.db_config, 2, Some(mock_port)).await;
    let now = NaiveDateTime::new(
        NaiveDate::from_weekday_of_month(2020, 1, Weekday::Mon, 1),
        NaiveTime::from_hms(1, 0, 0),
    );

    db::temperature_logs::create_temp_log(
        &test_config.db_config.pool,
        TemperatureLog {
            room_id: rooms[0].id,
            temp: 14.0,
            time: now.sub(Duration::minutes(30)),
        },
    )
    .await
    .expect("Failed to create temp log");

    let new_plug = Plug::new("test", &mock_ip, "admin", "password", &rooms[0].id, &true)
        .expect("Couldnt create plug");
    db::plugs::create_plug(&test_config.db_config.pool, &new_plug)
        .await
        .expect("Couldnt insert plug");

    db::rooms::update_room(
        &test_config.db_config.pool,
        &Room {
            id: rooms[0].id,
            name: rooms[0].name.clone(),
            min_temp: Some(22.0),
        },
    )
    .await
    .expect("Failed to update room");

    handler
        .main_handler(
            &PriceInfo {
                ext_price_level: PriceLevel::Normal,
                amount: 20.0,
                currency: "USD".to_string(),
                starts_at: Utc::now().naive_local(),
                price_level: None,
            },
            &now,
        )
        .await
        .expect("Handler failed");

    let received_requests = mock_server.received_requests().await.unwrap();
    assert_eq!(received_requests.len(), 1);
    let query_param = received_requests[0].url.query().expect("Missing query");
    assert_eq!(query_param, "turn=on");

    mock_server.reset().await;

    db::temperature_logs::create_temp_log(
        &test_config.db_config.pool,
        TemperatureLog {
            room_id: rooms[0].id,
            temp: 23.0,
            time: now.sub(Duration::minutes(20)),
        },
    )
    .await
    .expect("Failed to create temp log");

    handler
        .main_handler(
            &PriceInfo {
                ext_price_level: PriceLevel::Normal,
                amount: 20.0,
                currency: "USD".to_string(),
                starts_at: Utc::now().naive_local(),
                price_level: None,
            },
            &now,
        )
        .await
        .expect("Handler failed");

    let received_requests = mock_server.received_requests().await.unwrap();
    assert_eq!(received_requests.len(), 1);
    let query_param = received_requests[0].url.query().expect("Missing query");
    assert_eq!(query_param, "turn=off");

    mock_server.reset().await;

    let schedule = setup::schedule(vec![&rooms[0]]);

    db::schedules::create_schedule(&test_config.db_config.pool, schedule)
        .await
        .expect("Could insert schedule");

    db::temperature_logs::create_temp_log(
        &test_config.db_config.pool,
        TemperatureLog {
            room_id: rooms[0].id,
            temp: 21.0,
            time: now.sub(Duration::minutes(10)),
        },
    )
    .await
    .expect("Failed to create temp log");

    handler
        .main_handler(
            &PriceInfo {
                ext_price_level: PriceLevel::Normal,
                amount: 20.0,
                currency: "USD".to_string(),
                starts_at: Utc::now().naive_local(),
                price_level: None,
            },
            &now,
        )
        .await
        .expect("Handler failed");

    let received_requests = mock_server.received_requests().await.unwrap();
    assert_eq!(received_requests.len(), 1);
    let query_param = received_requests[0].url.query().expect("Missing query");
    assert_eq!(query_param, "turn=on");
}

#[tokio::test]
async fn button_handler() {
    let docker = Cli::default();
    let test_config = DatabaseTestConfig::new(&docker).await;
    let mock_server = MockServer::start().await;

    let mock_ip = mock_server.address().ip().to_string();
    let mock_port = mock_server.address().port();

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    let (handler, rooms) = setup(&test_config.db_config, 2, Some(mock_port)).await;

    let new_plug = Plug::new("test", &mock_ip, "admin", "password", &rooms[0].id, &false)
        .expect("Couldnt create plug");
    db::plugs::create_plug(&test_config.db_config.pool, &new_plug)
        .await
        .expect("Couldnt insert plug");

    let new_button = Button::new("test", "127.0.0.1", "a", "b", &[new_plug.id])
        .expect("Failed to create button");
    db::buttons::create_button(&test_config.db_config.pool, &new_button)
        .await
        .expect("failed to insert button");

    handler
        .button_handler(&new_button.id, &ActionType::ON)
        .await
        .expect("Handler failed");

    let received_requests = mock_server.received_requests().await.unwrap();
    assert_eq!(received_requests.len(), 1);
    let query_param = received_requests[0].url.query().expect("Missing query");
    assert_eq!(query_param, "turn=on");

    mock_server.reset().await;

    handler
        .button_handler(&new_button.id, &ActionType::OFF)
        .await
        .expect("Handler failed");

    let received_requests = mock_server.received_requests().await.unwrap();
    assert_eq!(received_requests.len(), 1);
    let query_param = received_requests[0].url.query().expect("Missing query");
    assert_eq!(query_param, "turn=off");

    mock_server.reset().await;

    let now = NaiveDateTime::new(
        NaiveDate::from_weekday_of_month(2020, 1, Weekday::Mon, 1),
        NaiveTime::from_hms(1, 0, 0),
    );

    handler
        .main_handler(
            &PriceInfo {
                ext_price_level: PriceLevel::Normal,
                amount: 20.0,
                currency: "USD".to_string(),
                starts_at: Utc::now().naive_local(),
                price_level: None,
            },
            &now,
        )
        .await
        .expect("Handler failed");

    let received_requests = mock_server.received_requests().await.unwrap();
    assert_eq!(received_requests.len(), 0);
}
