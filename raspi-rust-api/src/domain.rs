use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::str::FromStr;

use anyhow::{anyhow, Context};
use chrono::{NaiveDateTime, NaiveTime, Weekday};
use log::warn;
use serde::{Deserialize, Serialize};
use sqlx::types::ipnetwork::IpNetwork;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};
use tibber::{
    Consumption as TConsumption, EnergyUnits, PriceInfo as TPriceInfo, PriceLevel as TPriceLevel,
};
use uuid::Uuid;

#[derive(
    Debug, EnumString, EnumIter, Display, Eq, PartialEq, Copy, Clone, Serialize, Deserialize, Hash,
)]
pub enum PriceLevel {
    VeryCheap,
    Cheap,
    Normal,
    Expensive,
    VeryExpensive,
}

impl PriceLevel {
    pub fn index_of(&self) -> i32 {
        PriceLevel::iter()
            .position(|p| p == *self)
            .unwrap_or_else(|| panic!("Couldn't get index of price level: {}", self)) as i32
    }
}

impl From<i32> for PriceLevel {
    fn from(value: i32) -> Self {
        let indexes: Vec<(i32, PriceLevel)> =
            PriceLevel::iter().map(|p| (p.index_of(), p)).collect();
        if value < indexes.first().unwrap().0 {
            indexes.first().unwrap().1
        } else if value > indexes.last().unwrap().0 {
            indexes.last().unwrap().1
        } else {
            indexes.iter().find(|p| p.0 == value).unwrap().1
        }
    }
}

impl From<TPriceLevel> for PriceLevel {
    fn from(value: TPriceLevel) -> Self {
        match value {
            TPriceLevel::VeryCheap => PriceLevel::VeryCheap,
            TPriceLevel::Cheap => PriceLevel::Cheap,
            TPriceLevel::Normal => PriceLevel::Normal,
            TPriceLevel::Expensive => PriceLevel::Expensive,
            TPriceLevel::VeryExpensive => PriceLevel::VeryExpensive,
            TPriceLevel::Other(_) | TPriceLevel::None => {
                warn!(
                    "Encountered unexpected Tibber price level: {:?}, setting to Normal",
                    value
                );
                PriceLevel::Normal
            }
        }
    }
}

#[derive(Serialize, PartialEq, Debug)]
pub struct PriceInfo {
    pub amount: f64,
    pub currency: String,
    pub ext_price_level: PriceLevel,
    pub price_level: Option<PriceLevel>,
    pub starts_at: NaiveDateTime,
}

impl Display for PriceInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "Price: {} {} - Level: {}",
            &self.amount,
            &self.currency,
            &self.ext_price_level.to_string()
        ))
    }
}

impl From<TPriceInfo> for PriceInfo {
    fn from(value: TPriceInfo) -> Self {
        Self {
            amount: value.total,
            currency: value.currency,
            ext_price_level: value.level.into(),
            price_level: None,
            starts_at: value.starts_at.naive_local(),
        }
    }
}

