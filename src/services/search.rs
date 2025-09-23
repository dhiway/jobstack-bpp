use crate::config::AppConfig;
use crate::models::webhook::Context;

use serde_json::Value;

use crate::utils::{payload_generator::build_beckn_payload, shared::call_provider_db};

pub async fn handle_search(
    context: Context,
    message: Value,
    config: &AppConfig,
) -> anyhow::Result<Value> {
    let pagination = match message.get("pagination") {
        Some(Value::Null) | None => serde_json::json!({
            "page": 0,
            "limit": 50
        }),
        Some(p) => p.clone(),
    };

    let mut wrapped_message = serde_json::json!({
        "message": message,
        "pagination": pagination
    });
    if let Some(options) = message.get("options") {
        if !options.is_null() {
            wrapped_message["options"] = options.clone();
        }
    }
    let db_response = call_provider_db("/beckn/search", wrapped_message, config).await?;

    let result = build_beckn_payload(config, context, &db_response);

    // println!("response : {:?}", result);

    Ok(result)
}
