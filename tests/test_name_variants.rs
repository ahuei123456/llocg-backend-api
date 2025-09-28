use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use llocg_backend_api::create_router;
use std::collections::HashMap;
use tower::ServiceExt; // for `oneshot`

mod common;

#[tokio::test]
async fn test_name_variants_endpoints() {
    let state = common::setup_test_env().await;
    let app = create_router(state);

    // 1. Initially, GET all name_variants should return the defaults from migrations.
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/variants/names")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    // Deserialize the response body into a HashMap for type-safe assertions.
    let name_variants: HashMap<String, String> = serde_json::from_slice(&body).unwrap();
    assert_eq!(name_variants.len(), 2);
    assert_eq!(
        name_variants.get("Kanon Shibuya"),
        Some(&"Shibuya Kanon".to_string())
    );
    assert_eq!(
        name_variants.get("澁谷かのん"),
        Some(&"Shibuya Kanon".to_string())
    );

    // 2. POST a new name variant.
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/variants/names")
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    r#"{"variant_name": "Test Variant", "canonical_name": "Test Canonical"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    // 3. GET all variants again; it should contain the new one.
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/variants/names")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let name_variants: HashMap<String, String> = serde_json::from_slice(&body).unwrap();
    assert_eq!(name_variants.len(), 3);
    assert_eq!(
        name_variants.get("Test Variant"),
        Some(&"Test Canonical".to_string())
    );

    // 4. DELETE the variant.
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(http::Method::DELETE)
                .uri("/variants/names/Test%20Variant") // URL encode the space
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // 5. GET all variants again; it should be back to the defaults.
    let response = app.clone().oneshot(Request::builder().uri("/variants/names").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let name_variants: HashMap<String, String> = serde_json::from_slice(&body).unwrap();
    assert_eq!(name_variants.len(), 2);
}