impl PriceInfo {
    pub fn level(&self) -> PriceLevel {
        self.price_level.unwrap_or(self.ext_price_level)
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
    pub min_temp: Option<f64>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct TempSensor {
    pub id: String,
    pub room_id: Uuid,
    #[serde(default)]
    pub battery_level: Option<i32>,
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
    pub temps: HashMap<PriceLevel, f64>,
    pub days: Vec<Weekday>,
    pub time_windows: Vec<(NaiveTime, NaiveTime)>,
    pub room_ids: Vec<Uuid>,
}

impl Schedule {
    pub fn new(
        temps: HashMap<PriceLevel, f64>,
        days: Vec<Weekday>,
        time_windows: Vec<(NaiveTime, NaiveTime)>,
        room_ids: Vec<Uuid>,
    ) -> Result<Self, anyhow::Error> {
        if temps.is_empty() || days.is_empty() || time_windows.is_empty() || room_ids.is_empty() {
            return Err(anyhow!(
                "Schedule must include minimum price level to temperature mapping, one day, one time window and one room."
            ));
        }
        Ok(Self {
            id: Uuid::new_v4(),
            temps,
            days,
            time_windows,
            room_ids,
        })
    }

    pub fn get_temp(&self, price_level: &PriceLevel) -> f64 {
        if let Some(temp) = self.temps.get(price_level) {
            *temp
        } else if self.temps.len() == 1 {
            *self.temps.values().into_iter().collect::<Vec<&f64>>()[0]
        } else {
            let index = price_level.index_of();
            let min_price_level = self
                .temps
                .keys()
                .reduce(|acc, curr| {
                    if index - acc.index_of() > index - curr.index_of() {
                        acc
                    } else {
                        curr
                    }
                })
                .expect("min_price_level not found");
            let max_price_level = self
                .temps
                .keys()
                .reduce(|acc, curr| {
                    if index - acc.index_of() < index - curr.index_of() {
                        acc
                    } else {
                        curr
                    }
                })
                .expect("max_price_level not found");

            let x1 = min_price_level.index_of();
            let x2 = max_price_level.index_of();
            let y1 = self
                .temps
                .get(min_price_level)
                .expect("Temp for existing lower key not found");
            let y2 = self
                .temps
                .get(max_price_level)
                .expect("Temp for existing higher key not found");
            if index < x1 {
                *y1
            } else if index > x2 {
                *y2
            } else {
                (10.0 * (y1 + ((index - x1) as f64 * (y2 - y1) / (x2 - x1) as f64))).round() / 10.0
            }
        }
    }
}

#[derive(EnumString, Display, Deserialize, Serialize, Debug, Clone, Copy, Eq, PartialEq)]
pub enum ActionType {
    ON,
    OFF,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TempAction {
    pub id: Uuid,
    pub room_ids: Vec<Uuid>,
    pub action_type: TempActionType,
    pub expires_at: NaiveDateTime,
}

impl TempAction {
    pub fn new(
        expires_at: &NaiveDateTime,
        action_type: &TempActionType,
        room_ids: Vec<Uuid>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            room_ids,
            action_type: *action_type,
            expires_at: *expires_at,
        }
    }
}

#[derive(Deserialize)]
pub struct TempActionRequest {
    pub room_ids: Vec<Uuid>,
    pub action: ActionType,
    pub temp: Option<f64>,
    pub expires_at: NaiveDateTime,
}

#[derive(Serialize)]
pub struct TempActionResponse {
    pub id: Uuid,
    pub room_ids: Vec<Uuid>,
    pub action: ActionType,
    pub temp: Option<f64>,
    pub expires_at: NaiveDateTime,
}

impl Into<TempAction> for TempActionRequest {
    fn into(self) -> TempAction {
        let action_type = match self.action {
            ActionType::ON => TempActionType::ON(self.temp),
            ActionType::OFF => TempActionType::OFF,
        };
        TempAction::new(&self.expires_at, &action_type, self.room_ids)
    }
}

impl Into<TempActionResponse> for TempAction {
    fn into(self) -> TempActionResponse {
        let (action_type, temp) = match self.action_type {
            TempActionType::ON(t) => (ActionType::ON, t),
            TempActionType::OFF => (ActionType::OFF, None),
        };
        TempActionResponse {
            id: self.id,
            room_ids: self.room_ids,
            action: action_type,
            temp,
            expires_at: self.expires_at,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum TempActionType {
    ON(Option<f64>),
    OFF,
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

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct NotificationSettings {
    #[serde(skip_serializing)]
    #[serde(default)]
    pub id: Option<i32>,
    pub max_consumption: Option<i32>,
    pub max_consumption_timeout_minutes: i32,
    pub ntfy_topic: String,
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use chrono::{NaiveTime, Weekday};
    use strum::IntoEnumIterator;
    use uuid::Uuid;

    use crate::domain::{PriceLevel, Schedule};

    fn schedule() -> Schedule {
        Schedule::new(
            HashMap::from([
                (PriceLevel::VeryCheap, 21.0),
                (PriceLevel::VeryExpensive, 19.0),
            ]),
            vec![Weekday::Mon],
            vec![(NaiveTime::from_hms(0, 0, 0), NaiveTime::from_hms(1, 0, 0))],
            vec![Uuid::new_v4()],
        )
        .expect("Failed to create new schedule")
    }

    #[test]
    fn finds_correct_temp_when_match_exists() {
        assert_eq!(schedule().get_temp(&PriceLevel::VeryCheap), 21.0);
        assert_eq!(schedule().get_temp(&PriceLevel::VeryExpensive), 19.0);
    }

    #[test]
    fn calculates_correct_temp_when_no_match_exists() {
        PriceLevel::iter().for_each(|p| {
            dbg!(p);
            dbg!(p.index_of());
        });
        let sched = Schedule::new(
            HashMap::from([(PriceLevel::VeryCheap, 25.0), (PriceLevel::Expensive, 15.0)]),
            vec![Weekday::Mon],
            vec![(NaiveTime::from_hms(0, 0, 0), NaiveTime::from_hms(1, 0, 0))],
            vec![Uuid::new_v4()],
        )
        .expect("Failed to create new schedule");
        assert_eq!(sched.get_temp(&PriceLevel::Normal), 18.3);
    }

    #[test]
    fn calculates_correct_temp_when_only_lower_exists() {
        let sched = Schedule::new(
            HashMap::from([
                (PriceLevel::VeryCheap, 25.0),
                (PriceLevel::Cheap, 22.5),
                (PriceLevel::Normal, 15.0),
            ]),
            vec![Weekday::Mon],
            vec![(NaiveTime::from_hms(0, 0, 0), NaiveTime::from_hms(1, 0, 0))],
            vec![Uuid::new_v4()],
        )
        .expect("Failed to create new schedule");
        assert_eq!(sched.get_temp(&PriceLevel::VeryExpensive), 15.0);
    }

    #[test]
    fn calculates_correct_temp_when_only_higher_exists() {
        let sched = Schedule::new(
            HashMap::from([
                (PriceLevel::VeryExpensive, 17.5),
                (PriceLevel::Expensive, 20.0),
                (PriceLevel::Cheap, 25.0),
            ]),
            vec![Weekday::Mon],
            vec![(NaiveTime::from_hms(0, 0, 0), NaiveTime::from_hms(1, 0, 0))],
            vec![Uuid::new_v4()],
        )
        .expect("Failed to create new schedule");
        assert_eq!(sched.get_temp(&PriceLevel::VeryCheap), 25.0);
    }

    #[test]
    fn calculates_correct_when_only_one_temp() {
        let sched = Schedule::new(
            HashMap::from([(PriceLevel::Normal, 25.0)]),
            vec![Weekday::Mon],
            vec![(NaiveTime::from_hms(0, 0, 0), NaiveTime::from_hms(1, 0, 0))],
            vec![Uuid::new_v4()],
        )
        .expect("Failed to create new schedule");
        assert_eq!(sched.get_temp(&PriceLevel::VeryCheap), 25.0);
    }
}
