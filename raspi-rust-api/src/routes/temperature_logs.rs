use actix_web::get;
use actix_web::web;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::Scope;
use log::error;
use sqlx::PgPool;
use uuid::Uuid;
use web::Path;

use crate::db;

pub fn temperature_logs() -> Scope {
    web::scope("/temperature_logs")
        .service(get_temperature_logs)
        .service(get_room_temperature_logs)
}

#[get("/")]
async fn get_temperature_logs(pool: web::Data<PgPool>) -> impl Responder {
    match db::temperature_logs::get_temp_logs(pool.get_ref()).await {
        Ok(logs) => HttpResponse::Ok().json(logs),
        Err(e) => {
            error!("{:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[get("/{room_id}")]
async fn get_room_temperature_logs(room_id: Path<Uuid>, pool: web::Data<PgPool>) -> impl Responder {
    match db::temperature_logs::get_room_temp_logs(pool.get_ref(), &room_id.into_inner()).await {
        Ok(logs) => HttpResponse::Ok().json(logs),
        Err(e) => {
            error!("{:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
