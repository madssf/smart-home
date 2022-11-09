use std::sync::Arc;

use actix_web::{get, web, HttpResponse, Responder, Scope};
use log::error;
use serde::Serialize;
use sqlx::PgPool;
use tokio::sync::RwLock;

use crate::clients::tibber_client::TibberClient;
use crate::domain::Consumption;
use crate::service;
use crate::service::consumption_cache::ConsumptionCache;

pub fn prices(consumption_cache: web::Data<Arc<RwLock<ConsumptionCache>>>) -> Scope {
    web::scope("/prices")
        .app_data(consumption_cache)
        .service(get_current_price)
        .service(get_consumption)
        .service(get_live_consumption)
}

#[get("/current")]
async fn get_current_price(
    tibber_client: web::Data<Arc<TibberClient>>,
    pool: web::Data<Arc<PgPool>>,
) -> impl Responder {
    match service::prices::get_current_price(tibber_client.get_ref(), pool.get_ref()).await {
        Ok(price) => HttpResponse::Ok().json(price),
        Err(e) => {
            error!("{:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
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

#[get("/consumption")]
async fn get_consumption(tibber_client: web::Data<Arc<TibberClient>>) -> impl Responder {
    match tibber_client.get_ref().get_consumption().await {
        Ok(consumption) => {
            let json: Vec<ConsumptionGraphData> = consumption.iter().map(|v| v.into()).collect();
            HttpResponse::Ok().json(json)
        }
        Err(e) => {
            error!("{:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[get("/live_consumption")]
async fn get_live_consumption(
    consumption_cache: web::Data<Arc<RwLock<ConsumptionCache>>>,
) -> impl Responder {
    let cache = consumption_cache.read().await;
    HttpResponse::Ok().json(cache.get_all())
}
