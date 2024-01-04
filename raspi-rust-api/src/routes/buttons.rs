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

use crate::db;
use crate::domain::Button;
use crate::routes::lib::{error_response, internal_server_error};

pub fn buttons_router(pool: Arc<PgPool>) -> Router {
    Router::new()
        .route("/", get(get_buttons).post(create_button))
        .route("/:id", post(update_button).delete(delete_button))
        .layer(Extension(pool))
}

async fn get_buttons(Extension(pool): Extension<Arc<PgPool>>) -> impl IntoResponse {
    db::buttons::get_buttons(&pool)
        .await
        .map(|buttons| {
            (
                StatusCode::OK,
                Json(buttons.iter().map(|b| b.to_json()).collect::<Vec<_>>()),
            )
        })
        .map_err(internal_server_error)
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

async fn create_button(
    Extension(pool): Extension<Arc<PgPool>>,
    Json(body): Json<ButtonRequest>,
) -> Result<impl IntoResponse, impl IntoResponse> {
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
            return Err(error_response(
                format!("Failed to create button: {}", e),
                StatusCode::BAD_REQUEST,
            ));
        }
    };

    db::buttons::create_button(&pool, &new_button)
        .await
        .map(|_| StatusCode::OK)
        .map_err(|e| {
            error!("{}", e);
            error_response(
                format!("Failed to create button: {}", e),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })
}

async fn update_button(
    Extension(pool): Extension<Arc<PgPool>>,
    Path(id): Path<Uuid>,
    Json(body): Json<ButtonRequest>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let ip = match IpNetwork::from_str(&body.ip) {
        Ok(ip) => ip,
        Err(_) => {
            return Err(error_response(
                "Failed to parse IP address.".to_string(),
                StatusCode::BAD_REQUEST,
            ))
        }
    };

    match db::buttons::update_button(
        &pool,
        &Button {
            id,
            ip,
            name: body.name.clone(),
            username: body.username.clone(),
            password: body.password.clone(),
            plug_ids: body.plug_ids.clone(),
        },
    )
    .await
    {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => {
            error!("{}", e);
            Err(error_response(
                format!("Failed to update button: {}", e),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

async fn delete_button(
    Extension(pool): Extension<Arc<PgPool>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    db::buttons::delete_button(&pool, &id)
        .await
        .map(|_| StatusCode::OK)
        .map_err(internal_server_error)
}
