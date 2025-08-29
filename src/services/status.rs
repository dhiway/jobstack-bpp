use crate::config::AppConfig;
use crate::models::webhook::Context;

use serde_json::Value;

use crate::utils::{payload_generator::build_beckn_payload, shared::call_provider_db};

pub async fn handle_status(
    context: Context,
    message: Value,
    config: &AppConfig,
) -> anyhow::Result<Value> {
    let wrapped_message = serde_json::json!({
        "message": message,
        "context":context
    });
    let db_response = call_provider_db("/beckn/status", wrapped_message, config).await?;

    let result = build_beckn_payload(config, context, &db_response);

    println!("response : {:?}", result);

    Ok(result)
}
