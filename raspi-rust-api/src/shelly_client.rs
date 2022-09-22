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

pub fn get_status(plug: &Plug) -> Result<f64, Box<dyn Error>>  {
    let url = format!("http://{}:{}@{}/meter/0", plug.username, plug.password, plug.ip);
    let resp = reqwest::blocking::get(url)?.json::<PlugStatus>().unwrap();
    Ok(resp.power)

}

pub fn execute_action(plug: &Plug, action: &ActionType) -> Result<(), Box<dyn Error>> {
    let url = format!("http://{}:{}@{}/relay/0/command?turn={}", plug.username, plug.password, plug.ip, action.to_string());
    reqwest::blocking::get(url)?.text()?;
    println!("Turned {} {}", action.to_string(), plug.name);
    Ok(())
}