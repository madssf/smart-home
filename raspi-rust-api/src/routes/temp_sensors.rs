use std::sync::Arc;

use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{delete, get};
use axum::{Extension, Json, Router};
use log::info;
use sqlx::PgPool;

use crate::db;
use crate::domain::TempSensor;
use crate::routes::lib::internal_server_error;

pub fn temp_sensors_router(pool: Arc<PgPool>) -> Router {
    Router::new()
        .route("/", get(get_temp_sensors).post(create_sensor))
        .route("/:id", delete(delete_sensor))
        .layer(Extension(pool))
}

async fn get_temp_sensors(Extension(pool): Extension<Arc<PgPool>>) -> impl IntoResponse {
    db::temp_sensors::get_temp_sensors(&pool)
        .await
        .map(|sensors| (StatusCode::OK, Json(sensors)))
        .map_err(internal_server_error)
}

async fn create_sensor(
    Extension(pool): Extension<Arc<PgPool>>,
    Json(body): Json<TempSensor>,
) -> impl IntoResponse {
    db::temp_sensors::insert_temp_sensor(&pool, &body)
        .await
        .map(|_| StatusCode::OK)
        .map_err(internal_server_error)
}

async fn delete_sensor(
    Extension(pool): Extension<Arc<PgPool>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    info!("Deleting temp sensor with id: {}", id);
    db::temp_sensors::delete_temp_sensor(&pool, &id)
        .await
        .map(|_| StatusCode::OK)
        .map_err(internal_server_error)
}
