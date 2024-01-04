use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;

use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use log::info;
use sqlx::PgPool;
use tokio::sync::mpsc::Sender;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::clients::shelly_client::ShellyClient;
use crate::clients::tibber_client::TibberClient;
use crate::domain::{ActionType, WorkMessage};
use crate::routes;
use crate::service::consumption_cache::ConsumptionCache;

// This function initializes all the services that our application provides
pub async fn start(
    sender: Sender<WorkMessage>,
    tibber_client: Arc<TibberClient>,
    shelly_client: Arc<ShellyClient>,
    consumption_cache: Arc<RwLock<ConsumptionCache>>,
    pool: Arc<PgPool>,
) -> Router {
    Router::new()
        .route("/_/health", get(health))
        .route("/trigger_refresh", get(refresh))
        .route("/trigger_button/:button_id/:action", get(trigger_button))
        .nest("/buttons", routes::buttons::buttons_router(pool.clone()))
        .nest(
            "/notification_settings",
            routes::notification_settings::notification_settings_router(pool.clone()),
        )
        .nest(
            "/plugs",
            routes::plugs::plugs_router(pool.clone(), shelly_client.clone()),
        )
        .nest(
            "/prices",
            routes::prices::prices_router(pool.clone(), tibber_client.clone(), consumption_cache),
        )
        .nest("/rooms", routes::rooms::room_routes(pool.clone()))
        .nest(
            "/schedules",
            routes::schedules::schedules_router(pool.clone(), tibber_client.clone()),
        )
        .nest(
            "/temp_actions",
            routes::temp_actions::temp_actions_router(pool.clone()),
        )
        .nest(
            "/temp_sensors",
            routes::temp_sensors::temp_sensors_router(pool.clone()),
        )
        .nest(
            "/temperature_logs",
            routes::temperature_logs::temperature_logs_router(pool.clone()),
        )
        .layer(Extension(sender))
}

// The health check handler
async fn health() -> impl IntoResponse {
    "Healthy!"
}

async fn refresh(Extension(sender): Extension<Sender<WorkMessage>>) -> impl IntoResponse {
    match sender.send(WorkMessage::REFRESH).await {
        Ok(_) => (StatusCode::OK, Json("Refresh triggered".to_string())),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(format!("Failed to refresh: {}", e)),
        ),
    }
}

pub async fn serve_app(host: String, port: u16, server: Router) {
    let addr = SocketAddr::from((host.parse::<Ipv4Addr>().expect("Failed to parse IP"), port));

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind port");

    info!("Listening on {}", addr);

    axum::serve(listener, server)
        .await
        .expect("Failed to start server");
}

// The trigger button handler, adapted for axum
async fn trigger_button(
    Extension(sender): Extension<Sender<WorkMessage>>,
    Path((button_id, action)): Path<(Uuid, ActionType)>,
) -> impl IntoResponse {
    match sender.send(WorkMessage::BUTTON(button_id, action, 1)).await {
        Ok(_) => (StatusCode::OK, format!("Button {} triggered", button_id)),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to trigger button: {}", e),
        ),
    }
}
