use std::str::FromStr;
use std::sync::Arc;

use actix_web::{delete, get, post, web, HttpResponse, Responder, Scope};
use log::{error, info};
use uuid::Uuid;

use crate::db::plugs::PlugsClient;

pub fn plugs() -> Scope {
    web::scope("/plugs")
        .service(get_plugs)
        .service(create_plug)
        .service(update_plug)
        .service(delete_plug)
}

#[get("/")]
async fn get_plugs(plugs_client: web::Data<Arc<PlugsClient>>) -> impl Responder {
    info!("plugs");
    match plugs_client.get_ref().get_plugs().await {
        Ok(plugs) => HttpResponse::Ok().json(plugs),
        Err(e) => {
            error!("{:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[derive(serde::Deserialize)]
pub struct PlugRequest {
    name: String,
    ip: String,
    username: String,
    password: String,
}

#[post("/")]
async fn create_plug(
    plugs_client: web::Data<Arc<PlugsClient>>,
    body: web::Json<PlugRequest>,
) -> impl Responder {
    match plugs_client
        .get_ref()
        .create_plug(
            body.name.as_ref(),
            body.ip.as_ref(),
            body.username.as_ref(),
            body.password.as_ref(),
        )
        .await
    {
        Ok(plugs) => HttpResponse::Ok().json(plugs),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[post("/{id}")]
async fn update_plug(
    plugs_client: web::Data<Arc<PlugsClient>>,
    id: web::Path<String>,
    body: web::Json<PlugRequest>,
) -> impl Responder {
    let uuid = match Uuid::from_str(&id.into_inner()) {
        Ok(uuid) => uuid,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };
    match plugs_client
        .get_ref()
        .update_plug(
            &uuid,
            body.name.as_ref(),
            body.ip.as_ref(),
            body.username.as_ref(),
            body.password.as_ref(),
        )
        .await
    {
        Ok(plugs) => HttpResponse::Ok().json(plugs),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[delete("/{id}")]
async fn delete_plug(
    plugs_client: web::Data<Arc<PlugsClient>>,
    id: web::Path<String>,
) -> impl Responder {
    let uuid = match Uuid::from_str(&id.into_inner()) {
        Ok(uuid) => uuid,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };
    match plugs_client.get_ref().delete_plug(&uuid).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
