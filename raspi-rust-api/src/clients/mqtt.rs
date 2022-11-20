use std::str;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

use log::{debug, error, info, warn};
use rumqttc::{
    AsyncClient, ClientError, ConnectionError, Event, Incoming, MqttOptions, QoS, SubscribeFilter,
};
use serde::Deserialize;
use sqlx::PgPool;
use thiserror::Error;
use tokio::sync::mpsc::Sender;

use crate::db;
use crate::db::DbError;
use crate::domain::WorkMessage;

pub struct MqttClient {
    host: String,
    base_topic: String,
    pool: Arc<PgPool>,
    sender: Sender<WorkMessage>,
}

#[derive(Error, Debug)]
pub enum MqttClientError {
    #[error("DbError: {0}")]
    DbError(#[from] DbError),
    #[error("ClientError: {0}")]
    ClientError(#[from] ClientError),
    #[error("ConnectionError: {0}")]
    ConnectionError(#[from] ConnectionError),
}

impl MqttClient {
    pub fn new(
        host: String,
        base_topic: String,
        pool: Arc<PgPool>,
        sender: Sender<WorkMessage>,
    ) -> Self {
        Self {
            host,
            base_topic,
            pool,
            sender,
        }
    }

    pub async fn start(&self) {
        loop {
            info!("Starting MQTT subscriber");
            let res = self.subscribe_loop().await;
            error!(
                "MQTT subscriber quit unexpectedly, restarting in 5 seconds: {:?}",
                res
            );
            sleep(Duration::from_secs(5));
        }
    }

    async fn subscribe_loop(&self) -> Result<(), MqttClientError> {
        let mut mqttoptions = MqttOptions::new("smarthome", self.host.to_string(), 1883);
        mqttoptions.set_keep_alive(Duration::from_secs(15));

        let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

        let sensors = db::temp_sensors::get_temp_sensors(&self.pool).await?;
        if sensors.is_empty() {
            info!("No sensors, not starting");
            return Ok(());
        };

        let topics = sensors.iter().map(|sensor| SubscribeFilter {
            path: format!("{}/{}", self.base_topic, sensor.id),
            qos: QoS::AtMostOnce,
        });

        client.subscribe_many(topics).await?;
        loop {
            match eventloop.poll().await {
                Ok(notification) => match notification {
                    Event::Incoming(e) => match e {
                        Incoming::Publish(msg) => {
                            let topic = msg.topic;
                            let payload = msg.payload;
                            let string = str::from_utf8(&payload).expect("Failed to parse message");
                            debug!("Message on topic: {}", topic);
                            if let Ok(parsed) = serde_json::from_str::<SensorPayload>(string) {
                                if let Some(sensor) =
                                    sensors.iter().find(|sensor| topic.ends_with(&sensor.id))
                                {
                                    let sent = self
                                        .sender
                                        .send(WorkMessage::TEMP(sensor.room_id, parsed.temperature))
                                        .await;
                                    if sent.is_err() {
                                        error!(
                                            "Failed to send temperature work message - room_id: {}",
                                            sensor.room_id
                                        )
                                    }
                                } else {
                                    error!("No topic found: {}", topic)
                                }
                            } else {
                                warn!("Failed to parse message: {}", string)
                            }
                        }
                        _ => {
                            debug!("Incoming event: {:?}", e)
                        }
                    },
                    Event::Outgoing(e) => {
                        debug!("Outgoing event: {:?}", e)
                    }
                },
                Err(e) => {
                    error!("Connection error: {:?}", e);
                    return Err(MqttClientError::ConnectionError(e));
                }
            }
        }
    }
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct SensorPayload {
    pub battery: i64,
    pub humidity: f64,
    pub linkquality: i64,
    pub power_outage_count: i64,
    pub pressure: f64,
    pub temperature: f64,
    pub voltage: i64,
}
