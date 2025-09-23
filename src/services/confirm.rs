use crate::config::AppConfig;
use crate::models::webhook::Context;

use serde_json::Value;
use tracing::info;

use crate::utils::{payload_generator::build_beckn_payload, shared::call_provider_db};

pub async fn handle_confirm(
    context: Context,
    message: Value,
    config: &AppConfig,
) -> anyhow::Result<Value> {
    let wrapped_message = serde_json::json!({
        "message": message,
        "context": context
    });
    let db_response = call_provider_db("/beckn/confirm", wrapped_message, config).await?;

    let result = build_beckn_payload(config, context, &db_response);

    info!("response : {:?}", result);

    Ok(result)
}
