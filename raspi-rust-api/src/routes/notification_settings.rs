use std::sync::Arc;

use actix_web::{get, post, web, HttpResponse, Responder, Scope};
use log::error;
use sqlx::PgPool;

use crate::db;
use crate::domain::NotificationSettings;

pub fn notification_settings() -> Scope {
    web::scope("/notification_settings")
        .service(get_settings)
        .service(upsert_settings)
}

#[get("/")]
async fn get_settings(pool: web::Data<Arc<PgPool>>) -> impl Responder {
    match db::notification_settings::get_notification_settings(pool.get_ref()).await {
        Ok(settings) => HttpResponse::Ok().json(settings),
        Err(e) => {
            error!("{:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[post("/")]
async fn upsert_settings(
    pool: web::Data<Arc<PgPool>>,
    body: web::Json<NotificationSettings>,
) -> impl Responder {
    match db::notification_settings::upsert_notification_settings(
        pool.get_ref(),
        &body.into_inner(),
    )
    .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            error!("{:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
