//! Multi-department hospital architecture & inter-department patient transfers
//!
//! Enforces departmental data silos, structured clinical handovers (SBAR),
//! and patient movement between hospital wards and specialized departments.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Department {
    pub id: String,
    pub name: String,
    pub code: String,
    pub head_user_id: Option<String>,
    pub head_user_name: Option<String>,
    pub bed_count: u32,
    pub active_patients_count: u32,
    pub sub_specialties: Vec<String>,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferStatus {
    Requested,
    Accepted,
    Completed,
    Rejected,
    Cancelled,
}

impl TransferStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TransferStatus::Requested => "requested",
            TransferStatus::Accepted => "accepted",
            TransferStatus::Completed => "completed",
            TransferStatus::Rejected => "rejected",
            TransferStatus::Cancelled => "cancelled",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DepartmentTransferRequest {
    pub id: String,
    pub patient_id: String,
    pub patient_name: String,
    pub mrn: Option<String>,
    pub source_department: String,
    pub target_department: String,
    pub requesting_clinician_id: String,
    pub requesting_clinician_name: String,
    pub accepting_clinician_id: Option<String>,
    pub accepting_clinician_name: Option<String>,
    pub assigned_bed: Option<String>,
    pub urgency: String, // "routine", "urgent", "stat"
    pub reason: String,
    pub clinical_handover_notes: String,
    #[serde(default)]
    pub active_medications_summary: Vec<String>,
    #[serde(default)]
    pub pending_orders_summary: Vec<String>,
    #[serde(default)]
    pub allergies_summary: Vec<String>,
    pub status: TransferStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub handover_signed_by: Option<String>,
}

/// Returns the standard default departments for an enterprise hospital (T3/T4).
pub fn default_departments() -> Vec<Department> {
    vec![
        Department {
            id: "dept-emerg".into(),
            name: "Emergency & Trauma".into(),
            code: "EMERG".into(),
            head_user_id: Some("doc-emergency".into()),
            head_user_name: Some("Dr. Emergency Chief".into()),
            bed_count: 24,
            active_patients_count: 8,
            sub_specialties: vec!["Resuscitation".into(), "Triage".into(), "Trauma Bay".into()],
            description: "Acute emergency admissions, rapid triage, and resuscitation".into(),
        },
        Department {
            id: "dept-genmed".into(),
            name: "General Internal Medicine".into(),
            code: "MED".into(),
            head_user_id: Some("doc-depthead".into()),
            head_user_name: Some("Dr. HOD Medicine".into()),
            bed_count: 60,
            active_patients_count: 42,
            sub_specialties: vec!["Infectious Disease".into(), "Endocrinology".into(), "Geriatrics".into()],
            description: "Adult inpatient ward, chronic disease management, and diagnostic workups".into(),
        },
        Department {
            id: "dept-cardio".into(),
            name: "Cardiology & Cath Lab".into(),
            code: "CARD".into(),
            head_user_id: Some("doc-cardio".into()),
            head_user_name: Some("Dr. Cardio Specialist".into()),
            bed_count: 35,
            active_patients_count: 28,
            sub_specialties: vec!["Interventional".into(), "Electrophysiology".into(), "Heart Failure".into()],
            description: "Cardiac care unit, catheterization laboratory, and non-invasive telemetry".into(),
        },
        Department {
            id: "dept-surg".into(),
            name: "General Surgery & OT".into(),
            code: "SURG".into(),
            head_user_id: Some("doc-surgeon".into()),
            head_user_name: Some("Dr. Surgical Director".into()),
            bed_count: 45,
            active_patients_count: 30,
            sub_specialties: vec!["Laparoscopy".into(), "Trauma Surgery".into(), "Surgical ICU".into()],
            description: "Operating theaters, perioperative nursing, and surgical inpatient ward".into(),
        },
        Department {
            id: "dept-icu".into(),
            name: "Intensive Care Unit (ICU)".into(),
            code: "ICU".into(),
            head_user_id: Some("doc-intensivist".into()),
            head_user_name: Some("Dr. Chief Intensivist".into()),
            bed_count: 20,
            active_patients_count: 17,
            sub_specialties: vec!["Medical ICU".into(), "Neuro ICU".into(), "Ventilator Care".into()],
            description: "Critical care hemodynamic monitoring, continuous mechanical ventilation, and multi-organ support".into(),
        },
        Department {
            id: "dept-onco".into(),
            name: "Medical Oncology".into(),
            code: "ONC".into(),
            head_user_id: Some("doc-oncologist".into()),
            head_user_name: Some("Dr. Lead Oncologist".into()),
            bed_count: 30,
            active_patients_count: 19,
            sub_specialties: vec!["Chemotherapy Day Care".into(), "Hematology Oncology".into(), "Palliative Care".into()],
            description: "Chemotherapy infusion suites, oncology staging, and supportive cancer care".into(),
        },
        Department {
            id: "dept-rad".into(),
            name: "Radiology & Imaging".into(),
            code: "RAD".into(),
            head_user_id: Some("doc-radiologist".into()),
            head_user_name: Some("Dr. Chief Radiologist".into()),
            bed_count: 0,
            active_patients_count: 0,
            sub_specialties: vec!["CT".into(), "MRI".into(), "Ultrasound".into(), "Digital X-Ray".into()],
            description: "Diagnostic medical imaging, PACS storage, and interventional radiology".into(),
        },
        Department {
            id: "dept-path".into(),
            name: "Pathology & Biochemistry Labs".into(),
            code: "PATH".into(),
            head_user_id: Some("doc-pathologist".into()),
            head_user_name: Some("Dr. Head Pathologist".into()),
            bed_count: 0,
            active_patients_count: 0,
            sub_specialties: vec!["Hematology".into(), "Biochemistry".into(), "Histopathology".into(), "Microbiology".into()],
            description: "Clinical diagnostic testing, specimen accessioning, and automated analyzer benches".into(),
        },
        Department {
            id: "dept-pharm".into(),
            name: "Hospital Pharmacy & Dispensary".into(),
            code: "PHARM".into(),
            head_user_id: Some("pharm-head".into()),
            head_user_name: Some("Chief Pharmacist".into()),
            bed_count: 0,
            active_patients_count: 0,
            sub_specialties: vec!["Inpatient Dispense".into(), "Outpatient Pharmacy".into(), "Formulary & Buffer Stock".into()],
            description: "Drug dispensing queues, interaction checks, IV compounding, and inventory control".into(),
        },
    ]
}
