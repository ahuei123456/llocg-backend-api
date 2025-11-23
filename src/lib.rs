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
    pub sets_cache: Arc<RwLock<Vec<models::SetResponse>>>,
    pub groups_cache: Arc<RwLock<Vec<String>>>,
    pub units_cache: Arc<RwLock<Vec<String>>>,
    pub names_cache: Arc<RwLock<Vec<String>>>,
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

    // --- Populate the sets cache at startup ---
    println!("Loading sets into cache...");
    let sets = db::fetch_all_sets(&pool).await?;
    let sets_cache = Arc::new(RwLock::new(sets));
    println!("-> Loaded {} sets.", sets_cache.read().await.len());

    // --- Populate the groups cache at startup ---
    println!("Loading groups into cache...");
    let groups = db::fetch_all_groups(&pool).await?;
    let groups_cache = Arc::new(RwLock::new(groups));
    println!("-> Loaded {} groups.", groups_cache.read().await.len());

    // --- Populate the units cache at startup ---
    println!("Loading units into cache...");
    let units = db::fetch_all_units(&pool).await?;
    let units_cache = Arc::new(RwLock::new(units));
    println!("-> Loaded {} units.", units_cache.read().await.len());

    // --- Populate the names cache at startup ---
    println!("Loading names into cache...");
    let names = db::fetch_all_card_names(&pool).await?;
    let names_cache = Arc::new(RwLock::new(names));
    println!("-> Loaded {} names.", names_cache.read().await.len());

    Ok(ApiState {
        pool,
        rarity_cache,
        name_variant_cache,
        group_variant_cache,
        sets_cache,
        groups_cache,
        units_cache,
        names_cache,
    })
}

/// Creates the main Axum router for the application.
///
/// The router is configured with all the API endpoints and the shared application state.
///
/// # Endpoints
///
/// ## Cards
/// - `GET /cards`: [`handlers::cards::get_all`] - Get all cards. (Not Implemented)
/// - `POST /cards`: [`handlers::cards::create`] - Create a new card. Body: [`models::CreateCard`].
/// - `GET /cards/:id`: [`handlers::cards::get_by_id`] - Get a card by its ID. Returns: [`models::FullCard`].
/// - `POST /cards/bulk`: [`handlers::cards::create_bulk`] - Create multiple cards in bulk. Body: `Vec<[`models::CreateCard`]>`.
/// - `TODO`: `PUT /cards/:id` - Update a card.
/// - `TODO`: `PATCH /cards/:id` - Partially update a card.
/// - `TODO`: `DELETE /cards/:id` - Delete a card.
/// - `TODO`: `GET /cards/search?query` - Advanced card search.
///
/// ## Sets
/// - `GET /sets`: [`handlers::sets::get_all`] - Get all card sets. Returns: `Vec<[`models::Set`]>`.
/// - `POST /sets`: [`handlers::sets::add`] - Add a new card set. Body: [`models::CreateSet`].
/// - `DELETE /sets/:set_code`: [`handlers::sets::delete`] - Delete a card set by its code.
///
/// ## Groups
/// - `GET /groups`: [`handlers::groups::get_all`] - Get all groups. Returns: `Vec<[`models::Group`]>`.
/// - `POST /groups`: [`handlers::groups::add`] - Add a new group. Body: [`models::CreateGroup`].
/// - `DELETE /groups/:name`: [`handlers::groups::delete`] - Delete a group by its name.
///
/// ## Units
/// - `GET /units`: [`handlers::units::get_all`] - Get all units. Returns: `Vec<[`models::Unit`]>`.
/// - `POST /units`: [`handlers::units::add`] - Add a new unit. Body: [`models::CreateUnit`].
/// - `DELETE /units/:name`: [`handlers::units::delete`] - Delete a unit by its name.
///
/// ## Names
/// - `GET /names`: [`handlers::names::get_all`] - Get all distinct canonical card names.
///
/// ## Rarities
/// - `GET /rarities`: [`handlers::rarities::get_all`] - Get all rarities.
/// - `POST /rarities`: [`handlers::rarities::add`] - Add a new rarity. Body: [`models::CreateRarity`].
/// - `GET /rarities/:code`: [`handlers::rarities::get_by_code`] - Get a rarity by its code.
/// - `DELETE /rarities/:code`: [`handlers::rarities::delete`] - Delete a rarity by its code.
///
/// ## Name Variants
/// - `GET /variants/names`: [`handlers::variants::name_variants::get_all`] - Get all name variants.
/// - `POST /variants/names`: [`handlers::variants::name_variants::add`] - Add a new name variant. Body: [`models::CreateNameVariant`].
/// - `DELETE /variants/names/:variant`: [`handlers::variants::name_variants::delete`] - Delete a name variant.
///
/// ## Group Variants
/// - `GET /variants/groups`: [`handlers::variants::group_variants::get_all`] - Get all group variants.
/// - `POST /variants/groups`: [`handlers::variants::group_variants::add`] - Add a new group variant. Body: [`models::CreateGroupVariant`].
/// - `DELETE /variants/groups/:variant`: [`handlers::variants::group_variants::delete`] - Delete a group variant.
pub fn create_router(app_state: ApiState) -> Router {
    Router::new()
        // Card routes
        .route(
            "/cards",
            get(handlers::cards::get_all).post(handlers::cards::create),
        )
        .route("/cards/bulk", post(handlers::cards::create_bulk))
        .route("/cards/:id", get(handlers::cards::get_by_id))
        // Set, Group, and Unit routes
        .route(
            "/sets",
            get(handlers::sets::get_all).post(handlers::sets::add),
        )
        .route(
            "/sets/:set_code",
            axum::routing::delete(handlers::sets::delete),
        )
        .route(
            "/groups",
            get(handlers::groups::get_all).post(handlers::groups::add),
        )
        .route(
            "/groups/:name",
            axum::routing::delete(handlers::groups::delete),
        )
        .route(
            "/units",
            get(handlers::units::get_all).post(handlers::units::add),
        )
        .route(
            "/units/:name",
            axum::routing::delete(handlers::units::delete),
        )
        // Name routes
        .route("/names", get(handlers::names::get_all))
        // Rarity routes
        .route(
            "/rarities",
            get(handlers::rarities::get_all).post(handlers::rarities::add),
        )
        .route(
            "/rarities/:code",
            get(handlers::rarities::get_by_code).delete(handlers::rarities::delete),
        )
        // Name variant routes
        .route(
            "/variants/names",
            get(handlers::variants::name_variants::get_all)
                .post(handlers::variants::name_variants::add),
        )
        .route(
            "/variants/names/:variant",
            axum::routing::delete(handlers::variants::name_variants::delete),
        )
        // Group variant routes
        .route(
            "/variants/groups",
            get(handlers::variants::group_variants::get_all)
                .post(handlers::variants::group_variants::add),
        )
        .route(
            "/variants/groups/:variant",
            axum::routing::delete(handlers::variants::group_variants::delete),
        )
        .with_state(app_state)
}
