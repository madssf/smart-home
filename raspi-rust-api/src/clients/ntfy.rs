use std::time::Duration;

use thiserror::Error;

use crate::service::notifications::NotificationMessage;

pub struct NtfyClient {
    client: reqwest::Client,
}

#[derive(Error, Debug)]
pub enum NtfyClientError {
    #[error("Request error: {0}")]
    RequestError(#[from] reqwest::Error),
}

impl Default for NtfyClient {
    fn default() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(20))
                .build()
                .expect("Failed to build client"),
        }
    }
}

impl NtfyClient {
    pub async fn publish_notification(
        &self,
        message: &NotificationMessage,
        topic: &str,
    ) -> Result<(), NtfyClientError> {
        self.client
            .post(format!("https://ntfy.sh/{}", topic))
            .body(message.display())
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}
