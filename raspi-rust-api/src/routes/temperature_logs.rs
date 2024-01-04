use std::sync::Arc;

use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::db;
use crate::routes::lib::internal_server_error;
use crate::service::temperature_logs::{generate_temperature_graph, TimePeriod};

// Entry point to create router for temperature logs
pub fn temperature_logs_router(pool: Arc<PgPool>) -> Router {
    Router::new()
        .route("/", get(get_temperature_logs))
        .route("/:room_id/:time_period", get(get_room_temperature_logs))
        .route("/current", get(get_current_temps))
        .layer(Extension(pool))
}

// Handler to get all temperature logs
async fn get_temperature_logs(Extension(pool): Extension<Arc<PgPool>>) -> impl IntoResponse {
    db::temperature_logs::get_temp_logs(&pool)
        .await
        .map(Json)
        .map_err(internal_server_error)
}

// Parameters for the room temperature logs request
#[derive(Deserialize)]
pub struct RoomLogsParams {
    room_id: Uuid,
    time_period: TimePeriod,
}

// Handler to get temperature logs for a room
async fn get_room_temperature_logs(
    Path(params): Path<RoomLogsParams>,
    Extension(pool): Extension<Arc<PgPool>>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let room_logs = match db::temperature_logs::get_room_temp_logs(&pool, &params.room_id).await {
        Ok(logs) => logs,
        Err(e) => return Err(internal_server_error(e)),
    };

    Ok((
        StatusCode::OK,
        Json(generate_temperature_graph(room_logs, &params.time_period)),
    ))
}

// Struct to serialize the current room temperature response
#[derive(Serialize)]
struct RoomTemp {
    room_id: Uuid,
    room_name: String,
    temp: f64,
    time: NaiveDateTime,
}

// Handler to get current temperatures
async fn get_current_temps(Extension(pool): Extension<Arc<PgPool>>) -> impl IntoResponse {
    let rooms = db::rooms::get_rooms(&pool)
        .await
        .map_err(internal_server_error)?;

    db::temperature_logs::get_current_temps(&pool, &rooms)
        .await
        .map(|temps| {
            let room_temps: Vec<RoomTemp> = temps
                .into_iter()
                .filter_map(|(room_id, room_temp)| {
                    let room = rooms.iter().find(|room| room.id == room_id)?;
                    Some(RoomTemp {
                        room_id: room.id,
                        room_name: room.name.clone(),
                        temp: room_temp.temp,
                        time: room_temp.time,
                    })
                })
                .collect();
            Json(room_temps)
        })
        .map_err(internal_server_error)
}
