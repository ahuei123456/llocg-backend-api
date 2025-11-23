use crate::{
    AppState,
    db::{self, DbError},
    models::{CreateCard, FullCard},
};
use axum::{
    Json as AxumJson,
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};

/// API handler to get a single card by its ID.
pub async fn get_by_id(
    State(state): AppState,
    Path(id): Path<i64>,
) -> Result<Json<FullCard>, (StatusCode, String)> {
    match db::fetch_full_card(&state.pool, id).await {
        Ok(card) => Ok(Json(card)),
        Err(sqlx::Error::RowNotFound) => Err((StatusCode::NOT_FOUND, "Card not found".to_string())),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

/// API handler to create a new card.
pub async fn create(
    State(state): AppState,
    AxumJson(payload): AxumJson<CreateCard>,
) -> Result<(StatusCode, Json<FullCard>), (StatusCode, String)> {
    let rarity_cache = state.rarity_cache.read().await;
    let name_variant_cache = state.name_variant_cache.read().await;
    let group_variant_cache = state.group_variant_cache.read().await;

    match db::create_full_card(
        &state.pool,
        &rarity_cache,
        &name_variant_cache,
        &group_variant_cache,
        payload,
    )
    .await
    {
        Ok(card) => {
            // Invalidate and refresh names cache
            let mut names_cache = state.names_cache.write().await;
            *names_cache = db::fetch_all_card_names(&state.pool).await.unwrap_or_default();
            Ok((StatusCode::CREATED, Json(card)))
        }
        Err(DbError::GroupNotFound(name)) | Err(DbError::UnitNotFound(name)) => {
            // For missing entities, return a 400 Bad Request.
            Err((StatusCode::BAD_REQUEST, name))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

/// API handler to create multiple new cards in a single request.
pub async fn create_bulk(
    State(state): AppState,
    AxumJson(payload): AxumJson<Vec<CreateCard>>,
) -> Result<(StatusCode, Json<Vec<FullCard>>), (StatusCode, String)> {
    let rarity_cache = state.rarity_cache.read().await;
    let name_variant_cache = state.name_variant_cache.read().await;
    let group_variant_cache = state.group_variant_cache.read().await;

    match db::create_bulk_cards(
        &state.pool,
        &rarity_cache,
        &name_variant_cache,
        &group_variant_cache,
        payload,
    )
    .await
    {
        Ok(cards) => {
            // Invalidate and refresh names cache
            let mut names_cache = state.names_cache.write().await;
            *names_cache = db::fetch_all_card_names(&state.pool).await.unwrap_or_default();
            Ok((StatusCode::CREATED, Json(cards)))
        }
        Err(DbError::GroupNotFound(name)) | Err(DbError::UnitNotFound(name)) => {
            // For missing entities, return a 400 Bad Request.
            Err((StatusCode::BAD_REQUEST, name))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

/// API handler to get all cards (not yet implemented).
pub async fn get_all() -> StatusCode {
    StatusCode::NOT_IMPLEMENTED
}
