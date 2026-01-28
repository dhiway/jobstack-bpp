use crate::models::search::Pagination;
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;
use sqlx::{query, query_scalar, Error, PgPool, Row};
use tracing::info;

pub struct NewProfile {
    pub profile_id: String,
    pub beckn_structure: Option<Value>,
    pub metadata: Option<Value>,
    pub hash: String,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub transaction_id: String,
    pub bpp_id: String,
    pub bpp_uri: String,
}

#[derive(Debug, Serialize)]
pub struct PaginatedItems<T = serde_json::Value> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: u32,
    pub limit: u32,
}

pub async fn store_profiles(db_pool: &PgPool, profiles: &[NewProfile]) -> Result<(), Error> {
    if profiles.is_empty() {
        return Ok(());
    }
    let profile_ids: Vec<&str> = profiles.iter().map(|p| p.profile_id.as_str()).collect();

    let beckn_structures: Vec<Option<&Value>> = profiles
        .iter()
        .map(|p| p.beckn_structure.as_ref())
        .collect();

    let metadata: Vec<Option<&Value>> = profiles.iter().map(|p| p.metadata.as_ref()).collect();

    let hashes: Vec<&str> = profiles.iter().map(|p| p.hash.as_str()).collect();

    let last_synced_at: Vec<Option<DateTime<Utc>>> =
        profiles.iter().map(|p| p.last_synced_at).collect();

    let transaction_ids: Vec<&str> = profiles.iter().map(|p| p.transaction_id.as_str()).collect();

    let bpp_ids: Vec<&str> = profiles.iter().map(|p| p.bpp_id.as_str()).collect();

    let bpp_uris: Vec<&str> = profiles.iter().map(|p| p.bpp_uri.as_str()).collect();

    query(
        r#"
        INSERT INTO profiles (
            profile_id,
            beckn_structure,
            metadata,
            hash,
            last_synced_at,
            transaction_id,
            bpp_id,
            bpp_uri
        )
        SELECT
            profile_id,
            beckn_structure,
            metadata,
            hash,
            last_synced_at,
            transaction_id,
            bpp_id,
            bpp_uri
        FROM UNNEST(
            $1::text[],
            $2::jsonb[],
            $3::jsonb[],
            $4::text[],
            $5::timestamptz[],
            $6::text[],
            $7::text[],
            $8::text[]
        ) AS t(
            profile_id,
            beckn_structure,
            metadata,
            hash,
            last_synced_at,
            transaction_id,
            bpp_id,
            bpp_uri
        )
        ON CONFLICT (profile_id) DO UPDATE
        SET
            beckn_structure = CASE
                WHEN profiles.hash IS DISTINCT FROM EXCLUDED.hash
                THEN EXCLUDED.beckn_structure
                ELSE profiles.beckn_structure
            END,
            hash = CASE
                WHEN profiles.hash IS DISTINCT FROM EXCLUDED.hash
                THEN EXCLUDED.hash
                ELSE profiles.hash
            END,
            transaction_id = EXCLUDED.transaction_id,
            bpp_id = EXCLUDED.bpp_id,
            bpp_uri = EXCLUDED.bpp_uri,
            last_synced_at = EXCLUDED.last_synced_at
        "#,
    )
    .bind(&profile_ids)
    .bind(&beckn_structures)
    .bind(&metadata)
    .bind(&hashes)
    .bind(&last_synced_at)
    .bind(&transaction_ids)
    .bind(&bpp_ids)
    .bind(&bpp_uris)
    .execute(db_pool)
    .await?;

    Ok(())
}

pub async fn fetch_profiles(
    db_pool: &PgPool,
    pagination: Pagination,
) -> Result<PaginatedItems, sqlx::Error> {
    let page = pagination.page.unwrap_or(1).max(1);
    let limit = pagination.limit.unwrap_or(20).clamp(1, 1000);
    let offset = (page - 1) * limit;

    info!(
        "Fetching profiles (page-limit) - Page: {}, Limit: {}, Offset: {}",
        page, limit, offset
    );

    let total: i64 = query_scalar(
        r#"
        SELECT COUNT(*) 
        FROM profiles
        WHERE beckn_structure IS NOT NULL
        "#,
    )
    .fetch_one(db_pool)
    .await?;

    let rows = sqlx::query(
        r#"
        SELECT
            id,
            profile_id,
            beckn_structure,
            transaction_id,
            bpp_id,
            bpp_uri AS bpp_url,
            updated_at
        FROM profiles
        WHERE beckn_structure IS NOT NULL
        ORDER BY updated_at DESC, profile_id DESC
        LIMIT $1
        OFFSET $2
        "#,
    )
    .bind(limit as i64)
    .bind(offset as i64)
    .fetch_all(db_pool)
    .await?;

    let items = rows
        .into_iter()
        .map(|r| {
            serde_json::json!({
                "id": r.try_get::<i32, _>("id").ok(),
                "profile_id": r.try_get::<String, _>("profile_id").ok(),
                "beckn_structure": r.try_get::<Option<Value>, _>("beckn_structure").ok().flatten(),
                "transaction_id": r.try_get::<String, _>("transaction_id").ok(),
                "bpp_id": r.try_get::<String, _>("bpp_id").ok(),
                "bpp_url": r.try_get::<String, _>("bpp_url").ok(),
                "updated_at": r.try_get::<chrono::DateTime<chrono::Utc>, _>("updated_at").ok()
            })
        })
        .collect::<Vec<_>>();

    Ok(PaginatedItems {
        items,
        total,
        page: page as u32,
        limit: limit as u32,
    })
}
