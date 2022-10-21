use std::sync::Arc;

use actix_web::{delete, get, post, web, HttpResponse, Responder, Scope};
use log::error;
use uuid::Uuid;

use crate::db::rooms::RoomsClient;
use crate::domain::Room;

pub fn rooms() -> Scope {
    web::scope("/rooms")
        .service(get_rooms)
        .service(create_room)
        .service(update_room)
        .service(delete_room)
}

#[get("/")]
async fn get_rooms(rooms_client: web::Data<Arc<RoomsClient>>) -> impl Responder {
    match rooms_client.get_ref().get_rooms().await {
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
async fn create_room(
    rooms_client: web::Data<Arc<RoomsClient>>,
    body: web::Json<RoomRequest>,
) -> impl Responder {
    match rooms_client.get_ref().create_room(&body.name).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[post("/{id}")]
async fn update_room(
    rooms_client: web::Data<Arc<RoomsClient>>,
    id: web::Path<Uuid>,
    body: web::Json<RoomRequest>,
) -> impl Responder {
    match rooms_client
        .get_ref()
        .update_room(&Room {
            id: id.into_inner(),
            name: body.name.to_string(),
        })
        .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[delete("/{id}")]
async fn delete_room(
    rooms_client: web::Data<Arc<RoomsClient>>,
    id: web::Path<Uuid>,
) -> impl Responder {
    match rooms_client.get_ref().delete_room(&id.into_inner()).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
