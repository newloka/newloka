//! Attribute-Based Access Control (ABAC)
//!
//! Enforces policy at the data layer, not just the UI.
//! Supports department silos, team scope, emergency override, and role checks.

use serde::{Deserialize, Serialize};

/// ABAC policy request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PolicyRequest {
    pub subject: Subject,
    pub resource: Resource,
    pub action: Action,
    pub context: Context,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Subject {
    pub user_id: String,
    pub roles: Vec<crate::identity::Role>,
    pub department_id: Option<String>,
    pub team_ids: Vec<String>,
    pub session_valid: bool,
    pub emergency_override: bool,
    pub lab_affiliations: Vec<crate::cpoe::LabDepartment>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Resource {
    pub resource_type: String,
    pub resource_id: String,
    pub patient_id: Option<String>,
    pub department_id: Option<String>,
    pub owner_team_ids: Vec<String>,
    pub lab_department: Option<crate::cpoe::LabDepartment>,
    pub sensitivity: SensitivityLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Action {
    Read,
    Create,
    Update,
    Delete,
    SyncSend,
    SyncReceive,
    Override,
    Export,
    ResearchQuery,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SensitivityLevel {
    Normal,
    Restricted,
    Critical,
    ResearchOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Context {
    pub tier: crate::DeploymentTier,
    pub offline: bool,
    pub peer_node_id: Option<String>,
    pub time_of_day: String,
    pub lab_config: crate::cpoe::LabConfiguration,
    /// For lab/imaging staff: whether the patient has an active lab order
    /// that matches the subject's lab affiliation.
    pub patient_has_lab_order: bool,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            tier: crate::DeploymentTier::T1_SmallClinic,
            offline: false,
            peer_node_id: None,
            time_of_day: String::new(),
            lab_config: crate::cpoe::LabConfiguration::default(),
            patient_has_lab_order: false,
        }
    }
}

/// ABAC policy engine.
pub struct PolicyEngine;

impl PolicyEngine {
    pub fn evaluate(req: &PolicyRequest) -> PolicyDecision {
        // Deny if session invalid
        if !req.subject.session_valid {
            return PolicyDecision::Deny {
                reason: "Invalid or expired session".into(),
            };
        }

        // System role bypass for internal operations
        if req.subject.roles.contains(&crate::identity::Role::System) {
            return PolicyDecision::Allow;
        }

        // Emergency override
        if req.subject.emergency_override
            && (req.action == Action::Override
                || req.action == Action::Read
                || req.action == Action::Update)
        {
            return PolicyDecision::AllowWithAudit {
                reason: "Emergency override active".into(),
            };
        }

        let has_role = |r: crate::identity::Role| req.subject.roles.contains(&r);

        // Lab / Imaging specific restrictions on Patient creation
        if req.action == Action::Create
            && req.resource.resource_type == "Patient"
            && (has_role(crate::identity::Role::LabTechnician)
                || has_role(crate::identity::Role::ImagingTechnician))
        {
            return PolicyDecision::Deny {
                reason: "Lab and imaging staff cannot add patients".into(),
            };
        }

        // Lab / Imaging read access to Patient requires a lab order for that patient
        if req.action == Action::Read
            && req.resource.resource_type == "Patient"
            && (has_role(crate::identity::Role::LabTechnician)
                || has_role(crate::identity::Role::ImagingTechnician))
            && !req.context.patient_has_lab_order
            && !has_role(crate::identity::Role::Administrator)
            && !has_role(crate::identity::Role::Clinician)
            && !has_role(crate::identity::Role::DepartmentHead)
        {
            return PolicyDecision::Deny {
                reason: "Lab and imaging staff can only view patients with lab orders".into(),
            };
        }

        // Role-based action checks
        match req.action {
            Action::Create => {
                if !has_role(crate::identity::Role::Clinician)
                    && !has_role(crate::identity::Role::Nurse)
                    && !has_role(crate::identity::Role::Administrator)
                    && !has_role(crate::identity::Role::EmergencyOverride)
                    && !has_role(crate::identity::Role::DepartmentHead)
                    && !has_role(crate::identity::Role::ResidentDoctor)
                {
                    return PolicyDecision::Deny {
                        reason: "Insufficient role for create".into(),
                    };
                }
            }
            Action::Update | Action::Delete => {
                if !has_role(crate::identity::Role::Clinician)
                    && !has_role(crate::identity::Role::Administrator)
                    && !has_role(crate::identity::Role::EmergencyOverride)
                    && !has_role(crate::identity::Role::DepartmentHead)
                    && !has_role(crate::identity::Role::ResidentDoctor)
                {
                    return PolicyDecision::Deny {
                        reason: "Insufficient role for modify".into(),
                    };
                }
            }
            Action::Override => {
                if !has_role(crate::identity::Role::EmergencyOverride)
                    && !has_role(crate::identity::Role::Administrator)
                    && !has_role(crate::identity::Role::DepartmentHead)
                {
                    return PolicyDecision::Deny {
                        reason: "Override requires emergency or admin role".into(),
                    };
                }
            }
            Action::ResearchQuery
                if !has_role(crate::identity::Role::Researcher)
                    && !has_role(crate::identity::Role::Administrator) =>
            {
                return PolicyDecision::Deny {
                    reason: "Research role required".into(),
                };
            }
            _ => {}
        }

        // Lab report creation permissions
        if req.action == Action::Create
            && (req.resource.resource_type == "Observation"
                || req.resource.resource_type == "DiagnosticReport"
                || req.resource.resource_type == "DocumentReference")
        {
            if has_role(crate::identity::Role::LabTechnician)
                || has_role(crate::identity::Role::ImagingTechnician)
                || has_role(crate::identity::Role::Clinician)
                || has_role(crate::identity::Role::Administrator)
                || has_role(crate::identity::Role::DepartmentHead)
            {
                // Allowed specifically for lab/imaging reports
            } else {
                return PolicyDecision::Deny {
                    reason: "Insufficient role for report creation".into(),
                };
            }
        }

        // Department silo enforcement (T2+)
        if req.context.tier.supports_abac_silos() {
            if let Some(req_dept) = &req.subject.department_id {
                if let Some(res_dept) = &req.resource.department_id {
                    if req_dept != res_dept {
                        // Check if any shared team membership allows cross-department
                        let shared_team = req
                            .subject
                            .team_ids
                            .iter()
                            .any(|t| req.resource.owner_team_ids.contains(t));
                        if !shared_team && !has_role(crate::identity::Role::Administrator) {
                            return PolicyDecision::Deny {
                                reason: "Department silo violation".into(),
                            };
                        }
                    }
                }
            }
        }

        // Team-scope enforcement for resident doctors
        if req
            .subject
            .roles
            .contains(&crate::identity::Role::ResidentDoctor)
            && req.action != Action::Read
        {
            let shared_team = req
                .subject
                .team_ids
                .iter()
                .any(|t| req.resource.owner_team_ids.contains(t));
            if !shared_team
                && !has_role(crate::identity::Role::Administrator)
                && !has_role(crate::identity::Role::DepartmentHead)
            {
                return PolicyDecision::Deny {
                    reason: "Resident doctor can only modify resources within their team".into(),
                };
            }
        }

        // Sensitivity checks
        match req.resource.sensitivity {
            SensitivityLevel::Critical => {
                if !has_role(crate::identity::Role::Clinician)
                    && !has_role(crate::identity::Role::Administrator)
                    && !has_role(crate::identity::Role::EmergencyOverride)
                    && !has_role(crate::identity::Role::DepartmentHead)
                {
                    return PolicyDecision::Deny {
                        reason: "Critical sensitivity requires clinician or admin".into(),
                    };
                }
            }
            SensitivityLevel::ResearchOnly
                if !has_role(crate::identity::Role::Researcher)
                    && !has_role(crate::identity::Role::Administrator) =>
            {
                return PolicyDecision::Deny {
                    reason: "Research-only data access restricted".into(),
                };
            }
            _ => {}
        }

        PolicyDecision::Allow
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PolicyDecision {
    Allow,
    AllowWithAudit { reason: String },
    Deny { reason: String },
}
