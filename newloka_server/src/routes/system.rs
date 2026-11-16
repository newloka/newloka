//! System, Metadata, Configuration, Updater & Static Assets Routes

use axum::{
    extract::State,
    http::StatusCode,
    response::{Json, Redirect, Response},
    routing::{get, post},
    Router,
};
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::state::{AppState, NodeConfig};
use crate::updater;

pub const CONFIG_DOC_ID: &str = "instance-config";
pub const PRESCRIBER_DOC_ID: &str = "prescriber-profile";

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub node_id: String,
    pub tier: String,
}

#[derive(Serialize)]
pub struct CapabilityStatement {
    pub resource_type: String,
    pub status: String,
    pub kind: String,
    pub software: SoftwareInfo,
    pub fhir_version: String,
    pub rest: Vec<RestInfo>,
}

#[derive(Serialize)]
pub struct SoftwareInfo {
    pub name: String,
    pub version: String,
}

#[derive(Serialize)]
pub struct RestInfo {
    pub mode: String,
    pub resource: Vec<ResourceInfo>,
}

#[derive(Serialize)]
pub struct ResourceInfo {
    pub type_: String,
    pub interaction: Vec<InteractionInfo>,
}

#[derive(Serialize)]
pub struct InteractionInfo {
    pub code: String,
}

#[derive(Serialize, Deserialize)]
pub struct ConfigResponse {
    pub config: NodeConfig,
}

#[derive(RustEmbed)]
#[folder = "../newloka_web"]
pub struct Assets;

pub fn mime_type(path: &str) -> &'static str {
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

pub async fn embedded_static_handler(
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

pub fn router() -> Router<Arc<RwLock<AppState>>> {
    Router::new()
        .route(
            "/",
            get(|| async { Redirect::temporary("/static/index.html#rxpad") }),
        )
        .route("/health", get(health_handler))
        .route("/api/health", get(health_handler))
        .route("/metadata", get(capability_statement))
        .route("/config", get(get_config).put(put_config))
        .route("/sync/delta", post(sync_delta))
        .route("/sync/manifest", get(sync_manifest))
        .route("/api/system/version", get(system_version_handler))
        .route("/api/system/update/check", get(update_check_handler))
        .route("/api/system/update/apply", post(update_apply_handler))
        .route("/api/system/update/upload", post(update_upload_handler))
        .route("/api/system/restart", post(system_restart_handler))
        .route(
            "/api/config",
            get(get_config_handler).post(save_config_handler),
        )
        .route(
            "/api/prescriber/profile",
            get(get_prescriber_profile_handler).post(save_prescriber_profile_handler),
        )
        .nest_service("/static", tower::util::service_fn(embedded_static_handler))
        .fallback(|| async { Redirect::temporary("/static/index.html") })
}

pub async fn health_handler(State(state): State<Arc<RwLock<AppState>>>) -> Json<HealthResponse> {
    let state = state.read().await;
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        node_id: state.node_id.clone(),
        tier: state.config.tier.clone(),
    })
}

pub async fn capability_statement() -> Json<CapabilityStatement> {
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
                resource_info("MedicationDispense"),
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

pub async fn get_config(State(state): State<Arc<RwLock<AppState>>>) -> Json<ConfigResponse> {
    let state = state.read().await;
    Json(ConfigResponse {
        config: state.config.clone(),
    })
}

pub async fn put_config(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(body): Json<NodeConfig>,
) -> Json<ConfigResponse> {
    let mut state = state.write().await;
    let mut cfg = body;
    cfg.node_id = state.node_id.clone();
    state.config = cfg.clone();
    Json(ConfigResponse { config: cfg })
}

pub async fn sync_delta(Json(body): Json<serde_json::Value>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "from_node": body.get("from_node").and_then(|v| v.as_str()).unwrap_or("unknown"),
        "records": [],
        "conflicts": [],
        "timestamp": chrono::Utc::now().timestamp_millis()
    }))
}

pub async fn sync_manifest(
    State(state): State<Arc<RwLock<AppState>>>,
) -> Json<serde_json::Value> {
    let state = state.read().await;
    Json(serde_json::json!({
        "node_id": state.node_id,
        "tier": state.config.tier,
        "department": state.config.department,
        "sync_enabled": state.config.sync_enabled,
        "mesh_enabled": state.config.mesh_enabled,
        "last_sync": chrono::Utc::now().to_rfc3339(),
        "resource_types": [
            "Patient", "Encounter", "Observation", "Condition", "MedicationRequest",
            "Procedure", "DiagnosticReport", "AllergyIntolerance", "Flag", "DetectedIssue",
            "CarePlan", "FamilyMemberHistory", "Immunization", "DocumentReference",
            "ServiceRequest", "ImagingStudy", "Specimen", "Composition", "Whiteboard",
            "MedicationAdministration", "MedicationStatement", "Provenance"
        ]
    }))
}

pub async fn system_version_handler(
    State(state): State<Arc<RwLock<AppState>>>,
) -> Json<updater::SystemInfo> {
    let tier = {
        let s = state.read().await;
        s.config.tier.clone()
    };
    Json(updater::get_system_info(&tier))
}

pub async fn update_check_handler() -> Json<updater::UpdateCheckResponse> {
    Json(updater::check_for_updates().await)
}

