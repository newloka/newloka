//! Integration tests for New Loka Server

use axum::body::Body;
use axum::http::{Request, StatusCode};
use base64::Engine;
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
    let dmk = newloka_core::crypto::DeviceMasterKey {
        key: newloka_demo::DEMO_MASTER_KEY_BYTES,
    };
    let storage = newloka_storage::StorageEngine::open(&db_path(), "test-node".to_string(), dmk)
        .await
        .unwrap();
    newloka_server::demo::seed_demo_data(&storage, "test-node")
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

async fn test_app_unseeded() -> (axum::Router, Arc<RwLock<newloka_server::AppState>>) {
    let dmk = newloka_core::crypto::DeviceMasterKey::generate();
    let storage = newloka_storage::StorageEngine::open(&db_path(), "test-node".to_string(), dmk)
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
    let dmk = newloka_core::crypto::DeviceMasterKey {
        key: newloka_demo::DEMO_MASTER_KEY_BYTES,
    };
    let storage = newloka_storage::StorageEngine::open(&db_path(), "test-node".to_string(), dmk)
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
        "patientName": "Ramesh Patel",
        "ageSex": "54y / M",
        "contact": "+1-555-019-2834",
        "date": "15/11/2026",
        "weight": "72 kg",
        "bp": "130/84",
        "diagnosis": "Type 2 Diabetes Mellitus & Essential Hypertension",
        "followUp": "15/12/2026",
        "meds": [
            { "drug": "METFORMIN 500MG", "dose": "1 tab", "freq": "BD", "dur": "30d", "instr": "After meals" },
            { "drug": "TELMISARTAN 40MG", "dose": "1 tab", "freq": "OD", "dur": "30d", "instr": "Morning" }
        ],
        "labs": ["HbA1c", "Lipid Profile"],
        "serial": 4,
        "filename": "151126_0004_RameshPatel.pdf",
        "doctor": "Dr. Alex Mercer",
        "regNo": "NL-DOC-2026-001"
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
    assert_eq!(patient["gender"], "male");

    // 3. Verify MedicationRequests created
    let meds = storage
        .search_json("MedicationRequest", Some(&patient_id))
        .await
        .unwrap();
    assert_eq!(meds.len(), 2, "Expected 2 medication requests");

    // 4. Verify Blood Pressure Observation created
    let obs = storage
        .search_json("Observation", Some(&patient_id))
        .await
        .unwrap();
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
        "practitioner": "Dr. Alex Mercer",
        "lot_number": "LOT-2026-TEST",
        "quantity": "15 tabs",
        "notes": "Dispensed at clinic dispensary"
    });

    let dispense_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/rxpad/prescriptions/{}/dispense",
                    prescription_id
                ))
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
    let dispenses = storage
        .search_json("MedicationDispense", Some(&patient_id))
        .await
        .unwrap();
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
    let ct = resp
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    println!("Content-Type for /static/index.html: {}", ct);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let text = String::from_utf8_lossy(&body);
    println!(
        "Body preview for /static/index.html: {}",
        &text[..text.len().min(100)]
    );

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
    let ct_js = resp_js
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    println!("Content-Type for /static/js/app.js: {}", ct_js);
    let body_js = axum::body::to_bytes(resp_js.into_body(), usize::MAX)
        .await
        .unwrap();
    let text_js = String::from_utf8_lossy(&body_js);
    println!(
        "Body preview for /static/js/app.js: {}",
        &text_js[..text_js.len().min(100)]
    );
    assert!(
        ct_js.contains("javascript"),
        "Expected javascript, got: {}",
        ct_js
    );
    assert!(
        !text_js.starts_with("<!DOCTYPE"),
        "js/app.js returned HTML!"
    );
}

