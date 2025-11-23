use crate::{
    AppState, db,
    models::{CreateUnit},
};
use axum::{
    Json, Json as AxumJson,
    extract::{Path, State},
    http::StatusCode,
};

/// Handler to get all units from the database.
///
/// # Returns
/// - `200 OK` with a JSON array of all units.
/// - `500 Internal Server Error` if there's a database error.
pub async fn get_all(State(state): AppState) -> Json<Vec<String>> {
    let cache = state.units_cache.read().await;
    Json(cache.clone())
}

/// API handler to add a new unit.
pub async fn add(
    State(state): AppState,
    AxumJson(payload): AxumJson<CreateUnit>,
) -> Result<StatusCode, (StatusCode, String)> {
    match db::add_unit(&state.pool, &payload.name).await {
        Ok(_) => {
            // Invalidate and refresh cache
            let mut cache = state.units_cache.write().await;
            *cache = db::fetch_all_units(&state.pool).await.unwrap_or_default();
            Ok(StatusCode::CREATED)
        }
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => Err((
            StatusCode::CONFLICT,
            format!("Unit with name '{}' already exists.", payload.name),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )),
    }
}

/// API handler to delete a unit.
pub async fn delete(
    State(state): AppState,
    Path(name): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let result = db::delete_unit(&state.pool, &name)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected() > 0 {
        // Invalidate and refresh cache
        let mut cache = state.units_cache.write().await;
        *cache = db::fetch_all_units(&state.pool).await.unwrap_or_default();
    }

    Ok(StatusCode::NO_CONTENT)
}
