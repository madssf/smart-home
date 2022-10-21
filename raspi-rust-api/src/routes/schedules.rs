use std::sync::Arc;

use actix_web::{delete, get, post, web, HttpResponse, Responder, Scope};
use chrono::{NaiveTime, Weekday};
use log::error;
use uuid::Uuid;

use crate::db::schedules::SchedulesClient;
use crate::domain::{PriceLevel, Schedule};

pub fn schedules() -> Scope {
    web::scope("/schedules")
        .service(get_schedules)
        .service(create_schedule)
        .service(update_schedule)
        .service(delete_schedule)
}

#[get("/")]
async fn get_schedules(schedules_client: web::Data<Arc<SchedulesClient>>) -> impl Responder {
    match schedules_client.get_ref().get_schedules().await {
        Ok(schedules) => HttpResponse::Ok().json(schedules),
        Err(e) => {
            error!("{:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[derive(serde::Deserialize)]
pub struct ScheduleRequest {
    price_level: PriceLevel,
    days: Vec<Weekday>,
    time_windows: Vec<(NaiveTime, NaiveTime)>,
    temp: f64,
    room_ids: Vec<Uuid>,
}

impl TryInto<Schedule> for ScheduleRequest {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Schedule, Self::Error> {
        Schedule::new(
            &self.price_level,
            self.days,
            self.time_windows,
            self.temp,
            self.room_ids,
        )
    }
}

#[post("/")]
async fn create_schedule(
    schedules_client: web::Data<Arc<SchedulesClient>>,
    body: web::Json<ScheduleRequest>,
) -> impl Responder {
    let new_schedule = match body.into_inner().try_into() {
        Ok(schedule) => schedule,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };
    match schedules_client
        .get_ref()
        .create_schedule(new_schedule)
        .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[post("/{id}")]
async fn update_schedule(
    schedules_client: web::Data<Arc<SchedulesClient>>,
    id: web::Path<Uuid>,
    body: web::Json<ScheduleRequest>,
) -> impl Responder {
    match schedules_client
        .get_ref()
        .update_schedule(Schedule {
            id: id.into_inner(),
            price_level: body.price_level,
            days: body.days.clone(),
            time_windows: body.time_windows.clone(),
            temp: body.temp,
            room_ids: body.room_ids.clone(),
        })
        .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[delete("/{id}")]
async fn delete_schedule(
    schedules_client: web::Data<Arc<SchedulesClient>>,
    id: web::Path<Uuid>,
) -> impl Responder {
    match schedules_client
        .get_ref()
        .delete_schedule(&id.into_inner())
        .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
