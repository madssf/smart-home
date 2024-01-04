use std::sync::Arc;

use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use sqlx::PgPool;
use uuid::Uuid;

use crate::db;
use crate::domain::{
    ActionType, TempAction, TempActionRequest, TempActionResponse, TempActionType,
};
use crate::routes::lib::internal_server_error;

pub fn temp_actions_router(pool: Arc<PgPool>) -> Router {
    Router::new()
        .route("/", get(get_temp_actions).post(create_temp_action))
        .route("/:id", post(update_temp_action).delete(delete_temp_action))
        .layer(Extension(pool))
}

async fn get_temp_actions(Extension(pool): Extension<Arc<PgPool>>) -> impl IntoResponse {
    db::temp_actions::get_temp_actions(&pool)
        .await
        .map(|actions| {
            (
                StatusCode::OK,
                Json(
                    actions
                        .into_iter()
                        .map(|a| a.into())
                        .collect::<Vec<TempActionResponse>>(),
                ),
            )
        })
        .map_err(internal_server_error)
}

async fn create_temp_action(
    Extension(pool): Extension<Arc<PgPool>>,
    Json(body): Json<TempActionRequest>,
) -> impl IntoResponse {
    let new_action: TempAction = body.into();
    db::temp_actions::create_temp_action(&pool, new_action)
        .await
        .map(|_| StatusCode::OK)
        .map_err(internal_server_error)
}

async fn update_temp_action(
    Extension(pool): Extension<Arc<PgPool>>,
    Path(id): Path<Uuid>,
    Json(body): Json<TempActionRequest>,
) -> impl IntoResponse {
    let action_type = match body.action {
        ActionType::ON => TempActionType::ON(body.temp),
        ActionType::OFF => TempActionType::OFF,
    };
    let updated_action = TempAction {
        id,
        room_ids: body.room_ids.clone(),
        action_type,
        expires_at: body.expires_at,
        starts_at: body.starts_at,
    };
    db::temp_actions::update_temp_action(&pool, updated_action)
        .await
        .map(|_| StatusCode::OK)
        .map_err(internal_server_error)
}

async fn delete_temp_action(
    Extension(pool): Extension<Arc<PgPool>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    db::temp_actions::delete_temp_action(&pool, &id)
        .await
        .map(|_| StatusCode::OK)
        .map_err(internal_server_error)
}
