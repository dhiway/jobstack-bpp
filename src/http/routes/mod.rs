pub mod profiles;
pub mod webhook;
use crate::state::AppState;
use axum::{response::IntoResponse, routing::get, Json, Router};
use chrono::Utc;

use tower_http::cors::{Any, CorsLayer};

use crate::models::webhook::HealthResponse;
async fn health_check() -> impl IntoResponse {
    let response = HealthResponse {
        status: "OK",
        timestamp: Utc::now().to_rfc3339(),
    };

    Json(response)
}

pub fn create_routes(app_state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/", get(health_check))
        .nest("/api", profiles::routes(app_state.clone()))
        .merge(webhook::routes(app_state))
        .layer(cors)
}
