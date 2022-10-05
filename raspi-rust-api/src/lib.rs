use chrono::NaiveDateTime;
use strum_macros::{Display, EnumString};

use crate::scheduling::ActionType;

pub mod api;
pub mod clients;
pub mod db;
pub mod prices;
pub mod scheduling;
pub mod shelly_client;
pub mod work_handler;

#[derive(Debug, EnumString, Display, Eq, PartialEq)]
pub enum PriceLevel {
    CHEAP,
    NORMAL,
    EXPENSIVE,
}

#[derive(Debug)]
pub struct Plug {
    pub name: String,
    pub ip: String,
    pub username: String,
    pub password: String,
    pub temp_action: Option<TempAction>,
}

#[derive(Debug)]
pub struct TempAction {
    pub action_type: ActionType,
    pub expires_at: NaiveDateTime,
}

pub fn config_env_var(name: &str) -> String {
    std::env::var(name)
        .map_err(|e| format!("{}: {}", name, e))
        .expect(&*format!("Missing config env var: {}", name))
}
