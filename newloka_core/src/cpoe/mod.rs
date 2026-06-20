use serde::{Deserialize, Serialize};

/// Lab departments supported in New Loka CPOE.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum LabDepartment {
    Radiology,
    Biochemistry,
    Pathology,
    Microbiology,
}

impl LabDepartment {
    pub fn as_str(&self) -> &'static str {
        match self {
            LabDepartment::Radiology => "radiology",
            LabDepartment::Biochemistry => "biochemistry",
            LabDepartment::Pathology => "pathology",
            LabDepartment::Microbiology => "microbiology",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            LabDepartment::Radiology => "Radiology",
            LabDepartment::Biochemistry => "Biochemistry",
            LabDepartment::Pathology => "Pathology",
            LabDepartment::Microbiology => "Microbiology",
        }
    }
}

impl std::str::FromStr for LabDepartment {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "radiology" | "radio" => Ok(LabDepartment::Radiology),
            "biochemistry" | "biochem" => Ok(LabDepartment::Biochemistry),
            "pathology" | "patho" => Ok(LabDepartment::Pathology),
            "microbiology" | "micro" => Ok(LabDepartment::Microbiology),
            _ => Err(()),
        }
    }
}

/// Per-deployment lab configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LabConfiguration {
    pub enabled_labs: Vec<LabDepartment>,
    pub default_lab: Option<LabDepartment>,
    pub require_lab_selection: bool,
    pub cross_lab_read_requires_admin: bool,
    pub order_code_system: String,
}

impl Default for LabConfiguration {
    fn default() -> Self {
        Self {
            enabled_labs: vec![
                LabDepartment::Radiology,
                LabDepartment::Biochemistry,
                LabDepartment::Pathology,
                LabDepartment::Microbiology,
            ],
            default_lab: None,
            require_lab_selection: true,
            cross_lab_read_requires_admin: true,
            order_code_system: "http://newloka.org/lab-orders".into(),
        }
    }
}

impl LabConfiguration {
    pub fn for_tier(tier: crate::DeploymentTier) -> Self {
        match tier {
            crate::DeploymentTier::T0_SoloClinician => Self {
                enabled_labs: vec![],
                default_lab: None,
                require_lab_selection: false,
                cross_lab_read_requires_admin: false,
                order_code_system: "http://newloka.org/lab-orders".into(),
            },
            crate::DeploymentTier::T1_SmallClinic => Self {
                enabled_labs: vec![LabDepartment::Biochemistry, LabDepartment::Radiology],
                default_lab: Some(LabDepartment::Biochemistry),
                require_lab_selection: false,
                cross_lab_read_requires_admin: false,
                order_code_system: "http://newloka.org/lab-orders".into(),
            },
            _ => Self::default(),
        }
    }

    pub fn is_lab_enabled(&self, lab: &LabDepartment) -> bool {
        self.enabled_labs.contains(lab)
    }
}

/// Order lifecycle states for CPOE.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    Draft,
    Requested,
    Active,
    Completed,
    Cancelled,
}

impl OrderStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            OrderStatus::Draft => "draft",
            OrderStatus::Requested => "requested",
            OrderStatus::Active => "active",
            OrderStatus::Completed => "completed",
            OrderStatus::Cancelled => "cancelled",
        }
    }
}

/// Order priority levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderPriority {
    Routine,
    Urgent,
    Stat,
}

impl OrderPriority {
    pub fn as_str(&self) -> &'static str {
        match self {
            OrderPriority::Routine => "routine",
            OrderPriority::Urgent => "urgent",
            OrderPriority::Stat => "stat",
        }
    }
}

/// A CPOE order entry that maps to a FHIR ServiceRequest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrderEntry {
    pub order_id: String,
    pub patient_id: String,
    pub encounter_id: Option<String>,
    pub lab_department: LabDepartment,
    pub order_code: String,
    pub order_display: String,
    pub status: OrderStatus,
    pub priority: OrderPriority,
    pub requester_id: String,
    pub performer_ids: Vec<String>,
    pub authored_on: chrono::DateTime<chrono::Utc>,
    pub body_site: Option<String>,
    pub specimen_ids: Vec<String>,
    pub note: Option<String>,
}

/// ABAC helper: evaluates whether a subject can access a given lab.
pub fn lab_access_allowed(
    subject_labs: &[LabDepartment],
    resource_lab: &LabDepartment,
    subject_roles: &[crate::identity::Role],
    is_emergency: bool,
    lab_config: &LabConfiguration,
) -> bool {
    if is_emergency {
        return true;
    }

    if subject_roles.contains(&crate::identity::Role::Administrator)
        || subject_roles.contains(&crate::identity::Role::System)
    {
        return true;
    }

    if subject_labs.contains(resource_lab) {
        return true;
    }

    if !lab_config.cross_lab_read_requires_admin {
        return true;
    }

    false
}
