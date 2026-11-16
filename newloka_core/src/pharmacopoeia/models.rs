//! Pharmacopoeia Data Models
//!
//! Structured representations of Indian Pharmacopoeia (IP 2022) monographs,
//! National Formulary of India (NFI), regulatory schedules, clinical dosing,
//! contraindications, and evidence-based standardized order sets.

use serde::{Deserialize, Serialize};

/// Dosage form and route details for a pharmacopoeial formulation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DosageForm {
    pub form: String,
    pub strengths: Vec<String>,
    pub route: String,
}

/// A comprehensive pharmacopoeial drug entry / monograph.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PharmacopoeiaEntry {
    pub id: String,
    pub inn_name: String,
    pub ip_title: String,
    pub schedule: String,
    pub therapeutic_category: String,
    pub dosage_forms: Vec<DosageForm>,
    pub default_dose: String,
    pub default_frequency: String,
    pub default_duration: String,
    pub default_instructions: String,
    pub max_daily_dose: Option<String>,
    pub renal_dose_adjustment: Option<String>,
    pub contraindications: Vec<String>,
    pub pregnancy_category: String,
    pub nlem_level: Option<Vec<String>>,
    pub snomed_ct_code: Option<String>,
    pub common_brands_india: Vec<String>,
}

/// An individual order item inside a standardized clinical order set.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrderSetItem {
    pub id: String,
    pub item_type: String, // "medication", "laboratory", "radiology", "nursing", "diet"
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pharmacopoeia_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dose: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<String>,
    pub instructions: String,
    pub urgency: String, // "STAT", "Routine", "Urgent"
    pub is_default_checked: bool,
}

/// A standardized evidence-based clinical order set (protocol).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClinicalOrderSet {
    pub id: String,
    pub name: String,
    pub category: String,
    pub description: String,
    pub evidence_grade: String,
    pub items: Vec<OrderSetItem>,
}

/// Top-level container for the embedded Indian Pharmacopoeia reference database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PharmacopoeiaDatabase {
    pub version: String,
    pub published_date: String,
    pub reference_standard: String,
    #[serde(default)]
    pub total_monographs: usize,
    #[serde(default)]
    pub total_order_sets: usize,
    pub entries: Vec<PharmacopoeiaEntry>,
    pub order_sets: Vec<ClinicalOrderSet>,
}

/// Search result item returned by the pharmacopoeia index.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PharmacopoeiaSearchResult {
    pub entry: PharmacopoeiaEntry,
    pub matched_via: String, // "generic", "brand", "ip_title", "category"
    pub matched_brand: Option<String>,
    pub score: u32,
}

/// Summary of a therapeutic category with count of monographs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CategorySummary {
    pub name: String,
    pub count: usize,
}

/// Drug safety warning level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SafetyWarningLevel {
    Info,
    Moderate,
    Critical,
}

/// Individual clinical safety warning for prescription review.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DrugSafetyWarning {
    pub level: SafetyWarningLevel,
    pub warning_type: String, // "AllergyCrossReactivity", "DuplicateTherapy", "Contraindication", "ScheduleRestriction"
    pub drug_name: String,
    pub message: String,
    pub clinical_rationale: String,
}

/// Comprehensive safety report for an order set or prescription draft.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct DrugSafetyReport {
    pub has_critical: bool,
    pub warnings: Vec<DrugSafetyWarning>,
}
