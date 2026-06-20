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

pub mod demo;

pub mod state;
pub use crate::state::{AppState, NodeConfig};

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
    name: Option<String>,
    _count: Option<usize>,
}

#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
    tier: Option<String>,
}

#[derive(Serialize)]
pub struct LoginResponse {
    token: String,
    user_id: String,
    display_name: String,
    node_id: String,
    tier: String,
    expires_in: i64,
    roles: Vec<String>,
    department_id: Option<String>,
    team_ids: Vec<String>,
    lab_affiliations: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub roles: Vec<String>,
    pub department_id: Option<String>,
    pub team_ids: Vec<String>,
    pub lab_affiliations: Vec<String>,
    pub active: bool,
    pub created_at: String,
    pub last_login: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub display_name: String,
    pub password: String,
    pub roles: Vec<String>,
    pub department_id: Option<String>,
    pub team_ids: Vec<String>,
    pub lab_affiliations: Vec<String>,
    pub active: Option<bool>,
}

#[derive(Deserialize)]
pub struct EvaluateAbacRequest {
    pub subject_roles: Vec<String>,
    pub resource_type: String,
    pub action: String,
    pub patient_has_lab_order: Option<bool>,
}

#[derive(Serialize)]
struct ConfigResponse {
    config: NodeConfig,
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
            .status(StatusCode::OK)
            .header(axum::http::header::CONTENT_TYPE, mime_type("index.html"))
            .body(axum::body::Body::from(
                Assets::get("index.html")
                    .map(|c| c.data.into_owned())
                    .unwrap_or_default(),
            ))
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
        .route("/config", get(get_config).put(put_config))
        .route("/sync/delta", post(sync_delta))
        .route("/sync/manifest", get(sync_manifest))
        /* Patient */
        .route("/Patient", get(search_patients).post(create_patient))
        .route("/Patient/:id", get(get_patient).put(update_patient).delete(delete_patient))
        /* Encounter */
        .route("/Encounter", get(search_encounters).post(create_encounter))
        .route("/Encounter/:id", get(get_encounter).put(update_encounter).delete(delete_encounter))
        /* Observation */
        .route("/Observation", get(search_observations).post(create_observation))
        .route("/Observation/:id", get(get_observation).put(update_observation).delete(delete_observation))
        /* Condition */
        .route("/Condition", get(search_conditions).post(create_condition))
        .route("/Condition/:id", get(get_condition).put(update_condition).delete(delete_condition))
        /* MedicationRequest */
        .route("/MedicationRequest", get(search_medication_requests).post(create_medication_request))
        .route("/MedicationRequest/:id", get(get_medication_request).put(update_medication_request).delete(delete_medication_request))
        /* Procedure */
        .route("/Procedure", get(search_procedures).post(create_procedure))
        .route("/Procedure/:id", get(get_procedure).put(update_procedure).delete(delete_procedure))
        /* AllergyIntolerance */
        .route("/AllergyIntolerance", get(search_allergy_intolerances).post(create_allergy_intolerance))
        .route("/AllergyIntolerance/:id", get(get_allergy_intolerance).put(update_allergy_intolerance).delete(delete_allergy_intolerance))
        /* Flag */
        .route("/Flag", get(search_flags).post(create_flag))
        .route("/Flag/:id", get(get_flag).put(update_flag).delete(delete_flag))
        /* DetectedIssue */
        .route("/DetectedIssue", get(search_detected_issues).post(create_detected_issue))
        .route("/DetectedIssue/:id", get(get_detected_issue).put(update_detected_issue).delete(delete_detected_issue))
        /* Composition */
        .route("/Composition", get(search_compositions).post(create_composition))
        .route("/Composition/:id", get(get_composition).put(update_composition).delete(delete_composition))
        /* CarePlan */
        .route("/CarePlan", get(search_care_plans).post(create_care_plan))
        .route("/CarePlan/:id", get(get_care_plan).put(update_care_plan).delete(delete_care_plan))
        /* FamilyMemberHistory */
        .route("/FamilyMemberHistory", get(search_family_member_histories).post(create_family_member_history))
        .route("/FamilyMemberHistory/:id", get(get_family_member_history).put(update_family_member_history).delete(delete_family_member_history))
        /* Immunization */
        .route("/Immunization", get(search_immunizations).post(create_immunization))
        .route("/Immunization/:id", get(get_immunization).put(update_immunization).delete(delete_immunization))
        /* DocumentReference */
        .route("/DocumentReference", get(search_document_references).post(create_document_reference))
        .route("/DocumentReference/:id", get(get_document_reference).put(update_document_reference).delete(delete_document_reference))
        /* CPOE: ServiceRequest */
        .route("/ServiceRequest", get(search_service_requests).post(create_service_request))
        .route("/ServiceRequest/:id", get(get_service_request).put(update_service_request).delete(delete_service_request))
        /* CPOE: ImagingStudy */
        .route("/ImagingStudy", get(search_imaging_studies).post(create_imaging_study))
        .route("/ImagingStudy/:id", get(get_imaging_study).put(update_imaging_study).delete(delete_imaging_study))
        /* CPOE: Specimen */
        .route("/Specimen", get(search_specimens).post(create_specimen))
        .route("/Specimen/:id", get(get_specimen).put(update_specimen).delete(delete_specimen))
        /* MedicationAdministration */
        .route("/MedicationAdministration", get(search_medication_administrations).post(create_medication_administration))
        .route("/MedicationAdministration/:id", get(get_medication_administration).put(update_medication_administration).delete(delete_medication_administration))
        /* MedicationStatement */
        .route("/MedicationStatement", get(search_medication_statements).post(create_medication_statement))
        .route("/MedicationStatement/:id", get(get_medication_statement).put(update_medication_statement).delete(delete_medication_statement))
        /* Whiteboard */
        .route("/Whiteboard", get(search_whiteboards).post(create_whiteboard))
        .route("/Whiteboard/:id", get(get_whiteboard).put(update_whiteboard).delete(delete_whiteboard))
        /* Provenance */
        .route("/Provenance", get(search_provenances).post(create_provenance))
        .route("/Provenance/:id", get(get_provenance).put(update_provenance).delete(delete_provenance))
        /* Users */
        .route("/users", get(list_users).post(create_user))
        .route("/users/:id", get(get_user).put(update_user).delete(delete_user))
        /* ABAC */
        .route("/abac/evaluate", post(evaluate_abac))
        .route("/abac/policies", get(list_abac_policies))
        /* Auth */
        .route("/auth/login", post(login_handler))
        .route("/auth/session", get(session_handler))
        .route("/auth/logout", post(logout_handler))
        .nest_service("/static", tower::util::service_fn(embedded_static_handler))
        .fallback(|| async { axum::response::Redirect::temporary("/static/index.html") })
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
    seed_demo_users(&storage).await?;
    let storage = std::sync::Arc::new(storage);
    let config = NodeConfig {
        tier: std::env::var("NEWLOKA_TIER").unwrap_or_else(|_| "T1".to_string()),
        node_id: node_id.clone(),
        department: std::env::var("NEWLOKA_DEPARTMENT").unwrap_or_else(|_| "default".to_string()),
        sync_enabled: std::env::var("NEWLOKA_SYNC_ENABLED").unwrap_or_else(|_| "true".to_string())
            == "true",
        mesh_enabled: std::env::var("NEWLOKA_MESH_ENABLED").unwrap_or_else(|_| "false".to_string())
            == "true",
        offline_auth: std::env::var("NEWLOKA_OFFLINE_AUTH").unwrap_or_else(|_| "pin".to_string()),
        language: std::env::var("NEWLOKA_LANGUAGE").unwrap_or_else(|_| "en".to_string()),
        emergency_access: std::env::var("NEWLOKA_EMERGENCY_ACCESS")
            .unwrap_or_else(|_| "false".to_string())
            == "true",
        page_size: std::env::var("NEWLOKA_PAGE_SIZE")
            .unwrap_or_else(|_| "20".to_string())
            .parse()
            .unwrap_or(20),
        default_encounter_status: std::env::var("NEWLOKA_DEFAULT_ENCOUNTER_STATUS")
            .unwrap_or_else(|_| "in-progress".to_string()),
        offline_queue_auto_flush: std::env::var("NEWLOKA_OFFLINE_QUEUE_AUTO_FLUSH")
            .unwrap_or_else(|_| "true".to_string())
            == "true",
    };
    let state = Arc::new(RwLock::new(AppState::new(node_id, storage, config)));

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
        tier: state.config.tier.clone(),
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
                resource_info("AllergyIntolerance"),
                resource_info("Flag"),
                resource_info("DetectedIssue"),
                resource_info("CarePlan"),
                resource_info("FamilyMemberHistory"),
                resource_info("Immunization"),
                resource_info("DocumentReference"),
                resource_info("ServiceRequest"),
                resource_info("ImagingStudy"),
                resource_info("Specimen"),
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
        Ok(mut rows) => {
            if resource_type == "Patient" {
                if let Some(ref name_q) = q.name {
                    let needle = name_q.to_lowercase();
                    rows.retain(|r| {
                        if let Some(names) = r.get("name").and_then(|v| v.as_array()) {
                            for name in names {
                                let family = name
                                    .get("family")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_lowercase();
                                let given = name
                                    .get("given")
                                    .and_then(|v| v.as_array())
                                    .map(|arr| {
                                        arr.iter()
                                            .filter_map(|g| g.as_str())
                                            .collect::<Vec<_>>()
                                            .join(" ")
                                            .to_lowercase()
                                    })
                                    .unwrap_or_default();
                                let full = format!("{} {}", given, family).to_lowercase();
                                if full.contains(&needle)
                                    || family.contains(&needle)
                                    || given.contains(&needle)
                                {
                                    return true;
                                }
                            }
                        }
                        false
                    });
                }
            }
            let count = q._count.unwrap_or(state.config.page_size);
            let total = rows.len();
            let entry: Vec<serde_json::Value> = rows
                .into_iter()
                .take(count)
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
        .map(|s| s.replace("Patient/", ""))
        .or_else(|| {
            body.get("patient")
                .and_then(|s| s.get("reference"))
                .and_then(|r| r.as_str())
                .map(|s| s.replace("Patient/", ""))
        });
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
        .map(|s| s.replace("Patient/", ""))
        .or_else(|| {
            body.get("patient")
                .and_then(|s| s.get("reference"))
                .and_then(|r| r.as_str())
                .map(|s| s.replace("Patient/", ""))
        });
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
// AllergyIntolerance
// ---------------------------------------------------------------------------

async fn search_allergy_intolerances(
    State(s): State<Arc<RwLock<AppState>>>,
    Query(q): Query<SearchParams>,
) -> (StatusCode, Json<serde_json::Value>) {
    search_resource(State(s), Query(q), "AllergyIntolerance").await
}
async fn create_allergy_intolerance(
    State(s): State<Arc<RwLock<AppState>>>,
    Json(b): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    create_resource(State(s), Json(b), "AllergyIntolerance").await
}
async fn get_allergy_intolerance(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    get_resource(State(s), Path(id), "AllergyIntolerance").await
}
async fn update_allergy_intolerance(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
    Json(b): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    update_resource(State(s), Path(id), Json(b), "AllergyIntolerance").await
}
async fn delete_allergy_intolerance(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> StatusCode {
    delete_resource(State(s), Path(id), "AllergyIntolerance").await
}

// ---------------------------------------------------------------------------
// Flag
// ---------------------------------------------------------------------------

async fn search_flags(
    State(s): State<Arc<RwLock<AppState>>>,
    Query(q): Query<SearchParams>,
) -> (StatusCode, Json<serde_json::Value>) {
    search_resource(State(s), Query(q), "Flag").await
}
async fn create_flag(
    State(s): State<Arc<RwLock<AppState>>>,
    Json(b): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    create_resource(State(s), Json(b), "Flag").await
}
async fn get_flag(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    get_resource(State(s), Path(id), "Flag").await
}
async fn update_flag(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
    Json(b): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    update_resource(State(s), Path(id), Json(b), "Flag").await
}
async fn delete_flag(State(s): State<Arc<RwLock<AppState>>>, Path(id): Path<String>) -> StatusCode {
    delete_resource(State(s), Path(id), "Flag").await
}

// ---------------------------------------------------------------------------
// DetectedIssue
// ---------------------------------------------------------------------------

async fn search_detected_issues(
    State(s): State<Arc<RwLock<AppState>>>,
    Query(q): Query<SearchParams>,
) -> (StatusCode, Json<serde_json::Value>) {
    search_resource(State(s), Query(q), "DetectedIssue").await
}
async fn create_detected_issue(
    State(s): State<Arc<RwLock<AppState>>>,
    Json(b): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    create_resource(State(s), Json(b), "DetectedIssue").await
}
async fn get_detected_issue(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    get_resource(State(s), Path(id), "DetectedIssue").await
}
async fn update_detected_issue(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
    Json(b): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    update_resource(State(s), Path(id), Json(b), "DetectedIssue").await
}
async fn delete_detected_issue(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> StatusCode {
    delete_resource(State(s), Path(id), "DetectedIssue").await
}

// ---------------------------------------------------------------------------
// Composition
// ---------------------------------------------------------------------------

async fn search_compositions(
    State(s): State<Arc<RwLock<AppState>>>,
    Query(q): Query<SearchParams>,
) -> (StatusCode, Json<serde_json::Value>) {
    search_resource(State(s), Query(q), "Composition").await
}
async fn create_composition(
    State(s): State<Arc<RwLock<AppState>>>,
    Json(b): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    create_resource(State(s), Json(b), "Composition").await
}
async fn get_composition(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    get_resource(State(s), Path(id), "Composition").await
}
async fn update_composition(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
    Json(b): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    update_resource(State(s), Path(id), Json(b), "Composition").await
}
async fn delete_composition(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> StatusCode {
    delete_resource(State(s), Path(id), "Composition").await
}

// ---------------------------------------------------------------------------
// CarePlan
// ---------------------------------------------------------------------------

async fn search_care_plans(
    State(s): State<Arc<RwLock<AppState>>>,
    Query(q): Query<SearchParams>,
) -> (StatusCode, Json<serde_json::Value>) {
    search_resource(State(s), Query(q), "CarePlan").await
}
async fn create_care_plan(
    State(s): State<Arc<RwLock<AppState>>>,
    Json(b): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    create_resource(State(s), Json(b), "CarePlan").await
}
async fn get_care_plan(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    get_resource(State(s), Path(id), "CarePlan").await
}
async fn update_care_plan(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
    Json(b): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    update_resource(State(s), Path(id), Json(b), "CarePlan").await
}
async fn delete_care_plan(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> StatusCode {
    delete_resource(State(s), Path(id), "CarePlan").await
}

// ---------------------------------------------------------------------------
// FamilyMemberHistory
// ---------------------------------------------------------------------------

async fn search_family_member_histories(
    State(s): State<Arc<RwLock<AppState>>>,
    Query(q): Query<SearchParams>,
) -> (StatusCode, Json<serde_json::Value>) {
    search_resource(State(s), Query(q), "FamilyMemberHistory").await
}
async fn create_family_member_history(
    State(s): State<Arc<RwLock<AppState>>>,
    Json(b): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    create_resource(State(s), Json(b), "FamilyMemberHistory").await
}
async fn get_family_member_history(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    get_resource(State(s), Path(id), "FamilyMemberHistory").await
}
async fn update_family_member_history(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
    Json(b): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    update_resource(State(s), Path(id), Json(b), "FamilyMemberHistory").await
}
async fn delete_family_member_history(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> StatusCode {
    delete_resource(State(s), Path(id), "FamilyMemberHistory").await
}

// ---------------------------------------------------------------------------
// Immunization
// ---------------------------------------------------------------------------

async fn search_immunizations(
    State(s): State<Arc<RwLock<AppState>>>,
    Query(q): Query<SearchParams>,
) -> (StatusCode, Json<serde_json::Value>) {
    search_resource(State(s), Query(q), "Immunization").await
}
async fn create_immunization(
    State(s): State<Arc<RwLock<AppState>>>,
    Json(b): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    create_resource(State(s), Json(b), "Immunization").await
}
async fn get_immunization(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    get_resource(State(s), Path(id), "Immunization").await
}
async fn update_immunization(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
    Json(b): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    update_resource(State(s), Path(id), Json(b), "Immunization").await
}
async fn delete_immunization(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> StatusCode {
    delete_resource(State(s), Path(id), "Immunization").await
}

// ---------------------------------------------------------------------------
// DocumentReference
// ---------------------------------------------------------------------------

async fn search_document_references(
    State(s): State<Arc<RwLock<AppState>>>,
    Query(q): Query<SearchParams>,
) -> (StatusCode, Json<serde_json::Value>) {
    search_resource(State(s), Query(q), "DocumentReference").await
}
async fn create_document_reference(
    State(s): State<Arc<RwLock<AppState>>>,
    Json(b): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    create_resource(State(s), Json(b), "DocumentReference").await
}
async fn get_document_reference(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    get_resource(State(s), Path(id), "DocumentReference").await
}
async fn update_document_reference(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
    Json(b): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    update_resource(State(s), Path(id), Json(b), "DocumentReference").await
}
async fn delete_document_reference(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> StatusCode {
    delete_resource(State(s), Path(id), "DocumentReference").await
}

// ---------------------------------------------------------------------------
// CPOE: ServiceRequest
// ---------------------------------------------------------------------------

async fn search_service_requests(
    State(s): State<Arc<RwLock<AppState>>>,
    Query(q): Query<SearchParams>,
) -> (StatusCode, Json<serde_json::Value>) {
    search_resource(State(s), Query(q), "ServiceRequest").await
}
async fn create_service_request(
    State(s): State<Arc<RwLock<AppState>>>,
    Json(b): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    create_resource(State(s), Json(b), "ServiceRequest").await
}
async fn get_service_request(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    get_resource(State(s), Path(id), "ServiceRequest").await
}
async fn update_service_request(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
    Json(b): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    update_resource(State(s), Path(id), Json(b), "ServiceRequest").await
}
async fn delete_service_request(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> StatusCode {
    delete_resource(State(s), Path(id), "ServiceRequest").await
}

// ---------------------------------------------------------------------------
// MedicationAdministration
// ---------------------------------------------------------------------------

async fn search_medication_administrations(
    State(s): State<Arc<RwLock<AppState>>>,
    Query(q): Query<SearchParams>,
) -> (StatusCode, Json<serde_json::Value>) {
    search_resource(State(s), Query(q), "MedicationAdministration").await
}
async fn create_medication_administration(
    State(s): State<Arc<RwLock<AppState>>>,
    Json(b): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    create_resource(State(s), Json(b), "MedicationAdministration").await
}
async fn get_medication_administration(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    get_resource(State(s), Path(id), "MedicationAdministration").await
}
async fn update_medication_administration(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
    Json(b): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    update_resource(State(s), Path(id), Json(b), "MedicationAdministration").await
}
async fn delete_medication_administration(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> StatusCode {
    delete_resource(State(s), Path(id), "MedicationAdministration").await
}

// ---------------------------------------------------------------------------
// MedicationStatement
// ---------------------------------------------------------------------------

async fn search_medication_statements(
    State(s): State<Arc<RwLock<AppState>>>,
    Query(q): Query<SearchParams>,
) -> (StatusCode, Json<serde_json::Value>) {
    search_resource(State(s), Query(q), "MedicationStatement").await
}
async fn create_medication_statement(
    State(s): State<Arc<RwLock<AppState>>>,
    Json(b): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    create_resource(State(s), Json(b), "MedicationStatement").await
}
async fn get_medication_statement(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    get_resource(State(s), Path(id), "MedicationStatement").await
}
async fn update_medication_statement(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
    Json(b): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    update_resource(State(s), Path(id), Json(b), "MedicationStatement").await
}
async fn delete_medication_statement(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> StatusCode {
    delete_resource(State(s), Path(id), "MedicationStatement").await
}

// ---------------------------------------------------------------------------
// CPOE: ImagingStudy
// ---------------------------------------------------------------------------

async fn search_imaging_studies(
    State(s): State<Arc<RwLock<AppState>>>,
    Query(q): Query<SearchParams>,
) -> (StatusCode, Json<serde_json::Value>) {
    search_resource(State(s), Query(q), "ImagingStudy").await
}
async fn create_imaging_study(
    State(s): State<Arc<RwLock<AppState>>>,
    Json(b): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    create_resource(State(s), Json(b), "ImagingStudy").await
}
async fn get_imaging_study(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    get_resource(State(s), Path(id), "ImagingStudy").await
}
async fn update_imaging_study(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
    Json(b): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    update_resource(State(s), Path(id), Json(b), "ImagingStudy").await
}
async fn delete_imaging_study(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> StatusCode {
    delete_resource(State(s), Path(id), "ImagingStudy").await
}

// ---------------------------------------------------------------------------
// CPOE: Specimen
// ---------------------------------------------------------------------------

async fn search_specimens(
    State(s): State<Arc<RwLock<AppState>>>,
    Query(q): Query<SearchParams>,
) -> (StatusCode, Json<serde_json::Value>) {
    search_resource(State(s), Query(q), "Specimen").await
}
async fn create_specimen(
    State(s): State<Arc<RwLock<AppState>>>,
    Json(b): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    create_resource(State(s), Json(b), "Specimen").await
}
async fn get_specimen(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    get_resource(State(s), Path(id), "Specimen").await
}
async fn update_specimen(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
    Json(b): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    update_resource(State(s), Path(id), Json(b), "Specimen").await
}
async fn delete_specimen(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> StatusCode {
    delete_resource(State(s), Path(id), "Specimen").await
}
// ---------------------------------------------------------------------------
// Whiteboard
// ---------------------------------------------------------------------------

async fn search_whiteboards(
    State(s): State<Arc<RwLock<AppState>>>,
    Query(q): Query<SearchParams>,
) -> (StatusCode, Json<serde_json::Value>) {
    search_resource(State(s), Query(q), "Whiteboard").await
}
async fn create_whiteboard(
    State(s): State<Arc<RwLock<AppState>>>,
    Json(b): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    create_resource(State(s), Json(b), "Whiteboard").await
}
async fn get_whiteboard(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    get_resource(State(s), Path(id), "Whiteboard").await
}
async fn update_whiteboard(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
    Json(b): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    update_resource(State(s), Path(id), Json(b), "Whiteboard").await
}
async fn delete_whiteboard(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> StatusCode {
    delete_resource(State(s), Path(id), "Whiteboard").await
}
// ---------------------------------------------------------------------------
// Provenance
// ---------------------------------------------------------------------------

async fn search_provenances(
    State(s): State<Arc<RwLock<AppState>>>,
    Query(q): Query<SearchParams>,
) -> (StatusCode, Json<serde_json::Value>) {
    search_resource(State(s), Query(q), "Provenance").await
}
async fn create_provenance(
    State(s): State<Arc<RwLock<AppState>>>,
    Json(b): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    create_resource(State(s), Json(b), "Provenance").await
}
async fn get_provenance(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    get_resource(State(s), Path(id), "Provenance").await
}
async fn update_provenance(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
    Json(b): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    update_resource(State(s), Path(id), Json(b), "Provenance").await
}
async fn delete_provenance(
    State(s): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> StatusCode {
    delete_resource(State(s), Path(id), "Provenance").await
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
            "patient_id": r.patient_id,
            "resource_id": r.resource_id,
            "resource_type": r.resource_type,
            "details": r.details,
            "previous_hash": r.previous_hash,
            "entry_hash": r.entry_hash,
            "node_id": r.node_id,
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
        "tier": state.config.tier,
        "department": state.config.department,
        "sync_enabled": state.config.sync_enabled,
        "mesh_enabled": state.config.mesh_enabled,
        "last_sync": chrono::Utc::now().to_rfc3339(),
        "resource_types": ["Patient", "Encounter", "Observation", "Condition", "MedicationRequest", "Procedure", "DiagnosticReport", "AllergyIntolerance", "Flag", "DetectedIssue", "CarePlan", "FamilyMemberHistory", "Immunization", "DocumentReference", "ServiceRequest", "ImagingStudy", "Specimen", "Composition", "Whiteboard", "MedicationAdministration", "MedicationStatement", "Provenance"]
    }))
}

async fn login_handler(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(body): Json<LoginRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    if body.password.is_empty() {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Invalid credentials"})),
        );
    }
    let read = state.read().await;
    let user = match read.storage.get_user_by_username(&body.username).await {
        Ok(Some(u)) => u,
        _ => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Invalid credentials"})),
            );
        }
    };
    if !user.active {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Account inactive"})),
        );
    }
    let valid = newloka_core::identity::Authenticator::verify_password(
        &body.password,
        &user.salt,
        &user.password_hash,
    )
    .unwrap_or(false);
    if !valid {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Invalid credentials"})),
        );
    }
    let tier_str = body.tier.unwrap_or_else(|| read.config.tier.clone());
    let tier = parse_tier(&tier_str);
    let session = newloka_core::identity::Session::new(user.id.clone(), read.node_id.clone(), tier);
    let token = session.token.clone();
    let node_id = read.node_id.clone();
    let roles: Vec<String> = user.roles.iter().map(|r| r.as_str().to_string()).collect();
    let lab_affiliations: Vec<String> = user
        .lab_affiliations
        .iter()
        .map(|l| l.as_str().to_string())
        .collect();
    drop(read);
    let write = state.write().await;
    write.sessions.write().await.insert(token.clone(), session);
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "token": token,
            "user_id": user.id,
            "display_name": user.display_name,
            "node_id": node_id,
            "tier": tier_str,
            "expires_in": 43200,
            "roles": roles,
            "department_id": user.department_id,
            "team_ids": user.team_ids,
            "lab_affiliations": lab_affiliations,
        })),
    )
}

async fn logout_handler(
    State(state): State<Arc<RwLock<AppState>>>,
    headers: axum::http::HeaderMap,
) -> StatusCode {
    let token = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));
    if let Some(token) = token {
        let state = state.write().await;
        state.sessions.write().await.remove(token);
    }
    StatusCode::NO_CONTENT
}

async fn session_handler(
    State(state): State<Arc<RwLock<AppState>>>,
    headers: axum::http::HeaderMap,
) -> Json<serde_json::Value> {
    let token = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));
    let state = state.read().await;
    let session = if let Some(token) = token {
        state.sessions.read().await.get(token).cloned()
    } else {
        None
    };
    if let Some(s) = session {
        Json(serde_json::json!({
            "node_id": state.node_id,
            "tier": state.config.tier,
            "department": state.config.department,
            "sync_enabled": state.config.sync_enabled,
            "mesh_enabled": state.config.mesh_enabled,
            "emergency_access": state.config.emergency_access,
            "user_id": s.user_id,
            "expires_in": s.remaining_seconds(),
        }))
    } else {
        Json(serde_json::json!({
            "node_id": state.node_id,
            "tier": state.config.tier,
            "department": state.config.department,
            "sync_enabled": state.config.sync_enabled,
            "mesh_enabled": state.config.mesh_enabled,
            "emergency_access": state.config.emergency_access,
        }))
    }
}

