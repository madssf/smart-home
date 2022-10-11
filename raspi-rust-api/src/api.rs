use std::io::Result;

use actix_web::{get, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use log::{info, warn};
use serde::Deserialize;
use tokio::sync::mpsc::Sender;

use crate::WorkMessage;

pub struct AppState {
    sender: Sender<WorkMessage>,
}

pub async fn start(sender: Sender<WorkMessage>) -> Result<()> {
    info!("Starting API");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                sender: sender.clone(),
            }))
            .service(refresh)
            .service(health)
            .service(report_temp)
    })
    .shutdown_timeout(1)
    .bind(("0.0.0.0", 8080))
    .unwrap()
    .run()
    .await
}

#[get("/_/health")]
async fn health(_req: HttpRequest) -> impl Responder {
    HttpResponse::Ok().body("Healthy!")
}

#[get("/trigger_refresh")]
async fn refresh(data: web::Data<AppState>) -> impl Responder {
    match &data.sender.send(WorkMessage::REFRESH).await {
        Ok(_) => HttpResponse::Ok().body("Ok"),
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("Failed to refresh, error: {}", e))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ReportRequest {
    hum: i32,
    temp: f64,
}

#[get("/report_ht/{room}")]
async fn report_temp(
    room: web::Path<String>,
    body: web::Query<ReportRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    let room = room.into_inner();
    match data.sender.send(WorkMessage::TEMP(room, body.temp)).await {
        Ok(_) => HttpResponse::Ok(),
        Err(_) => HttpResponse::InternalServerError(),
    }
}
