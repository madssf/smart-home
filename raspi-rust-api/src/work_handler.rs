use std::collections::HashMap;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

use chrono::NaiveDateTime;
use itertools::Itertools;
use log::{error, info};
use thiserror::Error;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::{Receiver, Sender};
use uuid::Uuid;

use crate::db::plugs::PlugsClient;
use crate::db::rooms::RoomsClient;
use crate::db::schedules::SchedulesClient;
use crate::db::temp_actions::TempActionsClient;
use crate::db::temperature_logs::TemperatureLogsClient;
use crate::db::{DbClients, DbConfig, DbError};
use crate::domain::{ActionType, Room, TempAction, TemperatureLog, WorkMessage};
use crate::prices::{PriceError, PriceInfo};
use crate::shelly_client::ShellyClient;
use crate::{now, prices, scheduling};

#[derive(Error, Debug)]
pub enum WorkHandlerError {
    #[error("PriceError: {0}")]
    PriceError(#[from] PriceError),
    #[error("DbError: {0}")]
    DbError(#[from] DbError),
    #[error("SendError")]
    SendError,
}

pub struct WorkHandler {
    shelly_client: ShellyClient,
    sender: Sender<WorkMessage>,
    receiver: Receiver<WorkMessage>,
    rooms_client: Arc<RoomsClient>,
    plugs_client: Arc<PlugsClient>,
    temp_actions_client: Arc<TempActionsClient>,
    schedules_client: Arc<SchedulesClient>,
    temperature_logs_client: Arc<TemperatureLogsClient>,
}

impl WorkHandler {
    pub fn new(
        shelly_client: ShellyClient,
        sender: Sender<WorkMessage>,
        receiver: Receiver<WorkMessage>,
        db_config: &DbConfig,
    ) -> Self {
        let db_clients = DbClients::new(db_config);
        WorkHandler {
            shelly_client,
            sender,
            receiver,
            rooms_client: db_clients.rooms,
            plugs_client: db_clients.plugs,
            temp_actions_client: db_clients.temp_actions,
            schedules_client: db_clients.schedules,
            temperature_logs_client: db_clients.temperature_logs,
        }
    }

    pub async fn start(mut self) {
        info!("Starting work handler");
        loop {
            while let Ok(message) = self.receiver.try_recv() {
                info!("Got message {}", message.to_string());
                match message {
                    WorkMessage::REFRESH | WorkMessage::POLL => {
                        let now = now();
                        match prices::get_current_price().await {
                            Ok(price) => {
                                match self.main_handler(&price, &now).await {
                                    Ok(_) => {
                                        info!("Work handled.")
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
            sleep(Duration::from_secs(1))
        }
    }

    pub async fn temp_handler(&self, room_id: &Uuid, temp: &f64) -> Result<(), WorkHandlerError> {
        let now = now();
        self.temperature_logs_client
            .create_temp_log(TemperatureLog {
                room_id: *room_id,
                time: now,
                temp: *temp,
            })
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
        info!("Current local time: {}", &now);
        info!("Current price: {}", price);

        let all_actions = self.temp_actions_client.get_temp_actions().await?;
        let mut temp_actions = vec![];
        for action in all_actions {
            if action.expires_at < *now {
                self.temp_actions_client
                    .delete_temp_action(&action.id)
                    .await?;
            } else {
                temp_actions.push(action)
            }
        }

        info!("Found temp actions {:?}", temp_actions);

        let rooms = self.rooms_client.get_rooms().await?;
        let current_temps = self
            .temperature_logs_client
            .get_current_temps(rooms.clone())
            .await?;

        info!("Current temperatures: {:?}", &current_temps);

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

            let room_plugs = self.plugs_client.get_room_plugs(&room.id).await?;

            for plug in room_plugs {
                let result = self.shelly_client.execute_action(&plug, &action).await;
                if result.is_ok() {
                    info!("Turned plug {} {}", plug.name, action.to_string())
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
        current_temps: &HashMap<Uuid, f64>,
        temp_on: bool,
    ) -> Result<ActionType, DbError> {
        let room_schedules = self.schedules_client.get_room_schedules(&room.id).await?;
        let matching_schedule =
            scheduling::find_matching_schedule(room_schedules, &price.level, now);
        let current_temp = current_temps.get(&room.id);
        let action = if let (Some(schedule), Some(current_temp)) = (matching_schedule, current_temp)
        {
            if current_temp < &schedule.temp {
                ActionType::ON
            } else {
                ActionType::OFF
            }
        } else if temp_on {
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
                info!(
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