async fn get_config(State(state): State<Arc<RwLock<AppState>>>) -> Json<ConfigResponse> {
    let state = state.read().await;
    Json(ConfigResponse {
        config: state.config.clone(),
    })
}

async fn put_config(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(body): Json<NodeConfig>,
) -> Json<ConfigResponse> {
    let mut state = state.write().await;
    let mut cfg = body;
    cfg.node_id = state.node_id.clone();
    state.config = cfg.clone();
    Json(ConfigResponse { config: cfg })
}

async fn sync_delta(Json(body): Json<serde_json::Value>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "from_node": body.get("from_node").and_then(|v| v.as_str()).unwrap_or("unknown"),
        "records": [],
        "conflicts": [],
        "timestamp": chrono::Utc::now().timestamp_millis()
    }))
}

// ---------------------------------------------------------------------------
// User management CRUD
// ---------------------------------------------------------------------------

fn user_to_response(u: &newloka_core::identity::User) -> UserResponse {
    UserResponse {
        id: u.id.clone(),
        username: u.username.clone(),
        display_name: u.display_name.clone(),
        roles: u.roles.iter().map(|r| r.as_str().to_string()).collect(),
        department_id: u.department_id.clone(),
        team_ids: u.team_ids.clone(),
        lab_affiliations: u
            .lab_affiliations
            .iter()
            .map(|l| l.as_str().to_string())
            .collect(),
        active: u.active,
        created_at: u.created_at.to_rfc3339(),
        last_login: u.last_login.map(|d| d.to_rfc3339()),
    }
}

