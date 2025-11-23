use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use llocg_backend_api::{
    create_router,
};
use tower::ServiceExt; // for `oneshot`

mod common;

#[tokio::test]
async fn test_names_endpoints() {
    let state = common::setup_test_env().await;
    let app = create_router(state);

    // 1. GET all names should return 52 names.
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/names")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let names: Vec<String> = serde_json::from_slice(&body).unwrap();
    assert!(names.len() == 52);
}
