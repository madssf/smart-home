use std::collections::HashMap;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

use chrono::NaiveDateTime;
use itertools::Itertools;
use log::{debug, error, info};
use sqlx::PgPool;
use thiserror::Error;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::mpsc::error::SendError;
use uuid::Uuid;

use crate::{db, now, service};
use crate::clients::shelly_client::{ShellyClient, ShellyClientError};
use crate::clients::tibber_client::{TibberClient, TibberClientError};
use crate::db::DbError;
use crate::domain::{
    ActionType, Plug, PriceInfo, Room, TempAction, TempActionType, TemperatureLog, WorkMessage,
};

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
    #[error("No such button: {0}")]
    NoSuchButton(Uuid),
    #[error("Shelly error: {0}")]
    ShellyClientError(#[from] ShellyClientError),
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
                    WorkMessage::BUTTON(button_id, action, attempt) => {
                        match self.button_handler(&button_id, &action).await {
                            Ok(_) => {
                                debug!("Button work handled.")
                            }
                            Err(e) => {
                                error!("Button work failed, error: {}", e);
                                if attempt > 3 {
                                    error!("Button work failed 3 times, giving up.")
                                } else {
                                    let new_attempt = attempt + 1;
                                    info!("Retrying button work, attempt: {new_attempt}");
                                    if let Err(e) = self
                                        .sender
                                        .send(WorkMessage::BUTTON(button_id, action, new_attempt))
                                        .await
                                    {
                                        error!("Failed to send button work message when retrying: {:?}", e);
                                    }
                                }
                            }
                        }
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

    pub async fn button_handler(
        &self,
        button_id: &Uuid,
        action: &ActionType,
    ) -> Result<(), WorkHandlerError> {
        let button = db::buttons::get_button(&self.pool, button_id).await?;
        match button {
            None => Err(WorkHandlerError::NoSuchButton(*button_id)),
            Some(button) => {
                let all_plugs = db::plugs::get_plugs(&self.pool).await?;
                let plugs: Vec<&Plug> = all_plugs
                    .iter()
                    .filter(|p| button.plug_ids.contains(&p.id))
                    .collect();
                for plug in plugs {
                    self.shelly_client.execute_action(plug, action).await?;
                }

                Ok(())
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
            } else if action.starts_at.map_or(true, |t| t <= *now) {
                temp_actions.push(action)
            }
        }

        debug!("Found temp actions {:?}", temp_actions);

        let rooms = db::rooms::get_rooms(&self.pool).await?;
        let current_temps = db::temperature_logs::get_current_temps(&self.pool, &rooms).await?;

        debug!("Current temperatures: {:?}", &current_temps);

        for room in rooms {
            let room_temp_actions: Vec<TempAction> = temp_actions
                .clone()
                .into_iter()
                .filter(|a| a.room_ids.contains(&room.id))
                .sorted_by(|a, b| Ord::cmp(&a.expires_at, &b.expires_at))
                .collect();

            let action = self
                .get_action(now, price, &room, &current_temps, room_temp_actions.first())
                .await?;

            let room_plugs = db::plugs::get_room_plugs(&self.pool, &room.id).await?;

            for plug in room_plugs {
                if plug.scheduled {
                    match self.shelly_client.execute_action(&plug, &action).await {
                        Ok(_) => debug!("Turned plug {} {}", plug.name, action),
                        Err(e) => {
                            error!("Failed to turn plug {} {}, error: {}", plug.name, action, e)
                        }
                    }
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
        temp_action_opt: Option<&TempAction>,
    ) -> Result<ActionType, DbError> {
        let matching_schedule =
            db::schedules::get_matching_schedule(&self.pool, &room.id, now).await?;
        let current_temp = if let Some(temp) = current_temps.get(&room.id) {
            temp
        } else {
            return Ok(ActionType::OFF);
        };

        if let Some(abs_min_temp) = room.min_temp {
            if current_temp.temp < abs_min_temp {
                return Ok(ActionType::ON);
            }
        }

        if let Some(temp_action) = temp_action_opt {
            match temp_action.action_type {
                TempActionType::ON(temp_opt) => {
                    if let Some(temp) = temp_opt {
                        if current_temp.temp < temp {
                            return Ok(ActionType::ON);
                        }
                    } else {
                        return Ok(ActionType::ON);
                    }
                }
                TempActionType::OFF => return Ok(ActionType::OFF),
            }
        }

        if let Some(schedule) = matching_schedule {
            if current_temp.temp < schedule.get_temp(&price.level()) {
                Ok(ActionType::ON)
            } else {
                Ok(ActionType::OFF)
            }
        } else {
            Ok(ActionType::OFF)
        }
    }
}
