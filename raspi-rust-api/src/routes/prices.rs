use std::sync::Arc;

use actix_web::{get, web, HttpResponse, Responder, Scope};
use log::error;

use crate::prices::TibberClient;

pub fn prices(tibber_client: web::Data<Arc<TibberClient>>) -> Scope {
    web::scope("/prices")
        .app_data(tibber_client)
        .service(get_current_price)
        .service(get_consumption)
}

#[get("/current")]
async fn get_current_price(tibber_client: web::Data<Arc<TibberClient>>) -> impl Responder {
    match tibber_client.get_ref().get_current_price().await {
        Ok(price) => HttpResponse::Ok().json(price),
        Err(e) => {
            error!("{:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[get("/consumption")]
async fn get_consumption(tibber_client: web::Data<Arc<TibberClient>>) -> impl Responder {
    match tibber_client.get_ref().get_consumption().await {
        Ok(consumption) => HttpResponse::Ok().json(consumption),
        Err(e) => {
            error!("{:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
