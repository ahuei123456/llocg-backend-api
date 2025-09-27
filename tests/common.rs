use llocg_backend_api::{create_app_state, ApiState};

/// Helper function to set up a test environment with an in-memory DB.
/// This function now uses the same `create_app_state` as the main application,
/// ensuring the test environment is consistent with production.
pub async fn setup_test_env() -> ApiState {
    // The `create_app_state` function handles DB connection, migrations, and cache loading.
    // We pass the in-memory DB connection string to it.
    create_app_state("sqlite::memory:")
        .await
        .expect("Failed to create test app state.")
}