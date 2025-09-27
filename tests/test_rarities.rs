use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use llocg_backend_api::{create_router, models::RarityType};
use std::collections::HashMap;
use tower::ServiceExt; // for `oneshot`

mod common;

#[tokio::test]
async fn test_rarities_endpoints() {
    let state = common::setup_test_env().await;
    let app = create_router(state);

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
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    // Deserialize the response body into a HashMap for type-safe assertions.
    let rarities: HashMap<String, RarityType> = serde_json::from_slice(&body).unwrap();
    assert_eq!(rarities.len(), 2);
    assert_eq!(rarities.get("P"), Some(&RarityType::Parallel));
    assert_eq!(rarities.get("LLE"), Some(&RarityType::Parallel));
    

    // 2. POST a new rarity.
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/rarities")
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"rarity_code": "TEST", "rarity_type": "Regular"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    // 3. GET all rarities again; it should contain the new rarity.
    let response = app
        .clone()
        .oneshot(Request::builder().uri("/rarities").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let rarities: HashMap<String, RarityType> = serde_json::from_slice(&body).unwrap();
    assert_eq!(rarities.len(), 3);
    assert_eq!(rarities.get("P"), Some(&RarityType::Parallel));
    assert_eq!(rarities.get("LLE"), Some(&RarityType::Parallel));
    assert_eq!(rarities.get("TEST"), Some(&RarityType::Regular));

    // 4. DELETE the rarity.
    let response = app
        .clone()
        .oneshot(Request::builder().method(http::Method::DELETE).uri("/rarities/TEST").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // 5. GET all rarities again, it should be back to the default.
    let response = app
        .clone()
        .oneshot(Request::builder().uri("/rarities").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let rarities: HashMap<String, RarityType> = serde_json::from_slice(&body).unwrap();
    assert_eq!(rarities.len(), 2);
    assert_eq!(rarities.get("P"), Some(&RarityType::Parallel));
    assert_eq!(rarities.get("LLE"), Some(&RarityType::Parallel));
}