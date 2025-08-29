use crate::config::AppConfig;
use crate::models::webhook::Context;
use crate::utils::mock_responses::load_mock_response;

use crate::services::{
    confirm::handle_confirm, init::handle_init, search::handle_search, select::handle_select,
    status::handle_status,
};
use serde_json::Value;
use tracing::info;
use uuid::Uuid;

pub async fn generate_response(
    action: &str,
    context: Context,
    message: Value,
    config: &AppConfig,
) -> anyhow::Result<Value> {
    if !config.use_mock_bpp_response {
        match action {
            "search" => handle_search(context, message, config).await,
            "select" => handle_select(context, message, config).await,
            "init" => handle_init(context, message, config).await,
            "confirm" => handle_confirm(context, message, config).await,
            "status" => handle_status(context, message, config).await,
            _ => Ok(serde_json::json!({
                "context": context,
                "message": {
                    "error": format!("Unsupported action: {}", action)
                }
            })),
        }
    } else {
        info!("Fallback to mock response: ...");
        let mut mock = load_mock_response(action).unwrap_or_else(|| {
            serde_json::json!({
                "context": context,
                "message": {
                    "note": "Default mock response",
                    "action": action
                }
            })
        });

        if let Some(ctx) = mock.get_mut("context") {
            ctx["transaction_id"] = serde_json::json!(context.transaction_id.clone());
            ctx["message_id"] = serde_json::json!(context.message_id.clone());
        }

        if action == "confirm" {
            if let Some(order) = mock.get_mut("message").and_then(|msg| msg.get_mut("order")) {
                order["id"] = serde_json::json!(Uuid::new_v4().to_string());
            }
        }

        Ok(mock)
    }
}
