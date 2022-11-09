use log::warn;
use sqlx::PgPool;
use thiserror::Error;

use crate::clients::tibber_client::{TibberClient, TibberClientError};
use crate::db::DbError;
use crate::domain::PriceInfo;
use crate::{db, now};

#[derive(Error, Debug)]
pub enum PriceServiceError {
    #[error("Tibber Client Error {0}")]
    TibberClientError(#[from] TibberClientError),
    #[error("DbError {0}")]
    DbError(#[from] DbError),
}

pub async fn get_current_price(
    tibber_client: &TibberClient,
    pool: &PgPool,
) -> Result<PriceInfo, PriceServiceError> {
    if let Some(price) = db::prices::get_price(pool, &now()).await? {
        Ok(price)
    } else {
        warn!("Couldn't find price for timestamp in DB, fetching from Tibber");
        Ok(tibber_client.get_current_price().await?)
    }
}

pub async fn fetch_and_store_prices(
    tibber_client: &TibberClient,
    pool: &PgPool,
) -> Result<(), PriceServiceError> {
    let prices = tibber_client.get_hourly_prices().await?;
    db::prices::insert_prices(pool, &prices).await?;
    Ok(())
}
