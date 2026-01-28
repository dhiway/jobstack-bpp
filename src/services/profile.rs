use crate::db::profile::{fetch_profiles, store_profiles, NewProfile};
use crate::models::profiles::ProfileSearchRequest;
use crate::models::search::{Intent, Pagination, SearchMessage};
use crate::models::webhook::{Ack, AckResponse, AckStatus, WebhookPayload};
use crate::state::AppState;
use crate::utils::http_client::post_json;
use crate::utils::payload_generator::build_profile_beckn_request;

use axum::{extract::State, http::StatusCode, Json};
use chrono::Utc;
use deadpool_redis::redis::AsyncCommands;
use serde_json::Value;
use sha2::{Digest, Sha256};
use tracing::{error, info};
use uuid::Uuid;

fn hash_json(value: &Value) -> String {
    let canonical = serde_json::to_vec(value).unwrap();
    let mut hasher = Sha256::new();
    hasher.update(canonical);
    format!("{:x}", hasher.finalize())
}

fn extract_profiles_from_on_search(payload: &WebhookPayload, txn_id: &str) -> Vec<NewProfile> {
    let mut profiles = Vec::new();

    let providers = payload
        .message
        .get("catalog")
        .and_then(|c| c.get("providers"))
        .and_then(|p| p.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(&[]);

    for provider in providers {
        let items = provider
            .get("items")
            .and_then(|i| i.as_array())
            .map(|v| v.as_slice())
            .unwrap_or(&[]);

        for item in items {
            let profile_id = match item.get("id").and_then(|v| v.as_str()) {
                Some(id) => id.to_string(),
                None => continue,
            };

            let beckn_structure = item.clone();
            let hash = hash_json(&beckn_structure);

            profiles.push(NewProfile {
                profile_id,
                beckn_structure: Some(beckn_structure),
                metadata: None,
                hash,
                last_synced_at: Some(Utc::now()),
                transaction_id: txn_id.to_string(),
                bpp_id: payload.context.bpp_id.clone().unwrap_or_default(),
                bpp_uri: payload.context.bpp_uri.clone().unwrap_or_default(),
            });
        }
    }

    profiles
}

pub async fn handle_on_search(
    app_state: &AppState,
    payload: &WebhookPayload,
    txn_id: &str,
) -> Json<AckResponse> {
    let profiles = extract_profiles_from_on_search(payload, txn_id);

    if let Err(e) = store_profiles(&app_state.db_pool, &profiles).await {
        error!("store_profiles failed: {}", e);
    }

    let pagination = payload
        .message
        .get("pagination")
        .and_then(|p| p.as_object());

    let page = pagination
        .and_then(|p| p.get("page"))
        .and_then(|v| v.as_u64())
        .unwrap_or(1);

    let limit = pagination
        .and_then(|p| p.get("limit"))
        .and_then(|v| v.as_u64())
        .unwrap_or(50);

    let total = pagination
        .and_then(|p| p.get("total"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    if total == 0 {
        return ack();
    }

    let total_pages = (total + limit - 1) / limit;

    let bpp_id = payload.context.bpp_id.clone().unwrap_or_default();
    let mut redis = match app_state.redis_pool.get().await {
        Ok(c) => c,
        Err(e) => {
            error!("Redis connection failed: {}", e);
            return ack();
        }
    };

    let base_key = format!("pagination:{}:{}:{}", txn_id, bpp_id, limit);

    let received_key = format!("{}:received", base_key);

    let _: () = redis
        .hset_nx(&base_key, "total_pages", total_pages)
        .await
        .unwrap();

    let _: () = redis.sadd(&received_key, page).await.unwrap();

    let _: () = redis.expire(&base_key, 1800).await.unwrap();
    let _: () = redis.expire(&received_key, 1800).await.unwrap();

    let received_pages: Vec<u64> = redis.smembers(&received_key).await.unwrap();
    let next_page = received_pages.iter().max().copied().unwrap_or(0) + 1;

    if next_page <= total_pages {
        let trigger_key = format!("{}:triggered:{}", base_key, next_page);

        let triggered: bool = redis.set_nx(&trigger_key, 1).await.unwrap();
        let _: () = redis.expire(&trigger_key, 1800).await.unwrap();

        if triggered {
            info!(
                "Triggering next search page: {}/{} (txn_id={}, bpp_id={})",
                next_page, total_pages, txn_id, bpp_id
            );

            let message = SearchMessage {
                intent: Intent {
                    item: None,
                    provider: None,
                    fulfillment: None,
                },
                pagination: Some(Pagination {
                    page: Some(next_page as u32),
                    limit: Some(limit as u32),
                }),
            };

            let request_payload = build_profile_beckn_request(
                &app_state.config,
                txn_id,
                &format!("msg-profile-{}", Uuid::new_v4()),
                &message,
                "search",
                None,
                None,
            );

            let adapter_url = format!("{}/search", app_state.config.bap.caller_uri);

            if let Err(e) = post_json(&adapter_url, request_payload).await {
                error!("Failed to trigger next page {}: {}", next_page, e);
            }
        }
    }

    ack()
}

fn ack() -> Json<AckResponse> {
    Json(AckResponse {
        message: AckStatus {
            ack: Ack { status: "ACK" },
        },
    })
}

pub async fn handle_search(
    State(app_state): State<AppState>,
    Json(req): Json<ProfileSearchRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let pagination = req.pagination.unwrap_or_default();

    match fetch_profiles(&app_state.db_pool, pagination).await {
        Ok(result) => Ok(Json(serde_json::json!({
            "items": result.items,
            "total": result.total,
            "page": result.page,
            "limit": result.limit
        }))),

        Err(err) => {
            tracing::error!("fetch_profiles failed: {:?}", err);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to fetch profiles"
                })),
            ))
        }
    }
}
