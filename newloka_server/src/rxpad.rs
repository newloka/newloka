//! Rx Pad Integration Module
//!
//! Provides two-way synchronization between the standalone e-Prescription Pad
//! (`eRx Pad Pro.html`) and New Loka's FHIR R4 encrypted clinical database (`clinic.db`).

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        Html, Json,
    },
};
use base64::Engine;
use chrono::Utc;
use futures_core::Stream;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

use crate::AppState;

// ---------------------------------------------------------------------------
// Data Models
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RxPadMedItem {
    #[serde(default)]
    pub drug: String,
    pub dose: Option<String>,
    pub freq: Option<String>,
    pub dur: Option<String>,
    pub instr: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RxPadSyncRequest {
    #[serde(alias = "patient_name", rename = "patientName", default)]
    pub patient_name: String,
    #[serde(alias = "age_sex", rename = "ageSex", default)]
    pub age_sex: Option<String>,
    pub contact: Option<String>,
    pub date: Option<String>,
    pub weight: Option<String>,
    pub bp: Option<String>,
    pub diagnosis: Option<String>,
    #[serde(alias = "follow_up", rename = "followUp", default)]
    pub follow_up: Option<String>,
    #[serde(default)]
    pub meds: Vec<RxPadMedItem>,
    #[serde(default)]
    pub labs: Vec<String>,
    pub filename: Option<String>,
    pub serial: Option<u32>,
    pub doctor: Option<String>,
    #[serde(alias = "reg_no", rename = "regNo", default)]
    pub reg_no: Option<String>,
    #[serde(alias = "patient_id", rename = "patientId", default)]
    pub patient_id: Option<String>,
    #[serde(alias = "raw_snapshot", rename = "rawSnapshot", default)]
    pub raw_snapshot: Option<String>,
    #[serde(alias = "pdf_base64", rename = "pdfBase64", default)]
    pub pdf_base64: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RxPadSyncResponse {
    pub status: String,
    pub serial: u32,
    pub filename: String,
    pub patient_id: String,
    pub encounter_id: String,
    pub prescription_id: String,
    pub medication_request_ids: Vec<String>,
    pub service_request_ids: Vec<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RxPadPrescriptionSummary {
    pub id: String,
    pub serial: u32,
    pub date: String,
    pub patient_id: String,
    pub patient_name: String,
    pub age_sex: String,
    pub contact: String,
    pub diagnosis: String,
    pub meds_count: usize,
    pub meds_summary: String,
    pub labs_count: usize,
    pub filename: String,
    pub dispense_status: String,
    pub dispensed_at: Option<i64>,
    pub dispensed_by: Option<String>,
    #[serde(default)]
    pub meds: Vec<RxPadMedItem>,
    #[serde(default)]
    pub labs: Vec<String>,
    pub follow_up: Option<String>,
    pub weight: Option<String>,
    pub bp: Option<String>,
    pub snapshot: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DispenseRequest {
    #[serde(alias = "dispensed_by", alias = "dispensedBy", default)]
    pub practitioner: Option<String>,
    pub notes: Option<String>,
    #[serde(alias = "lot_number", alias = "lotNumber", default)]
    pub lot_number: Option<String>,
    pub quantity: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatientSearchResult {
    pub id: String,
    pub name: String,
    pub age_sex: String,
    pub contact: String,
}

#[derive(Debug, Deserialize)]
pub struct PatientSearchQuery {
    pub q: Option<String>,
}

// ---------------------------------------------------------------------------
// Route Handlers
// ---------------------------------------------------------------------------

/// Ingest or update a prescription from the pad into New Loka.
pub async fn sync_prescription(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(payload): Json<RxPadSyncRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let state = state.read().await;

    let patient_name = payload.patient_name.trim();
    if patient_name.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "patientName is required"})),
        );
    }

    let now_millis = Utc::now().timestamp_millis();
    let now_iso = Utc::now().to_rfc3339();
    let today_date_str = payload
        .date
        .clone()
        .unwrap_or_else(|| Utc::now().format("%d/%m/%Y").to_string());

    // 1. Resolve or Create Patient
    let patient_id = match resolve_or_create_patient(&state, &payload).await {
        Ok(id) => id,
        Err(e) => {
            tracing::error!("Failed to resolve/create patient: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("Patient resolution failed: {}", e)})),
            );
        }
    };

    // 2. Create Consultation Encounter
    let encounter_id = uuid::Uuid::new_v4().to_string();
    let encounter_json = serde_json::json!({
        "resourceType": "Encounter",
        "id": &encounter_id,
        "status": "finished",
        "class": {
            "system": "http://terminology.hl7.org/CodeSystem/v3-ActCode",
            "code": "AMB",
            "display": "Ambulatory"
        },
        "subject": { "reference": format!("Patient/{}", patient_id) },
        "period": { "start": &now_iso, "end": &now_iso },
        "reasonCode": [{ "text": payload.diagnosis.clone().unwrap_or_else(|| "Consultation & Prescription".to_string()) }]
    });
    let _ = state
        .storage
        .store_json("Encounter", &encounter_id, &encounter_json, Some(patient_id.clone()), None)
        .await;

    // 3. Observations: Blood Pressure & Body Weight
    if let Some(ref bp_str) = payload.bp {
        let bp_clean = bp_str.trim();
        if !bp_clean.is_empty() {
            let parts: Vec<&str> = bp_clean.split('/').collect();
            if parts.len() == 2 {
                if let (Ok(sys), Ok(dia)) = (parts[0].trim().parse::<f64>(), parts[1].trim().parse::<f64>()) {
                    let bp_id = uuid::Uuid::new_v4().to_string();
                    let bp_json = serde_json::json!({
                        "resourceType": "Observation",
                        "id": &bp_id,
                        "status": "final",
                        "category": [{
                            "coding": [{
                                "system": "http://terminology.hl7.org/CodeSystem/observation-category",
                                "code": "vital-signs",
                                "display": "Vital Signs"
                            }]
                        }],
                        "code": {
                            "coding": [{
                                "system": "http://loinc.org",
                                "code": "85354-9",
                                "display": "Blood pressure panel with all children optional"
                            }],
                            "text": "Blood Pressure"
                        },
                        "subject": { "reference": format!("Patient/{}", patient_id) },
                        "encounter": { "reference": format!("Encounter/{}", encounter_id) },
                        "effectiveDateTime": &now_iso,
                        "component": [
                            {
                                "code": { "coding": [{ "system": "http://loinc.org", "code": "8480-6", "display": "Systolic blood pressure" }] },
                                "valueQuantity": { "value": sys, "unit": "mmHg" }
                            },
                            {
                                "code": { "coding": [{ "system": "http://loinc.org", "code": "8462-4", "display": "Diastolic blood pressure" }] },
                                "valueQuantity": { "value": dia, "unit": "mmHg" }
                            }
                        ]
                    });
                    let _ = state
                        .storage
                        .store_json("Observation", &bp_id, &bp_json, Some(patient_id.clone()), None)
                        .await;
                }
            }
        }
    }

    if let Some(ref weight_str) = payload.weight {
        let w_clean = weight_str.trim().trim_end_matches("kg").trim_end_matches("KG").trim();
        if let Ok(w_val) = w_clean.parse::<f64>() {
            let w_id = uuid::Uuid::new_v4().to_string();
            let w_json = serde_json::json!({
                "resourceType": "Observation",
                "id": &w_id,
                "status": "final",
                "category": [{
                    "coding": [{
                        "system": "http://terminology.hl7.org/CodeSystem/observation-category",
                        "code": "vital-signs"
                    }]
                }],
                "code": {
                    "coding": [{ "system": "http://loinc.org", "code": "29463-7", "display": "Body weight" }],
                    "text": "Weight"
                },
                "subject": { "reference": format!("Patient/{}", patient_id) },
                "encounter": { "reference": format!("Encounter/{}", encounter_id) },
                "effectiveDateTime": &now_iso,
                "valueQuantity": { "value": w_val, "unit": "kg" }
            });
            let _ = state
                .storage
                .store_json("Observation", &w_id, &w_json, Some(patient_id.clone()), None)
                .await;
        }
    }

    // 4. Condition / Clinical Diagnosis
    if let Some(ref diag) = payload.diagnosis {
        let diag_clean = diag.trim();
        if !diag_clean.is_empty() {
            let cond_id = uuid::Uuid::new_v4().to_string();
            let cond_json = serde_json::json!({
                "resourceType": "Condition",
                "id": &cond_id,
                "clinicalStatus": {
                    "coding": [{
                        "system": "http://terminology.hl7.org/CodeSystem/condition-clinical",
                        "code": "active"
                    }]
                },
                "verificationStatus": {
                    "coding": [{
                        "system": "http://terminology.hl7.org/CodeSystem/condition-ver-status",
                        "code": "confirmed"
                    }]
                },
                "code": { "text": diag_clean },
                "subject": { "reference": format!("Patient/{}", patient_id) },
                "encounter": { "reference": format!("Encounter/{}", encounter_id) },
                "recordedDate": &now_iso
            });
            let _ = state
                .storage
                .store_json("Condition", &cond_id, &cond_json, Some(patient_id.clone()), None)
                .await;
        }
    }

    // Determine serial
    let serial = match payload.serial {
        Some(s) if s > 0 => s,
        _ => compute_next_serial(&state).await,
    };

    let filename = payload.filename.clone().unwrap_or_else(|| {
        let clean_name: String = patient_name
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect();
        format!(
            "{}_{:04}_{}.pdf",
            Utc::now().format("%d%m%y"),
            serial,
            clean_name
        )
    });

    // 5. Create MedicationRequest resources
    let mut med_ids = Vec::new();
    for med in &payload.meds {
        let drug = med.drug.trim();
        if drug.is_empty() {
            continue;
        }
        let med_id = uuid::Uuid::new_v4().to_string();
        let med_json = serde_json::json!({
            "resourceType": "MedicationRequest",
            "id": &med_id,
            "status": "active",
            "intent": "order",
            "subject": { "reference": format!("Patient/{}", patient_id) },
            "encounter": { "reference": format!("Encounter/{}", encounter_id) },
            "authoredOn": &now_iso,
            "requester": { "display": payload.doctor.clone().unwrap_or_else(|| "Dr. Yash Kulkarni".to_string()) },
            "medicationCodeableConcept": { "text": drug },
            "dosageInstruction": [{
                "text": med.instr.clone().unwrap_or_default(),
                "timing": { "code": { "text": med.freq.clone().unwrap_or_default() } },
                "doseAndRate": [{ "doseQuantity": { "value": med.dose.clone().unwrap_or_default() } }]
            }],
            "note": [{ "text": format!("Duration: {}. Prescribed via eRx Pad (Rx #{:04})", med.dur.as_deref().unwrap_or(""), serial) }],
            "identifier": [
                { "system": "urn:newloka:rxpad:serial", "value": format!("{:04}", serial) },
                { "system": "urn:newloka:rxpad:filename", "value": &filename }
            ]
        });
        let _ = state
            .storage
            .store_json("MedicationRequest", &med_id, &med_json, Some(patient_id.clone()), None)
            .await;
        med_ids.push(med_id);
    }

    // Automatically update formulary in clinic database with prescribed medications
    if !payload.meds.is_empty() {
        let _ = merge_formulary_items(&state, &payload.meds).await;
    }

    // 6. Create ServiceRequest for Lab Orders
    let mut srv_ids = Vec::new();
    for lab in &payload.labs {
        let lab_name = lab.trim();
        if lab_name.is_empty() {
            continue;
        }
        let srv_id = uuid::Uuid::new_v4().to_string();
        let srv_json = serde_json::json!({
            "resourceType": "ServiceRequest",
            "id": &srv_id,
            "status": "active",
            "intent": "order",
            "subject": { "reference": format!("Patient/{}", patient_id) },
            "encounter": { "reference": format!("Encounter/{}", encounter_id) },
            "authoredOn": &now_iso,
            "code": { "text": lab_name },
            "note": [{ "text": format!("Ordered via eRx Pad (Rx #{:04})", serial) }]
        });
        let _ = state
            .storage
            .store_json("ServiceRequest", &srv_id, &srv_json, Some(patient_id.clone()), None)
            .await;
        srv_ids.push(srv_id);
    }

    // 7. Store RxPad DocumentReference Snapshot
    let prescription_id = format!("rxpad-{:04}", serial);
    let doc_json = serde_json::json!({
        "resourceType": "DocumentReference",
        "id": &prescription_id,
        "status": "current",
        "type": {
            "coding": [{
                "system": "urn:newloka:doc-type",
                "code": "rxpad-prescription",
                "display": "e-Prescription Pad Record"
            }]
        },
        "subject": { "reference": format!("Patient/{}", patient_id) },
        "date": &now_iso,
        "description": format!("Prescription #{:04} for {}", serial, patient_name),
        "content": [{
            "attachment": {
                "title": &filename,
                "url": format!("/rxpad?serial={}", serial)
            }
        }],
        "_rxpad": {
            "serial": serial,
            "filename": &filename,
            "patientName": patient_name,
            "ageSex": payload.age_sex.clone().unwrap_or_default(),
            "contact": payload.contact.clone().unwrap_or_default(),
            "date": today_date_str,
            "weight": payload.weight.clone().unwrap_or_default(),
            "bp": payload.bp.clone().unwrap_or_default(),
            "diagnosis": payload.diagnosis.clone().unwrap_or_default(),
            "followUp": payload.follow_up.clone().unwrap_or_default(),
            "meds": payload.meds,
            "labs": payload.labs,
            "doctor": payload.doctor.clone().unwrap_or_else(|| "Dr. Yash Kulkarni".to_string()),
            "regNo": payload.reg_no.clone().unwrap_or_else(|| "MMC-20260608539".to_string()),
            "dispenseStatus": "prescribed",
            "medicationRequestIds": &med_ids,
            "serviceRequestIds": &srv_ids,
            "savedAt": now_millis
        }
    });

    let _ = state
        .storage
        .store_json("DocumentReference", &prescription_id, &doc_json, Some(patient_id.clone()), None)
        .await;

    tracing::info!(
        "Synced eRx Pad prescription #{:04} for patient {} ({})",
        serial,
        patient_name,
        patient_id
    );

    // Direct PDF File Archival to Clinic folder
    if let Some(ref b64) = payload.pdf_base64 {
        let b64_clean = if let Some(idx) = b64.find(',') {
            &b64[idx + 1..]
        } else {
            b64.as_str()
        };
        if let Ok(pdf_bytes) = base64::engine::general_purpose::STANDARD.decode(b64_clean.trim()) {
            if let Ok(db_path) = std::env::var("NEWLOKA_DB_PATH") {
                let p = std::path::Path::new(&db_path);
                if let Some(dir) = p.parent() {
                    let out_pdf = dir.join(&filename);
                    if let Err(e) = std::fs::write(&out_pdf, &pdf_bytes) {
                        tracing::warn!("Failed to save prescription PDF to {}: {}", out_pdf.display(), e);
                    } else {
                        tracing::info!("Saved prescription PDF directly to {}", out_pdf.display());
                    }
                }
            }
        }
    }

    // Broadcast real-time update event to all connected New Loka clients
    let _ = state.rxpad_tx.send("prescription_created".to_string());

    let resp = RxPadSyncResponse {
        status: "success".to_string(),
        serial,
        filename,
        patient_id,
        encounter_id,
        prescription_id,
        medication_request_ids: med_ids,
        service_request_ids: srv_ids,
        message: format!("Prescription #{:04} saved and synced to clinic.db", serial),
    };

    (StatusCode::OK, Json(serde_json::to_value(resp).unwrap()))
}

