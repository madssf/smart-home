use actix_web::dev::Server;
use actix_web::{get, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use log::info;
use serde::Deserialize;
use tokio::sync::mpsc::Sender;
use uuid::Uuid;

use crate::db::DbClients;
use crate::domain::WorkMessage;
use crate::routes::plugs::plugs;
use crate::routes::rooms::rooms;
use crate::routes::schedules::schedules;
use crate::routes::temp_actions::temp_actions;
use crate::routes::temperature_logs::temperature_logs;

pub async fn start(
    sender: Sender<WorkMessage>,
    host: String,
    port: u16,
    db_clients: DbClients,
) -> Result<Server, std::io::Error> {
    let sender = web::Data::new(sender);

    info!("Starting API on host {}, port {}", host, port);
    let server = HttpServer::new(move || {
        App::new()
            .app_data(sender.clone())
            .service(refresh)
            .service(health)
            .service(report_temp)
            .service(plugs(db_clients.plugs.clone()))
            .service(rooms(db_clients.rooms.clone()))
            .service(schedules(db_clients.schedules.clone()))
            .service(temp_actions(db_clients.temp_actions.clone()))
            .service(temperature_logs(db_clients.temperature_logs.clone()))
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
