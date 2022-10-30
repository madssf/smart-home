use thiserror::Error;
use tibber::{TibberSession, TimeResolution};
use tokio::task::JoinError;

use crate::domain::{Consumption, PriceInfo};

#[derive(Error, Debug)]
pub enum TibberClientError {
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

    pub async fn get_current_price(&self) -> Result<PriceInfo, TibberClientError> {
        let token = self.api_token.clone();
        let res = tokio::task::spawn_blocking(|| {
            let conn = TibberSession::new(token);
            let user = match conn.get_user() {
                Ok(user) => user,
                Err(_) => return Err(TibberClientError::FailedToGetUser),
            };

            if user.homes.is_empty() {
                return Err(TibberClientError::UserHasNoHome);
            }

            match conn.get_current_price(&user.homes[0]) {
                Ok(price_info) => Ok(PriceInfo::from_tibber_price_info(&price_info)),
                Err(_) => Err(TibberClientError::FailedToGetPrice),
            }
        })
        .await;

        match res {
            Ok(result) => result,
            Err(e) => Err(TibberClientError::ThreadError(e)),
        }
    }

    pub async fn get_consumption(&self) -> Result<Vec<Consumption>, TibberClientError> {
        let token = self.api_token.clone();
        let res = tokio::task::spawn_blocking(|| {
            let conn = TibberSession::new(token);
            let user = match conn.get_user() {
                Ok(user) => user,
                Err(_) => return Err(TibberClientError::FailedToGetUser),
            };

            if user.homes.is_empty() {
                return Err(TibberClientError::UserHasNoHome);
            }

            match conn.get_consuption(&user.homes[0], TimeResolution::Hourly, 24) {
                Ok(consumption) => Ok(consumption),
                Err(_) => Err(TibberClientError::FailedToGetPrice),
            }
        })
        .await;

        match res {
            Ok(result) => match result {
                Ok(consumption) => consumption.iter().map(Consumption::try_from).collect(),
                Err(e) => Err(e),
            },
            Err(e) => Err(TibberClientError::ThreadError(e)),
        }
    }
}
