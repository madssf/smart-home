use std::str::FromStr;

use actix_web::{delete, get, post, web, HttpResponse, Responder, Scope};
use log::error;
use sqlx::types::ipnetwork::IpNetwork;
use sqlx::PgPool;
use uuid::Uuid;

use crate::db;
use crate::domain::Plug;

pub fn plugs() -> Scope {
    web::scope("/plugs")
        .service(get_plugs)
        .service(create_plug)
        .service(update_plug)
        .service(delete_plug)
}

#[get("/")]
async fn get_plugs(pool: web::Data<PgPool>) -> impl Responder {
    match db::plugs::get_plugs(pool.get_ref()).await {
        Ok(plugs) => {
            let json: Vec<PlugResponse> = plugs.iter().map(|plug| plug.to_json()).collect();
            HttpResponse::Ok().json(json)
        }
        Err(e) => {
            error!("{:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[derive(serde::Serialize)]
pub struct PlugResponse {
    id: Uuid,
    name: String,
    ip: String,
    username: String,
    password: String,
    room_id: Uuid,
}

impl Plug {
    pub fn to_json(&self) -> PlugResponse {
        PlugResponse {
            id: self.id,
            name: self.name.clone(),
            ip: self.ip.ip().to_string(),
            username: self.username.clone(),
            password: self.password.clone(),
            room_id: self.room_id,
        }
    }
}

#[derive(serde::Deserialize)]
pub struct PlugRequest {
    name: String,
    ip: String,
    username: String,
    password: String,
    room_id: Uuid,
}

#[post("/")]
async fn create_plug(pool: web::Data<PgPool>, body: web::Json<PlugRequest>) -> impl Responder {
    let new_plug = match Plug::new(
        &body.name,
        &body.ip,
        &body.username,
        &body.password,
        &body.room_id,
    ) {
        Ok(plug) => plug,
        Err(e) => {
            error!("{}", e);
            return HttpResponse::BadRequest().json(e.to_string());
        }
    };

    match db::plugs::create_plug(pool.get_ref(), new_plug).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[post("/{id}")]
async fn update_plug(
    pool: web::Data<PgPool>,
    id: web::Path<Uuid>,
    body: web::Json<PlugRequest>,
) -> impl Responder {
    let ip = match IpNetwork::from_str(&body.ip) {
        Ok(ip) => ip,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    match db::plugs::update_plug(
        pool.get_ref(),
        Plug {
            id: id.into_inner(),
            ip,
            name: body.name.clone(),
            username: body.username.clone(),
            password: body.password.clone(),
            room_id: body.room_id,
        },
    )
    .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[delete("/{id}")]
async fn delete_plug(pool: web::Data<PgPool>, id: web::Path<Uuid>) -> impl Responder {
    match db::plugs::delete_plug(pool.get_ref(), &id.into_inner()).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
