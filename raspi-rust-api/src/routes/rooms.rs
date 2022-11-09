use std::sync::Arc;

use actix_web::{delete, get, post, web, HttpResponse, Responder, Scope};
use log::error;
use sqlx::PgPool;
use uuid::Uuid;

use crate::db;
use crate::domain::Room;

pub fn rooms() -> Scope {
    web::scope("/rooms")
        .service(get_rooms)
        .service(create_room)
        .service(update_room)
        .service(delete_room)
}

#[get("/")]
async fn get_rooms(pool: web::Data<Arc<PgPool>>) -> impl Responder {
    match db::rooms::get_rooms(pool.get_ref()).await {
        Ok(rooms) => HttpResponse::Ok().json(rooms),
        Err(e) => {
            error!("{:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[derive(serde::Deserialize)]
pub struct RoomRequest {
    name: String,
}

#[post("/")]
async fn create_room(pool: web::Data<Arc<PgPool>>, body: web::Json<RoomRequest>) -> impl Responder {
    match db::rooms::create_room(pool.get_ref(), &body.name).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[post("/{id}")]
async fn update_room(
    pool: web::Data<Arc<PgPool>>,
    id: web::Path<Uuid>,
    body: web::Json<RoomRequest>,
) -> impl Responder {
    match db::rooms::update_room(
        pool.get_ref(),
        &Room {
            id: id.into_inner(),
            name: body.name.to_string(),
        },
    )
    .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[delete("/{id}")]
async fn delete_room(pool: web::Data<Arc<PgPool>>, id: web::Path<Uuid>) -> impl Responder {
    match db::rooms::delete_room(pool.get_ref(), &id.into_inner()).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
