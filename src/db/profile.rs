use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{Error, PgPool};
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

    sqlx::query(
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
