//! Hospital Governance, Departments, SBAR Transfers, Reports & Audit Routes

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::state::AppState;

pub const INPATIENT_CENSUS_DOC_ID: &str = "inpatient-census";
pub const BILLING_INVOICES_DOC_ID: &str = "billing-invoices";
pub const DEPARTMENTS_DOC_ID: &str = "hospital-departments";
pub const TRANSFERS_DOC_ID: &str = "department-transfers";

pub fn router() -> Router<Arc<RwLock<AppState>>> {
    Router::new()
        .route("/AuditEvent", get(search_audit).post(create_audit))
        .route(
            "/api/inpatient/census",
            get(get_inpatient_census_handler).post(save_inpatient_census_handler),
        )
        .route(
            "/api/billing/invoices",
            get(get_billing_invoices_handler).post(create_billing_invoice_handler),
        )
        .route(
            "/api/billing/invoices/:id/pay",
            post(pay_billing_invoice_handler),
        )
        .route("/api/pharmacy/summary", get(get_pharmacy_summary_handler))
        .route(
            "/api/departments",
            get(list_departments_handler).post(save_department_handler),
        )
        .route(
            "/api/departments/transfers",
            get(list_transfers_handler).post(create_transfer_handler),
        )
        .route(
            "/api/departments/transfers/:id/accept",
            post(accept_transfer_handler),
        )
        .route(
            "/api/departments/transfers/:id/reject",
            post(reject_transfer_handler),
        )
        .route(
            "/api/reports/overview",
            get(hospital_reports_overview_handler),
        )
        .route(
            "/api/reports/department/:id",
            get(hospital_reports_department_handler),
        )
}

pub async fn search_audit(
    State(state): State<Arc<RwLock<AppState>>>,
) -> (StatusCode, Json<serde_json::Value>) {
    let state = state.read().await;
    let rows = state.storage.search_audit(200).await.unwrap_or_default();
    let total = rows.len();
    let entry: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|r| {
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
        })
        .collect();
    let results = serde_json::json!({
        "resourceType": "Bundle",
        "type": "searchset",
        "total": total,
        "entry": entry.into_iter().map(|r| serde_json::json!({ "resource": r })).collect::<Vec<_>>()
    });
    (StatusCode::OK, Json(results))
}

pub async fn create_audit(
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

pub async fn get_inpatient_census_handler(
    State(state): State<Arc<RwLock<AppState>>>,
) -> Json<serde_json::Value> {
    let state = state.read().await;
    if let Ok(Some(census)) = state.storage.get_json(INPATIENT_CENSUS_DOC_ID).await {
        return Json(census);
    }
    Json(serde_json::json!([]))
}

pub async fn save_inpatient_census_handler(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(payload): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    let state = state.read().await;
    match state
        .storage
        .store_json(
            "InpatientCensus",
            INPATIENT_CENSUS_DOC_ID,
            &payload,
            None,
            None,
        )
        .await
    {
        Ok(_) => (StatusCode::OK, Json(payload)),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Failed to save inpatient census: {}", e) })),
        ),
    }
}

pub async fn get_billing_invoices_handler(
    headers: axum::http::HeaderMap,
    State(state): State<Arc<RwLock<AppState>>>,
) -> Json<serde_json::Value> {
    let state = state.read().await;
    let is_billing_role = headers
        .get("x-user-role")
        .and_then(|v| v.to_str().ok())
        .map(|r| {
            r.eq_ignore_ascii_case("billing")
                || r.eq_ignore_ascii_case("cashier")
                || r.eq_ignore_ascii_case("reception")
        })
        .unwrap_or(false);

    let invoices: serde_json::Value =
        if let Ok(Some(inv)) = state.storage.get_json(BILLING_INVOICES_DOC_ID).await {
            inv
        } else {
            serde_json::json!([])
        };

    if is_billing_role {
        if let Some(list) = invoices.as_array() {
            let redacted_list: Vec<serde_json::Value> = list
                .iter()
                .map(|inv| {
                    let mut item = inv.clone();
                    if let Some(obj) = item.as_object_mut() {
                        obj.insert(
                            "diagnosis".to_string(),
                            serde_json::json!("🔒 [CLINICAL DATA REDACTED - ABAC CONFIDENTIALITY]"),
                        );
                    }
                    item
                })
                .collect();
            return Json(serde_json::Value::Array(redacted_list));
        }
    }

    Json(invoices)
}

