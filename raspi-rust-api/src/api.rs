use std::sync::Arc;

use actix_web::dev::Server;
use actix_web::{get, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use log::info;
use sqlx::PgPool;
use tokio::sync::mpsc::Sender;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::clients::shelly_client::ShellyClient;
use crate::clients::tibber_client::TibberClient;
use crate::domain::{ActionType, WorkMessage};
use crate::routes::buttons::buttons;
use crate::routes::notification_settings::notification_settings;
use crate::routes::plugs::plugs;
use crate::routes::prices::prices;
use crate::routes::rooms::rooms;
use crate::routes::schedules::schedules;
use crate::routes::temp_actions::temp_actions;
use crate::routes::temp_sensors::temp_sensors;
use crate::routes::temperature_logs::temperature_logs;
use crate::service::consumption_cache::ConsumptionCache;

pub async fn start(
    sender: Sender<WorkMessage>,
    host: String,
    port: u16,
    tibber_client: Arc<TibberClient>,
    shelly_client: Arc<ShellyClient>,
    consumption_cache: Arc<RwLock<ConsumptionCache>>,
    pool: Arc<PgPool>,
) -> Result<Server, std::io::Error> {
    let sender = web::Data::new(sender);
    let pool = web::Data::new(pool);
    let tibber_client = web::Data::new(tibber_client);
    let shelly_client = web::Data::new(shelly_client);
    let consumption_cache = web::Data::new(consumption_cache);
    info!("Starting API on host {}, port {}", host, port);
    let server = HttpServer::new(move || {
        App::new()
            .app_data(sender.clone())
            .app_data(pool.clone())
            .app_data(tibber_client.clone())
            .service(refresh)
            .service(health)
            .service(trigger_button)
            .service(plugs(shelly_client.clone()))
            .service(rooms())
            .service(buttons())
            .service(schedules())
            .service(temp_actions())
            .service(temperature_logs())
            .service(temp_sensors())
            .service(prices(consumption_cache.clone()))
            .service(notification_settings())
    })
    .shutdown_timeout(1)
    .bind((host, port))?
    .run();

    Ok(server)
}

#[get("/_/health")]
async fn health(_req: HttpRequest) -> impl Responder {
    HttpResponse::Ok().body("Healthy!")
}

#[get("/trigger_refresh")]
async fn refresh(sender: web::Data<Sender<WorkMessage>>) -> impl Responder {
    match sender.send(WorkMessage::REFRESH).await {
        Ok(_) => HttpResponse::Ok().body("Ok"),
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("Failed to refresh, error: {}", e))
        }
    }
}

#[get("/trigger_button/{button_id}/{action}")]
async fn trigger_button(
    sender: web::Data<Sender<WorkMessage>>,
    param: web::Path<(Uuid, ActionType)>,
) -> impl Responder {
    match sender.send(WorkMessage::BUTTON(param.0, param.1, 1)).await {
        Ok(_) => HttpResponse::Ok().body("Ok"),
        Err(e) => HttpResponse::InternalServerError()
            .body(format!("Failed to trigger button, error: {}", e)),
    }
}
