use std::io::Result;

use actix_web::{get, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use log::info;
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
            .service(greet)
    })
    .shutdown_timeout(1)
    .bind(("0.0.0.0", 8080))
    .unwrap()
    .run()
    .await
}

#[get("/_/health")]
async fn greet(_req: HttpRequest) -> impl Responder {
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
