use crate::{
    AppState, db,
    models::{CreateGroup},
};
use axum::{
    Json as AxumJson,
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};

/// Handler to get all groups from the database.
///
/// # Returns
/// - `200 OK` with a JSON array of all groups.
/// - `500 Internal Server Error` if there's a database error.
pub async fn get_all(State(state): AppState) -> Json<Vec<String>> {
    let cache = state.groups_cache.read().await;
    Json(cache.clone())
}

/// API handler to add a new group.
pub async fn add(
    State(state): AppState,
    AxumJson(payload): AxumJson<CreateGroup>,
) -> Result<StatusCode, (StatusCode, String)> {
    match db::add_group(&state.pool, &payload.name).await {
        Ok(_) => {
            // Invalidate and refresh cache
            let mut cache = state.groups_cache.write().await;
            *cache = db::fetch_all_groups(&state.pool).await.unwrap_or_default();
            Ok(StatusCode::CREATED)
        }
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => Err((
            StatusCode::CONFLICT,
            format!("Group with name '{}' already exists.", payload.name),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )),
    }
}

/// API handler to delete a group.
pub async fn delete(
    State(state): AppState,
    Path(name): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let result = db::delete_group(&state.pool, &name)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() > 0 {
        // Invalidate and refresh cache
        let mut cache = state.groups_cache.write().await;
        *cache = db::fetch_all_groups(&state.pool).await.unwrap_or_default();
    }

    Ok(StatusCode::NO_CONTENT)
}
