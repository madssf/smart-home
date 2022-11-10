use std::time::Duration;

use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;

use crate::domain::{ActionType, Plug};

pub struct ShellyClient {
    pub client: Client,
    port_suffix: Option<u16>,
}

impl Default for ShellyClient {
    fn default() -> Self {
        ShellyClient::new()
    }
}

impl ShellyClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(1))
            .build()
            .expect("Failed to create client");

        ShellyClient {
            client,
            port_suffix: None,
        }
    }

    pub fn new_with_port(port_suffix: u16) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(1))
            .build()
            .expect("Failed to create client");

        ShellyClient {
            client,
            port_suffix: Some(port_suffix),
        }
    }

    fn host(&self, plug: &Plug) -> String {
        match self.port_suffix {
            None => format!("{}", plug.ip.ip()),
            Some(port) => format!("{}:{}", plug.ip.ip(), port),
        }
    }

    pub async fn get_meter_values(&self, plug: &Plug) -> Result<MeterValues, ShellyClientError> {
        let url = format!(
            "http://{}:{}@{}/meter/0",
            plug.username,
            plug.password,
            self.host(plug)
        );
        let resp = self
            .client
            .get(url)
            .send()
            .await?
            .json::<MeterValues>()
            .await?;
        Ok(resp)
    }

    pub async fn get_plug_status(&self, plug: &Plug) -> Result<RelayStatus, ShellyClientError> {
        let url = format!(
            "http://{}:{}@{}/relay/0",
            plug.username,
            plug.password,
            self.host(plug)
        );
        let resp = self
            .client
            .get(url)
            .send()
            .await?
            .json::<RelayStatus>()
            .await?;
        Ok(resp)
    }

    pub async fn execute_action(
        &self,
        plug: &Plug,
        action: &ActionType,
    ) -> Result<(), ShellyClientError> {
        let url = format!(
            "http://{}:{}@{}/relay/0/command?turn={}",
            plug.username,
            plug.password,
            self.host(plug),
            action.to_string().to_lowercase(),
        );

        self.client.get(url).send().await?.text().await?;
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum ShellyClientError {
    #[error("Reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct MeterValues {
    pub power: f64,
    pub overpower: f32,
    pub is_valid: bool,
    pub timestamp: i32,
    pub counters: Vec<f32>,
    pub total: i32,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct RelayStatus {
    pub ison: bool,
    pub has_timer: bool,
    pub timer_started: i64,
    pub timer_duration: i64,
    pub timer_remaining: i64,
    pub overpower: bool,
    pub source: String,
}
