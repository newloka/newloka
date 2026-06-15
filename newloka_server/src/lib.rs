//! New Loka HTTP Server
//!
//! Provides REST API for T1+ deployments.
//! FHIR R4 compatible endpoints with ABAC enforcement.
//!
//! ## Environment Variables
//! - `NEWLOKA_NODE_ID` - Node identity (default: `server-node`)
//! - `NEWLOKA_DB_PATH` - SQLite connection string (default: in-memory)
//! - `NEWLOKA_MASTER_KEY` - Hex-encoded 32-byte device master key (**required** unless `demo` feature is enabled)

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Json, Redirect, Response},
    routing::{get, post},
    Router,
};
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;

mod demo;

#[derive(Clone)]
pub struct AppState {
    node_id: String,
    pub storage: std::sync::Arc<newloka_core::storage::StorageEngine>,
}

impl AppState {
    pub fn new(
        node_id: String,
        storage: std::sync::Arc<newloka_core::storage::StorageEngine>,
    ) -> Self {
        Self { node_id, storage }
    }
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
    patient: Option<String>,
    _count: Option<usize>,
}

/// Resolve the master key from environment or the `demo` feature.
fn resolve_master_key() -> anyhow::Result<newloka_core::crypto::DeviceMasterKey> {
    #[cfg(feature = "demo")]
    {
        tracing::warn!("DEMO FEATURE ENABLED: using hardcoded master key - NOT FOR PRODUCTION");
        Ok(newloka_core::crypto::DeviceMasterKey {
            key: *b"DEMOMASTERKEY1234567890123456789",
        })
    }

    #[cfg(not(feature = "demo"))]
    {
        let hex = std::env::var("NEWLOKA_MASTER_KEY")
            .map_err(|_| anyhow::anyhow!("NEWLOKA_MASTER_KEY environment variable is required in production builds (or enable the `demo` feature)"))?;
        let bytes = newloka_core::crypto::hex_decode(&hex)
            .map_err(|e| anyhow::anyhow!("invalid NEWLOKA_MASTER_KEY hex: {}", e))?;
        if bytes.len() != 32 {
            return Err(anyhow::anyhow!(
                "NEWLOKA_MASTER_KEY must be 64 hex characters (32 bytes), got {}",
                bytes.len()
            ));
        }
        let mut key = [0u8; 32];
        key.copy_from_slice(&bytes);
        Ok(newloka_core::crypto::DeviceMasterKey { key })
    }
}

/// Embedded static web assets (compiled into the binary).
#[derive(RustEmbed)]
#[folder = "../newloka_web"]
struct Assets;

fn mime_type(path: &str) -> &'static str {
    match path.rsplit('.').next() {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("png") => "image/png",
        Some("svg") => "image/svg+xml",
        _ => "application/octet-stream",
    }
}

async fn embedded_static_handler(
    req: axum::extract::Request,
) -> Result<Response<axum::body::Body>, std::convert::Infallible> {
    let path = req.uri().path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    match Assets::get(path) {
        Some(content) => {
            let ct = mime_type(path);
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header(axum::http::header::CONTENT_TYPE, ct)
                .body(axum::body::Body::from(content.data.into_owned()))
                .unwrap())
        }
        None => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(axum::body::Body::from("Not found"))
            .unwrap()),
    }
}

