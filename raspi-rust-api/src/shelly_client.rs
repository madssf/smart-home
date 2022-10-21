use std::time::Duration;

use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;

use crate::domain::{ActionType, Plug};

pub struct ShellyClient {
    pub client: Client,
}

impl Default for ShellyClient {
    fn default() -> Self {
        ShellyClient::new()
    }
}

impl ShellyClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create client");

        ShellyClient { client }
    }
}

#[derive(Error, Debug)]
pub enum ShellyClientError {
    #[error("Reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
}

#[derive(Debug, Deserialize)]
pub struct PlugStatus {
    power: f64,
    /*
    overpower: f32,
    is_valid: bool,
    timestamp: i32,
    counters: Vec<f32>,
    total: i32,
     */
}

pub async fn get_status(
    shelly_client: &ShellyClient,
    plug: &Plug,
) -> Result<f64, ShellyClientError> {
    let url = format!(
        "http://{}:{}@{}/meter/0",
        plug.username, plug.password, plug.ip
    );
    let resp = shelly_client
        .client
        .get(url)
        .send()
        .await?
        .json::<PlugStatus>()
        .await?;
    Ok(resp.power)
}

pub async fn execute_action(
    shelly_client: &ShellyClient,
    plug: &Plug,
    action: &ActionType,
) -> Result<(), ShellyClientError> {
    let url = format!(
        "http://{}:{}@{}/relay/0/command?turn={}",
        plug.username,
        plug.password,
        plug.ip.ip(),
        action.to_string().to_lowercase(),
    );

    shelly_client.client.get(url).send().await?.text().await?;
    Ok(())
}
