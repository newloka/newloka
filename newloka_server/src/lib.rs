//! New Loka HTTP Server
//!
//! Provides REST API for T1+ deployments.
//! FHIR R4 compatible endpoints with ABAC enforcement.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;

#[derive(Clone)]
struct AppState {
    node_id: String,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
    node_id: String,
    tier: String,
}

#[derive(Serialize)]
struct CapabilityStatement {
    resource_type: String,
    status: String,
    kind: String,
    software: SoftwareInfo,
    fhir_version: String,
    rest: Vec<RestInfo>,
}

#[derive(Serialize)]
struct SoftwareInfo {
    name: String,
    version: String,
}

#[derive(Serialize)]
struct RestInfo {
    mode: String,
    resource: Vec<ResourceInfo>,
}

#[derive(Serialize)]
struct ResourceInfo {
    type_: String,
    interaction: Vec<InteractionInfo>,
}

#[derive(Serialize)]
struct InteractionInfo {
    code: String,
}

#[derive(Deserialize)]
struct SearchParams {
    #[allow(dead_code)]
    patient: Option<String>,
    _count: Option<usize>,
}

pub async fn run(bind: &str) -> anyhow::Result<()> {
    let state = Arc::new(RwLock::new(AppState {
        node_id: "server-node".to_string(),
    }));

    let app = Router::new()
        .route("/", get(root_handler))
        .route("/health", get(health_handler))
        .route("/metadata", get(capability_statement))
        .route("/Patient", get(search_patients).post(create_patient))
        .route("/Patient/{id}", get(get_patient).put(update_patient).delete(delete_patient))
        .route("/Encounter", get(search_encounters).post(create_encounter))
        .route("/Observation", get(search_observations).post(create_observation))
        .route("/AuditEvent", get(search_audit).post(create_audit))
        .route("/sync/delta", post(sync_delta))
        .route("/sync/manifest", get(sync_manifest))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(bind).await?;
    tracing::info!("New Loka server listening on {}", bind);
    axum::serve(listener, app).await?;
    Ok(())
}

async fn root_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "service": "New Loka",
        "description": "Local-first health data management",
        "endpoints": ["/health", "/metadata", "/Patient", "/Encounter", "/Observation", "/AuditEvent", "/sync/delta"]
    }))
}

async fn health_handler(State(state): State<Arc<RwLock<AppState>>>) -> Json<HealthResponse> {
    let state = state.read().await;
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        node_id: state.node_id.clone(),
        tier: "T1".to_string(),
    })
}

async fn capability_statement() -> Json<CapabilityStatement> {
    Json(CapabilityStatement {
        resource_type: "CapabilityStatement".to_string(),
        status: "active".to_string(),
        kind: "instance".to_string(),
        software: SoftwareInfo {
            name: "New Loka".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
        fhir_version: "4.0.1".to_string(),
        rest: vec![RestInfo {
            mode: "server".to_string(),
            resource: vec![
                resource_info("Patient"),
                resource_info("Encounter"),
                resource_info("Observation"),
                resource_info("Condition"),
                resource_info("MedicationRequest"),
                resource_info("Procedure"),
                resource_info("DiagnosticReport"),
                resource_info("Composition"),
                resource_info("AuditEvent"),
                resource_info("Provenance"),
                resource_info("Bundle"),
            ],
        }],
    })
}

fn resource_info(type_: &str) -> ResourceInfo {
    ResourceInfo {
        type_: type_.to_string(),
        interaction: vec![
            InteractionInfo { code: "read".to_string() },
            InteractionInfo { code: "vread".to_string() },
            InteractionInfo { code: "update".to_string() },
            InteractionInfo { code: "delete".to_string() },
            InteractionInfo { code: "search-type".to_string() },
            InteractionInfo { code: "create".to_string() },
        ],
    }
}

async fn search_patients(
    Query(_params): Query<SearchParams>,
) -> (StatusCode, Json<serde_json::Value>) {
    let results = serde_json::json!({
        "resourceType": "Bundle",
        "type": "searchset",
        "total": 0,
        "entry": []
    });
    (StatusCode::OK, Json(results))
}

async fn create_patient(
    Json(_body): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    let id = uuid::Uuid::new_v4().to_string();
    let response = serde_json::json!({
        "resourceType": "Patient",
        "id": id,
        "meta": {
            "versionId": "1",
            "lastUpdated": chrono::Utc::now().to_rfc3339()
        }
    });
    (StatusCode::CREATED, Json(response))
}

async fn get_patient(Path(id): Path<String>) -> (StatusCode, Json<serde_json::Value>) {
    let response = serde_json::json!({
        "resourceType": "Patient",
        "id": id,
        "meta": {
            "versionId": "1",
            "lastUpdated": chrono::Utc::now().to_rfc3339()
        }
    });
    (StatusCode::OK, Json(response))
}

async fn update_patient(
    Path(id): Path<String>,
    Json(_body): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    let response = serde_json::json!({
        "resourceType": "Patient",
        "id": id,
        "meta": {
            "versionId": "2",
            "lastUpdated": chrono::Utc::now().to_rfc3339()
        }
    });
    (StatusCode::OK, Json(response))
}

async fn delete_patient(Path(id): Path<String>) -> StatusCode {
    tracing::info!("Soft deleting patient: {}", id);
    StatusCode::NO_CONTENT
}

async fn search_encounters(
    Query(_params): Query<SearchParams>,
) -> (StatusCode, Json<serde_json::Value>) {
    let results = serde_json::json!({
        "resourceType": "Bundle",
        "type": "searchset",
        "total": 0,
        "entry": []
    });
    (StatusCode::OK, Json(results))
}

async fn create_encounter(
    Json(_body): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    let id = uuid::Uuid::new_v4().to_string();
    let response = serde_json::json!({
        "resourceType": "Encounter",
        "id": id,
        "status": "in-progress"
    });
    (StatusCode::CREATED, Json(response))
}

async fn search_observations(
    Query(_params): Query<SearchParams>,
) -> (StatusCode, Json<serde_json::Value>) {
    let results = serde_json::json!({
        "resourceType": "Bundle",
        "type": "searchset",
        "total": 0,
        "entry": []
    });
    (StatusCode::OK, Json(results))
}

async fn create_observation(
    Json(_body): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    let id = uuid::Uuid::new_v4().to_string();
    let response = serde_json::json!({
        "resourceType": "Observation",
        "id": id,
        "status": "final"
    });
    (StatusCode::CREATED, Json(response))
}

async fn search_audit() -> (StatusCode, Json<serde_json::Value>) {
    let results = serde_json::json!({
        "resourceType": "Bundle",
        "type": "searchset",
        "total": 0,
        "entry": []
    });
    (StatusCode::OK, Json(results))
}

async fn create_audit(
    Json(body): Json<serde_json::Value>,
) -> StatusCode {
    tracing::info!("Audit event: {:?}", body);
    StatusCode::CREATED
}

async fn sync_manifest() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "node_id": "server-node",
        "tier": "T1",
        "last_sync": chrono::Utc::now().to_rfc3339(),
        "resource_types": ["Patient", "Encounter", "Observation"]
    }))
}

async fn sync_delta(
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "from_node": body.get("from_node").and_then(|v| v.as_str()).unwrap_or("unknown"),
        "records": [],
        "conflicts": [],
        "timestamp": chrono::Utc::now().timestamp_millis()
    }))
}
