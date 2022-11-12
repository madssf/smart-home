use std::collections::HashMap;
use std::sync::Arc;
use std::thread::sleep;

use chrono::{Duration, NaiveDateTime};
use log::{debug, error, info};
use sqlx::PgPool;
use strum_macros::Display;
use thiserror::Error;
use tokio::sync::mpsc::Receiver;

use crate::clients::ntfy::{NtfyClient, NtfyClientError};
use crate::db::DbError;
use crate::domain::NotificationSettings;
use crate::{db, now};

pub struct NotificationHandler {
    client: NtfyClient,
    pool: Arc<PgPool>,
    receiver: Receiver<NotificationMessage>,
    last_sent: HashMap<NotificationKey, NaiveDateTime>,
}

#[derive(Error, Debug)]
pub enum NotificationHandlerError {
    #[error("Publish failed: {0}")]
    PublishError(#[from] NtfyClientError),
    #[error("DbError: {0}")]
    DbError(#[from] DbError),
}

#[derive(Debug, Display, PartialEq, Eq)]
pub enum NotificationMessage {
    Consumption { watt_usage: i64 },
}

type NotificationKey = i32;

impl NotificationMessage {
    pub fn display(&self) -> String {
        match self {
            NotificationMessage::Consumption { watt_usage } => {
                format!("⚡️Current consumption {} W!️", watt_usage)
            }
        }
    }
    fn key(&self) -> NotificationKey {
        match self {
            NotificationMessage::Consumption { .. } => 1,
        }
    }
    fn timeout(&self, settings: &NotificationSettings) -> Duration {
        match self {
            NotificationMessage::Consumption { .. } => {
                Duration::minutes(settings.max_consumption_timeout_minutes as i64)
            }
        }
    }

    fn should_send(
        &self,
        last_sent: Option<&NaiveDateTime>,
        now: &NaiveDateTime,
        settings: &NotificationSettings,
    ) -> bool {
        let timeout_passed = if let Some(last_sent) = last_sent {
            *now - *last_sent > self.timeout(settings)
        } else {
            true
        };
        match self {
            NotificationMessage::Consumption { watt_usage } => {
                if let Some(max_usage) = settings.max_consumption {
                    *watt_usage > max_usage as i64 && timeout_passed
                } else {
                    false
                }
            }
        }
    }
}

impl NotificationHandler {
    pub fn new(receiver: Receiver<NotificationMessage>, pool: Arc<PgPool>) -> Self {
        Self {
            client: NtfyClient::default(),
            pool,
            receiver,
            last_sent: HashMap::new(),
        }
    }

    pub async fn start(&mut self) {
        loop {
            while let Ok(msg) = self.receiver.try_recv() {
                match self.handle_message(&msg).await {
                    Ok(_) => {
                        debug!("Message handled")
                    }
                    Err(e) => {
                        error!("Error when handling notification message: {}", e)
                    }
                }
            }
            sleep(core::time::Duration::from_millis(100))
        }
    }

    async fn handle_message(
        &mut self,
        msg: &NotificationMessage,
    ) -> Result<(), NotificationHandlerError> {
        let settings =
            db::notification_settings::get_notification_settings(self.pool.as_ref()).await?;
        if let Some(settings) = settings {
            if msg.should_send(self.last_sent.get(&msg.key()), &now(), &settings) {
                self.client
                    .publish_notification(msg, &settings.ntfy_topic)
                    .await?;
                self.last_sent.insert(msg.key(), now());
                info!("Notification published: {}", msg)
            }
        } else {
            debug!("No notification settings found, not handling message.")
        }
        Ok(())
    }
}