pub async fn create_billing_invoice_handler(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(payload): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    let state = state.read().await;
    let mut current_list: Vec<serde_json::Value> =
        if let Ok(Some(inv)) = state.storage.get_json(BILLING_INVOICES_DOC_ID).await {
            serde_json::from_value(inv).unwrap_or_default()
        } else {
            vec![]
        };

    let mut new_inv = payload.clone();
    if new_inv.get("id").is_none() {
        let serial = current_list.len() + 1;
        new_inv["id"] = serde_json::json!(format!("INV-2026-{:04}", serial));
    }
    current_list.insert(0, new_inv.clone());

    let _ = state
        .storage
        .store_json(
            "Invoice",
            BILLING_INVOICES_DOC_ID,
            &serde_json::Value::Array(current_list),
            None,
            None,
        )
        .await;
    (StatusCode::CREATED, Json(new_inv))
}

pub async fn pay_billing_invoice_handler(
    Path(id): Path<String>,
    State(state): State<Arc<RwLock<AppState>>>,
    Json(payment_info): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    let state = state.read().await;
    let mut current_list: Vec<serde_json::Value> =
        if let Ok(Some(inv)) = state.storage.get_json(BILLING_INVOICES_DOC_ID).await {
            serde_json::from_value(inv).unwrap_or_default()
        } else {
            vec![]
        };

    let mut found = false;
    for inv in current_list.iter_mut() {
        if inv.get("id").and_then(|v| v.as_str()) == Some(&id) {
            found = true;
            if let Some(obj) = inv.as_object_mut() {
                let net = obj.get("netAmount").and_then(|v| v.as_f64()).unwrap_or(0.0);
                obj.insert("paidAmount".to_string(), serde_json::json!(net));
                obj.insert("status".to_string(), serde_json::json!("Paid"));
                if let Some(m) = payment_info.get("paymentMethod") {
                    obj.insert("paymentMethod".to_string(), m.clone());
                } else {
                    obj.insert(
                        "paymentMethod".to_string(),
                        serde_json::json!("Cash / Recorded"),
                    );
                }
            }
            break;
        }
    }

    if found {
        let _ = state
            .storage
            .store_json(
                "Invoice",
                BILLING_INVOICES_DOC_ID,
                &serde_json::Value::Array(current_list),
                None,
                None,
            )
            .await;
        (
            StatusCode::OK,
            Json(serde_json::json!({ "status": "success", "invoiceId": id })),
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Invoice not found" })),
        )
    }
}

pub async fn get_pharmacy_summary_handler(
    State(state): State<Arc<RwLock<AppState>>>,
) -> Json<serde_json::Value> {
    let state = state.read().await;
    let all_prescriptions = crate::rxpad::get_prescription_summaries(&state).await;
    let pending_dispenses = all_prescriptions
        .iter()
        .filter(|p| p.dispense_status == "prescribed")
        .count();
    let dispensed_today = all_prescriptions
        .iter()
        .filter(|p| p.dispense_status == "dispensed")
        .count();

    Json(serde_json::json!({
        "pendingDispenses": pending_dispenses,
        "dispensedToday": dispensed_today,
        "criticalStockAlerts": [
            { "drug": "AMOXICILLIN + CLAVULANIC ACID 625MG", "available": 18, "threshold": 50, "status": "critical" },
            { "drug": "INSULIN GLARGINE 100 IU/ML", "available": 4, "threshold": 15, "status": "critical" },
            { "drug": "PARACETAMOL 650MG", "available": 95, "threshold": 100, "status": "warning" }
        ],
        "safetyAlerts": [
            {
                "id": "alt-01",
                "severity": "critical",
                "message": "HIGH RISK: Warfarin + Fluconazole co-prescribed in Ward 2A (Bed 04). Potential 3x INR spike!",
                "timestamp": "12 mins ago"
            },
            {
                "id": "alt-02",
                "severity": "critical",
                "message": "STAT ORDER: ICU Bed 02 Noradrenaline 4mg/2ml infusion stat dispense pending",
                "timestamp": "3 mins ago"
            },
            {
                "id": "alt-03",
                "severity": "warning",
                "message": "Formulary batch expiry warning: Cefixime 200mg batch #CFX-998 expires in 14 days",
                "timestamp": "1 hour ago"
            }
        ],
        "averageTatMins": 8.5
    }))
}

