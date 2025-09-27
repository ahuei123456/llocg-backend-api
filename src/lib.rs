use crate::models::RarityType;
use axum::{
    Router,
    routing::{get, post},
};
use sqlx::sqlite::SqlitePoolOptions;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

pub mod db;
pub mod handlers;
pub mod models;

/// A type alias for the database connection pool.
pub type Pool = sqlx::SqlitePool;

/// The shared state for our application.
#[derive(Clone)]
pub struct ApiState {
    pub pool: Pool,
    pub rarity_cache: Arc<RwLock<HashMap<String, models::RarityType>>>,
    pub name_variant_cache: Arc<RwLock<HashMap<String, String>>>,
    pub group_variant_cache: Arc<RwLock<HashMap<String, String>>>,
}

/// The shared state for our application, including the database connection pool.
pub type AppState = axum::extract::State<ApiState>;

/// Creates the application state from a database URL string.
pub async fn create_app_state(db_url: &str) -> Result<ApiState, Box<dyn std::error::Error>> {
    // Set up the database connection pool
    let pool: Pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await?;

    create_app_state_with_pool(pool).await
}

/// Creates the application state from an existing database pool and populates caches.
/// This is useful for tests where the pool is created and migrated manually.
pub async fn create_app_state_with_pool(
    pool: Pool,
) -> Result<ApiState, Box<dyn std::error::Error>> {
    // --- Populate the rarity cache at startup ---
    println!("Loading rarities into cache...");
    let rarities: Vec<(String, RarityType)> =
        sqlx::query_as("SELECT rarity_code, rarity_type FROM rarities")
            .fetch_all(&pool)
            .await?;
    let rarity_cache: Arc<RwLock<HashMap<String, RarityType>>> =
        Arc::new(RwLock::new(rarities.into_iter().collect()));
    println!(
        "-> Loaded {} rarity mappings.",
        rarity_cache.read().await.len()
    );

    // --- Populate the name variant cache at startup ---
    println!("Loading name variants into cache...");
    let name_variants: Vec<(String, String)> =
        sqlx::query_as("SELECT variant_name, canonical_name FROM name_variants")
            .fetch_all(&pool)
            .await?;
    let name_variant_cache: Arc<RwLock<HashMap<String, String>>> =
        Arc::new(RwLock::new(name_variants.into_iter().collect()));
    println!(
        "-> Loaded {} name variant mappings.",
        name_variant_cache.read().await.len()
    );

    // --- Populate the group variant cache at startup ---
    println!("Loading group variants into cache...");
    let group_variants: Vec<(String, String)> =
        sqlx::query_as("SELECT variant_name, canonical_name FROM group_variants")
            .fetch_all(&pool)
            .await?;
    let group_variant_cache: Arc<RwLock<HashMap<String, String>>> =
        Arc::new(RwLock::new(group_variants.into_iter().collect()));
    println!(
        "-> Loaded {} group variant mappings.",
        group_variant_cache.read().await.len()
    );

    Ok(ApiState {
        pool,
        rarity_cache,
        name_variant_cache,
        group_variant_cache,
    })
}

/// Creates the main Axum router for the application.
pub fn create_router(app_state: ApiState) -> Router {
    Router::new()
        .route(
            "/cards",
            get(handlers::cards::get_all).post(handlers::cards::create),
        )
        .route("/cards/:id", get(handlers::cards::get_by_id))
        .route(
            "/rarities",
            get(handlers::rarities::get_all).post(handlers::rarities::add),
        )
        .route(
            "/rarities/:code",
            get(handlers::rarities::get_by_code).delete(handlers::rarities::delete),
        )
        .route(
            "/variants/names",
            get(handlers::name_variants::get_all).post(handlers::name_variants::add),
        )
        .route(
            "/variants/names/:variant",
            axum::routing::delete(handlers::name_variants::delete),
        )
        .route(
            "/variants/groups",
            get(handlers::group_variants::get_all).post(handlers::group_variants::add),
        )
        .route(
            "/variants/groups/:variant",
            axum::routing::delete(handlers::group_variants::delete),
        )
        .with_state(app_state)
}
