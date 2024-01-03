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
use tower::ServiceBuilder;
use uuid::Uuid;

use crate::clients::shelly_client::ShellyClient;
use crate::clients::tibber_client::TibberClient;
use crate::domain::{ActionType, WorkMessage};
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
        // Define the rest of your routes here
        .layer(ServiceBuilder::new().layer(Extension(sender)))
        .layer(Extension(pool))
        .layer(Extension(tibber_client))
        .layer(Extension(shelly_client))
        .layer(Extension(consumption_cache))
}

// The health check handler
async fn health() -> impl IntoResponse {
    "Healthy!"
}

// The refresh handler, adapted for axum
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
