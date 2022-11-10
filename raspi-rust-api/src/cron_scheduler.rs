use std::sync::Arc;
use std::time::Duration;

use anyhow::anyhow;
use log::{info, warn};
use sqlx::PgPool;
use tokio::time::sleep;

use crate::clients::tibber_client::TibberClient;
use crate::service::prices;

pub async fn start(
    tibber_client: Arc<TibberClient>,
    pool: Arc<PgPool>,
) -> Result<(), anyhow::Error> {
    let task = tokio::task::spawn(async move {
        loop {
            match prices::fetch_and_store_prices(tibber_client.as_ref(), pool.as_ref()).await {
                Ok(_) => {
                    info!("Prices fetched and saved, waiting for 8 hours.");
                    sleep(Duration::from_secs(8 * 60 * 60)).await;
                }
                Err(e) => {
                    warn!(
                        "Failed to fetch prices, trying again in 5 minutes, error: {}",
                        e
                    );
                    sleep(Duration::from_secs(5 * 60)).await;
                }
            }
        }
    });
    match task.await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(e)),
    }
}
