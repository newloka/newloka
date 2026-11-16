//! Pure Clinical Domain Models & Calculators
//!
//! Enforces Section 3 of the New Loka Style Guide (Functional Core, Imperative Shell).

pub mod scoring;

pub use scoring::{
    calculate_egfr_ckd_epi, calculate_news2, calculate_pediatric_dose, evaluate_dka_risk,
    AvpuScore, BiologicalSex, CkdStage, ClinicalRiskTier, DkaSeverity, EgfrResult, News2Result,
};
