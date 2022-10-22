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
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create client");

        ShellyClient {
            client,
            port_suffix: None,
        }
    }

    pub fn new_with_port(port_suffix: u16) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create client");

        ShellyClient {
            client,
            port_suffix: Some(port_suffix),
        }
    }

    pub async fn get_status(&self, plug: &Plug) -> Result<f64, ShellyClientError> {
        let host_and_port = match self.port_suffix {
            None => format!("{}", plug.ip.ip()),
            Some(port) => format!("{}:{}", plug.ip.ip(), port),
        };
        let url = format!(
            "http://{}:{}@{}/meter/0",
            plug.username, plug.password, host_and_port
        );
        let resp = self
            .client
            .get(url)
            .send()
            .await?
            .json::<PlugStatus>()
            .await?;
        Ok(resp.power)
    }

    pub async fn execute_action(
        &self,
        plug: &Plug,
        action: &ActionType,
    ) -> Result<(), ShellyClientError> {
        let host_and_port = match self.port_suffix {
            None => format!("{}", plug.ip.ip()),
            Some(port) => format!("{}:{}", plug.ip.ip(), port),
        };
        let url = format!(
            "http://{}:{}@{}/relay/0/command?turn={}",
            plug.username,
            plug.password,
            host_and_port,
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
