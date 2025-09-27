use crate::{
    AppState, db,
    models::{CreateRarity, RarityType},
};
use axum::{
    Json as AxumJson,
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use std::collections::HashMap;

/// API handler to get all rarity mappings from the cache.
pub async fn get_all(State(state): AppState) -> Json<HashMap<String, RarityType>> {
    let cache = state.rarity_cache.read().await;
    Json(cache.clone())
}

/// API handler to get the type of a single rarity.
pub async fn get_by_code(State(state): AppState, Path(code): Path<String>) -> Json<RarityType> {
    let cache = state.rarity_cache.read().await;
    // Look up the code in the cache, defaulting to Regular if not found.
    let rarity_type = cache.get(&code).cloned().unwrap_or(RarityType::Regular);
    Json(rarity_type)
}

/// API handler to add a new rarity mapping.
pub async fn add(
    State(state): AppState,
    AxumJson(payload): AxumJson<CreateRarity>,
) -> Result<StatusCode, (StatusCode, String)> {
    // Acquire a write lock first to serialize access to this resource.
    let mut cache = state.rarity_cache.write().await;

    // Optimistically check the cache first to avoid a DB hit on a clear conflict.
    if cache.contains_key(&payload.rarity_code) {
        return Err((
            StatusCode::CONFLICT,
            format!("Rarity '{}' already exists.", payload.rarity_code),
        ));
    }

    // Now, attempt the database insert.
    match db::add_rarity(&state.pool, &payload.rarity_code, payload.rarity_type).await {
        Ok(_) => {
            // If the DB insert succeeds, update the cache and return success.
            cache.insert(payload.rarity_code, payload.rarity_type);
            Ok(StatusCode::CREATED)
        }
        // The DB can still fail with a unique violation if another process modified it.
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => Err((
            StatusCode::CONFLICT,
            format!("Rarity '{}' already exists.", payload.rarity_code),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )),
    }
}

/// API handler to delete a rarity mapping.
pub async fn delete(
    State(state): AppState,
    Path(code): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    // Acquire a write lock first to ensure the cache and DB operations are atomic.
    let mut cache = state.rarity_cache.write().await;

    // Attempt to delete from the database.
    let result = db::delete_rarity(&state.pool, &code)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // If the row was successfully deleted from the DB, remove it from the cache.
    if result.rows_affected() > 0 {
        cache.remove(&code);
    }

    Ok(StatusCode::NO_CONTENT)
}
