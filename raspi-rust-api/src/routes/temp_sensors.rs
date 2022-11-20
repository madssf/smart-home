use std::sync::Arc;

use actix_web::{delete, get, HttpResponse, post, Responder, Scope, web};
use log::error;
use sqlx::PgPool;

use crate::db;
use crate::domain::TempSensor;

pub fn temp_sensors() -> Scope {
    web::scope("/temp_sensors")
        .service(get_temp_sensors)
        .service(create_sensor)
        .service(delete_sensor)
}

#[get("/")]
async fn get_temp_sensors(pool: web::Data<Arc<PgPool>>) -> impl Responder {
    match db::temp_sensors::get_temp_sensors(pool.get_ref()).await {
        Ok(sensors) => HttpResponse::Ok().json(sensors),
        Err(e) => {
            error!("{:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[post("/")]
async fn create_sensor(
    pool: web::Data<Arc<PgPool>>,
    body: web::Json<TempSensor>,
) -> impl Responder {
    match db::temp_sensors::insert_temp_sensor(pool.get_ref(), &body.into_inner()).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[delete("/{id}")]
async fn delete_sensor(pool: web::Data<Arc<PgPool>>, id: web::Path<String>) -> impl Responder {
    match db::temp_sensors::delete_temp_sensor(pool.get_ref(), &id.into_inner()).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
