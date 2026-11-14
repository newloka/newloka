//! Consent and privacy governance
//!
//! Enforces patient consent directives for inter-facility transfers,
//! ABDM/ABHA exchange, and privacy-preserving research queries.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConsentDirective {
    pub id: String,
    pub patient_id: String,
    pub status: String, // "active", "revoked", "expired", "draft"
    pub scope: String,  // "research", "treatment-transfer", "emergency", "general"
    pub purpose: String, // "RESEARCH", "TREATMENT_TRANSFER", "EMERGENCY_OVERRIDE", "CARE_COORDINATION"
    pub recipient_node_id: Option<String>, // Specific node or None for all federation nodes
    pub recipient_name: Option<String>,
    pub data_categories: Vec<String>, // e.g. ["Observation", "Condition", "DiagnosticReport", "ALL"]
    pub valid_from: DateTime<Utc>,
    pub valid_to: DateTime<Utc>,
    pub revoked: bool,
    pub revocation_reason: Option<String>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub digital_signature: Option<String>,
    pub consent_notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConsentDecision {
    Permit {
        directive_id: String,
        reason: String,
    },
    Deny {
        reason: String,
    },
}

pub struct ConsentEngine;

impl ConsentEngine {
    /// Evaluates whether a patient record can be transferred or accessed based on active consent directives.
    pub fn check_consent(
        directives: &[ConsentDirective],
        patient_id: &str,
        recipient_node_id: &str,
        purpose: &str,
        resource_type: &str,
        now: DateTime<Utc>,
    ) -> ConsentDecision {
        // Find matching active directives for this patient
        let matching: Vec<&ConsentDirective> = directives
            .iter()
            .filter(|d| {
                d.patient_id == patient_id
                    && !d.revoked
                    && d.status == "active"
                    && d.valid_from <= now
                    && d.valid_to >= now
            })
            .collect();

        if matching.is_empty() {
            return ConsentDecision::Deny {
                reason: format!("No active consent directive found for patient {}", patient_id),
            };
        }

        for d in matching {
            // Check purpose
            let purpose_matches = d.purpose == "*"
                || d.purpose.eq_ignore_ascii_case(purpose)
                || (purpose == "RESEARCH" && d.scope == "research")
                || (purpose == "TREATMENT_TRANSFER" && d.scope == "treatment-transfer");

            if !purpose_matches {
                continue;
            }

            // Check recipient
            let recipient_matches = match &d.recipient_node_id {
                Some(r) => r == "*" || r == recipient_node_id,
                None => true, // Wildcard/open federation
            };

            if !recipient_matches {
                continue;
            }

            // Check data category
            let category_matches = d.data_categories.iter().any(|c| {
                c == "*" || c == "ALL" || c.eq_ignore_ascii_case(resource_type)
            });

            if !category_matches {
                continue;
            }

            return ConsentDecision::Permit {
                directive_id: d.id.clone(),
                reason: format!(
                    "Consent directive {} permits '{}' to '{}' for {}",
                    d.id, purpose, recipient_node_id, resource_type
                ),
            };
        }

        ConsentDecision::Deny {
            reason: format!(
                "Consent policy does not permit purpose '{}' or category '{}' for recipient '{}'",
                purpose, resource_type, recipient_node_id
            ),
        }
    }
}
