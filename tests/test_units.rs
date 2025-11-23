use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use llocg_backend_api::create_router;
use tower::ServiceExt; // for `oneshot`

mod common;

#[tokio::test]
async fn test_units_endpoints() {
    let state = common::setup_test_env().await;
    let app = create_router(state);

    // 1. Initially, GET all sets should return an empty list as none are added by default.
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/units")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let units: Vec<String> = serde_json::from_slice(&body).unwrap();
    assert!(units.len() == 20);

    // 2. POST a new unit.
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/units")
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"name": "AiScream!"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    // 3. GET all sets again; it should contain the new one.
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/units")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let units: Vec<String> = serde_json::from_slice(&body).unwrap();
    assert_eq!(units.len(), 21);
    assert_eq!(units[20], "AiScream!");

    // 4. POST a duplicate set to test conflict.
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(http::Method::POST)
                .uri("/units")
                .header(http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"name": "AiScream!"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CONFLICT);

    // 5. DELETE the set.
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(http::Method::DELETE)
                .uri("/units/AiScream!")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // 6. GET all sets again; it should be empty.
    let response = app
        .oneshot(
            Request::builder()
                .uri("/units")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let units: Vec<String> = serde_json::from_slice(&body).unwrap();
    assert!(units.len() == 20);
}