/// List all prescriptions issued through the Rx Pad and New Loka.
pub async fn list_prescriptions(
    State(state): State<Arc<RwLock<AppState>>>,
) -> (StatusCode, Json<Vec<RxPadPrescriptionSummary>>) {
    let state = state.read().await;

    let docs = state
        .storage
        .search_json("DocumentReference", None)
        .await
        .unwrap_or_default();

    let mut summaries = Vec::new();

    for doc in docs {
        let is_rxpad = doc
            .get("type")
            .and_then(|t| t.get("coding"))
            .and_then(|c| c.as_array())
            .map(|arr| {
                arr.iter().any(|item| {
                    item.get("code")
                        .and_then(|c| c.as_str())
                        .map(|s| s == "rxpad-prescription")
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false);

        if !is_rxpad {
            continue;
        }

        if let Some(pad) = doc.get("_rxpad") {
            let id = doc
                .get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let serial = pad.get("serial").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let date = pad
                .get("date")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let patient_id = doc
                .get("subject")
                .and_then(|s| s.get("reference"))
                .and_then(|r| r.as_str())
                .map(|s| s.replace("Patient/", ""))
                .unwrap_or_default();
            let patient_name = pad
                .get("patientName")
                .and_then(|v| v.as_str())
                .unwrap_or("Unnamed")
                .to_string();
            let age_sex = pad
                .get("ageSex")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let contact = pad
                .get("contact")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let diagnosis = pad
                .get("diagnosis")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let filename = pad
                .get("filename")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let dispense_status = pad
                .get("dispenseStatus")
                .and_then(|v| v.as_str())
                .unwrap_or("prescribed")
                .to_string();
            let dispensed_at = pad.get("dispensedAt").and_then(|v| v.as_i64());
            let dispensed_by = pad
                .get("dispensedBy")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let meds_vec: Vec<RxPadMedItem> = pad
                .get("meds")
                .and_then(|m| serde_json::from_value(m.clone()).ok())
                .unwrap_or_default();
            let meds_count = meds_vec.len();
            let meds_summary = meds_vec
                .iter()
                .filter_map(|m| if m.drug.trim().is_empty() { None } else { Some(m.drug.as_str()) })
                .take(3)
                .collect::<Vec<_>>()
                .join(", ");

            let labs_vec: Vec<String> = pad
                .get("labs")
                .and_then(|l| serde_json::from_value(l.clone()).ok())
                .unwrap_or_default();
            let labs_count = labs_vec.len();

            let follow_up = pad
                .get("followUp")
                .or_else(|| pad.get("follow_up"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let weight = pad
                .get("weight")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let bp = pad
                .get("bp")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            summaries.push(RxPadPrescriptionSummary {
                id,
                serial,
                date,
                patient_id,
                patient_name,
                age_sex,
                contact,
                diagnosis,
                meds_count,
                meds_summary,
                labs_count,
                filename,
                dispense_status,
                dispensed_at,
                dispensed_by,
                meds: meds_vec,
                labs: labs_vec,
                follow_up,
                weight,
                bp,
                snapshot: Some(pad.clone()),
            });
        }
    }

    summaries.sort_by(|a, b| b.serial.cmp(&a.serial));
    (StatusCode::OK, Json(summaries))
}

/// Retrieve full details of a specific prescription.
pub async fn get_prescription(
    State(state): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    let state = state.read().await;
    match state.storage.get_json(&id).await {
        Ok(Some(doc)) => (StatusCode::OK, Json(doc)),
        _ => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("Prescription {} not found", id)})),
        ),
    }
}

/// Fill / Dispense a prescription from New Loka.
/// Creates a MedicationDispense resource and marks the prescription as dispensed.
pub async fn dispense_prescription(
    State(state): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
    Json(payload): Json<DispenseRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let state = state.read().await;

    let mut doc = match state.storage.get_json(&id).await {
        Ok(Some(d)) => d,
        _ => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Prescription not found"})),
            );
        }
    };

    let now_millis = Utc::now().timestamp_millis();
    let now_iso = Utc::now().to_rfc3339();
    let practitioner = payload
        .practitioner
        .unwrap_or_else(|| "Dr. Yash Kulkarni".to_string());
    let quantity = payload
        .quantity
        .unwrap_or_else(|| "Full prescribed quantity".to_string());
    let lot_number = payload.lot_number.unwrap_or_else(|| "LOT-2026-09".to_string());
    let notes = payload.notes.unwrap_or_else(|| "Dispensed at clinic pharmacy".to_string());

    let patient_id = doc
        .get("subject")
        .and_then(|s| s.get("reference"))
        .and_then(|r| r.as_str())
        .map(|s| s.replace("Patient/", ""))
        .unwrap_or_default();

    let mut dispense_ids = Vec::new();

    // Create MedicationDispense for each medication
    if let Some(med_ids) = doc
        .get("_rxpad")
        .and_then(|p| p.get("medicationRequestIds"))
        .and_then(|m| m.as_array())
    {
        for mid in med_ids {
            if let Some(mid_str) = mid.as_str() {
                // Fetch the MedicationRequest
                let drug_name = if let Ok(Some(med_req)) = state.storage.get_json(mid_str).await {
                    med_req
                        .get("medicationCodeableConcept")
                        .and_then(|m| m.get("text"))
                        .and_then(|t| t.as_str())
                        .unwrap_or("Medication")
                        .to_string()
                } else {
                    "Medication".to_string()
                };

                let dispense_id = uuid::Uuid::new_v4().to_string();
                let dispense_json = serde_json::json!({
                    "resourceType": "MedicationDispense",
                    "id": &dispense_id,
                    "status": "completed",
                    "medicationCodeableConcept": { "text": drug_name },
                    "subject": { "reference": format!("Patient/{}", patient_id) },
                    "authorizingPrescription": [{ "reference": format!("MedicationRequest/{}", mid_str) }],
                    "quantity": { "value": &quantity },
                    "whenHandedOver": &now_iso,
                    "performer": [{ "actor": { "display": &practitioner } }],
                    "note": [{ "text": format!("Lot: {}. {}", lot_number, notes) }]
                });

                let _ = state
                    .storage
                    .store_json(
                        "MedicationDispense",
                        &dispense_id,
                        &dispense_json,
                        Some(patient_id.clone()),
                        None,
                    )
                    .await;
                dispense_ids.push(dispense_id);

                // Update MedicationRequest status to "completed"
                if let Ok(Some(mut med_req)) = state.storage.get_json(mid_str).await {
                    med_req["status"] = serde_json::json!("completed");
                    let _ = state
                        .storage
                        .store_json(
                            "MedicationRequest",
                            mid_str,
                            &med_req,
                            Some(patient_id.clone()),
                            None,
                        )
                        .await;
                }
            }
        }
    }

    // Update prescription document
    if let Some(pad) = doc.get_mut("_rxpad") {
        pad["dispenseStatus"] = serde_json::json!("dispensed");
        pad["dispensedAt"] = serde_json::json!(now_millis);
        pad["dispensedBy"] = serde_json::json!(&practitioner);
        pad["dispenseIds"] = serde_json::json!(dispense_ids);
    }

    let _ = state
        .storage
        .store_json(
            "DocumentReference",
            &id,
            &doc,
            Some(patient_id.clone()),
            None,
        )
        .await;

    tracing::info!("Prescription {} dispensed by {}", id, practitioner);

    // Broadcast real-time update event
    let _ = state.rxpad_tx.send("prescription_dispensed".to_string());

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "dispensed",
            "prescription_id": id,
            "dispensed_by": practitioner,
            "dispensed_at": now_iso,
            "dispense_ids": dispense_ids,
            "message": "Prescription successfully filled and dispensed in clinic database."
        })),
    )
}

