//! Audit logging module
//!
//! Append-only, cryptographically signed audit trail.
//! Every access, edit, sync, override, and correction is recorded.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Signed audit entry forming a tamper-evident chain.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditEntry {
    pub id: String,
    pub timestamp: chrono::DateTime<Utc>,
    pub event_type: AuditEventType,
    pub actor_id: String,
    pub node_id: String,
    pub patient_id: Option<String>,
    pub resource_id: Option<String>,
    pub resource_type: Option<String>,
    pub action: String,
    pub outcome: AuditOutcome,
    pub details: Option<String>,
    pub previous_hash: Option<String>,
    pub entry_hash: String,
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    Access,
    Create,
    Update,
    Delete,
    Sync,
    Override,
    Correction,
    Login,
    Logout,
    PolicyDeny,
    AiReview,
    Transfer,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AuditOutcome {
    Success,
    Failure,
    Denied,
    Warning,
    Escalated,
}

/// Audit engine maintains the signed chain.
pub struct AuditEngine {
    node_id: String,
    signer: crate::crypto::AuditSigner,
    last_hash: Option<String>,
    entries: Vec<AuditEntry>,
}

impl AuditEngine {
    pub fn new(node_id: String, signer: crate::crypto::AuditSigner) -> Self {
        Self {
            node_id,
            signer,
            last_hash: None,
            entries: vec![],
        }
    }

    pub fn log(
        &mut self,
        event_type: AuditEventType,
        actor_id: String,
        patient_id: Option<String>,
        resource_id: Option<String>,
        resource_type: Option<String>,
        action: String,
        outcome: AuditOutcome,
        details: Option<String>,
    ) -> crate::Result<AuditEntry> {
        let id = Uuid::new_v4().to_string();
        let timestamp = Utc::now();

        let mut preimage = Vec::new();
        preimage.extend_from_slice(id.as_bytes());
        preimage.extend_from_slice(&timestamp.timestamp_millis().to_be_bytes());
        preimage.extend_from_slice(actor_id.as_bytes());
        preimage.extend_from_slice(self.node_id.as_bytes());
        preimage.extend_from_slice(action.as_bytes());
        if let Some(prev) = &self.last_hash {
            preimage.extend_from_slice(prev.as_bytes());
        }

        let entry_hash = crate::crypto::hash_resource(&preimage);
        let signature = self.signer.sign(entry_hash.as_bytes());

        let entry = AuditEntry {
            id,
            timestamp,
            event_type,
            actor_id,
            node_id: self.node_id.clone(),
            patient_id,
            resource_id,
            resource_type,
            action,
            outcome,
            details,
            previous_hash: self.last_hash.clone(),
            entry_hash: entry_hash.clone(),
            signature,
        };

        self.last_hash = Some(entry_hash);
        self.entries.push(entry.clone());
        Ok(entry)
    }

    pub fn entries(&self) -> &[AuditEntry] {
        &self.entries
    }

    pub fn verify_chain(&self, public_key: &[u8]) -> crate::Result<bool> {
        let mut prev_hash: Option<String> = None;
        for entry in &self.entries {
            let mut preimage = Vec::new();
            preimage.extend_from_slice(entry.id.as_bytes());
            preimage.extend_from_slice(&entry.timestamp.timestamp_millis().to_be_bytes());
            preimage.extend_from_slice(entry.actor_id.as_bytes());
            preimage.extend_from_slice(entry.node_id.as_bytes());
            preimage.extend_from_slice(entry.action.as_bytes());
            if let Some(prev) = &prev_hash {
                preimage.extend_from_slice(prev.as_bytes());
            }
            let computed_hash = crate::crypto::hash_resource(&preimage);
            if computed_hash != entry.entry_hash {
                return Ok(false);
            }
            if !crate::crypto::verify_audit_signature(
                public_key,
                entry.entry_hash.as_bytes(),
                &entry.signature,
            )? {
                return Ok(false);
            }
            prev_hash = Some(entry.entry_hash.clone());
        }
        Ok(true)
    }
}
