use std::str::FromStr;

use anyhow::Context;
use chrono::{NaiveDateTime, NaiveTime, Weekday};
use serde::Serialize;
use sqlx::types::ipnetwork::IpNetwork;
use strum_macros::{Display, EnumString};
use uuid::Uuid;

#[derive(Debug, EnumString, Display, Eq, PartialEq, Copy, Clone)]
pub enum PriceLevel {
    CHEAP,
    NORMAL,
    EXPENSIVE,
}

#[derive(Debug, Clone, Serialize)]
pub struct Room {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Plug {
    pub id: Uuid,
    pub name: String,
    pub ip: IpNetwork,
    pub username: String,
    pub password: String,
    pub room_id: Uuid,
}

impl Plug {
    pub fn new(
        name: &str,
        ip: &str,
        username: &str,
        password: &str,
        room_id: &Uuid,
    ) -> Result<Plug, anyhow::Error> {
        Ok(Plug {
            id: Uuid::new_v4(),
            name: name.to_string(),
            ip: IpNetwork::from_str(ip).context(format!("Failed to parse IP: {}", ip))?,
            username: username.to_string(),
            password: password.to_string(),
            room_id: *room_id,
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Schedule {
    pub id: Uuid,
    pub price_level: PriceLevel,
    pub days: Vec<Weekday>,
    pub time_windows: Vec<(NaiveTime, NaiveTime)>,
    pub temp: f64,
    pub room_ids: Vec<Uuid>,
}

impl Schedule {
    pub fn new(
        price_level: &PriceLevel,
        days: Vec<Weekday>,
        time_windows: Vec<(NaiveTime, NaiveTime)>,
        temp: &f64,
        room_ids: Vec<Uuid>,
    ) -> Result<Self, anyhow::Error> {
        Ok(Schedule {
            id: Uuid::new_v4(),
            price_level: *price_level,
            days,
            time_windows,
            temp: *temp,
            room_ids,
        })
    }
}

#[derive(EnumString, Display, Debug, Clone, Copy, Eq, PartialEq)]
pub enum ActionType {
    ON,
    OFF,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TempAction {
    pub id: Uuid,
    pub room_ids: Vec<Uuid>,
    pub action_type: ActionType,
    pub expires_at: NaiveDateTime,
}

impl TempAction {
    pub fn new(
        expires_at: &NaiveDateTime,
        action_type: &str,
        room_ids: Vec<Uuid>,
    ) -> Result<Self, anyhow::Error> {
        Ok(TempAction {
            id: Uuid::new_v4(),
            room_ids,
            action_type: ActionType::from_str(action_type)
                .context(format!("Could not parse as Action: {}", action_type))?,
            expires_at: *expires_at,
        })
    }
}

#[derive(Display, Clone)]
pub enum WorkMessage {
    REFRESH,
    POLL,
    TEMP(String, f64),
}
