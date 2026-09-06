//! Integration tests for New Loka Server

use axum::body::Body;
use axum::http::{Request, StatusCode};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower::util::ServiceExt;

fn db_path() -> String {
    let path = std::env::temp_dir()
        .join(format!("newloka_test_{}.db", uuid::Uuid::new_v4()))
        .to_string_lossy()
        .replace("\\", "/");
    format!("sqlite:///{}?mode=rwc", path)
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
        newloka_server::NodeConfig::default(),
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

#[cfg(feature = "demo")]
#[tokio::test]
async fn test_demo_seed_data() {
    let dmk = newloka_core::crypto::DeviceMasterKey::generate();
    let storage =
        newloka_core::storage::StorageEngine::open(&db_path(), "test-node".to_string(), dmk)
            .await
            .unwrap();

    newloka_server::demo::seed_demo_data(&storage, "test-node")
        .await
        .unwrap();

    let patients = storage.search_json("Patient", None).await.unwrap();
    assert_eq!(patients.len(), 10, "Expected 10 demo patients");

    let conditions = storage.search_json("Condition", None).await.unwrap();
    assert!(!conditions.is_empty(), "Expected demo conditions");

    let observations = storage.search_json("Observation", None).await.unwrap();
    assert!(!observations.is_empty(), "Expected demo observations");

    let meds = storage
        .search_json("MedicationRequest", None)
        .await
        .unwrap();
    assert!(!meds.is_empty(), "Expected demo medications");

    let encounters = storage.search_json("Encounter", None).await.unwrap();
    assert!(!encounters.is_empty(), "Expected demo encounters");

    let allergies = storage
        .search_json("AllergyIntolerance", None)
        .await
        .unwrap();
    assert!(!allergies.is_empty(), "Expected demo allergies");

    let careplans = storage.search_json("CarePlan", None).await.unwrap();
    assert!(!careplans.is_empty(), "Expected demo care plans");

    let procedures = storage.search_json("Procedure", None).await.unwrap();
    assert!(!procedures.is_empty(), "Expected demo procedures");

    let immunizations = storage.search_json("Immunization", None).await.unwrap();
    assert!(!immunizations.is_empty(), "Expected demo immunizations");
}

#[tokio::test]
async fn test_rxpad_sync_and_dispense() {
    let (app, state) = test_app().await;

    let sync_payload = serde_json::json!({
        "patientName": "Punarnava Shastry",
        "ageSex": "51y / F",
        "contact": "+91-9833216533",
        "date": "02/09/2026",
        "weight": "65 kg",
        "bp": "126/84",
        "diagnosis": "hyperlipidemia, cystitis, atopy, prediabetes",
        "followUp": "02/03/2027",
        "meds": [
            { "drug": "NITROFURANTOIN 100MG", "dose": "1 tab", "freq": "BD", "dur": "5d", "instr": "After breakfast and dinner" },
            { "drug": "ZINCOVIT (VIT C) 500MG", "dose": "1 tab", "freq": "OD", "dur": "15d", "instr": "After lunch" }
        ],
        "labs": ["CBC (with diff)", "HbA1c + lipids"],
        "serial": 4,
        "filename": "020926_0004_PunarnavaShastry.pdf",
        "doctor": "Dr. Yash Kulkarni",
        "regNo": "MMC-20260608539"
    });

    // 1. Sync prescription from pad
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/rxpad/sync")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&sync_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let res: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(res["status"], "success");
    assert_eq!(res["serial"], 4);
    let patient_id = res["patient_id"].as_str().unwrap().to_string();
    let prescription_id = res["prescription_id"].as_str().unwrap().to_string();

    // 2. Verify Patient created in storage
    let storage = {
        let s = state.read().await;
        s.storage.clone()
    };
    let patient = storage.get_json(&patient_id).await.unwrap().unwrap();
    assert_eq!(patient["gender"], "female");

    // 3. Verify MedicationRequests created
    let meds = storage.search_json("MedicationRequest", Some(&patient_id)).await.unwrap();
    assert_eq!(meds.len(), 2, "Expected 2 medication requests");

    // 4. Verify Blood Pressure Observation created
    let obs = storage.search_json("Observation", Some(&patient_id)).await.unwrap();
    assert!(!obs.is_empty(), "Expected vitals observations");

    // 5. Query Prescriptions list
    let list_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/rxpad/prescriptions")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_resp.status(), StatusCode::OK);

    // 6. Dispense prescription
    let dispense_payload = serde_json::json!({
        "practitioner": "Dr. Yash Kulkarni",
        "lot_number": "LOT-2026-TEST",
        "quantity": "15 tabs",
        "notes": "Dispensed at clinic dispensary"
    });

    let dispense_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/rxpad/prescriptions/{}/dispense", prescription_id))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&dispense_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(dispense_resp.status(), StatusCode::OK);
    let disp_body = axum::body::to_bytes(dispense_resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let disp_res: serde_json::Value = serde_json::from_slice(&disp_body).unwrap();
    assert_eq!(disp_res["status"], "dispensed");

    // 7. Verify MedicationDispense in storage
    let dispenses = storage.search_json("MedicationDispense", Some(&patient_id)).await.unwrap();
    assert_eq!(dispenses.len(), 2, "Expected 2 MedicationDispense records");

    // 8. Verify next serial
    let serial_resp = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/rxpad/next-serial")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(serial_resp.status(), StatusCode::OK);
    let s_body = axum::body::to_bytes(serial_resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let s_res: serde_json::Value = serde_json::from_slice(&s_body).unwrap();
    assert_eq!(s_res["next_serial"], 5);
}

#[tokio::test]
async fn test_static_files() {
    let (app, _) = test_app().await;

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/static/index.html")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let ct = resp.headers().get("content-type").unwrap().to_str().unwrap().to_string();
    println!("Content-Type for /static/index.html: {}", ct);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let text = String::from_utf8_lossy(&body);
    println!("Body preview for /static/index.html: {}", &text[..text.len().min(100)]);

    let resp_js = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/static/js/app.js")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp_js.status(), StatusCode::OK);
    let ct_js = resp_js.headers().get("content-type").unwrap().to_str().unwrap().to_string();
    println!("Content-Type for /static/js/app.js: {}", ct_js);
    let body_js = axum::body::to_bytes(resp_js.into_body(), usize::MAX).await.unwrap();
    let text_js = String::from_utf8_lossy(&body_js);
    println!("Body preview for /static/js/app.js: {}", &text_js[..text_js.len().min(100)]);
    assert!(ct_js.contains("javascript"), "Expected javascript, got: {}", ct_js);
    assert!(!text_js.starts_with("<!DOCTYPE"), "js/app.js returned HTML!");

}


