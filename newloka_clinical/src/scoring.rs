//! Pure Clinical Scoring & Calculation Engines
//!
//! Enforces Section 3 of the New Loka Style Guide (Functional Core, Imperative Shell).
//! All algorithms here are 100% deterministic, side-effect free, zero-allocation
//! functions with explicit inputs and outputs.

use serde::{Deserialize, Serialize};

/// AVPU scale for consciousness assessment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AvpuScore {
    Alert,
    Voice,
    Pain,
    Unresponsive,
}

impl AvpuScore {
    pub fn as_str(&self) -> &'static str {
        match self {
            AvpuScore::Alert => "alert",
            AvpuScore::Voice => "voice",
            AvpuScore::Pain => "pain",
            AvpuScore::Unresponsive => "unresponsive",
        }
    }
}

/// Clinical Risk Tier according to the Royal College of Physicians NEWS2 standard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClinicalRiskTier {
    Low,    // Aggregate 0-4
    Medium, // Aggregate 5-6 or any single physiological parameter = 3
    High,   // Aggregate 7+
}

impl ClinicalRiskTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            ClinicalRiskTier::Low => "low",
            ClinicalRiskTier::Medium => "medium",
            ClinicalRiskTier::High => "high",
        }
    }
}

/// Result of a NEWS2 score evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct News2Result {
    pub total_score: u32,
    pub risk_tier: ClinicalRiskTier,
    pub triggers_stat_alert: bool,
}

/// Pure calculation of National Early Warning Score 2 (NEWS2).
///
/// Implemented strictly as specified in Section 3.2 of the New Loka Style Guide.
/// Zero heap allocations, zero side-effects, 100% deterministic.
pub fn calculate_news2(
    resp_rate: u32,
    spo2_percent: u32,
    is_on_supplemental_o2: bool,
    systolic_bp: u32,
    heart_rate: u32,
    conscious_level_avpu: AvpuScore,
    temperature_celsius: f32,
) -> News2Result {
    let mut score = 0;
    let mut has_extreme_trigger = false;

    // Respiration rate
    let rr_score = match resp_rate {
        0..=8 => 3,
        9..=11 => 1,
        12..=20 => 0,
        21..=24 => 2,
        _ => 3,
    };
    if rr_score == 3 {
        has_extreme_trigger = true;
    }
    score += rr_score;

    // Pulse oximetry Scale 1 (standard)
    let spo2_score = match spo2_percent {
        0..=91 => 3,
        92..=93 => 2,
        94..=95 => 1,
        _ => 0,
    };
    if spo2_score == 3 {
        has_extreme_trigger = true;
    }
    score += spo2_score;

    // Supplemental Oxygen
    if is_on_supplemental_o2 {
        score += 2;
    }

    // Systolic Blood Pressure
    let sbp_score = match systolic_bp {
        0..=90 => 3,
        91..=100 => 2,
        101..=110 => 1,
        111..=219 => 0,
        _ => 3,
    };
    if sbp_score == 3 {
        has_extreme_trigger = true;
    }
    score += sbp_score;

    // Pulse / Heart Rate
    let hr_score = match heart_rate {
        0..=40 => 3,
        41..=50 => 1,
        51..=90 => 0,
        91..=110 => 1,
        111..=130 => 2,
        _ => 3,
    };
    if hr_score == 3 {
        has_extreme_trigger = true;
    }
    score += hr_score;

    // Consciousness Level (AVPU)
    let avpu_score = match conscious_level_avpu {
        AvpuScore::Alert => 0,
        AvpuScore::Voice | AvpuScore::Pain | AvpuScore::Unresponsive => 3,
    };
    if avpu_score == 3 {
        has_extreme_trigger = true;
    }
    score += avpu_score;

    // Body Temperature (Celsius)
    let temp_score = if temperature_celsius <= 35.0 {
        3
    } else if temperature_celsius <= 36.0 {
        1
    } else if temperature_celsius <= 38.0 {
        0
    } else if temperature_celsius <= 39.0 {
        1
    } else {
        2
    };
    if temp_score == 3 {
        has_extreme_trigger = true;
    }
    score += temp_score;

    let risk_tier = if score >= 7 {
        ClinicalRiskTier::High
    } else if score >= 5 || has_extreme_trigger {
        ClinicalRiskTier::Medium
    } else {
        ClinicalRiskTier::Low
    };

    News2Result {
        total_score: score,
        risk_tier,
        triggers_stat_alert: score >= 7 || has_extreme_trigger,
    }
}

