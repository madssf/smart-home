use std::net::TcpStream;
use std::sync::Arc;
use std::time::Duration;

use async_tungstenite::tungstenite::stream::MaybeTlsStream;
use async_tungstenite::tungstenite::{
    client::IntoClientRequest, connect, http::HeaderValue, Error as WSClientError, Message,
    WebSocket,
};
use chrono::NaiveDateTime;
use log::{debug, error, warn};
use serde::Deserialize;
use thiserror::Error;
use tokio::sync::Mutex;
use tokio::time::sleep;

use crate::domain::LiveConsumption;
use crate::env_var;
use crate::service::consumption_cache::ConsumptionCache;

#[derive(Error, Debug)]
pub enum PowerSubscriberError {
    #[error("WebSocketError: {0}")]
    WebSocketError(#[from] WSClientError),
    #[error("No ACK received")]
    NoAckReceived,
    #[error("JSON Deserialize Error: {0}")]
    JSONDeserializeError(#[from] serde_json::Error),
    #[error("Chrono parse error: {0}")]
    ChronoParseError(#[from] chrono::ParseError),
}

pub async fn live_power_subscriber(consumption_cache: Arc<Mutex<ConsumptionCache>>) {
    loop {
        let subscriber = start_subscriber(consumption_cache.clone()).await;
        error!("Subscriber failed, restarting - error: {:?}", subscriber);
    }
}

async fn start_subscriber(
    consumption_cache: Arc<Mutex<ConsumptionCache>>,
) -> Result<(), PowerSubscriberError> {
    let mut socket = establish_subscription().await?;

    loop {
        let message = socket.read_message()?;
        let message_text = if let Ok(text) = message.clone().into_text() {
            text
        } else {
            warn!(
                "Couldn't convert message to text, skipping - message: {:?}",
                message
            );
            continue;
        };
        let response: LiveMeasurementResponse = match serde_json::from_str(&message_text) {
            Ok(response) => response,
            Err(_) => {
                debug!(
                    "Failed to parse json, skipping - message text: {}",
                    message_text
                );
                continue;
            }
        };
        debug!(
            "{} - {}",
            response.payload.data.live_measurement.timestamp,
            response.payload.data.live_measurement.power
        );
        consumption_cache.lock().await.add(LiveConsumption {
            timestamp: NaiveDateTime::parse_from_str(
                &response.payload.data.live_measurement.timestamp,
                "%Y-%m-%dT%H:%M:%S%.f%z",
            )?,
            power: response.payload.data.live_measurement.power,
        });
        sleep(Duration::from_millis(500)).await
    }
}

async fn establish_subscription(
) -> Result<WebSocket<MaybeTlsStream<TcpStream>>, PowerSubscriberError> {
    let mut request = "wss://api.tibber.com/v1-beta/gql/subscriptions"
        .into_client_request()
        .unwrap();
    request.headers_mut().insert(
        "Sec-WebSocket-Protocol",
        HeaderValue::from_str("graphql-transport-ws").expect("Failed to set header value"),
    );

    let (mut socket, _) = connect(request)?;

    debug!("WebSocket connection to Tibber established");

    debug!("Sending init message");
    socket.write_message(Message::text(init_message()))?;
    let res = socket.read_message()?.into_text()?;
    match res.contains("connection_ack") {
        true => {
            debug!("Sending subscribe message");
            socket.write_message(Message::text(subscribe_message()))?;
            Ok(socket)
        }
        false => Err(PowerSubscriberError::NoAckReceived),
    }
}

fn init_message() -> String {
    format!(
        r#"
    {{
        "type": "connection_init",
        "payload": {{
            "token": "{}"
        }}
    }}
    "#,
        env_var("TIBBER_API_TOKEN")
    )
}

fn subscribe_message() -> String {
    format!(
        r#"
    {{
    "id": "1",
    "type": "subscribe",
    "payload": {{
        "variables": {{}},
        "extensions": {{}},
        "query": "subscription {{\n  liveMeasurement(homeId: \"{}\") {{\n    timestamp\n    power\n    accumulatedConsumption\n    accumulatedCost\n    currency\n    minPower\n    averagePower\n    maxPower\n  }}\n}}"
    }}
    }}
    "#,
        env_var("TIBBER_HOME_ID")
    )
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct LiveMeasurement {
    pub timestamp: String,
    pub power: i64,
    #[serde(rename = "accumulatedConsumption")]
    pub accumulated_consumption: f64,
    #[serde(rename = "accumulatedCost")]
    pub accumulated_cost: f64,
    pub currency: String,
    #[serde(rename = "minPower")]
    pub min_power: i64,
    #[serde(rename = "averagePower")]
    pub average_power: f64,
    #[serde(rename = "maxPower")]
    pub max_power: i64,
}

#[derive(Deserialize)]
struct Data {
    #[serde(rename = "liveMeasurement")]
    pub live_measurement: LiveMeasurement,
}

#[derive(Deserialize)]
struct Payload {
    pub data: Data,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct LiveMeasurementResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub payload: Payload,
}
