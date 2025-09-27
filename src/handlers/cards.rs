use crate::{
    db,
    models::{Card, CreateCard, FullCard},
    AppState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    Json as AxumJson,
};

/// API handler to fetch all cards from the database.
pub async fn get_all(State(state): AppState) -> Result<Json<Vec<Card>>, (StatusCode, String)> {
    let cards = sqlx::query_as::<_, Card>("SELECT * FROM cards")
        .fetch_all(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(cards))
}

/// API handler to fetch a single, fully detailed card by its ID.
pub async fn get_by_id(
    State(state): AppState,
    Path(id): Path<i64>,
) -> Result<Json<FullCard>, (StatusCode, String)> {
    match db::fetch_full_card(&state.pool, id).await {
        Ok(card) => Ok(Json(card)),
        Err(sqlx::Error::RowNotFound) => Err((
            StatusCode::NOT_FOUND,
            format!("Card with ID {} not found", id),
        )),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

/// API handler to create a new card from a JSON payload.
pub async fn create(
    State(state): AppState,
    AxumJson(payload): AxumJson<CreateCard>,
) -> Result<(StatusCode, Json<FullCard>), (StatusCode, String)> {
    // Acquire read locks on the caches.
    let rarity_cache = state.rarity_cache.read().await;
    let name_variant_cache = state.name_variant_cache.read().await;
    match db::create_full_card(&state.pool, &rarity_cache, &name_variant_cache, payload).await {
        Ok(card) => Ok((StatusCode::CREATED, Json(card))),
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => Err((
            StatusCode::CONFLICT,
            "A card with this series, set, and number already exists.".to_string(),
        )),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}