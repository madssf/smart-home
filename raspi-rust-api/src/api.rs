use std::io::Result;

use actix_web::{get, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use log::{info, warn};
use serde::Deserialize;
use tokio::sync::mpsc::Sender;

pub struct AppState {
    sender: Sender<String>,
}

pub async fn start(sender: Sender<String>) -> Result<()> {
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
    match &data.sender.send("Refresh".to_string()).await {
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
    warn!(
        "Received hum: {}, temp: {} in {}",
        body.hum,
        body.temp,
        room.into_inner()
    );
    HttpResponse::Ok()
}
