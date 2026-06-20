//! New Loka Core Library
//!
//! A local-first, polymorphic health data management system core.
//! Provides FHIR R4 data models, CRDT storage, encryption, identity,
//! sync, audit, and ABAC policy enforcement.

pub mod abac;
pub mod audit;
pub mod cpoe;
pub mod crypto;
pub mod fhir;
pub mod identity;
pub mod storage;
pub mod sync;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum NewLokaError {
    #[error("storage error: {0}")]
    Storage(String),
    #[error("crypto error: {0}")]
    Crypto(String),
    #[error("identity error: {0}")]
    Identity(String),
    #[error("sync error: {0}")]
    Sync(String),
    #[error("audit error: {0}")]
    Audit(String),
    #[error("abac error: {0}")]
    Abac(String),
    #[error("fhir error: {0}")]
    Fhir(String),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, NewLokaError>;

/// Deployment tier determines which modules are active.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[allow(non_camel_case_types)]
pub enum DeploymentTier {
    T0_SoloClinician,
    T1_SmallClinic,
    T2_RuralHospital,
    T3_MultiDepartmentHospital,
    T4_ResearchFederation,
}

impl DeploymentTier {
    pub fn supports_mesh(&self) -> bool {
        matches!(
            self,
            DeploymentTier::T1_SmallClinic
                | DeploymentTier::T2_RuralHospital
                | DeploymentTier::T3_MultiDepartmentHospital
                | DeploymentTier::T4_ResearchFederation
        )
    }

    pub fn supports_abac_silos(&self) -> bool {
        matches!(
            self,
            DeploymentTier::T2_RuralHospital
                | DeploymentTier::T3_MultiDepartmentHospital
                | DeploymentTier::T4_ResearchFederation
        )
    }

    pub fn supports_federation(&self) -> bool {
        matches!(self, DeploymentTier::T4_ResearchFederation)
    }
}
