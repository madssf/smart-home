use std::collections::HashMap;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

use chrono::NaiveDateTime;
use itertools::Itertools;
use log::{debug, error, info};
use sqlx::PgPool;
use thiserror::Error;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::{Receiver, Sender};
use uuid::Uuid;

use crate::clients::shelly_client::ShellyClient;
use crate::clients::tibber_client::{TibberClient, TibberClientError};
use crate::db::DbError;
use crate::domain::{ActionType, PriceInfo, Room, TempAction, TemperatureLog, WorkMessage};
use crate::{db, now};

#[derive(Error, Debug)]
pub enum WorkHandlerError {
    #[error("PriceError: {0}")]
    PriceError(#[from] TibberClientError),
    #[error("DbError: {0}")]
    DbError(#[from] DbError),
    #[error("SendError")]
    SendError,
}

pub struct WorkHandler {
    shelly_client: Arc<ShellyClient>,
    tibber_client: Arc<TibberClient>,
    pool: Arc<PgPool>,
    sender: Sender<WorkMessage>,
    receiver: Receiver<WorkMessage>,
}

impl WorkHandler {
    pub fn new(
        shelly_client: Arc<ShellyClient>,
        tibber_client: Arc<TibberClient>,
        sender: Sender<WorkMessage>,
        receiver: Receiver<WorkMessage>,
        pool: Arc<PgPool>,
    ) -> Self {
        WorkHandler {
            shelly_client,
            tibber_client,
            pool,
            sender,
            receiver,
        }
    }

    pub async fn start(mut self) {
        info!("Starting work handler");
        loop {
            while let Ok(message) = self.receiver.try_recv() {
                debug!("Got message {}", message.to_string());
                match message {
                    WorkMessage::REFRESH | WorkMessage::POLL => {
                        let now = now();
                        match self.tibber_client.get_current_price().await {
                            Ok(price) => {
                                match self.main_handler(&price, &now).await {
                                    Ok(_) => {
                                        debug!("Work handled.")
                                    }
                                    Err(e) => error!("Work failed, error: {}", e),
                                };
                            }
                            Err(_) => error!("Failed to fetch price"),
                        }
                    }
                    WorkMessage::TEMP(room, temp) => {
                        match self.temp_handler(&room, &temp).await {
                            Ok(_) => {
                                info!("Temperature work handled.")
                            }
                            Err(e) => error!("Temperature work failed, error: {}", e),
                        };
                    }
                }
            }
            sleep(Duration::from_millis(100))
        }
    }

    pub async fn temp_handler(&self, room_id: &Uuid, temp: &f64) -> Result<(), WorkHandlerError> {
        let now = now();
        db::temperature_logs::create_temp_log(
            &self.pool,
            TemperatureLog {
                room_id: *room_id,
                time: now,
                temp: *temp,
            },
        )
        .await?;
        match self.sender.send(WorkMessage::REFRESH).await {
            Ok(_) => Ok(()),
            Err(_) => Err(WorkHandlerError::SendError),
        }
    }

    pub async fn main_handler(
        &self,
        price: &PriceInfo,
        now: &NaiveDateTime,
    ) -> Result<(), WorkHandlerError> {
        debug!("Current local time: {}", &now);
        debug!("Current price: {}", price);

        let all_actions = db::temp_actions::get_temp_actions(&self.pool).await?;
        let mut temp_actions = vec![];
        for action in all_actions {
            if action.expires_at < *now {
                db::temp_actions::delete_temp_action(&self.pool, &action.id).await?;
            } else {
                temp_actions.push(action)
            }
        }

        debug!("Found temp actions {:?}", temp_actions);

        let rooms = db::rooms::get_rooms(&self.pool).await?;
        let current_temps = db::temperature_logs::get_current_temps(&self.pool, &rooms).await?;

        debug!("Current temperatures: {:?}", &current_temps);

        for room in rooms {
            let room_temp_actions: Vec<&TempAction> = temp_actions
                .iter()
                .filter(|a| a.room_ids.contains(&room.id))
                .sorted_by(|a, b| Ord::cmp(&a.expires_at, &b.expires_at))
                .collect();

            let action = if let Some(action) = room_temp_actions.first() {
                if action.action_type == ActionType::OFF {
                    ActionType::OFF
                } else {
                    self.get_action(now, price, &room, &current_temps, true)
                        .await?
                }
            } else {
                self.get_action(now, price, &room, &current_temps, false)
                    .await?
            };

            let room_plugs = db::plugs::get_room_plugs(&self.pool, &room.id).await?;

            for plug in room_plugs {
                let result = self.shelly_client.execute_action(&plug, &action).await;
                if result.is_ok() {
                    debug!("Turned plug {} {}", plug.name, action.to_string())
                } else {
                    error!("Failed to turn plug {} {}", plug.name, action.to_string())
                }
            }
        }

        Ok(())
    }

    async fn get_action(
        &self,
        now: &NaiveDateTime,
        price: &PriceInfo,
        room: &Room,
        current_temps: &HashMap<Uuid, TemperatureLog>,
        temp_action_on: bool,
    ) -> Result<ActionType, DbError> {
        let matching_schedule =
            db::schedules::get_matching_schedule(&self.pool, &room.id, now).await?;
        let current_temp = current_temps.get(&room.id);
        let action = if let (Some(schedule), Some(current_temp)) = (matching_schedule, current_temp)
        {
            if current_temp.temp < schedule.get_temp(&price.level) {
                ActionType::ON
            } else {
                ActionType::OFF
            }
        } else if temp_action_on {
            ActionType::ON
        } else {
            ActionType::OFF
        };
        Ok(action)
    }
}

pub async fn poll(
    sender: Sender<WorkMessage>,
    sleep_duration_in_minutes: u64,
) -> Result<(), SendError<String>> {
    loop {
        match sender.send(WorkMessage::POLL).await {
            Ok(_) => {
                debug!(
                    "Sent message, sleeping for {} minutes",
                    sleep_duration_in_minutes
                );
                sleep(Duration::from_secs(sleep_duration_in_minutes * 60))
            }
            Err(e) => {
                error!("Failed to send message, error {}", e);
                sleep(Duration::from_secs(sleep_duration_in_minutes * 10))
            }
        }
    }
}