/// Build the Axum router for the given application state.
pub fn app(state: Arc<RwLock<AppState>>) -> Router {
    Router::new()
        .route("/", get(|| async { Redirect::temporary("/static/index.html") }))
        .route("/health", get(health_handler))
        .route("/metadata", get(capability_statement))
        .route("/AuditEvent", get(search_audit).post(create_audit))
        .route("/sync/delta", post(sync_delta))
        .route("/sync/manifest", get(sync_manifest))
        /* Patient */
        .route("/Patient", get(search_patients).post(create_patient))
        .route("/Patient/{id}", get(get_patient).put(update_patient).delete(delete_patient))
        /* Encounter */
        .route("/Encounter", get(search_encounters).post(create_encounter))
        .route("/Encounter/{id}", get(get_encounter).put(update_encounter).delete(delete_encounter))
        /* Observation */
        .route("/Observation", get(search_observations).post(create_observation))
        .route("/Observation/{id}", get(get_observation).put(update_observation).delete(delete_observation))
        /* Condition */
        .route("/Condition", get(search_conditions).post(create_condition))
        .route("/Condition/{id}", get(get_condition).put(update_condition).delete(delete_condition))
        /* MedicationRequest */
        .route("/MedicationRequest", get(search_medication_requests).post(create_medication_request))
        .route("/MedicationRequest/{id}", get(get_medication_request).put(update_medication_request).delete(delete_medication_request))
        /* Procedure */
        .route("/Procedure", get(search_procedures).post(create_procedure))
        .route("/Procedure/{id}", get(get_procedure).put(update_procedure).delete(delete_procedure))
        .nest_service("/static", tower::util::service_fn(embedded_static_handler))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

/// Start the New Loka HTTP server.
pub async fn run(bind: &str) -> anyhow::Result<()> {
    let node_id = std::env::var("NEWLOKA_NODE_ID").unwrap_or_else(|_| "server-node".to_string());
    let db_path = std::env::var("NEWLOKA_DB_PATH")
        .unwrap_or_else(|_| "sqlite::memory:?cache=shared".to_string());
    tracing::info!("Using database: {}", db_path);

    let dmk = resolve_master_key()?;
    let storage =
        newloka_core::storage::StorageEngine::open(&db_path, node_id.clone(), dmk).await?;
    if storage.is_empty().await.unwrap_or(false) {
        tracing::info!("Database empty -- seeding demo dataset");
        demo::seed_demo_data(&storage, &node_id).await?;
    }
    let storage = std::sync::Arc::new(storage);
    let state = Arc::new(RwLock::new(AppState::new(node_id, storage)));

    let app = app(state);
    let listener = tokio::net::TcpListener::bind(bind).await?;
    tracing::info!("New Loka server listening on {}", bind);
    axum::serve(listener, app).await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

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
            InteractionInfo {
                code: "read".to_string(),
            },
            InteractionInfo {
                code: "vread".to_string(),
            },
            InteractionInfo {
                code: "update".to_string(),
            },
            InteractionInfo {
                code: "delete".to_string(),
            },
            InteractionInfo {
                code: "search-type".to_string(),
            },
            InteractionInfo {
                code: "create".to_string(),
            },
        ],
    }
}

// ---------------------------------------------------------------------------
// Generic resource helpers
// ---------------------------------------------------------------------------

async fn search_resource(
    State(state): State<Arc<RwLock<AppState>>>,
    Query(q): Query<SearchParams>,
    resource_type: &str,
) -> (StatusCode, Json<serde_json::Value>) {
    let state = state.read().await;
    let patient_id = q.patient.as_deref();
    match state.storage.search_json(resource_type, patient_id).await {
        Ok(rows) => {
            let total = rows.len();
            let entry: Vec<serde_json::Value> = rows
                .into_iter()
                .map(|r| serde_json::json!({ "resource": r }))
                .collect();
            let results = serde_json::json!({
                "resourceType": "Bundle",
                "type": "searchset",
                "total": total,
                "entry": entry
            });
            (StatusCode::OK, Json(results))
        }
        Err(e) => {
            tracing::error!("Search {} failed: {}", resource_type, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("{}", e)})),
            )
        }
    }
}

async fn create_resource(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(mut body): Json<serde_json::Value>,
    resource_type: &str,
) -> (StatusCode, Json<serde_json::Value>) {
    let state = state.read().await;
    let id = body
        .get("id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    body["id"] = serde_json::json!(&id);
    let patient_id = body
        .get("subject")
        .and_then(|s| s.get("reference"))
        .and_then(|r| r.as_str())
        .map(|s| s.replace("Patient/", ""));
    match state
        .storage
        .store_json(resource_type, &id, &body, patient_id, None)
        .await
    {
        Ok(_) => (StatusCode::CREATED, Json(body)),
        Err(e) => {
            tracing::error!("Create {} failed: {}", resource_type, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("{}", e)})),
            )
        }
    }
}

async fn get_resource(
    State(state): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
    resource_type: &str,
) -> (StatusCode, Json<serde_json::Value>) {
    let state = state.read().await;
    match state.storage.get_json(&id).await {
        Ok(Some(json)) => {
            let actual_type = json
                .get("resourceType")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if actual_type == resource_type {
                (StatusCode::OK, Json(json))
            } else {
                (
                    StatusCode::NOT_FOUND,
                    Json(
                        serde_json::json!({"resourceType": "OperationOutcome", "issue": [{"severity": "error", "code": "not-found", "diagnostics": format!("Resource {} is not a {}", id, resource_type)}]}),
                    ),
                )
            }
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(
                serde_json::json!({"resourceType": "OperationOutcome", "issue": [{"severity": "error", "code": "not-found", "diagnostics": format!("{} not found", resource_type)}]}),
            ),
        ),
        Err(e) => {
            tracing::error!("Get {} {} failed: {}", resource_type, id, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("{}", e)})),
            )
        }
    }
}

