use std::ops::Sub;

use chrono::Duration;
use itertools::Itertools;
use log::{info, warn};
use sqlx::PgPool;
use thiserror::Error;

use crate::clients::tibber_client::{TibberClient, TibberClientError, TibberDailyPrice};
use crate::db::DbError;
use crate::domain::{PriceInfo, PriceLevel};
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
    let new_prices = tibber_client.get_hourly_prices().await?;
    let daily_prices = tibber_client.get_daily_prices().await?;
    db::prices::insert_prices(pool, &calculate_price_levels(daily_prices, new_prices)).await?;
    Ok(())
}

pub fn calculate_price_levels(
    daily_prices: Vec<TibberDailyPrice>,
    new_prices: Vec<PriceInfo>,
) -> Vec<PriceInfo> {
    if daily_prices.is_empty() {
        warn!("Got no daily prices from tibber");
        return new_prices;
    }

    if daily_prices[daily_prices.len() - 1].starts_at.naive_local() < now().sub(Duration::days(2)) {
        warn!("Got old daily prices from Tibber");
        return new_prices;
    }

    let sorted: Vec<TibberDailyPrice> = daily_prices
        .into_iter()
        .sorted_by(|a, b| a.total.partial_cmp(&b.total).unwrap())
        .collect();

    let median = sorted.get(sorted.len() / 2).unwrap().total;

    if median <= 0.0 {
        warn!("Got 0 or negative average daily price from Tibber");
        return new_prices;
    }

    info!("Calculating prices with daily median of: {}", &median);

    new_prices
        .into_iter()
        .map(|mut price_info| {
            price_info.price_level = Some(get_price_level(&price_info, median));
            price_info
        })
        .collect()
}

fn get_price_level(price_info: &PriceInfo, median: f64) -> PriceLevel {
    let ratio = price_info.amount / median;
    let daily_level = if ratio < 0.5 {
        PriceLevel::VeryCheap
    } else if ratio < 0.85 {
        PriceLevel::Cheap
    } else if ratio < 1.15 {
        PriceLevel::Normal
    } else if ratio < 1.5 {
        PriceLevel::Expensive
    } else {
        PriceLevel::VeryExpensive
    };
    let daily_index = daily_level.index_of() as f64;
    let hourly_index = price_info.ext_price_level.index_of() as f64;
    let actual_index = ((2.0 * daily_index + hourly_index) / 3.0).round() as i32;
    PriceLevel::from(actual_index)
}
