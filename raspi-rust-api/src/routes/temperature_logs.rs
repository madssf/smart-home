use std::sync::Arc;

use actix_web::get;
use actix_web::web;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::Scope;
use log::error;

use crate::db::temperature_logs::TemperatureLogsClient;

pub fn temperature_logs() -> Scope {
    web::scope("/temperature_logs").service(get_temperature_logs)
}

#[get("/")]
async fn get_temperature_logs(
    temperature_logs_client: web::Data<Arc<TemperatureLogsClient>>,
) -> impl Responder {
    match temperature_logs_client.get_ref().get_temp_logs().await {
        Ok(logs) => HttpResponse::Ok().json(logs),
        Err(e) => {
            error!("{:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
