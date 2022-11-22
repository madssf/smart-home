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
use crate::{db, now, service};

#[derive(Error, Debug)]
pub enum WorkHandlerError {
    #[error("PriceError: {0}")]
    PriceError(#[from] TibberClientError),
    #[error("DbError: {0}")]
    DbError(#[from] DbError),
    #[error("SendError")]
    SendError,
    #[error("Unexpected error: {0}")]
    UnexpectedError(#[from] anyhow::Error),
}

pub struct WorkHandler {
    shelly_client: Arc<ShellyClient>,
    tibber_client: Arc<TibberClient>,
    pool: Arc<PgPool>,
    sender: Sender<WorkMessage>,
    receiver: Receiver<WorkMessage>,
    poll_interval_mins: u64,
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
            poll_interval_mins: 1,
        }
    }

    pub async fn start(mut self) {
        info!("Starting work handler");
        let poll_sender = self.sender.clone();
        let poll_interval = self.poll_interval_mins;
        tokio::task::spawn(async move { self.listener().await });
        tokio::task::spawn(async move { Self::poll(poll_sender, poll_interval).await });
    }

    async fn listener(&mut self) -> Result<(), WorkHandlerError> {
        loop {
            while let Ok(message) = self.receiver.try_recv() {
                debug!("Got message {}", message.to_string());
                match message {
                    WorkMessage::REFRESH | WorkMessage::POLL => {
                        let now = now();
                        match service::prices::get_current_price(
                            self.tibber_client.as_ref(),
                            self.pool.as_ref(),
                        )
                        .await
                        {
                            Ok(price) => {
                                match self.main_handler(&price, &now).await {
                                    Ok(_) => {
                                        debug!("Work handled.")
                                    }
                                    Err(e) => error!("Work failed, error: {}", e),
                                };
                            }
                            Err(_) => error!("Failed to get price"),
                        }
                    }
                    WorkMessage::TEMP(room_id, temp) => {
                        match self.temperature_handler(&room_id, &temp).await {
                            Ok(_) => {
                                debug!("Temperature work handled.")
                            }
                            Err(e) => error!("Temperature work failed, error: {}", e),
                        };
                    }
                }
            }
            sleep(Duration::from_millis(100))
        }
    }

    async fn poll(
        sender: Sender<WorkMessage>,
        poll_interval_mins: u64,
    ) -> Result<(), SendError<String>> {
        loop {
            match sender.send(WorkMessage::POLL).await {
                Ok(_) => {
                    debug!("Sent message, sleeping for {} minutes", poll_interval_mins);
                    sleep(Duration::from_secs(poll_interval_mins * 60))
                }
                Err(e) => {
                    error!("Failed to send message, error {}", e);
                    sleep(Duration::from_secs(poll_interval_mins * 10))
                }
            }
        }
    }

    pub async fn temperature_handler(
        &self,
        room_id: &Uuid,
        temp: &f64,
    ) -> Result<(), WorkHandlerError> {
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
                match self.shelly_client.execute_action(&plug, &action).await {
                    Ok(_) => debug!("Turned plug {} {}", plug.name, action),
                    Err(e) => error!("Failed to turn plug {} {}, error: {}", plug.name, action, e),
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
        let current_temp = if let Some(temp) = current_temps.get(&room.id) {
            temp
        } else {
            return Ok(ActionType::OFF);
        };

        let less_than_abs_min = if let Some(temp) = room.min_temp {
            current_temp.temp < temp
        } else {
            false
        };
        let action = if let Some(schedule) = matching_schedule {
            if current_temp.temp < schedule.get_temp(&price.level()) {
                ActionType::ON
            } else {
                ActionType::OFF
            }
        } else if temp_action_on || less_than_abs_min {
            ActionType::ON
        } else {
            ActionType::OFF
        };
        Ok(action)
    }
}
