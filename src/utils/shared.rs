use crate::config::AppConfig;
use crate::utils::http_client::post_json;
use anyhow::Result;
use serde_json::Value;
use std::sync::Arc;

pub async fn send_to_bpp_caller(
    action: &str,
    payload: Value,
    config: Arc<AppConfig>,
) -> Result<Value> {
    let bpp_url = &config.bpp.caller_uri;
    let full_url = format!("{}/on_{}", bpp_url.trim_end_matches('/'), action);
    post_json(&full_url, payload).await
}

pub async fn call_provider_db(path: &str, payload: Value, config: &AppConfig) -> Result<Value> {
    let db_url = &config.provider_db.db_uri;
    let full_url = format!(
        "{}/{}",
        db_url.trim_end_matches('/'),
        path.trim_start_matches('/')
    );
    post_json(&full_url, payload).await
}
