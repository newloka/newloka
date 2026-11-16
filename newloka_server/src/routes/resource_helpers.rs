//! Generic FHIR Resource CRUD Helpers
//!
//! Applies WATER Strike 3 pattern extraction for encrypted CRUD operations.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::state::AppState;

#[derive(Deserialize, Debug, Clone, Default)]
pub struct SearchParams {
    pub patient: Option<String>,
    pub name: Option<String>,
    pub _count: Option<usize>,
}

pub async fn search_resource(
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

pub async fn create_resource(
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

pub async fn get_resource(
    State(state): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
    resource_type: &str,
) -> (StatusCode, Json<serde_json::Value>) {
    let state = state.read().await;
    match state.storage.get_json(&id).await {
        Ok(Some(r)) => {
            if r.get("resourceType").and_then(|v| v.as_str()) == Some(resource_type) {
                (StatusCode::OK, Json(r))
            } else {
                (
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({"error": "Resource not found"})),
                )
            }
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Resource not found"})),
        ),
        Err(e) => {
            tracing::error!("Get {} failed: {}", resource_type, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("{}", e)})),
            )
        }
    }
}

pub async fn update_resource(
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
            tracing::error!("Update {} failed: {}", resource_type, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("{}", e)})),
            )
        }
    }
}

pub async fn delete_resource(
    State(state): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
    resource_type: &str,
) -> StatusCode {
    let state = state.read().await;
    match state.storage.soft_delete(&id).await {
        Ok(true) => StatusCode::NO_CONTENT,
        Ok(false) => StatusCode::NOT_FOUND,
        Err(e) => {
            tracing::error!("Delete {} failed: {}", resource_type, e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