#[tokio::test]
async fn test_rxpad_pdf_and_events() {
    let (app, state) = test_app().await;

    // 1. Subscribe to SSE events
    let mut rx = {
        let s = state.read().await;
        s.rxpad_tx.subscribe()
    };

    // 2. Sync with pdf_base64
    let dummy_pdf = b"%PDF-1.4 mock pdf content for testing";
    let pdf_b64 = base64::engine::general_purpose::STANDARD.encode(dummy_pdf);
    let filename = "060926_0005_TestPatient.pdf";

    let payload = serde_json::json!({
        "patientName": "Test Event Patient",
        "date": "06/09/2026",
        "serial": 5,
        "filename": filename,
        "pdfBase64": format!("data:application/pdf;base64,{}", pdf_b64),
        "meds": [{ "drug": "PARACETAMOL 500MG", "dose": "1 tab", "freq": "TDS" }]
    });

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/rxpad/sync")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    // 3. Verify event received on broadcast
    let event = rx.try_recv().expect("Expected prescription_created event");
    assert_eq!(event, "prescription_created");

    // 4. Dispense and check event
    let disp_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/rxpad/prescriptions/rxpad-0005/dispense")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&serde_json::json!({})).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(disp_resp.status(), StatusCode::OK);
    let disp_event = rx
        .try_recv()
        .expect("Expected prescription_dispensed event");
    assert_eq!(disp_event, "prescription_dispensed");

    // 5. Delete and check event
    let del_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/rxpad/prescriptions/rxpad-0005")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(del_resp.status(), StatusCode::OK);
    let del_event = rx.try_recv().expect("Expected prescription_deleted event");
    assert_eq!(del_event, "prescription_deleted");
}

#[tokio::test]
async fn test_config_and_prescriber_endpoints() {
    let (app, _) = test_app().await;

    // Test GET /api/config
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/config")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let cfg: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(cfg["tier"], "T0");
    assert_eq!(cfg["defaultLandingView"], "rxpad");

    // Test GET /api/prescriber/profile
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/prescriber/profile")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let prescriber: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(prescriber["isLegalPrescriber"], true);
    assert!(prescriber["clinicianName"]
        .as_str()
        .unwrap()
        .contains("Dr."));
    assert_eq!(prescriber["qualifications"], "MBBS, MD");
    assert_eq!(prescriber["department"], "");
    assert_eq!(prescriber["clinicHeader"]["clinicName"], "New Loka Clinic");

    // Test GET /api/rxpad/template (RxPad Configurable Template & Box Behavior Engine)
    let res_tpl = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/rxpad/template")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_tpl.status(), StatusCode::OK);
    let body_tpl = axum::body::to_bytes(res_tpl.into_body(), usize::MAX)
        .await
        .unwrap();
    let tpl: serde_json::Value = serde_json::from_slice(&body_tpl).unwrap();
    assert_eq!(tpl["header"]["entityName"], "New Loka Clinic");
    assert_eq!(tpl["header"]["qualifications"], "MBBS, MD");
    assert_eq!(tpl["header"]["showDepartment"], false);
    assert_eq!(tpl["bindings"]["doctorName"], "persist");
    assert_eq!(tpl["bindings"]["currentDate"], "fetch");
    assert_eq!(tpl["bindings"]["medications"], "db_sync");
    assert_eq!(tpl["visibility"]["department"], false);
    assert_eq!(tpl["visibility"]["weight"], false);

    // Test POST /api/rxpad/template
    let mut updated_tpl = tpl.clone();
    updated_tpl["header"]["entityName"] = serde_json::json!("New Loka Health Center");
    let res_post = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/rxpad/template")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&updated_tpl).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_post.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_unconfigured_empty_database_graceful_nulls() {
    let (app, _) = test_app_unseeded().await;

    // 1. GET /api/config -> clinic is null
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/config")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let cfg: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(cfg["clinic"].is_null(), "Unconfigured clinic must be null");

    // 2. GET /api/prescriber/profile -> clinicianName is empty, clinicHeader is null
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/prescriber/profile")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let prof: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(prof["clinicianName"], "");
    assert!(
        prof["clinicHeader"].is_null(),
        "Unconfigured clinicHeader must be null"
    );

    // 3. GET /api/rxpad/template -> standard unconfigured template with empty fields & false visibility
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/rxpad/template")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let tpl: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(tpl["header"]["entityName"], "");
    assert_eq!(tpl["header"]["doctorName"], "");
    assert_eq!(tpl["visibility"]["entityName"], false);

    // 4. GET /api/inpatient/census -> empty array []
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/inpatient/census")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let census: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(census, serde_json::json!([]));

    // 5. GET /api/billing/invoices -> empty array []
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/billing/invoices")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let inv: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(inv, serde_json::json!([]));
}

