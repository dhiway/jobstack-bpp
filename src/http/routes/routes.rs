use crate::config::AppConfig;
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use std::sync::Arc;
use tracing::info;

use crate::models::webhook::{Ack, AckResponse, AckStatus, HealthResponse, WebhookPayload};
use crate::workers::processor::spawn_processing_task;

async fn health_check() -> impl IntoResponse {
    let response = HealthResponse {
        status: "OK",
        timestamp: Utc::now().to_rfc3339(),
    };

    Json(response)
}

pub async fn webhook_handler(
    Path(action): Path<String>,
    State(config): State<Arc<AppConfig>>,
    Json(payload): Json<WebhookPayload>,
) -> impl IntoResponse {
    info!(
        "webhook called: action = {:?}, context = {:?}, message = {:?}",
        action, payload.context, payload.message
    );
    if action.starts_with("on_") {
        info!(
            "Skipping processing since action starts with 'on_': {:?}",
            action
        );
        let ack = AckResponse {
            message: AckStatus {
                ack: Ack { status: "ACK" },
            },
        };
        return Json(ack);
    }

    spawn_processing_task(payload.context, payload.message, action, config);

    let ack = AckResponse {
        message: AckStatus {
            ack: Ack { status: "ACK" },
        },
    };

    Json(ack)
}

pub fn create_routes(config: AppConfig) -> Router {
    let shared_config = Arc::new(config);

    Router::new()
        .route("/", get(health_check))
        .route("/webhook/{action}", post(webhook_handler))
        .with_state(shared_config)
}
