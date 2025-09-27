use crate::{AppState, models::CreateGroupVariant};
use axum::{
    Json as AxumJson,
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use std::collections::HashMap;

/// API handler to get all group variant mappings from the cache.
pub async fn get_all(State(state): AppState) -> Json<HashMap<String, String>> {
    let cache = state.group_variant_cache.read().await;
    Json(cache.clone())
}

/// API handler to add a new group variant mapping.
pub async fn add(
    State(state): AppState,
    AxumJson(payload): AxumJson<CreateGroupVariant>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut cache = state.group_variant_cache.write().await;

    if cache.contains_key(&payload.variant_name) {
        return Err((
            StatusCode::CONFLICT,
            format!(
                "Group variant name '{}' already exists.",
                payload.variant_name
            ),
        ));
    }

    match sqlx::query("INSERT INTO group_variants (variant_name, canonical_name) VALUES (?, ?)")
        .bind(&payload.variant_name)
        .bind(&payload.canonical_name)
        .execute(&state.pool)
        .await
    {
        Ok(_) => {
            cache.insert(payload.variant_name, payload.canonical_name);
            Ok(StatusCode::CREATED)
        }
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => Err((
            StatusCode::CONFLICT,
            format!(
                "Group variant name '{}' already exists.",
                payload.variant_name
            ),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("DB error: {}", e),
        )),
    }
}

/// API handler to delete a group variant mapping.
pub async fn delete(
    State(state): AppState,
    Path(variant): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut cache = state.group_variant_cache.write().await;

    let result = sqlx::query("DELETE FROM group_variants WHERE variant_name = ?")
        .bind(&variant)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() > 0 {
        cache.remove(&variant);
    }

    Ok(StatusCode::NO_CONTENT)
}
