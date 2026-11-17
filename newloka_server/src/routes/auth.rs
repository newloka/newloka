//! Authentication, Identity, LDAP & ABAC Routes

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::state::AppState;

pub const DIRECTORY_CONFIG_DOC_ID: &str = "enterprise-directory-config";

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub tier: Option<String>,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user_id: String,
    pub display_name: String,
    pub node_id: String,
    pub tier: String,
    pub expires_in: i64,
    pub roles: Vec<String>,
    pub department_id: Option<String>,
    pub team_ids: Vec<String>,
    pub lab_affiliations: Vec<String>,
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

#[derive(Deserialize)]
pub struct EnterpriseLoginRequest {
    pub username: String,
    pub password: String,
    pub tier: Option<String>,
}

pub fn parse_tier(s: &str) -> newloka_core::DeploymentTier {
    match s.to_uppercase().as_str() {
        "T0" => newloka_core::DeploymentTier::T0_SoloClinician,
        "T1" => newloka_core::DeploymentTier::T1_SmallClinic,
        "T2" => newloka_core::DeploymentTier::T2_RuralHospital,
        "T3" => newloka_core::DeploymentTier::T3_MultiDepartmentHospital,
        "T4" => newloka_core::DeploymentTier::T4_ResearchFederation,
        _ => newloka_core::DeploymentTier::T1_SmallClinic,
    }
}

pub fn user_to_response(u: &newloka_core::identity::User) -> UserResponse {
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

pub fn router() -> Router<Arc<RwLock<AppState>>> {
    Router::new()
        .route("/users", get(list_users).post(create_user))
        .route(
            "/users/:id",
            get(get_user).put(update_user).delete(delete_user),
        )
        .route("/abac/evaluate", post(evaluate_abac))
        .route("/abac/policies", get(list_abac_policies))
        .route("/auth/login", post(login_handler))
        .route("/auth/session", get(session_handler))
        .route("/auth/logout", post(logout_handler))
        .route(
            "/api/auth/enterprise/login",
            post(enterprise_ldap_login_handler),
        )
        .route(
            "/api/auth/directory/config",
            get(get_directory_config_handler).post(save_directory_config_handler),
        )
}

pub async fn login_handler(
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

    // Async hygiene: offload CPU-heavy Argon2id password verification
    let password = body.password.clone();
    let salt = user.salt.clone();
    let hash = user.password_hash.clone();
    let valid = tokio::task::spawn_blocking(move || {
        newloka_core::identity::Authenticator::verify_password(&password, &salt, &hash)
            .unwrap_or(false)
    })
    .await
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

pub async fn logout_handler(
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

pub async fn session_handler(
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

pub async fn list_users(
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

pub async fn get_user(
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

pub async fn create_user(
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

    let password = body.password.clone();
    let hash_result = tokio::task::spawn_blocking(move || {
        newloka_core::identity::Authenticator::hash_password(&password)
    })
    .await;

    let (salt, hash) = match hash_result {
        Ok(Ok(h)) => h,
        _ => {
            tracing::error!("Password hashing failed");
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

pub async fn update_user(
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
        let password = body.password.clone();
        let hash_result = tokio::task::spawn_blocking(move || {
            newloka_core::identity::Authenticator::hash_password(&password)
        })
        .await;

        let (salt, hash) = match hash_result {
            Ok(Ok(h)) => h,
            _ => {
                tracing::error!("Password hashing failed");
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

pub async fn delete_user(
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

pub async fn evaluate_abac(
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

pub async fn list_abac_policies() -> Json<serde_json::Value> {
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

pub async fn get_directory_config_handler(
    State(state): State<Arc<RwLock<AppState>>>,
) -> Json<serde_json::Value> {
    let state = state.read().await;
    if let Ok(Some(cfg)) = state.storage.get_json(DIRECTORY_CONFIG_DOC_ID).await {
        return Json(cfg);
    }
    let default_cfg = newloka_core::identity::EnterpriseDirectoryConfig::default();
    let val = serde_json::to_value(&default_cfg).unwrap_or(serde_json::json!({}));
    Json(val)
}

pub async fn save_directory_config_handler(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(payload): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    let state = state.read().await;
    match state
        .storage
        .store_json(
            "EnterpriseDirectory",
            DIRECTORY_CONFIG_DOC_ID,
            &payload,
            None,
            None,
        )
        .await
    {
        Ok(_) => (StatusCode::OK, Json(payload)),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

pub async fn enterprise_ldap_login_handler(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(body): Json<EnterpriseLoginRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let (config, node_id, default_tier) = {
        let read = state.read().await;
        let cfg: newloka_core::identity::EnterpriseDirectoryConfig =
            match read.storage.get_json(DIRECTORY_CONFIG_DOC_ID).await {
                Ok(Some(val)) => serde_json::from_value(val).unwrap_or_default(),
                _ => newloka_core::identity::EnterpriseDirectoryConfig {
                    enabled: true,
                    ..Default::default()
                },
            };
        (cfg, read.node_id.clone(), read.config.tier.clone())
    };

    if let Some(auth_res) = config.authenticate(&body.username, &body.password) {
        let tier_str = body.tier.unwrap_or(default_tier);
        let tier = parse_tier(&tier_str);
        let session =
            newloka_core::identity::Session::new(auth_res.username.clone(), node_id.clone(), tier);
        let token = session.token.clone();

        {
            let write = state.write().await;
            write.sessions.write().await.insert(token.clone(), session);
        }

        let roles_str: Vec<String> = auth_res
            .roles
            .iter()
            .map(|r| format!("{:?}", r).to_lowercase())
            .collect();
        (
            StatusCode::OK,
            Json(serde_json::json!({
                "token": token,
                "user_id": auth_res.username,
                "display_name": auth_res.display_name,
                "node_id": node_id,
                "tier": tier_str,
                "roles": roles_str,
                "department_id": auth_res.department_id,
                "team_ids": auth_res.team_ids,
                "directory_type": "ActiveDirectory_LDAP",
                "expires_in": 43200
            })),
        )
    } else {
        (
            StatusCode::UNAUTHORIZED,
            Json(
                serde_json::json!({ "error": "Invalid enterprise credentials or directory authentication rejected" }),
            ),
        )
    }
}
