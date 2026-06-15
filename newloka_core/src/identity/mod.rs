//! Identity and session management
//!
//! Handles user authentication, offline-capable sessions,
//! role assignments, and department/team membership.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub roles: Vec<Role>,
    pub department_id: Option<String>,
    pub team_ids: Vec<String>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub password_hash: String,
    pub salt: Vec<u8>,
    pub totp_secret: Option<String>,
    pub last_login: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Role {
    Clinician,
    Nurse,
    Pharmacist,
    LabTechnician,
    Administrator,
    System,
    Researcher,
    EmergencyOverride,
}

impl Role {
    pub fn can_create_patient(&self) -> bool {
        matches!(
            self,
            Role::Clinician | Role::Nurse | Role::Administrator | Role::EmergencyOverride
        )
    }

    pub fn can_prescribe(&self) -> bool {
        matches!(
            self,
            Role::Clinician | Role::Pharmacist | Role::EmergencyOverride
        )
    }

    pub fn can_override(&self) -> bool {
        matches!(self, Role::EmergencyOverride | Role::Administrator)
    }

    pub fn can_admin(&self) -> bool {
        matches!(self, Role::Administrator | Role::System)
    }

    pub fn can_access_research(&self) -> bool {
        matches!(self, Role::Researcher | Role::Administrator)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Session {
    pub token: String,
    pub user_id: String,
    pub node_id: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub tier: crate::DeploymentTier,
    pub emergency_override: bool,
    pub override_reason: Option<String>,
}

impl Session {
    pub fn new(user_id: String, node_id: String, tier: crate::DeploymentTier) -> Self {
        let now = Utc::now();
        Self {
            token: Uuid::new_v4().to_string(),
            user_id,
            node_id,
            created_at: now,
            expires_at: now + Duration::try_hours(12).expect("valid hours"),
            tier,
            emergency_override: false,
            override_reason: None,
        }
    }

    pub fn with_emergency_override(mut self, reason: String) -> Self {
        self.emergency_override = true;
        self.override_reason = Some(reason);
        self
    }

    pub fn is_valid(&self) -> bool {
        Utc::now() < self.expires_at
    }

    pub fn remaining_seconds(&self) -> i64 {
        let remaining = self.expires_at - Utc::now();
        remaining.num_seconds().max(0)
    }
}

/// Authenticator for offline-capable password verification.
pub struct Authenticator;

impl Authenticator {
    pub fn verify_password(password: &str, salt: &[u8], stored_hash: &str) -> crate::Result<bool> {
        let dmk = crate::crypto::DeviceMasterKey::derive_from_password(password, salt);
        let computed_hash = crate::crypto::hash_resource(&[dmk.key.as_slice(), salt].concat());
        Ok(computed_hash == stored_hash)
    }

    pub fn hash_password(password: &str) -> crate::Result<(Vec<u8>, String)> {
        let salt = crate::crypto::generate_salt();
        let dmk = crate::crypto::DeviceMasterKey::derive_from_password(password, &salt);
        let hash = crate::crypto::hash_resource(&[dmk.key.as_slice(), &salt].concat());
        Ok((salt.to_vec(), hash))
    }
}

/// Node identity for mesh networking.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NodeIdentity {
    pub node_id: String,
    pub display_name: String,
    pub public_key: Vec<u8>,
    pub tier: crate::DeploymentTier,
    pub trusted_peers: Vec<String>,
    pub created_at: DateTime<Utc>,
}

impl NodeIdentity {
    pub fn generate(
        display_name: String,
        tier: crate::DeploymentTier,
    ) -> (Self, crate::crypto::AuditSigner) {
        let node_id = Uuid::new_v4().to_string();
        let signer = crate::crypto::AuditSigner::generate();
        let public_key = signer.verifying_key();
        let identity = Self {
            node_id,
            display_name,
            public_key,
            tier,
            trusted_peers: vec![],
            created_at: Utc::now(),
        };
        (identity, signer)
    }
}