async fn list_users(
    State(state): State<Arc<RwLock<AppState>>>,
) -> (StatusCode, Json<Vec<UserResponse>>) {
    let state = state.read().await;
    match state.storage.list_users().await {
        Ok(users) => {
            let resp: Vec<UserResponse> = users.iter().map(user_to_response).collect();
            (StatusCode::OK, Json(resp))
        }
        Err(e) => {
            tracing::error!("Failed to list users: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![]))
        }
    }
}

async fn get_user(
    State(state): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    let state = state.read().await;
    match state.storage.get_user_by_id(&id).await {
        Ok(Some(u)) => {
            let resp = user_to_response(&u);
            (StatusCode::OK, Json(serde_json::json!(resp)))
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "User not found"})),
        ),
        Err(e) => {
            tracing::error!("Failed to get user: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Internal error"})),
            )
        }
    }
}

async fn create_user(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(body): Json<CreateUserRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let state = state.write().await;
    let existing = state
        .storage
        .get_user_by_username(&body.username)
        .await
        .ok()
        .flatten();
    if existing.is_some() {
        return (
            StatusCode::CONFLICT,
            Json(serde_json::json!({"error": "Username already exists"})),
        );
    }
    let (salt, hash) = match newloka_core::identity::Authenticator::hash_password(&body.password) {
        Ok(h) => h,
        Err(e) => {
            tracing::error!("Password hashing failed: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Internal error"})),
            );
        }
    };
    let roles: Vec<newloka_core::identity::Role> =
        body.roles.iter().filter_map(|r| r.parse().ok()).collect();
    let lab_affiliations: Vec<newloka_core::cpoe::LabDepartment> = body
        .lab_affiliations
        .iter()
        .filter_map(|l| l.parse().ok())
        .collect();
    let user = newloka_core::identity::User {
        id: uuid::Uuid::new_v4().to_string(),
        username: body.username,
        display_name: body.display_name,
        roles,
        department_id: body.department_id,
        team_ids: body.team_ids,
        lab_affiliations,
        active: body.active.unwrap_or(true),
        created_at: chrono::Utc::now(),
        password_hash: hash,
        salt,
        totp_secret: None,
        last_login: None,
    };
    if let Err(e) = state.storage.store_user(&user).await {
        tracing::error!("Failed to store user: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Internal error"})),
        );
    }
    (
        StatusCode::CREATED,
        Json(serde_json::json!(user_to_response(&user))),
    )
}

