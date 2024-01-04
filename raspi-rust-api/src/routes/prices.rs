use std::sync::Arc;

use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Extension, Json, Router};
use serde::Serialize;
use sqlx::PgPool;
use tokio::sync::RwLock;

use crate::clients::tibber_client::TibberClient;
use crate::domain::{Consumption, LiveConsumption};
use crate::routes::lib::internal_server_error;
use crate::service;
use crate::service::consumption_cache::ConsumptionCache;

pub fn prices_router(
    pool: Arc<PgPool>,
    tibber_client: Arc<TibberClient>,
    consumption_cache: Arc<RwLock<ConsumptionCache>>,
) -> Router {
    Router::new()
        .route("/current", get(get_current_price))
        .route("/consumption", get(get_consumption))
        .route("/live_consumption", get(get_live_consumption))
        .layer(Extension(pool))
        .layer(Extension(tibber_client))
        .layer(Extension(consumption_cache))
}

async fn get_current_price(
    Extension(tibber_client): Extension<Arc<TibberClient>>,
    Extension(pool): Extension<Arc<PgPool>>,
) -> impl IntoResponse {
    service::prices::get_current_price(&tibber_client, &pool)
        .await
        .map(Json)
        .map_err(internal_server_error)
}

#[derive(Serialize)]
struct ConsumptionGraphData {
    label: String,
    kwh: Option<f64>,
}

impl From<&Consumption> for ConsumptionGraphData {
    fn from(value: &Consumption) -> Self {
        Self {
            label: value.to.format("%H:%M").to_string(),
            kwh: value.kwh,
        }
    }
}

async fn get_consumption(
    Extension(tibber_client): Extension<Arc<TibberClient>>,
) -> impl IntoResponse {
    tibber_client
        .get_consumption()
        .await
        .map(|consumption| {
            let json: Vec<ConsumptionGraphData> = consumption.iter().map(|v| v.into()).collect();
            Json(json)
        })
        .map_err(internal_server_error)
}

async fn get_live_consumption(
    Extension(consumption_cache): Extension<Arc<RwLock<ConsumptionCache>>>,
) -> impl IntoResponse {
    let cache = consumption_cache.read().await;
    let res = cache
        .get_all()
        .into_iter()
        .cloned()
        .collect::<Vec<LiveConsumption>>();
    Json(res)
}
