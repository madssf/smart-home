use std::fmt::{Display, Formatter};
use std::str::FromStr;

use anyhow::Context;
use chrono::{NaiveDateTime, NaiveTime, Weekday};
use serde::{Deserialize, Serialize};
use sqlx::types::ipnetwork::IpNetwork;
use strum_macros::{Display, EnumString};
use tibber::{
    Consumption as TConsumption, EnergyUnits, PriceInfo as TPriceInfo, PriceLevel as TPriceLevel,
};
use uuid::Uuid;

#[derive(Debug, EnumString, Display, Eq, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum PriceLevel {
    CHEAP,
    NORMAL,
    EXPENSIVE,
}

impl From<TPriceLevel> for PriceLevel {
    fn from(value: TPriceLevel) -> Self {
        match value {
            TPriceLevel::VeryCheap => PriceLevel::CHEAP,
            TPriceLevel::Cheap => PriceLevel::CHEAP,
            TPriceLevel::Normal => PriceLevel::NORMAL,
            TPriceLevel::Expensive => PriceLevel::EXPENSIVE,
            TPriceLevel::VeryExpensive => PriceLevel::EXPENSIVE,
            TPriceLevel::Other(_) => PriceLevel::NORMAL,
            TPriceLevel::None => PriceLevel::NORMAL,
        }
    }
}

#[derive(Serialize)]
pub struct PriceInfo {
    pub amount: f64,
    pub currency: String,
    pub level: PriceLevel,
    pub starts_at: NaiveDateTime,
}

impl Display for PriceInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "Price: {} {} - Level: {}",
            &self.amount,
            &self.currency,
            &self.level.to_string()
        ))
    }
}

impl From<TPriceInfo> for PriceInfo {
    fn from(value: TPriceInfo) -> Self {
        Self {
            amount: value.total,
            currency: value.currency,
            level: value.level.into(),
            starts_at: value.starts_at.naive_local(),
        }
    }
}

#[derive(Serialize)]
pub struct Consumption {
    pub from: NaiveDateTime,
    pub to: NaiveDateTime,
    pub kwh: Option<f64>,
    pub cost: f64,
}

impl From<&TConsumption> for Consumption {
    fn from(value: &TConsumption) -> Self {
        Self {
            from: value.from.naive_local(),
            to: value.to.naive_local(),
            kwh: match value.energy {
                EnergyUnits::kWh(kwh) => Some(kwh),
                EnergyUnits::None => None,
            },
            cost: value.cost,
        }
    }
}
#[derive(Eq, PartialEq, Debug, Copy, Clone, Serialize)]
pub struct LiveConsumption {
    pub timestamp: NaiveDateTime,
    pub power: i64,
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

#[derive(Debug, PartialEq, Clone, Serialize)]
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
        temp: f64,
        room_ids: Vec<Uuid>,
    ) -> Result<Self, anyhow::Error> {
        Ok(Schedule {
            id: Uuid::new_v4(),
            price_level: *price_level,
            days,
            time_windows,
            temp,
            room_ids,
        })
    }
}

#[derive(EnumString, Display, Deserialize, Serialize, Debug, Clone, Copy, Eq, PartialEq)]
pub enum ActionType {
    ON,
    OFF,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct TempAction {
    pub id: Uuid,
    pub room_ids: Vec<Uuid>,
    pub action_type: ActionType,
    pub expires_at: NaiveDateTime,
}

impl TempAction {
    pub fn new(
        expires_at: &NaiveDateTime,
        action_type: &ActionType,
        room_ids: Vec<Uuid>,
    ) -> Result<Self, anyhow::Error> {
        Ok(TempAction {
            id: Uuid::new_v4(),
            room_ids,
            action_type: *action_type,
            expires_at: *expires_at,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TemperatureLog {
    pub room_id: Uuid,
    pub time: NaiveDateTime,
    pub temp: f64,
}

#[derive(Display, Clone)]
pub enum WorkMessage {
    REFRESH,
    POLL,
    TEMP(Uuid, f64),
}
