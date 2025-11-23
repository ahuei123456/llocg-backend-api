use crate::{AppState};
use axum::{Json, extract::State};

/// Handler to get all distinct canonical card names from the database.
///
/// # Returns
/// - `200 OK` with a JSON array of all card names.
/// - `500 Internal Server Error` if there's a database error.
pub async fn get_all(State(state): AppState) -> Json<Vec<String>> {
    let cache = state.names_cache.read().await;
    Json(cache.clone())
}