/// Biological sex for clinical pharmacokinetic and renal equations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BiologicalSex {
    Female,
    Male,
}

/// Chronic Kidney Disease (CKD) staging based on KDIGO guidelines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CkdStage {
    G1Normal,         // eGFR >= 90
    G2MildlyDecreased, // eGFR 60-89
    G3aMildToModerate, // eGFR 45-59
    G3bModerateToSevere, // eGFR 30-44
    G4SeverelyDecreased, // eGFR 15-29
    G5KidneyFailure,   // eGFR < 15
}

/// Result of eGFR CKD-EPI (2021 race-free equation) evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EgfrResult {
    pub egfr_value: f64,
    pub stage: CkdStage,
    pub requires_dose_adjustment: bool,
}

/// Pure calculation of Estimated Glomerular Filtration Rate using CKD-EPI 2021 Refit (Race-Free).
///
/// Formula:
/// eGFR = 142 * min(Scr/kappa, 1)^alpha * max(Scr/kappa, 1)^-1.200 * 0.9938^Age * (1.012 if female)
///
/// Where:
/// - Female: kappa = 0.7, alpha = -0.241
/// - Male: kappa = 0.9, alpha = -0.302
pub fn calculate_egfr_ckd_epi(
    serum_creatinine_mg_dl: f64,
    age_years: u32,
    sex: BiologicalSex,
) -> Option<EgfrResult> {
    if serum_creatinine_mg_dl <= 0.0 || age_years == 0 {
        return None;
    }

    let (kappa, alpha, sex_factor) = match sex {
        BiologicalSex::Female => (0.7, -0.241, 1.012),
        BiologicalSex::Male => (0.9, -0.302, 1.000),
    };

    let scr_ratio = serum_creatinine_mg_dl / kappa;
    let min_part = scr_ratio.min(1.0).powf(alpha);
    let max_part = scr_ratio.max(1.0).powf(-1.200);
    let age_part = 0.9938_f64.powf(age_years as f64);

    let egfr_value = 142.0 * min_part * max_part * age_part * sex_factor;
    let rounded_egfr = (egfr_value * 10.0).round() / 10.0;

    let stage = if rounded_egfr >= 90.0 {
        CkdStage::G1Normal
    } else if rounded_egfr >= 60.0 {
        CkdStage::G2MildlyDecreased
    } else if rounded_egfr >= 45.0 {
        CkdStage::G3aMildToModerate
    } else if rounded_egfr >= 30.0 {
        CkdStage::G3bModerateToSevere
    } else if rounded_egfr >= 15.0 {
        CkdStage::G4SeverelyDecreased
    } else {
        CkdStage::G5KidneyFailure
    };

    Some(EgfrResult {
        egfr_value: rounded_egfr,
        stage,
        requires_dose_adjustment: rounded_egfr < 60.0,
    })
}

/// Diabetic Ketoacidosis (DKA) severity triage evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DkaSeverity {
    None,
    Mild,
    Moderate,
    Severe,
}

/// Pure evaluation of Diabetic Ketoacidosis risk based on ADA Diagnostic Criteria.
pub fn evaluate_dka_risk(
    blood_glucose_mg_dl: f64,
    arterial_ph: f64,
    serum_bicarbonate_meq_l: f64,
    urine_or_blood_ketones_positive: bool,
) -> DkaSeverity {
    if blood_glucose_mg_dl < 250.0 || !urine_or_blood_ketones_positive {
        return DkaSeverity::None;
    }

    if arterial_ph < 7.00 || serum_bicarbonate_meq_l < 10.0 {
        DkaSeverity::Severe
    } else if (7.00..=7.24).contains(&arterial_ph) || (10.0..=14.9).contains(&serum_bicarbonate_meq_l) {
        DkaSeverity::Moderate
    } else if (7.25..=7.30).contains(&arterial_ph) || (15.0..=18.0).contains(&serum_bicarbonate_meq_l) {
        DkaSeverity::Mild
    } else {
        DkaSeverity::None
    }
}