async fn update_user(
    State(state): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
    Json(body): Json<CreateUserRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let state = state.write().await;
    let mut user = match state.storage.get_user_by_id(&id).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "User not found"})),
            )
        }
        Err(e) => {
            tracing::error!("Failed to get user: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Internal error"})),
            );
        }
    };
    if !body.password.is_empty() {
        let (salt, hash) =
            match newloka_core::identity::Authenticator::hash_password(&body.password) {
                Ok(h) => h,
                Err(e) => {
                    tracing::error!("Password hashing failed: {}", e);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({"error": "Internal error"})),
                    );
                }
            };
        user.salt = salt;
        user.password_hash = hash;
    }
    user.display_name = body.display_name;
    user.roles = body.roles.iter().filter_map(|r| r.parse().ok()).collect();
    user.department_id = body.department_id;
    user.team_ids = body.team_ids;
    user.lab_affiliations = body
        .lab_affiliations
        .iter()
        .filter_map(|l| l.parse().ok())
        .collect();
    if let Some(a) = body.active {
        user.active = a;
    }
    if let Err(e) = state.storage.store_user(&user).await {
        tracing::error!("Failed to store user: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Internal error"})),
        );
    }
    (
        StatusCode::OK,
        Json(serde_json::json!(user_to_response(&user))),
    )
}

