use crate::services::profile::{handle_market_insights, handle_search, handle_talent_search};
use crate::state::AppState;
use axum::{routing::post, Router};

pub fn routes(app_state: AppState) -> Router {
    Router::new()
        .route("/v1/search", post(handle_search))
        .route("/v1/talent/search", post(handle_talent_search))
        .route("/v1/talent/insights", post(handle_market_insights))
        .with_state(app_state)
}
