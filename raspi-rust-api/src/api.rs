use std::io::Result;
use std::sync::Arc;

use actix_web::{get, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use log::info;
use serde::Deserialize;
use tokio::sync::mpsc::Sender;
use uuid::Uuid;

use crate::db::plugs::PlugsClient;
use crate::db::schedules::SchedulesClient;
use crate::db::temp_actions::TempActionsClient;
use crate::db::temperature_logs::TemperatureLogsClient;
use crate::domain::WorkMessage;
use crate::routes::plugs::plugs;
use crate::routes::rooms::rooms;
use crate::routes::schedules::schedules;
use crate::routes::temp_actions::temp_actions;
use crate::routes::temperature_logs::temperature_logs;

pub async fn start(
    sender: Sender<WorkMessage>,
    port: u16,
    plugs_client: Arc<PlugsClient>,
    schedules_client: Arc<SchedulesClient>,
    temp_actions_client: Arc<TempActionsClient>,
    temperature_logs_client: Arc<TemperatureLogsClient>,
) -> Result<()> {
    let sender = web::Data::new(sender);
    let plugs_client = web::Data::new(plugs_client);
    let schedules_client = web::Data::new(schedules_client);
    let temp_actions_client = web::Data::new(temp_actions_client);
    let temperature_logs_client = web::Data::new(temperature_logs_client);
    info!("Starting API");
    HttpServer::new(move || {
        App::new()
            .app_data(sender.clone())
            .app_data(plugs_client.clone())
            .app_data(schedules_client.clone())
            .app_data(temp_actions_client.clone())
            .app_data(temperature_logs_client.clone())
            .service(refresh)
            .service(health)
            .service(report_temp)
            .service(plugs())
            .service(rooms())
            .service(schedules())
            .service(temp_actions())
            .service(temperature_logs())
    })
    .shutdown_timeout(1)
    .bind(("0.0.0.0", port))
    .unwrap()
    .run()
    .await
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

#[derive(Debug, Deserialize)]
pub struct ReportRequest {
    // hum: i32,
    temp: f64,
}

#[get("/report_ht/{room}")]
async fn report_temp(
    room: web::Path<Uuid>,
    body: web::Query<ReportRequest>,
    sender: web::Data<Sender<WorkMessage>>,
) -> impl Responder {
    let room = room.into_inner();
    match sender.send(WorkMessage::TEMP(room, body.temp)).await {
        Ok(_) => HttpResponse::Ok(),
        Err(_) => HttpResponse::InternalServerError(),
    }
}