/// Returns the next continuous serial number for the clinic.
pub async fn get_next_serial(
    State(state): State<Arc<RwLock<AppState>>>,
) -> (StatusCode, Json<serde_json::Value>) {
    let state = state.read().await;
    let next_serial = compute_next_serial(&state).await;
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "next_serial": next_serial,
            "formatted": format!("{:04}", next_serial)
        })),
    )
}

/// Search existing patients for autocomplete on the pad.
pub async fn search_patients(
    State(state): State<Arc<RwLock<AppState>>>,
    Query(q): Query<PatientSearchQuery>,
) -> (StatusCode, Json<Vec<PatientSearchResult>>) {
    let state = state.read().await;
    let query_str = q.q.unwrap_or_default().trim().to_lowercase();

    let patients = state
        .storage
        .search_json("Patient", None)
        .await
        .unwrap_or_default();

    let mut results = Vec::new();

    for p in patients {
        let id = p.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let name = p
            .get("name")
            .and_then(|n| n.as_array())
            .and_then(|arr| arr.first())
            .and_then(|n| n.get("text").or_else(|| n.get("family")))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let contact = p
            .get("telecom")
            .and_then(|t| t.as_array())
            .and_then(|arr| arr.first())
            .and_then(|t| t.get("value"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let gender = p.get("gender").and_then(|v| v.as_str()).unwrap_or("");
        let birth_date = p.get("birthDate").and_then(|v| v.as_str()).unwrap_or("");
        let age_str = if !birth_date.is_empty() {
            if let Ok(year) = birth_date[..4.min(birth_date.len())].parse::<i32>() {
                let current_year = 2026;
                format!("{}y", current_year - year)
            } else {
                "".to_string()
            }
        } else {
            "".to_string()
        };

        let age_sex = if !age_str.is_empty() && !gender.is_empty() {
            format!("{} / {}", age_str, &gender[..1].to_uppercase())
        } else if !gender.is_empty() {
            gender.to_string()
        } else {
            age_str
        };

        if query_str.is_empty()
            || name.to_lowercase().contains(&query_str)
            || contact.contains(&query_str)
        {
            results.push(PatientSearchResult {
                id,
                name,
                age_sex,
                contact,
            });
        }
    }

    results.truncate(15);
    (StatusCode::OK, Json(results))
}

/// Get the clinic's centralized formulary.
pub async fn get_formulary(
    State(state): State<Arc<RwLock<AppState>>>,
) -> (StatusCode, Json<Vec<RxPadMedItem>>) {
    let state = state.read().await;
    let items = fetch_formulary_items(&state).await;
    (StatusCode::OK, Json(items))
}

/// Update or merge into the clinic's centralized formulary.
pub async fn update_formulary(
    State(state): State<Arc<RwLock<AppState>>>,
    Json(payload): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    let state = state.read().await;
    let incoming_items: Vec<RxPadMedItem> = if let Ok(items) = serde_json::from_value(payload.clone()) {
        items
    } else if let Some(items) = payload.get("items") {
        serde_json::from_value(items.clone()).unwrap_or_default()
    } else {
        vec![]
    };

    let updated = merge_formulary_items(&state, &incoming_items).await;
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "success",
            "count": updated.len(),
            "items": updated
        })),
    )
}

