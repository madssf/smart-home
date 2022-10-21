use std::str::FromStr;
use std::sync::Arc;

use actix_web::{delete, get, post, web, HttpResponse, Responder, Scope};
use log::error;
use sqlx::types::ipnetwork::IpNetwork;
use uuid::Uuid;

use crate::db::plugs::PlugsClient;
use crate::domain::Plug;

pub fn plugs(plugs_client: Arc<PlugsClient>) -> Scope {
    let plugs_client = web::Data::new(plugs_client);

    web::scope("/plugs")
        .app_data(plugs_client)
        .service(get_plugs)
        .service(create_plug)
        .service(update_plug)
        .service(delete_plug)
}

#[get("/")]
async fn get_plugs(plugs_client: web::Data<Arc<PlugsClient>>) -> impl Responder {
    match plugs_client.get_ref().get_plugs().await {
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
async fn create_plug(
    plugs_client: web::Data<Arc<PlugsClient>>,
    body: web::Json<PlugRequest>,
) -> impl Responder {
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

    match plugs_client.get_ref().create_plug(new_plug).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[post("/{id}")]
async fn update_plug(
    plugs_client: web::Data<Arc<PlugsClient>>,
    id: web::Path<Uuid>,
    body: web::Json<PlugRequest>,
) -> impl Responder {
    let ip = match IpNetwork::from_str(&body.ip) {
        Ok(ip) => ip,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    match plugs_client
        .get_ref()
        .update_plug(Plug {
            id: id.into_inner(),
            ip,
            name: body.name.clone(),
            username: body.username.clone(),
            password: body.password.clone(),
            room_id: body.room_id,
        })
        .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[delete("/{id}")]
async fn delete_plug(
    plugs_client: web::Data<Arc<PlugsClient>>,
    id: web::Path<Uuid>,
) -> impl Responder {
    match plugs_client.get_ref().delete_plug(&id.into_inner()).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}
