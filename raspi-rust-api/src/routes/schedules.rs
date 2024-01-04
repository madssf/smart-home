use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use chrono::{NaiveTime, Weekday};
use log::error;
use sqlx::PgPool;
use uuid::Uuid;

use crate::clients::tibber_client::TibberClient;
use crate::db::rooms;
use crate::domain::{PriceLevel, Schedule};
use crate::routes::lib::error_response;
use crate::{db, now, service};

// Router definition for the schedules module
pub fn schedules_router(pool: Arc<PgPool>, tibber_client: Arc<TibberClient>) -> Router {
    Router::new()
        .route("/", get(get_schedules).post(create_schedule))
        .route("/:id", post(update_schedule).delete(delete_schedule))
        .route("/active", get(get_active_schedules))
        .layer(Extension(pool))
        .layer(Extension(tibber_client))
}

// Handler to get all schedules
async fn get_schedules(Extension(pool): Extension<Arc<PgPool>>) -> impl IntoResponse {
    match db::schedules::get_schedules(&pool).await {
        Ok(schedules) => (StatusCode::OK, Json(schedules)).into_response(),
        Err(e) => {
            error!("{:?}", e);
            error_response(
                "Failed to get schedules.".to_string(),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into_response()
        }
    }
}

#[derive(serde::Deserialize)]
pub struct ScheduleRequest {
    pub temps: HashMap<PriceLevel, f64>,
    pub days: Vec<Weekday>,
    pub time_windows: Vec<(NaiveTime, NaiveTime)>,
    pub room_ids: Vec<Uuid>,
}

impl TryInto<Schedule> for ScheduleRequest {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Schedule, Self::Error> {
        Schedule::new(self.temps, self.days, self.time_windows, self.room_ids)
    }
}

async fn create_schedule(
    Extension(pool): Extension<Arc<PgPool>>,
    Json(body): Json<ScheduleRequest>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let new_schedule = match body.try_into() {
        Ok(schedule) => schedule,
        Err(e) => {
            error!("{}", e);
            return Err(error_response(
                format!("Failed to create schedule: {}", e),
                StatusCode::BAD_REQUEST,
            ));
        }
    };

    match db::schedules::create_schedule(&pool, new_schedule).await {
        Ok(_) => Ok((StatusCode::OK, Json("Schedule created successfully."))),
        Err(e) => {
            error!("{}", e.to_string());
            Err(error_response(
                "Failed to create schedule.".to_string(),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

// Handler to update a schedule by ID
async fn update_schedule(
    Extension(pool): Extension<Arc<PgPool>>,
    Path(id): Path<Uuid>,
    Json(body): Json<ScheduleRequest>,
) -> impl IntoResponse {
    match db::schedules::update_schedule(
        &pool,
        Schedule {
            id,
            temps: body.temps.clone(),
            days: body.days.clone(),
            time_windows: body.time_windows.clone(),
            room_ids: body.room_ids.clone(),
        },
    )
    .await
    {
        Ok(_) => (StatusCode::OK).into_response(),
        Err(_) => error_response(
            "Failed to update schedule.".to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        )
        .into_response(),
    }
}

// Handler to delete a schedule by ID
async fn delete_schedule(
    Extension(pool): Extension<Arc<PgPool>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match db::schedules::delete_schedule(&pool, &id).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => error_response(
            format!("Failed to delete schedule: {}", e),
            StatusCode::INTERNAL_SERVER_ERROR,
        )
        .into_response(),
    }
}

#[derive(serde::Serialize)]
pub struct ActiveSchedule {
    pub room_id: Uuid,
    pub schedule: Option<Schedule>,
    pub temp: Option<f64>,
}
async fn get_active_schedules(
    Extension(pool): Extension<Arc<PgPool>>,
    Extension(tibber_client): Extension<Arc<TibberClient>>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let rooms = match rooms::get_rooms(&pool).await {
        Ok(rooms) => rooms,
        Err(e) => {
            error!("{:?}", e);
            return Err(error_response(
                "Failed to get rooms.".to_string(),
                StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };

    let price_info = match service::prices::get_current_price(&tibber_client, &pool).await {
        Ok(price) => price,
        Err(e) => {
            error!("{:?}", e);
            return Err(error_response(
                "Failed to get current price.".to_string(),
                StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };
    let mut active_schedules: Vec<ActiveSchedule> = vec![];
    for room in rooms {
        match db::schedules::get_matching_schedule(&pool, &room.id, &now()).await {
            Ok(schedule) => {
                let temp =
                    if let Some(schedule) = schedule.clone() {
                        Some(schedule.get_temp(
                            &price_info.price_level.unwrap_or(price_info.ext_price_level),
                        ))
                    } else {
                        None
                    };
                active_schedules.push(ActiveSchedule {
                    room_id: room.id,
                    schedule,
                    temp,
                })
            }
            Err(e) => {
                error!("{:?}", e);
                return Err(error_response(
                    format!("Failed to get schedule for room {}, {}", room.id, e),
                    StatusCode::INTERNAL_SERVER_ERROR,
                ));
            }
        };
    }
    Ok((StatusCode::OK, Json(active_schedules)))
}
