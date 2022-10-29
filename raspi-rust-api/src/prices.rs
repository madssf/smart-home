use std::fmt::{Display, Formatter};

use chrono::NaiveDateTime;
use serde::Serialize;
use thiserror::Error;
use tibber::{
    Consumption as TConsumption, EnergyUnits, PriceInfo as TPriceInfo, TibberSession,
    TimeResolution,
};
use tokio::task::JoinError;

use crate::domain::PriceLevel;
use crate::prices::PriceError::ThreadError;

#[derive(Serialize)]
pub struct PriceInfo {
    pub amount: f64,
    pub currency: String,
    pub level: PriceLevel,
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

impl PriceInfo {
    fn from_tibber_price_info(tibber_price_info: &TPriceInfo) -> Self {
        PriceInfo {
            amount: tibber_price_info.total,
            currency: String::from(&tibber_price_info.currency),
            level: PriceLevel::from_tibber_price_level(&tibber_price_info.level),
        }
    }
}

#[derive(Serialize)]
pub struct Consumption {
    from: NaiveDateTime,
    to: NaiveDateTime,
    kwh: Option<f64>,
    cost: f64,
}

impl TryFrom<&TConsumption> for Consumption {
    type Error = PriceError;

    fn try_from(value: &TConsumption) -> Result<Self, Self::Error> {
        Ok(Self {
            from: value.from.naive_local(),
            to: value.to.naive_local(),
            kwh: match value.energy {
                EnergyUnits::kWh(kwh) => Some(kwh),
                EnergyUnits::None => None,
            },
            cost: value.cost,
        })
    }
}

#[derive(Error, Debug)]
pub enum PriceError {
    #[error("Failed to get user")]
    FailedToGetUser,
    #[error("User has no home")]
    UserHasNoHome,
    #[error("Failed to get price")]
    FailedToGetPrice,
    #[error("Thread error: {0}")]
    ThreadError(JoinError),
}

#[derive(Clone)]
pub struct TibberClient {
    api_token: String,
}

impl TibberClient {
    pub fn new(api_token: String) -> Self {
        Self { api_token }
    }

    pub async fn get_current_price(&self) -> Result<PriceInfo, PriceError> {
        let token = self.api_token.clone();
        let res = tokio::task::spawn_blocking(|| {
            let conn = TibberSession::new(token);
            let user = match conn.get_user() {
                Ok(user) => user,
                Err(_) => return Err(PriceError::FailedToGetUser),
            };

            if user.homes.is_empty() {
                return Err(PriceError::UserHasNoHome);
            }

            match conn.get_current_price(&user.homes[0]) {
                Ok(price_info) => Ok(PriceInfo::from_tibber_price_info(&price_info)),
                Err(_) => Err(PriceError::FailedToGetPrice),
            }
        })
        .await;

        match res {
            Ok(result) => result,
            Err(e) => Err(ThreadError(e)),
        }
    }

    pub async fn get_consumption(&self) -> Result<Vec<Consumption>, PriceError> {
        let token = self.api_token.clone();
        let res = tokio::task::spawn_blocking(|| {
            let conn = TibberSession::new(token);
            let user = match conn.get_user() {
                Ok(user) => user,
                Err(_) => return Err(PriceError::FailedToGetUser),
            };

            if user.homes.is_empty() {
                return Err(PriceError::UserHasNoHome);
            }

            match conn.get_consuption(&user.homes[0], TimeResolution::Hourly, 24) {
                Ok(consumption) => Ok(consumption),
                Err(_) => Err(PriceError::FailedToGetPrice),
            }
        })
        .await;

        match res {
            Ok(result) => match result {
                Ok(consumption) => consumption.iter().map(Consumption::try_from).collect(),
                Err(e) => Err(e),
            },
            Err(e) => Err(ThreadError(e)),
        }
    }
}