pub async fn update_apply_handler(
    payload: Option<Json<updater::UpdateApplyRequest>>,
) -> Result<Json<updater::UpdateApplyResponse>, (StatusCode, Json<serde_json::Value>)> {
    let asset_url = payload.and_then(|p| p.0.asset_url);
    match updater::apply_update(asset_url).await {
        Ok(res) => Ok(Json(res)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "status": "error",
                "message": format!("Update failed: {:#}", e),
                "requires_restart": false,
                "updated_files": []
            })),
        )),
    }
}

pub async fn update_upload_handler(
    body: axum::body::Bytes,
) -> Result<Json<updater::UpdateApplyResponse>, (StatusCode, Json<serde_json::Value>)> {
    if body.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "status": "error",
                "message": "Uploaded update package is empty.",
                "requires_restart": false,
                "updated_files": []
            })),
        ));
    }
    match updater::apply_binary_or_archive(&body) {
        Ok(res) => Ok(Json(res)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "status": "error",
                "message": format!("Failed to apply offline update package: {:#}", e),
                "requires_restart": false,
                "updated_files": []
            })),
        )),
    }
}

pub async fn system_restart_handler(
) -> Result<Json<updater::RestartResponse>, (StatusCode, Json<serde_json::Value>)> {
    match updater::restart_server() {
        Ok(res) => Ok(Json(res)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "status": "error",
                "message": format!("Failed to restart server: {:#}", e),
            })),
        )),
    }
}

pub async fn get_config_handler(
    State(state): State<Arc<RwLock<AppState>>>,
) -> Json<serde_json::Value> {
    let state = state.read().await;
    if let Ok(Some(cfg)) = state.storage.get_json(CONFIG_DOC_ID).await {
        return Json(cfg);
    }
    if let Ok(Some(cfg)) = state.storage.get_json("config-default").await {
        return Json(cfg);
    }
    Json(serde_json::json!({
        "tier": "T0",
        "clinicProfile": "solo",
        "defaultLandingView": "rxpad",
        "roleLandingViews": {
            "clinician": "rxpad",
            "resident_doctor": "inpatientRegister",
            "billing": "billing",
            "pharmacist": "deptSummary",
            "department_head": "deptSummary",
            "administrator": "dashboard"
        },
        "installedModules": [
            "dashboard", "rxpad", "inpatientRegister", "billing", "deptSummary",
            "patients", "patientChart", "clinicalNotes", "cpoeOrders", "vitalsFlowsheet",
            "resultsReview", "drugInteraction", "medications", "encounters", "observations",
            "conditions", "procedures", "alerts", "carePlans", "familyHistory", "immunizations",
            "documents", "appointmentSchedule", "mar", "handoffSbar", "whiteboard", "ingest",
            "departments", "reports", "research", "consent", "federation",
            "audit", "settings"
        ],
        "clinic": null,
        "sync": {
            "meshPeers": [],
            "centralHospitalUrl": "",
            "centralHospitalToken": "",
            "centralSyncEnabled": false
        }
    }))
}

pub async fn save_config_handler(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(payload): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    let state = state.read().await;
    match state
        .storage
        .store_json("Config", CONFIG_DOC_ID, &payload, None, None)
        .await
    {
        Ok(_) => (StatusCode::OK, Json(payload)),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Failed to save config: {}", e) })),
        ),
    }
}

pub async fn get_prescriber_profile_handler(
    State(state): State<Arc<RwLock<AppState>>>,
) -> Json<serde_json::Value> {
    let state = state.read().await;
    if let Ok(Some(mut profile)) = state.storage.get_json(PRESCRIBER_DOC_ID).await {
        if profile
            .get("logoBase64")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .is_empty()
        {
            if let Some(logo) = profile
                .get("clinicHeader")
                .and_then(|h| h.get("logoBase64"))
                .cloned()
            {
                if let Some(obj) = profile.as_object_mut() {
                    obj.insert("logoBase64".to_string(), logo.clone());
                    obj.insert("logo_data_url".to_string(), logo);
                }
            }
        }
        return Json(profile);
    }
    if let Ok(Some(mut profile)) = state.storage.get_json("prescriber-profile-default").await {
        if profile
            .get("logoBase64")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .is_empty()
        {
            if let Some(logo) = profile
                .get("clinicHeader")
                .and_then(|h| h.get("logoBase64"))
                .cloned()
            {
                if let Some(obj) = profile.as_object_mut() {
                    obj.insert("logoBase64".to_string(), logo.clone());
                    obj.insert("logo_data_url".to_string(), logo);
                }
            }
        }
        return Json(profile);
    }
    Json(serde_json::json!({
        "clinicianName": "",
        "clinician_name": "",
        "qualifications": "",
        "regNo": "",
        "registration_no": "",
        "specialty": "",
        "department": "",
        "isLegalPrescriber": true,
        "is_legal_prescriber": true,
        "signatureBase64": "",
        "signature_data_url": "",
        "signature": "",
        "logoBase64": "",
        "logo_data_url": "",
        "clinicHeader": null
    }))
}

pub async fn save_prescriber_profile_handler(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(payload): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    let state = state.read().await;
    match state
        .storage
        .store_json("Practitioner", PRESCRIBER_DOC_ID, &payload, None, None)
        .await
    {
        Ok(_) => (StatusCode::OK, Json(payload)),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Failed to save prescriber profile: {}", e) })),
        ),
    }
}
