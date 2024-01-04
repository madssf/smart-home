use std::sync::Arc;

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{
    extract::{Extension, Json, Path},
    routing::{get, post},
    Router,
};
use log::error;
use sqlx::PgPool;
use uuid::Uuid;

use crate::db;
use crate::domain::Room;
use crate::routes::lib::error_response;

// Define the `RoomRequest` struct that corresponds to the request payload when creating or updating a room
#[derive(serde::Deserialize)]
pub struct RoomRequest {
    name: String,
    min_temp: Option<f64>,
}

pub fn room_routes(pool: Arc<PgPool>) -> Router {
    // Create a router for room routes
    Router::new()
        .route("/", get(get_rooms).post(create_room))
        .route("/:id", post(update_room).delete(delete_room))
        .layer(Extension(pool))
}

// The ``get_rooms` handler, adapted for Axum
async fn get_rooms(Extension(pool): Extension<Arc<PgPool>>) -> impl IntoResponse {
    match db::rooms::get_rooms(&pool).await {
        Ok(rooms) => (StatusCode::OK, Json(rooms)).into_response(),
        Err(e) => {
            error!("{:?}", e);
            error_response(
                format!("Failed to get rooms: {:?}", e),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into_response()
        }
    }
}

// Adapt `create_room` handler for Axum
async fn create_room(
    Extension(pool): Extension<Arc<PgPool>>,
    Json(body): Json<RoomRequest>,
) -> impl IntoResponse {
    match db::rooms::create_room(&pool, &body.name, &body.min_temp).await {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            error!("Failed to create room: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

// Adapt `update_room` handler for Axum
async fn update_room(
    Extension(pool): Extension<Arc<PgPool>>,
    Path(id): Path<Uuid>,
    Json(body): Json<RoomRequest>,
) -> impl IntoResponse {
    match db::rooms::update_room(
        &pool,
        &Room {
            id,
            name: body.name,
            min_temp: body.min_temp,
        },
    )
    .await
    {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

// Adapt `delete_room` handler for Axum
async fn delete_room(
    Extension(pool): Extension<Arc<PgPool>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match db::rooms::delete_room(&pool, &id).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
