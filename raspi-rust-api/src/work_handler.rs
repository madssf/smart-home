use std::env;
use std::thread::sleep;
use std::time::Duration;

use chrono::{TimeZone, Utc};
use chrono_tz::Tz;
use log::{error, info, warn};
use thiserror::Error;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::clients::{FirestoreClient, ShellyClient};
use crate::db::DbError;
use crate::prices::{PriceError, PriceInfo};
use crate::scheduling::{ActionType, SchedulingError};
use crate::{db, prices, scheduling, shelly_client};

#[derive(Error, Debug)]
pub enum WorkHandlerError {
    #[error("PriceError: {0}")]
    PriceError(#[from] PriceError),
    #[error("SchedulingError: {0}")]
    SchedulingError(#[from] SchedulingError),
    #[error("DbError: {0}")]
    DbError(#[from] DbError),
}

pub struct WorkHandler {
    firestore_client: FirestoreClient,
    shelly_client: ShellyClient,
    receiver: Receiver<String>,
}

impl WorkHandler {
    pub fn new(
        firestore_client: FirestoreClient,
        shelly_client: ShellyClient,
        receiver: Receiver<String>,
    ) -> Self {
        WorkHandler {
            firestore_client,
            shelly_client,
            receiver,
        }
    }

    pub async fn start(mut self) {
        info!("Starting work handler");
        loop {
            while let Ok(message) = self.receiver.try_recv() {
                info!("Got message {}", message);
                match self.handle().await {
                    Ok(_) => {
                        info!("Work handled.")
                    }
                    Err(e) => error!("Work failed, error: {}", e),
                };
            }
            sleep(Duration::from_secs(1))
        }
    }

    pub async fn handle(&self) -> Result<(), WorkHandlerError> {
        let utc = Utc::now().naive_utc();
        let tz: Tz = env::var("TIME_ZONE")
            .expect("Missing TIME_ZONE env var")
            .parse()
            .expect("Failed to parse timezone");
        let now = tz.from_utc_datetime(&utc).naive_local();

        info!("Current local time: {}", &now);

        let price: PriceInfo = prices::get_current_price().await?;

        info!("Current price: {}", &price);

        let plugs = db::get_plugs(&self.firestore_client).await?;
        let temp_actions = db::get_temp_actions(&self.firestore_client).await?;
        info!("Found temp actions {:?}", temp_actions);

        let action: ActionType =
            scheduling::get_action(&self.firestore_client, &price.level, &now).await?;

        info!("Got action: {}", &action.to_string());

        for plug in plugs {
            info!("Processing plug: {}", &plug.name);
            if let Ok(power_usage) = shelly_client::get_status(&self.shelly_client, &plug).await {
                info!("Current power usage: {} W", power_usage);
                info!(
                    "Equals hourly price of: {:.3} {}",
                    price.amount / 1000.0 * power_usage,
                    price.currency
                );
            };

            let actual_action = if let Some(temp_action) = temp_actions
                .iter()
                .find(|action| action.plug_ids.contains(&plug.id))
            {
                if temp_action.expires_at > now {
                    info!(
                        "Found temp action {} on plug {}",
                        temp_action.action_type.to_string(),
                        &plug.name
                    );
                    temp_action.action_type
                } else {
                    match db::delete_temp_action(&self.firestore_client, &temp_action.id).await {
                        Ok(_) => info!("Deleted temp action: {}", &temp_action.id),
                        Err(e) => warn!(
                            "Failed to delete temp action: {}, error: {}",
                            &temp_action.id,
                            e.to_string()
                        ),
                    }
                    action
                }
            } else {
                action
            };

            match shelly_client::execute_action(&self.shelly_client, &plug, &actual_action).await {
                Ok(_) => info!(
                    "Action executed on plug {}: {}",
                    &plug.name,
                    &action.to_string()
                ),
                Err(e) => error!(
                    "Action failed on plug {}: {} - error: {}",
                    &plug.name,
                    &action.to_string(),
                    e,
                ),
            }
        }
        Ok(())
    }
}

pub async fn poll(
    sender: Sender<String>,
    sleep_duration_in_minutes: u64,
) -> Result<(), SendError<String>> {
    loop {
        match sender.send("Poll".to_string()).await {
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
