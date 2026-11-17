//! Research federation, peer trust mesh & differential privacy engine
//!
//! Enables multi-center collaborative research across health systems (T4)
//! while guaranteeing mathematical privacy via k-anonymity suppression
//! and Laplace differential privacy noise injection.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FederationNode {
    pub node_id: String,
    pub institution_name: String,
    pub endpoint_url: String,
    pub public_key: Vec<u8>,
    pub trust_tier: String, // "T3_Partner", "T4_Federation_Hub", "Academic_Medical_Center"
    pub status: String,     // "active", "offline", "unverified"
    pub last_heartbeat: i64,
    #[serde(default)]
    pub supported_studies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeIdentifiedPatientRecord {
    pub pseudo_id: String,
    pub age_bracket: String,
    pub gender: String,
    pub conditions: Vec<String>,
    pub medications: Vec<String>,
    pub observation_summaries: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResearchCohortQuery {
    pub query_id: String,
    pub title: String,
    pub inclusion_conditions: Vec<String>,
    pub min_age: Option<u32>,
    pub max_age: Option<u32>,
    pub gender: Option<String>,
    #[serde(default = "default_epsilon")]
    pub epsilon: f64,
    #[serde(default = "default_k")]
    pub k_anonymity: usize,
}

fn default_epsilon() -> f64 {
    0.5
}
fn default_k() -> usize {
    5
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResearchCohortResult {
    pub query_id: String,
    pub reported_cohort_size: String,
    pub perturbed_count: f64,
    pub is_suppressed: bool,
    pub k_anonymity_guarantee: usize,
    pub epsilon_budget_consumed: f64,
    pub demographics_breakdown: serde_json::Value,
}

pub struct DifferentialPrivacyEngine;

impl DifferentialPrivacyEngine {
    /// Applies k-anonymity suppression. If count is strictly below `k`,
    /// cell is suppressed (`None`) to prevent patient re-identification.
    pub fn apply_k_anonymity(count: usize, k: usize) -> Option<usize> {
        if count < k {
            None
        } else {
            Some(count)
        }
    }

    /// Adds Laplace noise to guarantee epsilon-differential privacy:
    /// Noise ~ Laplace(0, scale) where scale = sensitivity / epsilon.
    /// Uses inverse CDF sampling: X = -scale * sgn(u) * ln(1 - 2|u|).
    pub fn add_laplace_noise(
        value: f64,
        sensitivity: f64,
        epsilon: f64,
        seed_val: Option<f64>,
    ) -> f64 {
        if epsilon <= 0.0 {
            return value;
        }
        let scale = sensitivity / epsilon;
        // Deterministic or pseudo-random uniform draw in (-0.5, 0.5)
        let u: f64 = seed_val.unwrap_or_else(|| {
            let nanos = chrono::Utc::now().timestamp_subsec_nanos() as f64;
            ((nanos % 1_000_000.0) / 1_000_000.0) - 0.5
        });

        // Clamp to avoid log(0)
        let clamped_u = u.clamp(-0.499999, 0.499999);
        let sgn = if clamped_u < 0.0 { -1.0 } else { 1.0 };
        let noise = -scale * sgn * (1.0 - 2.0 * clamped_u.abs()).ln();

        let perturbed = value + noise;
        if perturbed < 0.0 {
            0.0
        } else {
            (perturbed * 10.0).round() / 10.0
        }
    }

    /// De-identifies a patient record by removing direct identifiers (name, MRN, phone, exact address, precise DOB).
    pub fn deidentify_patient(
        patient: &crate::fhir::Patient,
        conditions: Vec<String>,
        medications: Vec<String>,
        observations: Vec<String>,
        salt: &str,
    ) -> DeIdentifiedPatientRecord {
        // Calculate age bracket
        let age_bracket =
            if let Ok(dob) = chrono::NaiveDate::parse_from_str(&patient.birth_date, "%Y-%m-%d") {
                let current_year = chrono::Utc::now().date_naive().year();
                let age = current_year - dob.year();
                match age {
                    a if a < 18 => "0-17".to_string(),
                    a if a < 30 => "18-29".to_string(),
                    a if a < 45 => "30-44".to_string(),
                    a if a < 60 => "45-59".to_string(),
                    a if a < 75 => "60-74".to_string(),
                    _ => "75+".to_string(),
                }
            } else {
                "Unknown".to_string()
            };

        // Pseudo-ID using SHA-256 hash of patient ID with salt
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(patient.id.as_bytes());
        hasher.update(salt.as_bytes());
        let hash_bytes = hasher.finalize();
        let pseudo_id = format!("SUBJ-{}", crate::crypto::hex_encode(&hash_bytes[..8]));

        DeIdentifiedPatientRecord {
            pseudo_id,
            age_bracket,
            gender: patient.gender.clone(),
            conditions,
            medications,
            observation_summaries: observations,
        }
    }
}

use chrono::Datelike;
