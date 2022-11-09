use std::sync::Arc;

use actix_web::get;
use actix_web::web;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::Scope;
use chrono::NaiveDateTime;
use log::error;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use web::Path;

use crate::db;
use crate::service::temperature_logs::{generate_temperature_graph, TimePeriod};

pub fn temperature_logs() -> Scope {
    web::scope("/temperature_logs")
        .service(get_temperature_logs)
        .service(get_room_temperature_logs)
        .service(get_current_temps)
}

#[get("/")]
async fn get_temperature_logs(pool: web::Data<Arc<PgPool>>) -> impl Responder {
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
    pool: web::Data<Arc<PgPool>>,
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

#[derive(Serialize)]
struct RoomTemp {
    room_id: Uuid,
    room_name: String,
    temp: f64,
    time: NaiveDateTime,
}

#[get("/current")]
async fn get_current_temps(pool: web::Data<Arc<PgPool>>) -> impl Responder {
    let rooms = match db::rooms::get_rooms(pool.get_ref()).await {
        Ok(rooms) => rooms,
        Err(e) => {
            error!("{:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    match db::temperature_logs::get_current_temps(pool.get_ref(), &rooms).await {
        Ok(temps) => {
            let mut room_temps: Vec<RoomTemp> = vec![];
            temps.iter().for_each(|(room_id, room_temp)| {
                if let Some(room) = rooms.iter().find(|room| &room.id == room_id) {
                    room_temps.push(RoomTemp {
                        room_id: room.id,
                        room_name: room.name.clone(),
                        temp: room_temp.temp,
                        time: room_temp.time,
                    });
                }
            });
            HttpResponse::Ok().json(room_temps)
        }
        Err(e) => {
            error!("{:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
