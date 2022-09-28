use std::error::Error;

use serde::Deserialize;

use crate::{plugs::Plug, scheduling::ActionType};

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

pub async fn get_status(client: &reqwest::Client, plug: &Plug) -> Result<f64, Box<dyn Error>> {
    let url = format!(
        "http://{}:{}@{}/meter/0",
        plug.username, plug.password, plug.ip
    );
    let resp = client.get(url).send().await?.json::<PlugStatus>().await?;
    Ok(resp.power)
}

pub async fn execute_action(
    client: &reqwest::Client,
    plug: &Plug,
    action: &ActionType,
) -> Result<(), Box<dyn Error>> {
    let url = format!(
        "http://{}:{}@{}/relay/0/command?turn={}",
        plug.username,
        plug.password,
        plug.ip,
        action.to_string()
    );

    client.get(url).send().await?.text().await?;
    Ok(())
}
