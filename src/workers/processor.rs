use crate::config::AppConfig;
use crate::models::webhook::Context;
use crate::services::webhook::generate_response;
use crate::utils::shared::send_to_bpp_caller;
use serde_json::Value;
use std::sync::Arc;
use tokio::task;
use tracing::error;

pub fn spawn_processing_task(
    context: Context,
    message: Value,
    action: String,
    config: Arc<AppConfig>,
) {
    task::spawn({
        let config = config.clone();
        async move {
            match generate_response(&action, context, message, &config).await {
                Ok(response) => {
                    if let Err(e) = send_to_bpp_caller(&action, response, config).await {
                        error!("Error sending to BPP client: {:?}", e);
                    }
                }
                Err(e) => {
                    error!("Error generating response: {:?}", e);
                }
            }
        }
    });
}
