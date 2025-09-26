use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post},
    Router, Json as AxumJson,
};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;

mod db;
mod models;
use models::{Card, CreateCard, CreateRarity, FullCard, RarityType};

/// The shared state for our application.
#[derive(Clone)]
struct ApiState {
    pool: Pool,
    rarity_cache: Arc<RwLock<HashMap<String, RarityType>>>,
}

/// The shared state for our application, including the database connection pool.
type AppState = State<ApiState>;

/// A type alias for the database connection pool.
type Pool = SqlitePool;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenvy::dotenv().expect("Failed to read .env file");
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Set up the database connection pool
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    // --- Populate the rarity cache at startup ---
    println!("Loading rarities into cache...");
    let rarities: Vec<(String, RarityType)> =
        sqlx::query_as("SELECT rarity_code, rarity_type FROM rarities")
            .fetch_all(&pool)
            .await?;
    let rarity_cache: Arc<RwLock<HashMap<String, RarityType>>> = Arc::new(RwLock::new(rarities.into_iter().collect()));
    println!("-> Loaded {} rarity mappings.", rarity_cache.read().await.len());

    let app_state = ApiState {
        pool,
        rarity_cache,
    };

    // Define our application's routes
    let app = Router::new()
        .route("/cards", get(get_all_cards).post(create_card))
        .route("/cards/:id", get(get_card_by_id))
        .route("/rarities", get(get_all_rarities).post(add_rarity))
        .route(
            "/rarities/:code",
            get(get_rarity_by_code).delete(delete_rarity),
        )
        .with_state(app_state);

    // Start the server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// API handler to fetch all cards from the database.
async fn get_all_cards(
    State(state): AppState,
) -> Result<Json<Vec<Card>>, (StatusCode, String)> {
    let cards = sqlx::query_as::<_, Card>("SELECT * FROM cards")
        .fetch_all(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(cards))
}

/// API handler to fetch a single, fully detailed card by its ID.
async fn get_card_by_id(
    State(state): AppState,
    Path(id): Path<i64>,
) -> Result<Json<FullCard>, (StatusCode, String)> {
    match db::fetch_full_card(&state.pool, id).await {
        Ok(card) => Ok(Json(card)),
        Err(sqlx::Error::RowNotFound) => Err((
            StatusCode::NOT_FOUND,
            format!("Card with ID {} not found", id),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{self, Request, StatusCode},
    };
    use tower::ServiceExt; // for `oneshot`

    /// Helper function to set up a test environment with an in-memory DB.
    async fn setup_test_env() -> ApiState {
        // Use an in-memory SQLite database for testing.
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .expect("Failed to create in-memory db pool.");

        // Run migrations on the in-memory database.
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations on in-memory db.");

        // The cache will be empty initially, which is what we want for a clean test.
        let rarity_cache = Arc::new(RwLock::new(HashMap::new()));

        ApiState {
            pool,
            rarity_cache,
        }
    }

    #[tokio::test]
    async fn test_rarities_endpoints() {
        let state = setup_test_env().await;
        let app = Router::new()
            .route("/rarities", get(get_all_rarities).post(add_rarity))
            .route("/rarities/:code", delete(delete_rarity))
            .with_state(state);

        // 1. Initially, GET all rarities should return an empty map.
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/rarities")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(&body[..], b"{}");

        // 2. POST a new rarity.
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/rarities")
                    .header(http::header::CONTENT_TYPE, "application/json")
                    .body(Body::from(r#"{"rarity_code": "TEST", "rarity_type": "Parallel"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        // 3. GET all rarities again; it should contain the new rarity.
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/rarities")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(&body[..], b"{\"TEST\":\"Parallel\"}");

        // 4. DELETE the rarity.
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(http::Method::DELETE)
                    .uri("/rarities/TEST")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }
}

/// API handler to create a new card from a JSON payload.
async fn create_card(
    State(state): AppState,
    AxumJson(payload): AxumJson<CreateCard>,
) -> Result<(StatusCode, Json<FullCard>), (StatusCode, String)> {
    // Acquire a read lock on the cache.
    let cache = state.rarity_cache.read().await;
    match db::create_full_card(&state.pool, &cache, payload).await {
        Ok(card) => Ok((StatusCode::CREATED, Json(card))),
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
            Err((
                StatusCode::CONFLICT,
                "A card with this series, set, and number already exists.".to_string(),
            ))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )),
    }
}

/// API handler to get all rarity mappings from the cache.
async fn get_all_rarities(
    State(state): AppState,
) -> Json<HashMap<String, RarityType>> {
    let cache = state.rarity_cache.read().await;
    Json(cache.clone())
}

/// API handler to get the type of a single rarity.
async fn get_rarity_by_code(
    State(state): AppState,
    Path(code): Path<String>,
) -> Json<RarityType> {
    let cache = state.rarity_cache.read().await;
    // Look up the code in the cache, defaulting to Regular if not found.
    let rarity_type = cache.get(&code).cloned().unwrap_or(RarityType::Regular);
    Json(rarity_type)
}

/// API handler to add a new rarity mapping.
async fn add_rarity(
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
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
            Err((
                StatusCode::CONFLICT,
                format!("Rarity '{}' already exists.", payload.rarity_code),
            ))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )),
    }
}

/// API handler to delete a rarity mapping.
async fn delete_rarity(
    State(state): AppState,
    Path(code): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    // Acquire a write lock first to ensure the cache and DB operations are atomic.
    let mut cache = state.rarity_cache.write().await;

    // Attempt to delete from the database.
    match db::delete_rarity(&state.pool, &code).await {
        Ok(rows_affected) => {
            // If the DB row was deleted, also remove it from the cache.
            if rows_affected > 0 {
                cache.remove(&code);
            }
            // Per HTTP spec, DELETE should be idempotent. Return success even if the
            // resource was already gone.
            Ok(StatusCode::NO_CONTENT)
        }
        // If the database operation fails for any other reason, we return an error
        // without modifying the cache.
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )),
    }
}
