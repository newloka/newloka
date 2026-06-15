//! Integration tests for New Loka Server

use axum::body::Body;
use axum::http::{Request, StatusCode};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower::util::ServiceExt;

fn db_path() -> String {
    format!(
        "file:/{}/newloka_test_{}.db?mode=rwc",
        std::env::temp_dir().to_string_lossy().replace("\\\\", "/"),
        uuid::Uuid::new_v4()
    )
}

async fn test_app() -> (axum::Router, Arc<RwLock<newloka_server::AppState>>) {
    let dmk = newloka_core::crypto::DeviceMasterKey::generate();
    let storage =
        newloka_core::storage::StorageEngine::open(&db_path(), "test-node".to_string(), dmk)
            .await
            .unwrap();
    let storage = Arc::new(storage);
    let state = Arc::new(RwLock::new(newloka_server::AppState::new(
        "test-node".to_string(),
        storage,
    )));
    (newloka_server::app(state.clone()), state)
}

#[tokio::test]
async fn test_health_endpoint() {
    let (app, _) = test_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_metadata_endpoint() {
    let (app, _) = test_app().await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/metadata")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_patient_create_and_not_found() {
    let (app, state) = test_app().await;

    // Create a patient
    let payload = serde_json::json!({
        "resourceType": "Patient",
        "id": "pat-test-001",
        "name": [{ "family": "Doe", "given": ["Jane"] }],
        "gender": "female",
        "birthDate": "1990-01-01"
    });
    let create = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/Patient")
                .header("Content-Type", "application/fhir+json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create.status(), StatusCode::CREATED);

    // Verify persistence directly through storage
    let s = state.read().await;
    let direct = s.storage.get_json("pat-test-001").await.unwrap();
    assert!(
        direct.is_some(),
        "Patient should exist in storage after POST"
    );
    drop(s);

    // Verify that a missing patient returns 404
    let missing = app
        .oneshot(
            Request::builder()
                .uri("/Patient/does-not-exist")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_audit_event_create_and_search() {
    let (app, _) = test_app().await;

    let payload = serde_json::json!({
        "resourceType": "AuditEvent",
        "id": "aud-test-001",
        "action": "C",
        "recorded": "2026-06-15T10:00:00Z",
        "outcome": "Success"
    });
    let create = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/AuditEvent")
                .header("Content-Type", "application/fhir+json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create.status(), StatusCode::CREATED);

    let search = app
        .oneshot(
            Request::builder()
                .uri("/AuditEvent")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(search.status(), StatusCode::OK);
}