/// Pure calculation of pediatric dose by weight with adult ceiling cap.
pub fn calculate_pediatric_dose(
    weight_kg: f64,
    dose_mg_per_kg: f64,
    max_adult_single_dose_mg: f64,
) -> f64 {
    if weight_kg <= 0.0 || dose_mg_per_kg <= 0.0 {
        return 0.0;
    }
    let calculated = weight_kg * dose_mg_per_kg;
    if calculated > max_adult_single_dose_mg && max_adult_single_dose_mg > 0.0 {
        max_adult_single_dose_mg
    } else {
        (calculated * 10.0).round() / 10.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_news2_normal_patient_returns_low_risk() {
        let res = calculate_news2(16, 98, false, 120, 72, AvpuScore::Alert, 36.8);
        assert_eq!(res.total_score, 0);
        assert_eq!(res.risk_tier, ClinicalRiskTier::Low);
        assert!(!res.triggers_stat_alert);
    }

    #[test]
    fn test_news2_single_extreme_parameter_triggers_medium_risk() {
        // Severe tachypnoea (RR 26 -> score 3), everything else normal
        let res = calculate_news2(26, 98, false, 120, 72, AvpuScore::Alert, 36.8);
        assert_eq!(res.total_score, 3);
        assert_eq!(res.risk_tier, ClinicalRiskTier::Medium);
        assert!(res.triggers_stat_alert);
    }

    #[test]
    fn test_news2_critical_patient_triggers_high_risk() {
        // Hypoxic on oxygen, tachycardic, hypotensive, altered consciousness
        let res = calculate_news2(28, 88, true, 85, 135, AvpuScore::Voice, 39.5);
        // RR: 3, SpO2: 3, O2: 2, SBP: 3, HR: 3, AVPU: 3, Temp: 2 = 19
        assert_eq!(res.total_score, 19);
        assert_eq!(res.risk_tier, ClinicalRiskTier::High);
        assert!(res.triggers_stat_alert);
    }

    #[test]
    fn test_egfr_male_and_female_calculation() {
        // Male, 50 years, creatinine 1.0 mg/dL
        let male_res = calculate_egfr_ckd_epi(1.0, 50, BiologicalSex::Male).unwrap();
        assert!(male_res.egfr_value >= 90.0);
        assert_eq!(male_res.stage, CkdStage::G1Normal);
        assert!(!male_res.requires_dose_adjustment);

        // Female, 75 years, creatinine 2.5 mg/dL (severe impairment)
        let female_res = calculate_egfr_ckd_epi(2.5, 75, BiologicalSex::Female).unwrap();
        assert!(female_res.egfr_value < 25.0);
        assert!(matches!(female_res.stage, CkdStage::G4SeverelyDecreased | CkdStage::G5KidneyFailure));
        assert!(female_res.requires_dose_adjustment);
    }

    #[test]
    fn test_dka_severity_grading() {
        assert_eq!(evaluate_dka_risk(350.0, 6.95, 8.0, true), DkaSeverity::Severe);
        assert_eq!(evaluate_dka_risk(300.0, 7.15, 12.0, true), DkaSeverity::Moderate);
        assert_eq!(evaluate_dka_risk(280.0, 7.28, 16.0, true), DkaSeverity::Mild);
        assert_eq!(evaluate_dka_risk(200.0, 7.10, 10.0, false), DkaSeverity::None);
    }

    #[test]
    fn test_pediatric_dose_capping() {
        // 20kg child at 15 mg/kg = 300mg (cap 500mg) -> 300mg
        assert_eq!(calculate_pediatric_dose(20.0, 15.0, 500.0), 300.0);
        // 45kg child at 15 mg/kg = 675mg (cap 500mg) -> 500mg
        assert_eq!(calculate_pediatric_dose(45.0, 15.0, 500.0), 500.0);
    }
}