async fn update_resource(
    State(state): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
    Json(mut body): Json<serde_json::Value>,
    resource_type: &str,
) -> (StatusCode, Json<serde_json::Value>) {
    let state = state.read().await;
    body["id"] = serde_json::json!(&id);
    let patient_id = body
        .get("subject")
        .and_then(|s| s.get("reference"))
        .and_then(|r| r.as_str())
        .map(|s| s.replace("Patient/", ""));
    match state
        .storage
        .store_json(resource_type, &id, &body, patient_id, None)
        .await
    {
        Ok(_) => (StatusCode::OK, Json(body)),
        Err(e) => {
            tracing::error!("Update {} {} failed: {}", resource_type, id, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("{}", e)})),
            )
        }
    }
}

async fn delete_resource(
    State(state): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
    resource_type: &str,
) -> StatusCode {
    let state = state.read().await;
    match state.storage.get_json(&id).await {
        Ok(Some(json)) => {
            let actual_type = json
                .get("resourceType")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if actual_type == resource_type {
                match state.storage.soft_delete(&id).await {
                    Ok(true) => StatusCode::NO_CONTENT,
                    Ok(false) => StatusCode::NOT_FOUND,
                    Err(e) => {
                        tracing::error!("Delete {} {} failed: {}", resource_type, id, e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    }
                }
            } else {
                StatusCode::NOT_FOUND
            }
        }
        Ok(None) => StatusCode::NOT_FOUND,
        Err(e) => {
            tracing::error!("Delete {} {} failed: {}", resource_type, id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

// ---------------------------------------------------------------------------
// Patient
// ---------------------------------------------------------------------------

async fn search_patients(
    State(s): State<Arc<RwLock<AppState>>>,
    Query(q): Query<SearchParams>,
) -> (StatusCode, Json<serde_json::Value>) {
    search_resource(State(s), Query(q), "Patient").await
}
async fn create_patient(
    State(s): State<Arc<RwLock<AppState>>>,
    Json(b): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    create_resource(State(s), Json(b), "Patient").await
}
async fn get_patient(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    get_resource(State(s), Path(id), "Patient").await
}
async fn update_patient(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
    Json(b): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    update_resource(State(s), Path(id), Json(b), "Patient").await
}
async fn delete_patient(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> StatusCode {
    delete_resource(State(s), Path(id), "Patient").await
}

// ---------------------------------------------------------------------------
// Encounter
// ---------------------------------------------------------------------------

async fn search_encounters(
    State(s): State<Arc<RwLock<AppState>>>,
    Query(q): Query<SearchParams>,
) -> (StatusCode, Json<serde_json::Value>) {
    search_resource(State(s), Query(q), "Encounter").await
}
async fn create_encounter(
    State(s): State<Arc<RwLock<AppState>>>,
    Json(b): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    create_resource(State(s), Json(b), "Encounter").await
}
async fn get_encounter(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    get_resource(State(s), Path(id), "Encounter").await
}
async fn update_encounter(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
    Json(b): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    update_resource(State(s), Path(id), Json(b), "Encounter").await
}
async fn delete_encounter(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> StatusCode {
    delete_resource(State(s), Path(id), "Encounter").await
}

// ---------------------------------------------------------------------------
// Observation
// ---------------------------------------------------------------------------

async fn search_observations(
    State(s): State<Arc<RwLock<AppState>>>,
    Query(q): Query<SearchParams>,
) -> (StatusCode, Json<serde_json::Value>) {
    search_resource(State(s), Query(q), "Observation").await
}
async fn create_observation(
    State(s): State<Arc<RwLock<AppState>>>,
    Json(b): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    create_resource(State(s), Json(b), "Observation").await
}
async fn get_observation(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    get_resource(State(s), Path(id), "Observation").await
}
async fn update_observation(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
    Json(b): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    update_resource(State(s), Path(id), Json(b), "Observation").await
}
async fn delete_observation(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> StatusCode {
    delete_resource(State(s), Path(id), "Observation").await
}

// ---------------------------------------------------------------------------
// Condition
// ---------------------------------------------------------------------------

async fn search_conditions(
    State(s): State<Arc<RwLock<AppState>>>,
    Query(q): Query<SearchParams>,
) -> (StatusCode, Json<serde_json::Value>) {
    search_resource(State(s), Query(q), "Condition").await
}
async fn create_condition(
    State(s): State<Arc<RwLock<AppState>>>,
    Json(b): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    create_resource(State(s), Json(b), "Condition").await
}
async fn get_condition(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    get_resource(State(s), Path(id), "Condition").await
}
async fn update_condition(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
    Json(b): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    update_resource(State(s), Path(id), Json(b), "Condition").await
}
async fn delete_condition(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> StatusCode {
    delete_resource(State(s), Path(id), "Condition").await
}

// ---------------------------------------------------------------------------
// MedicationRequest
// ---------------------------------------------------------------------------

async fn search_medication_requests(
    State(s): State<Arc<RwLock<AppState>>>,
    Query(q): Query<SearchParams>,
) -> (StatusCode, Json<serde_json::Value>) {
    search_resource(State(s), Query(q), "MedicationRequest").await
}
async fn create_medication_request(
    State(s): State<Arc<RwLock<AppState>>>,
    Json(b): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    create_resource(State(s), Json(b), "MedicationRequest").await
}
async fn get_medication_request(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    get_resource(State(s), Path(id), "MedicationRequest").await
}
async fn update_medication_request(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
    Json(b): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    update_resource(State(s), Path(id), Json(b), "MedicationRequest").await
}
async fn delete_medication_request(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> StatusCode {
    delete_resource(State(s), Path(id), "MedicationRequest").await
}

// ---------------------------------------------------------------------------
// Procedure
// ---------------------------------------------------------------------------

async fn search_procedures(
    State(s): State<Arc<RwLock<AppState>>>,
    Query(q): Query<SearchParams>,
) -> (StatusCode, Json<serde_json::Value>) {
    search_resource(State(s), Query(q), "Procedure").await
}
async fn create_procedure(
    State(s): State<Arc<RwLock<AppState>>>,
    Json(b): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    create_resource(State(s), Json(b), "Procedure").await
}
async fn get_procedure(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    get_resource(State(s), Path(id), "Procedure").await
}
async fn update_procedure(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
    Json(b): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    update_resource(State(s), Path(id), Json(b), "Procedure").await
}
async fn delete_procedure(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> StatusCode {
    delete_resource(State(s), Path(id), "Procedure").await
}

// ---------------------------------------------------------------------------
// Audit
// ---------------------------------------------------------------------------

async fn search_audit(
    State(state): State<Arc<RwLock<AppState>>>,
) -> (StatusCode, Json<serde_json::Value>) {
    let state = state.read().await;
    let rows = state.storage.search_audit(200).await.unwrap_or_default();
    let total = rows.len();
    let entry: Vec<serde_json::Value> = rows.into_iter().map(|r| {
        serde_json::json!({
            "resourceType": "AuditEvent",
            "id": r.id,
            "type": { "coding": [{ "code": r.event_type }] },
            "action": r.action,
            "recorded": chrono::DateTime::from_timestamp_millis(r.timestamp).map(|dt| dt.to_rfc3339()).unwrap_or_default(),
            "outcome": r.outcome,
            "agent": [{ "who": { "reference": r.actor_id, "display": r.actor_id } }],
        })
    }).collect();
    let results = serde_json::json!({
        "resourceType": "Bundle",
        "type": "searchset",
        "total": total,
        "entry": entry.into_iter().map(|r| serde_json::json!({ "resource": r })).collect::<Vec<_>>()
    });
    (StatusCode::OK, Json(results))
}

async fn create_audit(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(mut body): Json<serde_json::Value>,
) -> StatusCode {
    let state = state.read().await;
    let id = body
        .get("id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    body["id"] = serde_json::json!(&id);
    match state
        .storage
        .store_json("AuditEvent", &id, &body, None, None)
        .await
    {
        Ok(_) => StatusCode::CREATED,
        Err(e) => {
            tracing::error!("Failed to store audit event: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

// ---------------------------------------------------------------------------
// Sync stubs
// ---------------------------------------------------------------------------

async fn sync_manifest(State(state): State<Arc<RwLock<AppState>>>) -> Json<serde_json::Value> {
    let state = state.read().await;
    Json(serde_json::json!({
        "node_id": state.node_id,
        "tier": "T1",
        "last_sync": chrono::Utc::now().to_rfc3339(),
        "resource_types": ["Patient", "Encounter", "Observation"]
    }))
}

async fn sync_delta(Json(body): Json<serde_json::Value>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "from_node": body.get("from_node").and_then(|v| v.as_str()).unwrap_or("unknown"),
        "records": [],
        "conflicts": [],
        "timestamp": chrono::Utc::now().timestamp_millis()
    }))
}
