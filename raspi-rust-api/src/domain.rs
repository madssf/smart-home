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
    fn index_of(&self) -> i32 {
        PriceLevel::iter()
            .position(|p| p == *self)
            .unwrap_or_else(|| panic!("Couldn't get index of price level: {}", self)) as i32
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
            let x1 = min_price_level.index_of() as f64;
            let x2 = max_price_level.index_of() as f64;
            let y1 = self
                .temps
                .get(min_price_level)
                .expect("Temp for existing lower key not found");
            let y2 = self
                .temps
                .get(max_price_level)
                .expect("Temp for existing higher key not found");
            (10.0 * (y1 + ((index as f64 - x1) * (y2 - y1) / (x2 - x1)))).round() / 10.0
        }
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
        assert_eq!(sched.get_temp(&PriceLevel::VeryExpensive), 5.0);
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
        assert_eq!(sched.get_temp(&PriceLevel::VeryCheap), 27.5);
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
