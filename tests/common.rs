use llocg_backend_api::ApiState;
use sqlx::sqlite::SqlitePoolOptions;

/// Helper function to set up a test environment with an in-memory DB.
pub async fn setup_test_env() -> ApiState {
    // 1. Create an in-memory SQLite database pool.
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .expect("Failed to create in-memory database pool.");

    // 2. Run the migrations.
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations on in-memory database.");

    // 3. Create the app state with the migrated database.
    llocg_backend_api::create_app_state_with_pool(pool)
        .await
        .expect("Failed to create test app state.")
}