#[tokio::test]
async fn test_pharmacopoeia_endpoints_and_order_sets() {
    let (app, _) = test_app().await;

    // 1. GET /api/pharmacopoeia/info
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/pharmacopoeia/info")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let info: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(info["version"], "2026.11");
    assert!(info["monographsCount"].as_u64().unwrap() >= 50);
    assert!(info["orderSetsCount"].as_u64().unwrap() >= 10);

    // 2. GET /api/pharmacopoeia/search?q=paracetamol
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/pharmacopoeia/search?q=paracetamol")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let results: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert!(!results.is_empty());
    assert_eq!(results[0]["entry"]["inn_name"], "Paracetamol");

    // 3. GET /api/pharmacopoeia/search?q=Dolo (Indian brand search)
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/pharmacopoeia/search?q=Dolo")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let results: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert!(!results.is_empty());
    assert_eq!(results[0]["entry"]["inn_name"], "Paracetamol");
    assert_eq!(results[0]["matched_via"], "brand");

    // 4. GET /api/pharmacopoeia/entry/ip-paracetamol
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/pharmacopoeia/entry/ip-paracetamol")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let entry: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(entry["id"], "ip-paracetamol");
    assert_eq!(entry["schedule"], "OTC");

    // 5. GET /api/pharmacopoeia/categories
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/pharmacopoeia/categories")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let categories: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert!(!categories.is_empty());
    assert!(categories
        .iter()
        .any(|c| c["name"].as_str().unwrap().contains("Cardiovascular")));

    // 6. GET /api/pharmacopoeia/order-sets
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/pharmacopoeia/order-sets")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let sets: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert!(sets.len() >= 10);

    // 7. GET /api/pharmacopoeia/order-sets/orderset-acs-chestpain
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/pharmacopoeia/order-sets/orderset-acs-chestpain")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let set: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(set["id"], "orderset-acs-chestpain");
    assert!(set["items"].as_array().unwrap().len() >= 4);

    // 8. POST /api/pharmacopoeia/check-safety
    let safety_req = serde_json::json!({
        "drugs": ["Augmentin 625 Duo", "Ibuprofen 400mg", "Diclofenac 50mg"],
        "allergies": ["Penicillin"]
    });
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/pharmacopoeia/check-safety")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_vec(&safety_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let report: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(report["has_critical"], true);
    let warnings = report["warnings"].as_array().unwrap();
    assert!(warnings
        .iter()
        .any(|w| w["warning_type"] == "AllergyCrossReactivity"));
    assert!(warnings
        .iter()
        .any(|w| w["warning_type"] == "DuplicateTherapy"));

    // 9. GET /api/rxpad/formulary - verify pharmacopoeia enrichment
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/rxpad/formulary")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let formulary: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert!(
        formulary.len() > 15,
        "Expected pharmacopoeia to enrich formulary items"
    );
    assert!(formulary
        .iter()
        .any(|f| f["drug"].as_str().unwrap().contains("PARACETAMOL")));
}

