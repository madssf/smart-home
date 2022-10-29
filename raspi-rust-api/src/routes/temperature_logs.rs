use actix_web::get;
use actix_web::web;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::Scope;
use log::error;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;
use web::Path;

use crate::db;
use crate::service::temperature_logs::{generate_temperature_graph, TimePeriod};

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

#[derive(Deserialize)]
struct RoomLogsParams {
    room_id: Uuid,
    time_period: TimePeriod,
}

#[get("/{room_id}/{time_period}")]
async fn get_room_temperature_logs(
    path: Path<RoomLogsParams>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    let room_logs =
        match db::temperature_logs::get_room_temp_logs(pool.get_ref(), &path.room_id).await {
            Ok(logs) => logs,
            Err(e) => {
                error!("{:?}", e);
                return HttpResponse::InternalServerError().finish();
            }
        };

    HttpResponse::Ok().json(generate_temperature_graph(room_logs, &path.time_period))
}
