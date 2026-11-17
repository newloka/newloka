//! New Loka Clinical Algorithms & Decision Support
//!
//! Pure, deterministic, zero-allocation calculations for clinical acuity,
//! renal clearance, ketoacidosis triage, pediatric dosing, and document ingestion.

pub mod ingest;
pub mod scoring;

pub use ingest::*;
pub use scoring::*;
