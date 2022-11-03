use thiserror::Error;
use tibber::{HomeId, TibberSession, TimeResolution};
use tokio::task::JoinError;

use crate::domain::{Consumption, PriceInfo};

#[derive(Error, Debug)]
pub enum TibberClientError {
    #[error("User has no home")]
    UserHasNoHome,
    #[error("Request failure")]
    RequestFailure,
    #[error("Join error: {0}")]
    JoinError(#[from] JoinError),
}

#[derive(Clone)]
pub struct TibberClient {
    api_token: String,
}

impl TibberClient {
    pub fn new(api_token: String) -> Self {
        Self { api_token }
    }

    async fn prepare_request(&self) -> Result<(TibberSession, HomeId), TibberClientError> {
        let token = self.api_token.clone();
        let res = tokio::task::spawn_blocking(|| {
            let conn = TibberSession::new(token);
            let user = match conn.get_user() {
                Ok(user) => user,
                Err(_) => return Err(TibberClientError::RequestFailure),
            };

            if user.homes.is_empty() {
                return Err(TibberClientError::UserHasNoHome);
            }
            Ok(user.homes[0].clone())
        })
        .await?;
        Ok((TibberSession::new(self.api_token.clone()), res?))
    }

    pub async fn get_current_price(&self) -> Result<PriceInfo, TibberClientError> {
        let (session, home_id) = self.prepare_request().await?;
        tokio::task::spawn_blocking(move || match session.get_current_price(&home_id) {
            Ok(price_info) => Ok(price_info.into()),
            Err(_) => Err(TibberClientError::RequestFailure),
        })
        .await?
    }

    pub async fn get_consumption(&self) -> Result<Vec<Consumption>, TibberClientError> {
        let (session, home_id) = self.prepare_request().await?;
        Ok(tokio::task::spawn_blocking(move || {
            match session.get_consuption(&home_id, TimeResolution::Hourly, 24) {
                Ok(consumption) => Ok(consumption),
                Err(_) => Err(TibberClientError::RequestFailure),
            }
        })
        .await??
        .iter()
        .map(Consumption::from)
        .collect())
    }
    
    pub async fn get_prices_today(&self) -> Result<Vec<PriceInfo>, TibberClientError> {
        let (session, home_id) = self.prepare_request().await?;
        let res = tokio::task::spawn_blocking(move || match session.get_prices_today(&home_id) {
            Ok(consumption) => Ok(consumption),
            Err(_) => Err(TibberClientError::RequestFailure),
        })
        .await?;
        match res {
            Ok(v) => Ok(v.into_iter().map(|a| a.into()).collect()),
            Err(e) => Err(e),
        }
    }
}
