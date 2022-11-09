use std::collections::HashMap;
use std::sync::Arc;

use actix_web::{delete, get, post, web, HttpResponse, Responder, Scope};
use chrono::{NaiveTime, Weekday};
use log::error;
use sqlx::PgPool;
use uuid::Uuid;

use crate::clients::tibber_client::TibberClient;
use crate::db::rooms;
use crate::domain::{PriceLevel, Schedule};
use crate::{db, now};

pub fn schedules() -> Scope {
    web::scope("/schedules")
        .service(get_schedules)
        .service(create_schedule)
        .service(update_schedule)
        .service(delete_schedule)
        .service(get_active_schedules)
}

#[get("/")]
async fn get_schedules(pool: web::Data<PgPool>) -> impl Responder {
    match db::schedules::get_schedules(pool.get_ref()).await {
        Ok(schedules) => HttpResponse::Ok().json(schedules),
        Err(e) => {
            error!("{:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[derive(serde::Deserialize)]
pub struct ScheduleRequest {
    pub temps: HashMap<PriceLevel, f64>,
    pub days: Vec<Weekday>,
    pub time_windows: Vec<(NaiveTime, NaiveTime)>,
    pub room_ids: Vec<Uuid>,
}

impl TryInto<Schedule> for ScheduleRequest {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Schedule, Self::Error> {
        Schedule::new(self.temps, self.days, self.time_windows, self.room_ids)
    }
}

#[post("/")]
async fn create_schedule(
    pool: web::Data<PgPool>,
    body: web::Json<ScheduleRequest>,
) -> impl Responder {
    let new_schedule = match body.into_inner().try_into() {
        Ok(schedule) => schedule,
        Err(e) => {
            error!("{}", e);
            return HttpResponse::BadRequest().finish();
        }
    };
    match db::schedules::create_schedule(pool.get_ref(), new_schedule).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            error!("{}", e.to_string());
            HttpResponse::BadRequest().finish()
        }
    }
}

#[post("/{id}")]
async fn update_schedule(
    pool: web::Data<PgPool>,
    id: web::Path<Uuid>,
    body: web::Json<ScheduleRequest>,
) -> impl Responder {
    match db::schedules::update_schedule(
        pool.get_ref(),
        Schedule {
            id: id.into_inner(),
            temps: body.temps.clone(),
            days: body.days.clone(),
            time_windows: body.time_windows.clone(),
            room_ids: body.room_ids.clone(),
        },
    )
    .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[delete("/{id}")]
async fn delete_schedule(pool: web::Data<PgPool>, id: web::Path<Uuid>) -> impl Responder {
    match db::schedules::delete_schedule(pool.get_ref(), &id.into_inner()).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[derive(serde::Serialize)]
pub struct ActiveSchedule {
    pub room_id: Uuid,
    pub schedule: Option<Schedule>,
    pub temp: Option<f64>,
}

#[get("/active")]
async fn get_active_schedules(
    pool: web::Data<PgPool>,
    tibber_client: web::Data<Arc<TibberClient>>,
) -> impl Responder {
    let rooms = match rooms::get_rooms(pool.get_ref()).await {
        Ok(rooms) => rooms,
        Err(e) => {
            error!("{:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };
    let price_info = match tibber_client.get_current_price().await {
        Ok(price) => price,
        Err(e) => {
            error!("{:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };
    let mut active_schedules: Vec<ActiveSchedule> = vec![];
    for room in rooms {
        match db::schedules::get_matching_schedule(pool.get_ref(), &room.id, &now()).await {
            Ok(schedule) => {
                let temp = if let Some(schedule) = schedule.clone() {
                    Some(schedule.get_temp(&price_info.ext_price_level))
                } else {
                    None
                };
                active_schedules.push(ActiveSchedule {
                    room_id: room.id,
                    schedule,
                    temp,
                })
            }
            Err(e) => {
                error!("{:?}", e);
                return HttpResponse::InternalServerError().finish();
            }
        };
    }

    HttpResponse::Ok().json(active_schedules)
}