/// Delete a prescription and its associated clinical entities.
pub async fn delete_prescription(
    State(state): State<Arc<RwLock<AppState>>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    let state = state.read().await;
    let doc = match state.storage.get_json(&id).await {
        Ok(Some(d)) => d,
        _ => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": format!("Prescription {} not found", id)})),
            );
        }
    };

    let mut deleted_count = 0;
    if let Some(pad) = doc.get("_rxpad") {
        if let Some(med_ids) = pad.get("medicationRequestIds").and_then(|m| m.as_array()) {
            for mid in med_ids {
                if let Some(mid_str) = mid.as_str() {
                    if state.storage.soft_delete(mid_str).await.is_ok() {
                        deleted_count += 1;
                    }
                }
            }
        }
        if let Some(srv_ids) = pad.get("serviceRequestIds").and_then(|m| m.as_array()) {
            for sid in srv_ids {
                if let Some(sid_str) = sid.as_str() {
                    if state.storage.soft_delete(sid_str).await.is_ok() {
                        deleted_count += 1;
                    }
                }
            }
        }
        if let Some(disp_ids) = pad.get("dispenseIds").and_then(|m| m.as_array()) {
            for did in disp_ids {
                if let Some(did_str) = did.as_str() {
                    if state.storage.soft_delete(did_str).await.is_ok() {
                        deleted_count += 1;
                    }
                }
            }
        }
    }

    let _ = state.storage.soft_delete(&id).await;
    deleted_count += 1;

    // Broadcast real-time update event
    let _ = state.rxpad_tx.send("prescription_deleted".to_string());

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "deleted",
            "prescription_id": id,
            "deleted_resources": deleted_count
        })),
    )
}

