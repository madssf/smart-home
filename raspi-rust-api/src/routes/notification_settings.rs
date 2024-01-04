use std::sync::Arc;

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Extension, Json, Router};
use sqlx::PgPool;

use crate::db;
use crate::domain::NotificationSettings;
use crate::routes::lib::internal_server_error;

pub fn notification_settings_router(pool: Arc<PgPool>) -> Router {
    Router::new()
        .route("/", get(get_settings).post(upsert_settings))
        .layer(Extension(pool))
}

async fn get_settings(Extension(pool): Extension<Arc<PgPool>>) -> impl IntoResponse {
    db::notification_settings::get_notification_settings(&pool)
        .await
        .map(|settings| (StatusCode::OK, Json(settings)))
        .map_err(internal_server_error)
}

async fn upsert_settings(
    Extension(pool): Extension<Arc<PgPool>>,
    Json(body): Json<NotificationSettings>,
) -> impl IntoResponse {
    db::notification_settings::upsert_notification_settings(&pool, &body)
        .await
        .map(|_| StatusCode::OK)
        .map_err(internal_server_error)
}
