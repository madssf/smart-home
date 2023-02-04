use std::str::FromStr;
use std::sync::Arc;

use actix_web::{delete, get, post, web, HttpResponse, Responder, Scope};
use log::error;
use sqlx::types::ipnetwork::IpNetwork;
use sqlx::PgPool;
use uuid::Uuid;

use crate::db;
use crate::domain::Button;

pub fn buttons() -> Scope {
    web::scope("/buttons")
        .service(get_buttons)
        .service(create_button)
        .service(update_button)
        .service(delete_button)
}

#[get("/")]
async fn get_buttons(pool: web::Data<Arc<PgPool>>) -> impl Responder {
    match db::buttons::get_buttons(pool.get_ref()).await {
        Ok(buttons) => {
            let json: Vec<ButtonResponse> = buttons.iter().map(|b| b.to_json()).collect();

            HttpResponse::Ok().json(json)
        }
        Err(e) => {
            error!("{:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[derive(serde::Serialize)]
pub struct ButtonResponse {
    id: Uuid,
    name: String,
    ip: String,
    username: String,
    password: String,
    plug_ids: Vec<Uuid>,
}

impl Button {
    pub fn to_json(&self) -> ButtonResponse {
        ButtonResponse {
            id: self.id,
            name: self.name.clone(),
            ip: self.ip.ip().to_string(),
            username: self.username.clone(),
            password: self.password.clone(),
            plug_ids: self.plug_ids.clone(),
        }
    }
}

#[derive(serde::Deserialize)]
pub struct ButtonRequest {
    name: String,
    ip: String,
    username: String,
    password: String,
    plug_ids: Vec<Uuid>,
}

#[post("/")]
async fn create_button(
    pool: web::Data<Arc<PgPool>>,
    body: web::Json<ButtonRequest>,
) -> impl Responder {
    let new_button = match Button::new(
        &body.name,
        &body.ip,
        &body.username,
        &body.password,
        &body.plug_ids,
    ) {
        Ok(button) => button,
        Err(e) => {
            error!("{}", e);
            return HttpResponse::BadRequest().json(e.to_string());
        }
    };

    match db::buttons::create_button(pool.get_ref(), &new_button).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[post("/{id}")]
async fn update_button(
    pool: web::Data<Arc<PgPool>>,
    id: web::Path<Uuid>,
    body: web::Json<ButtonRequest>,
) -> impl Responder {
    let ip = match IpNetwork::from_str(&body.ip) {
        Ok(ip) => ip,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    match db::buttons::update_button(
        pool.get_ref(),
        &Button {
            id: id.into_inner(),
            ip,
            name: body.name.clone(),
            username: body.username.clone(),
            password: body.password.clone(),
            plug_ids: body.plug_ids.clone(),
        },
    )
    .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[delete("/{id}")]
async fn delete_button(pool: web::Data<Arc<PgPool>>, id: web::Path<Uuid>) -> impl Responder {
    match db::buttons::delete_button(pool.get_ref(), &id.into_inner()).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
