use std::net::TcpStream;
use std::sync::Arc;
use std::time::Duration;

use async_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use async_tungstenite::tungstenite::protocol::CloseFrame;
use async_tungstenite::tungstenite::stream::MaybeTlsStream;
use async_tungstenite::tungstenite::{
    client::IntoClientRequest, connect, http::HeaderValue, Error as WSClientError, Message,
    WebSocket,
};
use chrono::NaiveDateTime;
use log::{debug, error, info, warn};
use reqwest::header::InvalidHeaderValue;
use serde::Deserialize;
use thiserror::Error;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::domain::LiveConsumption;
use crate::service::consumption_cache::ConsumptionCache;
use crate::{env_var, now};

#[derive(Error, Debug)]
pub enum PowerSubscriberError {
    #[error("WebSocketError: {0}")]
    WebSocketError(#[from] WSClientError),
    #[error("No ACK received")]
    NoAckReceived,
    #[error("Server closed connection")]
    ServerClosedConnection,
    #[error("JSON Deserialize Error: {0}")]
    JSONDeserializeError(#[from] serde_json::Error),
    #[error("Chrono parse error: {0}")]
    ChronoParseError(#[from] chrono::ParseError),
    #[error("Invalid header value")]
    InvalidHeaderValue(#[from] InvalidHeaderValue),
    #[error("Unexpected error")]
    UnexpectedError,
}

pub struct TibberSubscriber {
    consumption_cache: Arc<RwLock<ConsumptionCache>>,
}

impl TibberSubscriber {
    pub fn new(consumption_cache: Arc<RwLock<ConsumptionCache>>) -> Self {
        Self { consumption_cache }
    }

    pub async fn subscribe(&self) -> Result<(), PowerSubscriberError> {
        let mut backoff_counter = 2;
        loop {
            let started_at = now();
            let subscription = self.run_subscriber().await;
            let failed_at = now();
            let backoff = if (failed_at - started_at) > chrono::Duration::minutes(5) {
                info!("Subscriber alive for more than 5 minutes, resetting backoff");
                2
            } else {
                backoff_counter
            };

            error!(
                "Subscriber failed, restarting in {} seconds - error: {:?}",
                backoff, subscription,
            );

            tokio::time::sleep(Duration::from_secs(backoff)).await;

            info!("Restarting subscriber now!");

            backoff_counter = if backoff < 60 { backoff * 2 } else { backoff };
        }
    }

    async fn establish_subscription(
        &self,
    ) -> Result<WebSocket<MaybeTlsStream<TcpStream>>, PowerSubscriberError> {
        let mut request =
            "wss://websocket-api.tibber.com/v1-beta/gql/subscriptions".into_client_request()?;

        request.headers_mut().insert(
            "Sec-WebSocket-Protocol",
            HeaderValue::from_str("graphql-transport-ws")?,
        );

        request
            .headers_mut()
            .insert("User-Agent", HeaderValue::from_str("smarthome/0.0.1")?);

        info!("Trying to establish subscription");

        let (mut socket, _) = connect(request)?;

        match socket.get_mut() {
            MaybeTlsStream::NativeTls(t) => t
                .get_mut()
                .set_read_timeout(Some(Duration::from_secs(10)))
                .expect("Failed to set read timeout"),
            _ => panic!("Unexpected TLS Stream"),
        }
        info!("WebSocket connection to Tibber established, sending init message");

        socket.write_message(Message::text(init_message()))?;

        info!("Init message sent");

        let res = socket.read_message()?.into_text()?;
        match res.contains("connection_ack") {
            true => {
                info!("Sending subscribe message");
                socket.write_message(Message::text(subscribe_message()))?;
                info!("Subscribe message sent");
                Ok(socket)
            }
            false => {
                socket.close(Some(CloseFrame {
                    code: CloseCode::Normal,
                    reason: Default::default(),
                }))?;
                warn!("No ACK received, sent close message");
                Err(PowerSubscriberError::NoAckReceived)
            }
        }
    }

    async fn run_subscriber(&self) -> Result<(), PowerSubscriberError> {
        let socket = self.establish_subscription().await?;
        info!("Subscription established, starting subscribe loop");
        let error = match self.subscribe_loop(socket).await {
            Ok(_) => {
                error!("Subscriber unexpectedly stopped");
                PowerSubscriberError::UnexpectedError
            }
            Err(error) => {
                error!("Subscriber loop returned an error: {:?}", error);
                error
            }
        };
        Err(error)
    }

    async fn subscribe_loop(
        &self,
        mut socket: WebSocket<MaybeTlsStream<TcpStream>>,
    ) -> Result<(), PowerSubscriberError> {
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
                    if !message_text.is_empty() {
                        if message_text.to_lowercase() == "going away" {
                            info!("Tibber sent close message, returning error to restart");
                            return Err(PowerSubscriberError::ServerClosedConnection);
                        }
                        warn!(
                            "Failed to parse json, skipping - message text: {}",
                            message_text
                        );
                    }
                    continue;
                }
            };
            debug!(
                "{} - {}",
                response.payload.data.live_measurement.timestamp,
                response.payload.data.live_measurement.power
            );
            self.consumption_cache
                .write()
                .await
                .add(LiveConsumption {
                    timestamp: NaiveDateTime::parse_from_str(
                        &response.payload.data.live_measurement.timestamp,
                        "%Y-%m-%dT%H:%M:%S%.f%z",
                    )?,
                    power: response.payload.data.live_measurement.power,
                })
                .await;
        }
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
    "id": "{}",
    "type": "subscribe",
    "payload": {{
        "variables": {{}},
        "extensions": {{}},
        "query": "subscription {{\n  liveMeasurement(homeId: \"{}\") {{\n    timestamp\n    power\n    accumulatedConsumption\n    accumulatedCost\n    currency\n    minPower\n    averagePower\n    maxPower\n  }}\n}}"
    }}
    }}
    "#,
        Uuid::new_v4(),
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
