use crate::{
    AppState, db,
    models::{CreateSet, SetResponse},
};
use axum::{
    Json, Json as AxumJson,
    extract::{Path, State},
    http::StatusCode,
};

/// Handler to get all sets from the database.
///
/// # Returns
/// - `200 OK` with a JSON array of all sets.
/// - `500 Internal Server Error` if there's a database error.
pub async fn get_all(State(state): AppState) -> Json<Vec<SetResponse>> {
    let cache = state.sets_cache.read().await;
    Json(cache.clone())
}

/// API handler to add a new set.
pub async fn add(
    State(state): AppState,
    AxumJson(payload): AxumJson<CreateSet>,
) -> Result<StatusCode, (StatusCode, String)> {
    match db::add_set(&state.pool, &payload.set_code, &payload.name).await {
        Ok(_) => {
            // Invalidate and refresh cache
            let mut cache = state.sets_cache.write().await;
            *cache = db::fetch_all_sets(&state.pool).await.unwrap_or_default();
            Ok(StatusCode::CREATED)
        }
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => Err((
            StatusCode::CONFLICT,
            format!("Set with code '{}' already exists.", payload.set_code),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )),
    }
}

/// API handler to delete a set.
pub async fn delete(
    State(state): AppState,
    Path(set_code): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let result = db::delete_set(&state.pool, &set_code)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() > 0 {
        // Invalidate and refresh cache
        let mut cache = state.sets_cache.write().await;
        *cache = db::fetch_all_sets(&state.pool).await.unwrap_or_default();
    }

    Ok(StatusCode::NO_CONTENT)
}