async fn delete_user(
    State(state): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> StatusCode {
    let state = state.read().await;
    match state.storage.delete_user(&id).await {
        Ok(true) => StatusCode::NO_CONTENT,
        Ok(false) => StatusCode::NOT_FOUND,
        Err(e) => {
            tracing::error!("Failed to delete user: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

// ---------------------------------------------------------------------------
// ABAC policy admin
// ---------------------------------------------------------------------------

async fn evaluate_abac(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(body): Json<EvaluateAbacRequest>,
) -> Json<serde_json::Value> {
    let state = state.read().await;
    let tier = parse_tier(&state.config.tier);
    let roles: Vec<newloka_core::identity::Role> = body
        .subject_roles
        .iter()
        .filter_map(|r| r.parse().ok())
        .collect();
    let lab_affiliations = if roles.contains(&newloka_core::identity::Role::LabTechnician) {
        vec![newloka_core::cpoe::LabDepartment::Pathology]
    } else if roles.contains(&newloka_core::identity::Role::ImagingTechnician) {
        vec![newloka_core::cpoe::LabDepartment::Radiology]
    } else {
        vec![]
    };
    let action = match body.action.to_lowercase().as_str() {
        "create" => newloka_core::abac::Action::Create,
        "read" => newloka_core::abac::Action::Read,
        "update" => newloka_core::abac::Action::Update,
        "delete" => newloka_core::abac::Action::Delete,
        "syncsend" => newloka_core::abac::Action::SyncSend,
        "syncreceive" => newloka_core::abac::Action::SyncReceive,
        "override" => newloka_core::abac::Action::Override,
        "export" => newloka_core::abac::Action::Export,
        "researchquery" => newloka_core::abac::Action::ResearchQuery,
        _ => newloka_core::abac::Action::Read,
    };
    let sensitivity = newloka_core::abac::SensitivityLevel::Normal;
    let req = newloka_core::abac::PolicyRequest {
        subject: newloka_core::abac::Subject {
            user_id: "test-user".to_string(),
            roles: roles.clone(),
            department_id: None,
            team_ids: vec![],
            session_valid: true,
            emergency_override: false,
            lab_affiliations,
        },
        resource: newloka_core::abac::Resource {
            resource_type: body.resource_type.clone(),
            resource_id: "test-resource".to_string(),
            patient_id: None,
            department_id: None,
            owner_team_ids: vec![],
            lab_department: None,
            sensitivity,
        },
        action,
        context: newloka_core::abac::Context {
            tier,
            offline: false,
            peer_node_id: None,
            time_of_day: chrono::Local::now().format("%H:%M").to_string(),
            lab_config: newloka_core::cpoe::LabConfiguration::for_tier(tier),
            patient_has_lab_order: body.patient_has_lab_order.unwrap_or(false),
        },
    };
    let decision = newloka_core::abac::PolicyEngine::evaluate(&req);
    let (allowed, reason) = match decision {
        newloka_core::abac::PolicyDecision::Allow => (true, "Policy allows".to_string()),
        newloka_core::abac::PolicyDecision::AllowWithAudit { reason } => (true, reason),
        newloka_core::abac::PolicyDecision::Deny { reason } => (false, reason),
    };
    Json(serde_json::json!({
        "allowed": allowed,
        "reason": reason,
        "subject_roles": body.subject_roles,
        "resource_type": body.resource_type,
        "action": body.action,
    }))
}

async fn list_abac_policies() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "policies": [
            { "name": "Session validity", "description": "All requests require a valid session" },
            { "name": "Emergency override", "description": "Emergency role can read/update/override with audit" },
            { "name": "Patient creation", "description": "Lab and imaging staff cannot create patients" },
            { "name": "Lab patient read", "description": "Lab/imaging can only read patients with lab orders" },
            { "name": "Role-based create", "description": "Create requires clinician, nurse, admin, dept head, or resident" },
            { "name": "Role-based update/delete", "description": "Modify requires clinician, admin, dept head, or resident" },
            { "name": "Override privilege", "description": "Override requires emergency, admin, or dept head" },
            { "name": "Research query", "description": "Research queries require researcher or admin" },
            { "name": "Lab report creation", "description": "Lab/imaging reports can be created by lab/imaging staff, clinicians, admin, dept head" },
            { "name": "Department silos", "description": "T2+ enforces department boundaries unless shared team or admin" },
            { "name": "Resident team scope", "description": "Residents can only modify resources within their team" },
            { "name": "Sensitivity critical", "description": "Critical resources require clinician, admin, dept head, or emergency" },
            { "name": "Sensitivity research", "description": "Research-only data requires researcher or admin" },
        ]
    }))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn parse_tier(s: &str) -> newloka_core::DeploymentTier {
    match s {
        "T0" => newloka_core::DeploymentTier::T0_SoloClinician,
        "T1" => newloka_core::DeploymentTier::T1_SmallClinic,
        "T2" => newloka_core::DeploymentTier::T2_RuralHospital,
        "T3" => newloka_core::DeploymentTier::T3_MultiDepartmentHospital,
        "T4" => newloka_core::DeploymentTier::T4_ResearchFederation,
        _ => newloka_core::DeploymentTier::T1_SmallClinic,
    }
}

async fn seed_demo_users(storage: &newloka_core::storage::StorageEngine) -> anyhow::Result<()> {
    let existing = storage.get_user_by_username("admin").await.ok().flatten();
    if existing.is_some() {
        return Ok(());
    }
    let users = vec![
        (
            "admin",
            "New Loka Admin",
            vec!["administrator"],
            None,
            vec![],
            vec![],
        ),
        (
            "pathlab",
            "Pathology Lab",
            vec!["lab_technician"],
            Some("Pathology".to_string()),
            vec!["path-team".to_string()],
            vec!["pathology"],
        ),
        (
            "imaging",
            "Imaging Tech",
            vec!["imaging_technician"],
            Some("Radiology".to_string()),
            vec!["radio-team".to_string()],
            vec!["radiology"],
        ),
        (
            "resident",
            "Resident Doctor",
            vec!["resident_doctor"],
            Some("Internal Medicine".to_string()),
            vec!["team-alpha".to_string()],
            vec![],
        ),
        (
            "depthead",
            "Department Head",
            vec!["department_head"],
            Some("Internal Medicine".to_string()),
            vec!["team-alpha".to_string(), "mgmt-team".to_string()],
            vec![],
        ),
    ];
    for (username, display_name, roles, dept, teams, labs) in users {
        let (salt, hash) = newloka_core::identity::Authenticator::hash_password(username)?;
        let parsed_roles: Vec<newloka_core::identity::Role> =
            roles.iter().filter_map(|r| r.parse().ok()).collect();
        let parsed_labs: Vec<newloka_core::cpoe::LabDepartment> =
            labs.iter().filter_map(|l| l.parse().ok()).collect();
        let user = newloka_core::identity::User {
            id: uuid::Uuid::new_v4().to_string(),
            username: username.to_string(),
            display_name: display_name.to_string(),
            roles: parsed_roles,
            department_id: dept,
            team_ids: teams,
            lab_affiliations: parsed_labs,
            active: true,
            created_at: chrono::Utc::now(),
            password_hash: hash,
            salt,
            totp_secret: None,
            last_login: None,
        };
        storage.store_user(&user).await?;
    }
    tracing::info!("Seeded demo users");
    Ok(())
}
