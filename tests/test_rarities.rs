use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use llocg_backend_api::create_router;
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
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(&body[..], b"{}"); // Empty because migrations don't insert rarities.

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
        .oneshot(Request::builder().uri("/rarities").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(&body[..], b"{\"TEST\":\"Parallel\"}");

    // 4. DELETE the rarity.
    let response = app
        .oneshot(Request::builder().method(http::Method::DELETE).uri("/rarities/TEST").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}