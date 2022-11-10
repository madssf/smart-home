use std::time::Duration;

use chrono::{DateTime, FixedOffset};
use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;
use tibber::{HomeId, PriceInfo as TPriceInfo, TibberSession, TimeResolution};
use tokio::task::JoinError;

use crate::domain::{Consumption, PriceInfo};
use crate::env_var;

const TIBBER_BASE_URL: &str = "https://api.tibber.com/v1-beta/gql";

#[derive(Error, Debug)]
pub enum TibberClientError {
    #[error("User has no home")]
    UserHasNoHome,
    #[error("Request failure")]
    RequestFailure,
    #[error("Reqwest client failure {0}")]
    ReqwestClientFailure(#[from] reqwest::Error),
    #[error("Join error: {0}")]
    JoinError(#[from] JoinError),
}

#[derive(Clone)]
pub struct TibberClient {
    api_token: String,
    client: Client,
}

impl TibberClient {
    pub fn new(api_token: String) -> Self {
        Self {
            api_token,
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("Failed to build client"),
        }
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

    pub async fn get_hourly_prices(&self) -> Result<Vec<PriceInfo>, TibberClientError> {
        let (session, home_id) = self.prepare_request().await?;
        let res = tokio::task::spawn_blocking(move || {
            match (
                session.get_prices_today(&home_id),
                session.get_prices_tomorrow(&home_id),
            ) {
                (Ok(today), Ok(tomorrow)) => Ok(today
                    .into_iter()
                    .chain(tomorrow.into_iter())
                    .collect::<Vec<TPriceInfo>>()),
                _ => Err(TibberClientError::RequestFailure),
            }
        })
        .await?;
        match res {
            Ok(v) => Ok(v.into_iter().map(|a| a.into()).collect()),
            Err(e) => Err(e),
        }
    }

    pub async fn get_daily_prices(&self) -> Result<Vec<TibberDailyPrice>, TibberClientError> {
        let body = r#"
            {"query":"{\n  viewer {\n    homes {\n      currentSubscription{\n        priceInfo{\n          range(resolution: DAILY, last: 100) {\n            nodes {\n              total,\n              startsAt,\n            }\n          }\n        }\n      }\n    }\n  }\n}\n"}
             "#.to_string();

        Ok(self
            .client
            .post(TIBBER_BASE_URL)
            .header(
                "Authorization",
                format!("Bearer {}", env_var("TIBBER_API_TOKEN")),
            )
            .header("content-type", "application/json")
            .body(body.clone())
            .send()
            .await?
            .json::<TibberRootResponse>()
            .await?
            .data
            .viewer
            .homes[0]
            .current_subscription
            .price_info
            .range
            .nodes
            .clone())
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct TibberDailyPrice {
    pub total: f64,
    #[serde(rename = "startsAt")]
    pub starts_at: DateTime<FixedOffset>,
}

#[derive(Deserialize)]
struct Range {
    pub nodes: Vec<TibberDailyPrice>,
}

#[derive(Deserialize)]
struct TibberPrices {
    pub range: Range,
}

#[derive(Deserialize)]
struct CurrentSubscription {
    #[serde(rename = "priceInfo")]
    pub price_info: TibberPrices,
}

#[derive(Deserialize)]
struct Home {
    #[serde(rename = "currentSubscription")]
    pub current_subscription: CurrentSubscription,
}

#[derive(Deserialize)]
struct Viewer {
    pub homes: Vec<Home>,
}

#[derive(Deserialize)]
struct Data {
    pub viewer: Viewer,
}

#[derive(Deserialize)]
struct TibberRootResponse {
    pub data: Data,
}
