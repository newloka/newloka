//! Pharmacopoeia & Clinical Safety Shield Routes

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct PharmacopoeiaSearchQuery {
    #[serde(default)]
    pub q: String,
    pub category: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct SafetyCheckRequest {
    #[serde(default)]
    pub drugs: Vec<String>,
    #[serde(default)]
    pub allergies: Vec<String>,
}

pub fn router() -> Router<Arc<RwLock<AppState>>> {
    Router::new()
        .route("/api/pharmacopoeia/info", get(pharmacopoeia_info_handler))
        .route("/api/pharmacopoeia/search", get(pharmacopoeia_search_handler))
        .route("/api/pharmacopoeia/entry/:id", get(pharmacopoeia_get_entry_handler))
        .route("/api/pharmacopoeia/categories", get(pharmacopoeia_categories_handler))
        .route("/api/pharmacopoeia/order-sets", get(pharmacopoeia_list_order_sets_handler))
        .route("/api/pharmacopoeia/order-sets/:id", get(pharmacopoeia_get_order_set_handler))
        .route("/api/pharmacopoeia/check-safety", post(pharmacopoeia_check_safety_handler))
}

pub async fn pharmacopoeia_info_handler(
    State(state): State<Arc<RwLock<AppState>>>,
) -> Json<serde_json::Value> {
    let state = state.read().await;
    let (version, published_date, reference_standard, monographs_count, ordersets_count) =
        state.pharmacopoeia.metadata();
    Json(serde_json::json!({
        "version": version,
        "publishedDate": published_date,
        "referenceStandard": reference_standard,
        "monographsCount": monographs_count,
        "orderSetsCount": ordersets_count,
    }))
}

pub async fn pharmacopoeia_search_handler(
    State(state): State<Arc<RwLock<AppState>>>,
    Query(params): Query<PharmacopoeiaSearchQuery>,
) -> Json<Vec<newloka_core::pharmacopoeia::PharmacopoeiaSearchResult>> {
    let state = state.read().await;
    let limit = params.limit.unwrap_or(25).min(100);
    let results = state
        .pharmacopoeia
        .search(&params.q, params.category.as_deref(), limit);
    Json(results)
}

pub async fn pharmacopoeia_get_entry_handler(
    State(state): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    let state = state.read().await;
    if let Some(entry) = state.pharmacopoeia.get_entry(&id) {
        (StatusCode::OK, Json(serde_json::to_value(entry).unwrap()))
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Not Found",
                "message": format!("Pharmacopoeia entry '{}' not found", id)
            })),
        )
    }
}

pub async fn pharmacopoeia_categories_handler(
    State(state): State<Arc<RwLock<AppState>>>,
) -> Json<Vec<newloka_core::pharmacopoeia::CategorySummary>> {
    let state = state.read().await;
    Json(state.pharmacopoeia.list_categories())
}

pub async fn pharmacopoeia_list_order_sets_handler(
    State(state): State<Arc<RwLock<AppState>>>,
) -> Json<Vec<newloka_core::pharmacopoeia::ClinicalOrderSet>> {
    let state = state.read().await;
    let sets: Vec<newloka_core::pharmacopoeia::ClinicalOrderSet> = state
        .pharmacopoeia
        .list_order_sets()
        .into_iter()
        .cloned()
        .collect();
    Json(sets)
}

pub async fn pharmacopoeia_get_order_set_handler(
    State(state): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    let state = state.read().await;
    if let Some(set) = state.pharmacopoeia.get_order_set(&id) {
        (StatusCode::OK, Json(serde_json::to_value(set).unwrap()))
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Not Found",
                "message": format!("Order set '{}' not found", id)
            })),
        )
    }
}

pub async fn pharmacopoeia_check_safety_handler(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(body): Json<SafetyCheckRequest>,
) -> Json<newloka_core::pharmacopoeia::DrugSafetyReport> {
    let state = state.read().await;
    let report = state
        .pharmacopoeia
        .check_safety(&body.drugs, &body.allergies);
    Json(report)
}