#[tokio::test]
async fn test_inpatient_and_billing_with_abac() {
    let (app, _) = test_app().await;

    // Test GET /api/inpatient/census
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/inpatient/census")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let census: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(census.as_array().unwrap().len() >= 3);

    // Test GET /api/billing/invoices as Clinician (full clinical correlation)
    let res_doc = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/billing/invoices")
                .header("x-user-role", "clinician")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_doc.status(), StatusCode::OK);
    let body_doc = axum::body::to_bytes(res_doc.into_body(), usize::MAX)
        .await
        .unwrap();
    let inv_doc: serde_json::Value = serde_json::from_slice(&body_doc).unwrap();
    let diag_doc = inv_doc[0]["diagnosis"].as_str().unwrap();
    assert!(!diag_doc.contains("REDACTED"));

    // Test GET /api/billing/invoices as Billing Staff (ABAC clinical data redacted)
    let res_bill = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/billing/invoices")
                .header("x-user-role", "billing")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_bill.status(), StatusCode::OK);
    let body_bill = axum::body::to_bytes(res_bill.into_body(), usize::MAX)
        .await
        .unwrap();
    let inv_bill: serde_json::Value = serde_json::from_slice(&body_bill).unwrap();
    let diag_bill = inv_bill[0]["diagnosis"].as_str().unwrap();
    assert!(diag_bill.contains("REDACTED"));

    // Test GET /api/pharmacy/summary
    let res_pharm = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/pharmacy/summary")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_pharm.status(), StatusCode::OK);
    let body_pharm = axum::body::to_bytes(res_pharm.into_body(), usize::MAX)
        .await
        .unwrap();
    let pharm: serde_json::Value = serde_json::from_slice(&body_pharm).unwrap();
    assert!(pharm["criticalStockAlerts"].as_array().unwrap().len() >= 2);
    assert!(pharm["safetyAlerts"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_departments_and_transfers_endpoints() {
    let (app, _) = test_app().await;

    // 1. GET /api/departments
    let res_depts = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/departments")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_depts.status(), StatusCode::OK);
    let body_depts = axum::body::to_bytes(res_depts.into_body(), usize::MAX)
        .await
        .unwrap();
    let depts: serde_json::Value = serde_json::from_slice(&body_depts).unwrap();
    assert!(depts.as_array().unwrap().len() >= 6);

    // 2. GET /api/departments/transfers
    let res_tr_list = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/departments/transfers")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_tr_list.status(), StatusCode::OK);

    // 3. POST /api/departments/transfers
    let payload_tr = serde_json::json!({
        "id": "tr-test-99",
        "patient_id": "pat-test-001",
        "patient_name": "Ramesh Sharma",
        "mrn": "MRN-101",
        "source_department": "Emergency & Trauma",
        "target_department": "Cardiology & Cath Lab",
        "requesting_clinician_id": "doc_emergency",
        "requesting_clinician_name": "Dr. Acute",
        "urgency": "urgent",
        "reason": "Suspected STEMI",
        "clinical_handover_notes": "Chest pain with anterior ST elevations.",
        "active_medications_summary": ["Aspirin 325mg"],
        "pending_orders_summary": ["Stat Troponin"],
        "allergies_summary": ["None"]
    });

    let res_create = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/departments/transfers")
                .header("Content-Type", "application/json")
                .body(Body::from(payload_tr.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_create.status(), StatusCode::CREATED);

    // 4. POST /api/departments/transfers/tr-test-99/accept
    let accept_payload = serde_json::json!({
        "accepting_clinician_id": "doc_attending",
        "accepting_clinician_name": "Dr. Cardio Attending",
        "assigned_bed": "CCU Bed 05"
    });

    let res_accept = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/departments/transfers/tr-test-99/accept")
                .header("Content-Type", "application/json")
                .body(Body::from(accept_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_accept.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_enterprise_ldap_login_and_config() {
    let (app, _) = test_app().await;

    // GET /api/auth/directory/config
    let res_cfg = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/auth/directory/config")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_cfg.status(), StatusCode::OK);

    // POST /api/auth/enterprise/login (valid doctor)
    let login_payload = serde_json::json!({
        "username": "dr.mercer@hospital.local",
        "password": "valid_enterprise_password_2026",
        "tier": "T3"
    });
    let res_login = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/enterprise/login")
                .header("Content-Type", "application/json")
                .body(Body::from(login_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_login.status(), StatusCode::OK);
    let body_login = axum::body::to_bytes(res_login.into_body(), usize::MAX)
        .await
        .unwrap();
    let login_res: serde_json::Value = serde_json::from_slice(&body_login).unwrap();
    assert!(!login_res["token"].as_str().unwrap().is_empty());
    assert_eq!(login_res["tier"].as_str().unwrap(), "T3");

    // POST /api/auth/enterprise/login (empty password -> 401)
    let bad_payload = serde_json::json!({
        "username": "dr.mercer@hospital.local",
        "password": ""
    });
    let res_bad = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/enterprise/login")
                .header("Content-Type", "application/json")
                .body(Body::from(bad_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_bad.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_hospital_reports_endpoints() {
    let (app, _) = test_app().await;

    let res_rep = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/reports/overview")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_rep.status(), StatusCode::OK);
    let body_rep = axum::body::to_bytes(res_rep.into_body(), usize::MAX)
        .await
        .unwrap();
    let rep: serde_json::Value = serde_json::from_slice(&body_rep).unwrap();
    assert!(rep["metrics"]["totalHospitalBeds"].as_u64().unwrap() > 0);
    assert!(rep["departmentCensusBreakdown"].as_array().unwrap().len() >= 5);

    let res_dept = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/reports/department/dept-cardio")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_dept.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_research_federation_and_differential_privacy() {
    let (app, _) = test_app().await;

    // GET /api/federation/nodes
    let res_nodes = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/federation/nodes")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_nodes.status(), StatusCode::OK);

    // POST /api/federation/ping
    let res_ping = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/federation/ping")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "peer_url": "https://federation.aiims.edu/newloka" })
                        .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_ping.status(), StatusCode::OK);

    // POST /api/research/query
    let query_payload = serde_json::json!({
        "query_id": "q-diabetes-01",
        "title": "Type 2 Diabetes HbA1c Cohort",
        "inclusion_conditions": ["E11"],
        "min_age": 40,
        "max_age": 80,
        "gender": null,
        "epsilon": 0.5,
        "k_anonymity": 5
    });
    let res_query = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/research/query")
                .header("Content-Type", "application/json")
                .body(Body::from(query_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_query.status(), StatusCode::OK);
    let body_query = axum::body::to_bytes(res_query.into_body(), usize::MAX)
        .await
        .unwrap();
    let query_res: serde_json::Value = serde_json::from_slice(&body_query).unwrap();
    assert_eq!(query_res["k_anonymity_guarantee"], 5);

    // POST /api/research/export
    let res_exp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/research/export")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "cohort_id": "cohort-001" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_exp.status(), StatusCode::OK);
    let body_exp = axum::body::to_bytes(res_exp.into_body(), usize::MAX)
        .await
        .unwrap();
    let exp_res: serde_json::Value = serde_json::from_slice(&body_exp).unwrap();
    assert!(exp_res["privacyGuarantees"]["piiStripped"]
        .as_bool()
        .unwrap());
}

#[tokio::test]
async fn test_fhir_consent_and_directive_verification() {
    let (app, _) = test_app().await;

    // 1. POST /Consent (FHIR R4 resource)
    let fhir_consent = serde_json::json!({
        "resourceType": "Consent",
        "id": "consent-fhir-001",
        "status": "active",
        "scope": {
            "coding": [{ "system": "http://terminology.hl7.org/CodeSystem/consentscope", "code": "research" }]
        },
        "category": [{
            "coding": [{ "system": "http://loinc.org", "code": "59284-0" }]
        }]
    });

    let res_fhir = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/Consent")
                .header("Content-Type", "application/fhir+json")
                .body(Body::from(fhir_consent.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_fhir.status(), StatusCode::CREATED);

    // 2. GET /Consent/consent-fhir-001
    let res_get_c = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/Consent/consent-fhir-001")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_get_c.status(), StatusCode::OK);

    // 3. POST /api/consent/directives
    let directive_payload = serde_json::json!({
        "id": "dir-test-01",
        "patient_id": "pat-test-001",
        "status": "active",
        "scope": "research",
        "purpose": "RESEARCH",
        "recipient_node_id": "node-aiims-delhi",
        "recipient_name": "AIIMS Central Research",
        "data_categories": ["Observation", "Condition"],
        "valid_from": "2026-01-01T00:00:00Z",
        "valid_to": "2027-12-31T23:59:59Z",
        "revoked": false,
        "consent_notes": "Patient authorized diabetes research"
    });

    let res_dir = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/consent/directives")
                .header("Content-Type", "application/json")
                .body(Body::from(directive_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_dir.status(), StatusCode::CREATED);

    // 4. POST /api/consent/verify -> permitted: true
    let verify_payload = serde_json::json!({
        "patient_id": "pat-test-001",
        "recipient_node_id": "node-aiims-delhi",
        "purpose": "RESEARCH",
        "resource_type": "Observation"
    });
    let res_ver = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/consent/verify")
                .header("Content-Type", "application/json")
                .body(Body::from(verify_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_ver.status(), StatusCode::OK);
    let body_ver = axum::body::to_bytes(res_ver.into_body(), usize::MAX)
        .await
        .unwrap();
    let ver_res: serde_json::Value = serde_json::from_slice(&body_ver).unwrap();
    assert!(ver_res["permitted"].as_bool().unwrap());

    // 5. POST /api/consent/directives/dir-test-01/revoke
    let res_revoke = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/consent/directives/dir-test-01/revoke")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "reason": "Patient requested opt-out" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_revoke.status(), StatusCode::OK);

    // 6. POST /api/consent/verify -> permitted: false after revocation
    let res_ver_after = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/consent/verify")
                .header("Content-Type", "application/json")
                .body(Body::from(verify_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_ver_after.status(), StatusCode::OK);
    let body_after = axum::body::to_bytes(res_ver_after.into_body(), usize::MAX)
        .await
        .unwrap();
    let ver_after_res: serde_json::Value = serde_json::from_slice(&body_after).unwrap();
    assert!(!ver_after_res["permitted"].as_bool().unwrap());
}

#[tokio::test]
async fn test_ingest_samples_extract_and_commit() {
    let (app, _) = test_app().await;

    // 1. GET /api/ingest/samples
    let res_samples = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/ingest/samples")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_samples.status(), StatusCode::OK);
    let body_s = axum::body::to_bytes(res_samples.into_body(), usize::MAX)
        .await
        .unwrap();
    let samples_json: serde_json::Value = serde_json::from_slice(&body_s).unwrap();
    let cases = samples_json["cases"].as_array().unwrap();
    assert!(cases.len() >= 4);

    // 2. Create Target Patient
    let patient_payload = serde_json::json!({
        "resourceType": "Patient",
        "id": "pat-ingest-001",
        "name": [{ "text": "Rajesh Kumar", "family": "Kumar", "given": ["Rajesh"] }],
        "gender": "male",
        "birthDate": "1958-04-12"
    });
    let res_pat = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/Patient")
                .header("Content-Type", "application/json")
                .body(Body::from(patient_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(res_pat.status().is_success());

    // 3. POST /api/ingest/extract
    let acs_case = cases.iter().find(|c| c["id"] == "acs").unwrap();
    let extract_payload = serde_json::json!({
        "text": acs_case["text"].as_str().unwrap(),
        "patient_id": "pat-ingest-001"
    });
    let res_extract = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/ingest/extract")
                .header("Content-Type", "application/json")
                .body(Body::from(extract_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_extract.status(), StatusCode::OK);
    let body_ext = axum::body::to_bytes(res_extract.into_body(), usize::MAX)
        .await
        .unwrap();
    let report: serde_json::Value = serde_json::from_slice(&body_ext).unwrap();
    assert!(report["conditions"].as_array().unwrap().len() >= 3);
    assert!(report["allergies"].as_array().unwrap().len() >= 1);
    assert!(report["medications"].as_array().unwrap().len() >= 4);
    assert!(report["observations"].as_array().unwrap().len() >= 3);
    assert!(report["has_critical_alerts"].as_bool().unwrap());

    // 4. POST /api/ingest/commit
    let commit_payload = serde_json::json!({
        "patient_id": "pat-ingest-001",
        "source_text": acs_case["text"].as_str().unwrap(),
        "conditions": report["conditions"],
        "allergies": report["allergies"],
        "medications": report["medications"],
        "observations": report["observations"],
        "clinician_id": "dr.sharma@hospital.local"
    });
    let res_commit = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/ingest/commit")
                .header("Content-Type", "application/json")
                .body(Body::from(commit_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_commit.status(), StatusCode::OK);
    let body_com = axum::body::to_bytes(res_commit.into_body(), usize::MAX)
        .await
        .unwrap();
    let commit_res: serde_json::Value = serde_json::from_slice(&body_com).unwrap();
    assert_eq!(commit_res["status"], "committed");
    assert!(commit_res["created_resource_ids"].as_array().unwrap().len() >= 11);

    // 5. Verify Resources Stored for Patient
    let res_conds = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/Condition?patient=pat-ingest-001")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_conds.status(), StatusCode::OK);
    let body_c = axum::body::to_bytes(res_conds.into_body(), usize::MAX)
        .await
        .unwrap();
    let conds_bundle: serde_json::Value = serde_json::from_slice(&body_c).unwrap();
    assert!(conds_bundle["entry"].as_array().unwrap().len() >= 3);

    // 6. Test Historical Allergy Conflict Detection
    let amox_payload = serde_json::json!({
        "text": "Rx: Amoxicillin-Clavulanate 625mg PO TDS for severe dental infection.",
        "patient_id": "pat-ingest-001"
    });
    let res_amox = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/ingest/extract")
                .header("Content-Type", "application/json")
                .body(Body::from(amox_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_amox.status(), StatusCode::OK);
    let body_amox = axum::body::to_bytes(res_amox.into_body(), usize::MAX)
        .await
        .unwrap();
    let rep_amox: serde_json::Value = serde_json::from_slice(&body_amox).unwrap();
    assert!(rep_amox["has_critical_alerts"].as_bool().unwrap());
    let warnings = rep_amox["safety_warnings"].as_array().unwrap();
    assert!(warnings
        .iter()
        .any(|w| w["warning_type"] == "HistoricalAllergyConflict"));
}
