use std::io::Result;

use actix_web::{get, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use log::{info, warn};
use serde::Deserialize;
use sqlx::{PgConnection, PgPool};
use tokio::sync::mpsc::Sender;

use crate::WorkMessage;

pub async fn start(sender: Sender<WorkMessage>, port: u16) -> Result<()> {
    let sender = web::Data::new(sender);
    info!("Starting API");
    HttpServer::new(move || {
        App::new()
            .app_data(sender.clone())
            .service(refresh)
            .service(health)
            .service(report_temp)
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
    hum: i32,
    temp: f64,
}

#[get("/report_ht/{room}")]
async fn report_temp(
    room: web::Path<String>,
    body: web::Query<ReportRequest>,
    sender: web::Data<Sender<WorkMessage>>,
) -> impl Responder {
    let room = room.into_inner();
    match sender.send(WorkMessage::TEMP(room, body.temp)).await {
        Ok(_) => HttpResponse::Ok(),
        Err(_) => HttpResponse::InternalServerError(),
    }
}