/// Serve the prescription pad HTML directly to the browser.
pub async fn serve_rxpad() -> Html<String> {
    let local_path = std::path::Path::new(r"D:\Medical\Clinic\eRx Pad Pro.html");
    if local_path.exists() {
        if let Ok(content) = std::fs::read_to_string(local_path) {
            return Html(content);
        }
    }

    // Fallback if file path differs
    Html(include_str!("../../newloka_web/index.html").to_string())
}

/// Real-time Server-Sent Events stream for eRx Pad updates.
pub async fn rxpad_events_handler(
    State(state): State<Arc<RwLock<AppState>>>,
) -> Sse<impl Stream<Item = Result<Event, std::convert::Infallible>>> {
    let rx = {
        let st = state.read().await;
        st.rxpad_tx.subscribe()
    };

    let stream = BroadcastStream::new(rx).filter_map(|msg| match msg {
        Ok(m) => Some(Ok(Event::default().data(m))),
        Err(_) => None,
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}


// ---------------------------------------------------------------------------
// Helper Functions
// ---------------------------------------------------------------------------

async fn compute_next_serial(state: &AppState) -> u32 {
    let docs = state
        .storage
        .search_json("DocumentReference", None)
        .await
        .unwrap_or_default();

    let mut max_serial: u32 = 0;
    for doc in docs {
        if let Some(pad) = doc.get("_rxpad") {
            if let Some(s) = pad.get("serial").and_then(|v| v.as_u64()) {
                if (s as u32) > max_serial {
                    max_serial = s as u32;
                }
            }
        }
    }

    // If max_serial is 0, check if we had existing historical PDFs 0001, 0002, 0003
    if max_serial < 3 {
        3 + 1
    } else {
        max_serial + 1
    }
}

async fn resolve_or_create_patient(
    state: &AppState,
    req: &RxPadSyncRequest,
) -> anyhow::Result<String> {
    let patient_name = req.patient_name.trim();
    let contact_str = req.contact.as_deref().unwrap_or("").trim();

    // Check existing patients
    let existing = state
        .storage
        .search_json("Patient", None)
        .await
        .unwrap_or_default();

    for p in existing {
        let name = p
            .get("name")
            .and_then(|n| n.as_array())
            .and_then(|arr| arr.first())
            .and_then(|n| n.get("text").or_else(|| n.get("family")))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let contact = p
            .get("telecom")
            .and_then(|t| t.as_array())
            .and_then(|arr| arr.first())
            .and_then(|t| t.get("value"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if name.eq_ignore_ascii_case(patient_name)
            || (!contact_str.is_empty() && contact == contact_str)
        {
            if let Some(id) = p.get("id").and_then(|v| v.as_str()) {
                return Ok(id.to_string());
            }
        }
    }

    // Create new patient
    let patient_id = uuid::Uuid::new_v4().to_string();
    let parts: Vec<&str> = patient_name.split_whitespace().collect();
    let (given, family) = if parts.len() > 1 {
        let given_parts = parts[..parts.len() - 1].to_vec();
        let family_part = parts.last().unwrap_or(&"");
        (given_parts, *family_part)
    } else {
        (vec![], patient_name)
    };

    let mut gender = "unknown";
    let mut birth_date = "".to_string();
    if let Some(ref as_str) = req.age_sex {
        let as_lower = as_str.to_lowercase();
        if as_lower.contains('f') || as_lower.contains("female") {
            gender = "female";
        } else if as_lower.contains('m') || as_lower.contains("male") {
            gender = "male";
        }

        // Extract numbers for age
        let digits: String = as_str.chars().filter(|c| c.is_ascii_digit()).collect();
        if let Ok(age) = digits.parse::<i32>() {
            let current_year = 2026;
            birth_date = format!("{}-01-01", current_year - age);
        }
    }

    let patient_json = serde_json::json!({
        "resourceType": "Patient",
        "id": &patient_id,
        "identifier": [{
            "system": "urn:newloka:mrn",
            "value": format!("MRN-{}", &patient_id[..6].to_uppercase())
        }],
        "active": true,
        "name": [{
            "text": patient_name,
            "family": family,
            "given": given
        }],
        "gender": gender,
        "birthDate": birth_date,
        "telecom": [{
            "system": "phone",
            "value": contact_str
        }]
    });

    state
        .storage
        .store_json("Patient", &patient_id, &patient_json, None, None)
        .await?;

    Ok(patient_id)
}

const FORMULARY_DOC_ID: &str = "rxpad-formulary";

async fn fetch_formulary_items(state: &AppState) -> Vec<RxPadMedItem> {
    if let Ok(Some(doc)) = state.storage.get_json(FORMULARY_DOC_ID).await {
        if let Some(items) = doc.get("items") {
            if let Ok(list) = serde_json::from_value::<Vec<RxPadMedItem>>(items.clone()) {
                if !list.is_empty() {
                    return list;
                }
            }
        }
    }

    // Default clinic formulary
    vec![
        RxPadMedItem {
            drug: "NITROFURANTOIN 100MG".to_string(),
            dose: Some("1 Tab".to_string()),
            freq: Some("BD".to_string()),
            dur: Some("5d".to_string()),
            instr: Some("After breakfast and dinner".to_string()),
        },
        RxPadMedItem {
            drug: "ZINCOVIT (VIT C) 500MG".to_string(),
            dose: Some("1 Tab".to_string()),
            freq: Some("OD".to_string()),
            dur: Some("15d".to_string()),
            instr: Some("After lunch".to_string()),
        },
        RxPadMedItem {
            drug: "T. FEXOFENADINE 180MG".to_string(),
            dose: Some("1 tab".to_string()),
            freq: Some("OD".to_string()),
            dur: Some("10 d".to_string()),
            instr: Some("Before breakfast / Disp: Send 10 such tablets".to_string()),
        },
        RxPadMedItem {
            drug: "T. VIT B-COMPLEX".to_string(),
            dose: Some("1 tab".to_string()),
            freq: Some("OD".to_string()),
            dur: Some("10 d".to_string()),
            instr: Some("After breakfast / Disp: Send 10 such tablets".to_string()),
        },
        RxPadMedItem {
            drug: "T. CAL+VITD3".to_string(),
            dose: Some("1 tab".to_string()),
            freq: Some("OD".to_string()),
            dur: Some("10 d".to_string()),
            instr: Some("After lunch / Disp: Send 10 such tablets".to_string()),
        },
        RxPadMedItem {
            drug: "T. RIZATRIPTAN 10MG".to_string(),
            dose: Some("1 tab".to_string()),
            freq: Some("PRN".to_string()),
            dur: None,
            instr: Some("Min 2 hr b/w tabs, max 3/day / Disp: Send 8 such tablets".to_string()),
        },
        RxPadMedItem {
            drug: "T. VALACICLOVIR 1G".to_string(),
            dose: Some("1 tab".to_string()),
            freq: Some("BD".to_string()),
            dur: Some("10d".to_string()),
            instr: Some("after meals".to_string()),
        },
        RxPadMedItem {
            drug: "T. VALACICLOVIR 500MG".to_string(),
            dose: Some("1 tab".to_string()),
            freq: Some("BD".to_string()),
            dur: Some("5d".to_string()),
            instr: Some("after meals (FOR FLARES)".to_string()),
        },
        RxPadMedItem {
            drug: "INJ BENZATHINE PENICILLIN G 2.4M IU".to_string(),
            dose: Some("1 inj".to_string()),
            freq: Some("1 weekly".to_string()),
            dur: Some("3wk".to_string()),
            instr: Some("inj deep IM".to_string()),
        },
    ]
}

async fn merge_formulary_items(state: &AppState, new_items: &[RxPadMedItem]) -> Vec<RxPadMedItem> {
    let mut current = fetch_formulary_items(state).await;

    for item in new_items {
        let clean_drug = item.drug.trim();
        if clean_drug.is_empty() {
            continue;
        }

        if let Some(existing) = current
            .iter_mut()
            .find(|x| x.drug.eq_ignore_ascii_case(clean_drug))
        {
            if let Some(ref d) = item.dose {
                if !d.trim().is_empty() {
                    existing.dose = Some(d.trim().to_string());
                }
            }
            if let Some(ref f) = item.freq {
                if !f.trim().is_empty() {
                    existing.freq = Some(f.trim().to_string());
                }
            }
            if let Some(ref dur) = item.dur {
                if !dur.trim().is_empty() {
                    existing.dur = Some(dur.trim().to_string());
                }
            }
            if let Some(ref instr) = item.instr {
                if !instr.trim().is_empty() {
                    existing.instr = Some(instr.trim().to_string());
                }
            }
        } else {
            current.push(RxPadMedItem {
                drug: clean_drug.to_string(),
                dose: item.dose.as_ref().map(|s| s.trim().to_string()),
                freq: item.freq.as_ref().map(|s| s.trim().to_string()),
                dur: item.dur.as_ref().map(|s| s.trim().to_string()),
                instr: item.instr.as_ref().map(|s| s.trim().to_string()),
            });
        }
    }

    let doc_json = serde_json::json!({
        "resourceType": "DocumentReference",
        "id": FORMULARY_DOC_ID,
        "status": "current",
        "type": {
            "coding": [{
                "system": "urn:newloka:doc-type",
                "code": "rxpad-formulary",
                "display": "eRx Pad Formulary"
            }]
        },
        "description": "Clinic Drug Formulary and Order Defaults",
        "items": &current
    });

    let _ = state
        .storage
        .store_json("DocumentReference", FORMULARY_DOC_ID, &doc_json, None, None)
        .await;

    current
}
