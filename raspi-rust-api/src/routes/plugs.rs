use std::str::FromStr;
use std::sync::Arc;

use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use log::error;
use sqlx::types::ipnetwork::IpNetwork;
use sqlx::PgPool;
use uuid::Uuid;

use crate::clients::shelly_client::ShellyClient;
use crate::domain::Plug;
use crate::routes::lib::{error_response, internal_server_error};
use crate::{db, service};

pub fn plugs_router(pool: Arc<PgPool>, shelly_client: Arc<ShellyClient>) -> Router {
    Router::new()
        .route("/", get(get_plugs).post(create_plug))
        .route("/:id", post(update_plug).delete(delete_plug))
        .route("/status", get(get_plug_statuses))
        .layer(Extension(pool))
        .layer(Extension(shelly_client))
}

async fn get_plugs(Extension(pool): Extension<Arc<PgPool>>) -> impl IntoResponse {
    db::plugs::get_plugs(&pool)
        .await
        .map(|plugs| Json(plugs.into_iter().map(|p| p.to_json()).collect::<Vec<_>>()))
        .map_err(internal_server_error)
}

#[derive(serde::Serialize)]
pub struct PlugResponse {
    id: Uuid,
    name: String,
    ip: String,
    username: String,
    password: String,
    room_id: Uuid,
    scheduled: bool,
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
            scheduled: self.scheduled,
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
    scheduled: bool,
}

async fn create_plug(
    Extension(pool): Extension<Arc<PgPool>>,
    Json(body): Json<PlugRequest>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let new_plug = match Plug::new(
        &body.name,
        &body.ip,
        &body.username,
        &body.password,
        &body.room_id,
        &body.scheduled,
    ) {
        Ok(plug) => plug,
        Err(e) => {
            error!("{}", e);
            return Err(error_response(
                format!("Failed to create plug: {}", e),
                StatusCode::BAD_REQUEST,
            ));
        }
    };

    db::plugs::create_plug(&pool, &new_plug)
        .await
        .map(|_| (StatusCode::OK, Json("Plug created successfully.")))
        .map_err(|e| {
            error!("{}", e.to_string());
            error_response(
                "Failed to create plug.".to_string(),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })
}

async fn update_plug(
    Extension(pool): Extension<Arc<PgPool>>,
    Path(id): Path<Uuid>,
    Json(body): Json<PlugRequest>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let ip = match IpNetwork::from_str(&body.ip) {
        Ok(ip) => ip,
        Err(_) => {
            return Err(error_response(
                "Invalid IP address.".to_string(),
                StatusCode::BAD_REQUEST,
            ))
        }
    };

    match db::plugs::update_plug(
        &pool,
        Plug {
            id,
            ip,
            name: body.name.clone(),
            username: body.username.clone(),
            password: body.password.clone(),
            room_id: body.room_id,
            scheduled: body.scheduled,
        },
    )
    .await
    {
        Ok(_) => Ok((StatusCode::OK, Json("Plug updated successfully."))),
        Err(e) => {
            error!("{}", e.to_string());
            Err(error_response(
                "Failed to update plug.".to_string(),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

async fn delete_plug(
    Extension(pool): Extension<Arc<PgPool>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match db::plugs::delete_plug(&pool, &id).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

async fn get_plug_statuses(
    Extension(pool): Extension<Arc<PgPool>>,
    Extension(shelly_client): Extension<Arc<ShellyClient>>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let plugs = match db::plugs::get_plugs(&pool).await {
        Ok(plugs) => plugs,
        Err(e) => {
            error!("{:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    match service::plugs::get_plug_statuses(&plugs, &shelly_client).await {
        Ok(plug_statuses) => Ok(Json(plug_statuses)),
        Err(e) => {
            error!("{:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