pub async fn list_departments_handler(
    State(state): State<Arc<RwLock<AppState>>>,
) -> Json<serde_json::Value> {
    let state = state.read().await;
    if let Ok(Some(depts)) = state.storage.get_json(DEPARTMENTS_DOC_ID).await {
        return Json(depts);
    }
    Json(serde_json::json!([]))
}

pub async fn save_department_handler(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(payload): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    let state = state.read().await;
    match state
        .storage
        .store_json("Department", DEPARTMENTS_DOC_ID, &payload, None, None)
        .await
    {
        Ok(_) => (StatusCode::OK, Json(payload)),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

pub async fn list_transfers_handler(
    State(state): State<Arc<RwLock<AppState>>>,
) -> Json<serde_json::Value> {
    let state = state.read().await;
    if let Ok(Some(transfers)) = state.storage.get_json(TRANSFERS_DOC_ID).await {
        return Json(transfers);
    }
    Json(serde_json::json!([]))
}

pub async fn create_transfer_handler(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(mut payload): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    let state = state.read().await;
    if payload
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .is_empty()
    {
        let short_id = format!("tr-{}", &uuid::Uuid::new_v4().to_string()[..8]);
        payload["id"] = serde_json::json!(short_id);
    }
    payload["status"] = serde_json::json!("requested");
    let now = chrono::Utc::now().to_rfc3339();
    payload["created_at"] = serde_json::json!(now);
    payload["updated_at"] = serde_json::json!(now);

    let mut current_transfers = match state.storage.get_json(TRANSFERS_DOC_ID).await {
        Ok(Some(serde_json::Value::Array(arr))) => arr,
        _ => vec![],
    };
    current_transfers.insert(0, payload.clone());
    let _ = state
        .storage
        .store_json(
            "Transfer",
            TRANSFERS_DOC_ID,
            &serde_json::Value::Array(current_transfers),
            None,
            None,
        )
        .await;

    (StatusCode::CREATED, Json(payload))
}

pub async fn accept_transfer_handler(
    State(state): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    let state = state.read().await;
    let mut current_transfers = match state.storage.get_json(TRANSFERS_DOC_ID).await {
        Ok(Some(serde_json::Value::Array(arr))) => arr,
        _ => vec![],
    };

    let mut found = false;
    for t in &mut current_transfers {
        if t.get("id").and_then(|v| v.as_str()) == Some(&id) {
            t["status"] = serde_json::json!("accepted");
            t["updated_at"] = serde_json::json!(chrono::Utc::now().to_rfc3339());
            if let Some(clinician) = payload
                .get("accepting_clinician_name")
                .and_then(|v| v.as_str())
            {
                t["accepting_clinician_name"] = serde_json::json!(clinician);
                t["handover_signed_by"] = serde_json::json!(clinician);
            }
            if let Some(clinician_id) = payload
                .get("accepting_clinician_id")
                .and_then(|v| v.as_str())
            {
                t["accepting_clinician_id"] = serde_json::json!(clinician_id);
            }
            if let Some(bed) = payload.get("assigned_bed").and_then(|v| v.as_str()) {
                t["assigned_bed"] = serde_json::json!(bed);
            }
            found = true;
            break;
        }
    }

    if found {
        let _ = state
            .storage
            .store_json(
                "Transfer",
                TRANSFERS_DOC_ID,
                &serde_json::Value::Array(current_transfers),
                None,
                None,
            )
            .await;
        (
            StatusCode::OK,
            Json(serde_json::json!({ "status": "accepted", "id": id })),
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Transfer not found" })),
        )
    }
}

pub async fn reject_transfer_handler(
    State(state): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    let state = state.read().await;
    let mut current_transfers = match state.storage.get_json(TRANSFERS_DOC_ID).await {
        Ok(Some(serde_json::Value::Array(arr))) => arr,
        _ => vec![],
    };

    let mut found = false;
    for t in &mut current_transfers {
        if t.get("id").and_then(|v| v.as_str()) == Some(&id) {
            t["status"] = serde_json::json!("rejected");
            t["updated_at"] = serde_json::json!(chrono::Utc::now().to_rfc3339());
            if let Some(reason) = payload.get("reason").and_then(|v| v.as_str()) {
                t["rejection_reason"] = serde_json::json!(reason);
            }
            found = true;
            break;
        }
    }

    if found {
        let _ = state
            .storage
            .store_json(
                "Transfer",
                TRANSFERS_DOC_ID,
                &serde_json::Value::Array(current_transfers),
                None,
                None,
            )
            .await;
        (
            StatusCode::OK,
            Json(serde_json::json!({ "status": "rejected", "id": id })),
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Transfer not found" })),
        )
    }
}

pub async fn hospital_reports_overview_handler(
    State(state): State<Arc<RwLock<AppState>>>,
) -> Json<serde_json::Value> {
    let state = state.read().await;
    let patients_count = state
        .storage
        .search_json("Patient", None)
        .await
        .unwrap_or_default()
        .len();

    let census_val = state
        .storage
        .get_json(INPATIENT_CENSUS_DOC_ID)
        .await
        .ok()
        .flatten();
    let inpatient_count = census_val
        .as_ref()
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(4);

    let depts_val = state
        .storage
        .get_json(DEPARTMENTS_DOC_ID)
        .await
        .ok()
        .flatten();
    let depts_count = depts_val
        .as_ref()
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(9);

    let transfers_val = state
        .storage
        .get_json(TRANSFERS_DOC_ID)
        .await
        .ok()
        .flatten();
    let transfers_count = transfers_val
        .as_ref()
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(1);

    let total_beds = 189;
    let occupied_beds = inpatient_count + 112;
    let occupancy_rate = ((occupied_beds as f64 / total_beds as f64) * 1000.0).round() / 10.0;

    Json(serde_json::json!({
        "hospitalName": "New Loka Multi-Specialty Hospital & Medical Center",
        "tier": state.config.tier,
        "metrics": {
            "totalRegisteredPatients": patients_count,
            "totalHospitalBeds": total_beds,
            "occupiedBeds": occupied_beds,
            "occupancyRatePercent": occupancy_rate,
            "activeInpatients": occupied_beds,
            "activeDepartments": depts_count,
            "pendingInterDeptTransfers": transfers_count,
            "averageLengthOfStayDays": 4.2,
            "emergencyDoorToDoctorMinutes": 14.5,
            "medicationAdministrationAdherencePercent": 99.4,
            "cpoeDiagnosticTurnaroundHours": 2.1,
            "breakGlassEmergencyOverrides24h": 1
        },
        "departmentCensusBreakdown": [
            { "department": "Emergency & Trauma", "code": "EMERG", "beds": 24, "occupied": 18, "occupancy": "75%", "acuityAvgNEWS2": 4.2 },
            { "department": "General Internal Medicine", "code": "MED", "beds": 60, "occupied": 48, "occupancy": "80%", "acuityAvgNEWS2": 1.8 },
            { "department": "Cardiology & Cath Lab", "code": "CARD", "beds": 35, "occupied": 30, "occupancy": "85.7%", "acuityAvgNEWS2": 3.9 },
            { "department": "General Surgery & OT", "code": "SURG", "beds": 45, "occupied": 32, "occupancy": "71.1%", "acuityAvgNEWS2": 1.2 },
            { "department": "Intensive Care Unit (ICU)", "code": "ICU", "beds": 20, "occupied": 17, "occupancy": "85.0%", "acuityAvgNEWS2": 6.8 },
            { "department": "Medical Oncology", "code": "ONC", "beds": 30, "occupied": 21, "occupancy": "70.0%", "acuityAvgNEWS2": 2.4 }
        ],
        "qualityAndSafety": {
            "sentinelEvents": 0,
            "infectionControlIsolationBeds": 6,
            "antibioticStewardshipCompliancePercent": 96.8,
            "criticalPanicLabEscalationsToday": 3
        }
    }))
}

pub async fn hospital_reports_department_handler(
    State(_state): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "departmentId": id,
        "activeStaffOnDuty": 14,
        "physicians": 4,
        "nurses": 8,
        "alliedHealth": 2,
        "roundsCompletedPercent": 100.0,
        "averageDischargeHours": 11.2,
        "satisfactionScore": 4.8
    }))
}
