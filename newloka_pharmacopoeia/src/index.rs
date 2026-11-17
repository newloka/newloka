//! Pharmacopoeia Index & Search Engine
//!
//! Provides ultra-low-latency in-memory search, brand-to-generic resolution,
//! clinical order set management, and pharmacovigilance safety screening.

use std::collections::HashMap;

use super::models::{
    CategorySummary, ClinicalOrderSet, DrugSafetyReport, DrugSafetyWarning, PharmacopoeiaDatabase,
    PharmacopoeiaEntry, PharmacopoeiaSearchResult, SafetyWarningLevel,
};

/// The embedded Indian Pharmacopoeia reference JSON payload.
pub const EMBEDDED_PHARMACOPOEIA_JSON: &str = include_str!("data/indian_pharmacopoeia.json");

/// In-memory search index and clinical engine.
#[derive(Debug, Clone)]
pub struct PharmacopoeiaIndex {
    database: PharmacopoeiaDatabase,
    by_id: HashMap<String, PharmacopoeiaEntry>,
    order_sets_by_id: HashMap<String, ClinicalOrderSet>,
}

impl Default for PharmacopoeiaIndex {
    fn default() -> Self {
        Self::load_embedded().expect("Embedded pharmacopoeia JSON must be valid")
    }
}

impl PharmacopoeiaIndex {
    /// Load the pharmacopoeia index from the embedded JSON dataset.
    pub fn load_embedded() -> Result<Self, serde_json::Error> {
        let database: PharmacopoeiaDatabase = serde_json::from_str(EMBEDDED_PHARMACOPOEIA_JSON)?;
        let mut by_id = HashMap::with_capacity(database.entries.len());
        for entry in &database.entries {
            by_id.insert(entry.id.clone(), entry.clone());
        }

        let mut order_sets_by_id = HashMap::with_capacity(database.order_sets.len());
        for set in &database.order_sets {
            order_sets_by_id.insert(set.id.clone(), set.clone());
        }

        Ok(Self {
            database,
            by_id,
            order_sets_by_id,
        })
    }

    /// Retrieve raw database metadata.
    pub fn metadata(&self) -> (&str, &str, &str, usize, usize) {
        (
            &self.database.version,
            &self.database.published_date,
            &self.database.reference_standard,
            self.database.entries.len(),
            self.database.order_sets.len(),
        )
    }

    /// Get a pharmacopoeia monograph entry by unique ID (e.g. "ip-paracetamol").
    pub fn get_entry(&self, id: &str) -> Option<&PharmacopoeiaEntry> {
        self.by_id.get(id)
    }

    /// Retrieve an evidence-based clinical order set by ID.
    pub fn get_order_set(&self, id: &str) -> Option<&ClinicalOrderSet> {
        self.order_sets_by_id.get(id)
    }

    /// List all standardized clinical order sets.
    pub fn list_order_sets(&self) -> Vec<&ClinicalOrderSet> {
        self.database.order_sets.iter().collect()
    }

    /// List all unique therapeutic categories with entry counts.
    pub fn list_categories(&self) -> Vec<CategorySummary> {
        let mut counts: HashMap<String, usize> = HashMap::new();
        for entry in &self.database.entries {
            *counts
                .entry(entry.therapeutic_category.clone())
                .or_insert(0) += 1;
        }

        let mut res: Vec<CategorySummary> = counts
            .into_iter()
            .map(|(name, count)| CategorySummary { name, count })
            .collect();
        res.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.name.cmp(&b.name)));
        res
    }

    /// Search the pharmacopoeia by text query and optional category filter.
    /// Supports generic INN names, IP titles, Indian brand equivalents, and dosage forms.
    pub fn search(
        &self,
        query: &str,
        category: Option<&str>,
        limit: usize,
    ) -> Vec<PharmacopoeiaSearchResult> {
        let q = query.trim().to_lowercase();
        let cat_filter = category.map(|c| c.trim().to_lowercase());

        let mut results = Vec::new();

        for entry in &self.database.entries {
            // Apply category filter if specified
            if let Some(ref cat) = cat_filter {
                if !cat.is_empty() && !entry.therapeutic_category.to_lowercase().contains(cat) {
                    continue;
                }
            }

            // If query is empty, return category items
            if q.is_empty() {
                results.push(PharmacopoeiaSearchResult {
                    entry: entry.clone(),
                    matched_via: "category".to_string(),
                    matched_brand: None,
                    score: 10,
                });
                continue;
            }

            let inn_lower = entry.inn_name.to_lowercase();
            let title_lower = entry.ip_title.to_lowercase();

            // 1. Exact match on generic name or IP title
            if inn_lower == q || title_lower == q {
                results.push(PharmacopoeiaSearchResult {
                    entry: entry.clone(),
                    matched_via: "generic".to_string(),
                    matched_brand: None,
                    score: 100,
                });
                continue;
            }

            // 2. Exact match on Indian brand equivalent
            let exact_brand = entry
                .common_brands_india
                .iter()
                .find(|b| b.to_lowercase() == q);
            if let Some(b) = exact_brand {
                results.push(PharmacopoeiaSearchResult {
                    entry: entry.clone(),
                    matched_via: "brand".to_string(),
                    matched_brand: Some(b.clone()),
                    score: 95,
                });
                continue;
            }

            // 3. Prefix match on generic name
            if inn_lower.starts_with(&q) {
                results.push(PharmacopoeiaSearchResult {
                    entry: entry.clone(),
                    matched_via: "generic".to_string(),
                    matched_brand: None,
                    score: 80,
                });
                continue;
            }

            // 4. Prefix match on brand name
            let prefix_brand = entry
                .common_brands_india
                .iter()
                .find(|b| b.to_lowercase().starts_with(&q));
            if let Some(b) = prefix_brand {
                results.push(PharmacopoeiaSearchResult {
                    entry: entry.clone(),
                    matched_via: "brand".to_string(),
                    matched_brand: Some(b.clone()),
                    score: 75,
                });
                continue;
            }

            // 5. Substring match on generic or IP title
            if inn_lower.contains(&q) || title_lower.contains(&q) {
                results.push(PharmacopoeiaSearchResult {
                    entry: entry.clone(),
                    matched_via: "generic".to_string(),
                    matched_brand: None,
                    score: 50,
                });
                continue;
            }

            // 6. Substring match on brand name
            let sub_brand = entry
                .common_brands_india
                .iter()
                .find(|b| b.to_lowercase().contains(&q));
            if let Some(b) = sub_brand {
                results.push(PharmacopoeiaSearchResult {
                    entry: entry.clone(),
                    matched_via: "brand".to_string(),
                    matched_brand: Some(b.clone()),
                    score: 40,
                });
                continue;
            }

            // 7. Substring match on therapeutic category or ID
            if entry.id.to_lowercase().contains(&q)
                || entry.therapeutic_category.to_lowercase().contains(&q)
            {
                results.push(PharmacopoeiaSearchResult {
                    entry: entry.clone(),
                    matched_via: "category".to_string(),
                    matched_brand: None,
                    score: 20,
                });
            }
        }

        // Sort descending by score, then alphabetically by INN name
        results.sort_by(|a, b| {
            b.score
                .cmp(&a.score)
                .then_with(|| a.entry.inn_name.cmp(&b.entry.inn_name))
        });

        if limit > 0 && results.len() > limit {
            results.truncate(limit);
        }

        results
    }

    /// Perform clinical safety checks across a list of proposed drug names and known allergies.
    pub fn check_safety(&self, drugs: &[String], known_allergies: &[String]) -> DrugSafetyReport {
        let mut report = DrugSafetyReport::default();

        let clean_allergies: Vec<String> = known_allergies
            .iter()
            .map(|a| a.trim().to_lowercase())
            .filter(|a| !a.is_empty())
            .collect();

        // Track categories for duplicate therapy detection
        let mut category_counts: HashMap<String, Vec<String>> = HashMap::new();

        for raw_drug in drugs {
            let d_clean = raw_drug.trim().to_lowercase();
            if d_clean.is_empty() {
                continue;
            }

            // Attempt to resolve drug into index
            let entry = self.resolve_drug(&d_clean);

            // 1. Allergy Cross-Reactivity Checks
            for allergy in &clean_allergies {
                if allergy.contains("penicillin") || allergy.contains("beta-lactam") {
                    if d_clean.contains("amox")
                        || d_clean.contains("augmentin")
                        || d_clean.contains("clavam")
                        || d_clean.contains("pipzo")
                        || d_clean.contains("piperacillin")
                        || d_clean.contains("ampicillin")
                    {
                        report.warnings.push(DrugSafetyWarning {
                            level: SafetyWarningLevel::Critical,
                            warning_type: "AllergyCrossReactivity".to_string(),
                            drug_name: raw_drug.clone(),
                            message: format!(
                                "Critical Penicillin Allergy Alert: Patient has documented '{}' allergy.",
                                allergy
                            ),
                            clinical_rationale: "Penicillins and aminopenicillins carry severe anaphylaxis risk in penicillin-allergic patients.".to_string(),
                        });
                        report.has_critical = true;
                    } else if d_clean.contains("ceftriax")
                        || d_clean.contains("cefixime")
                        || d_clean.contains("monocef")
                        || d_clean.contains("taxim")
                    {
                        report.warnings.push(DrugSafetyWarning {
                            level: SafetyWarningLevel::Moderate,
                            warning_type: "AllergyCrossReactivity".to_string(),
                            drug_name: raw_drug.clone(),
                            message: "Cephalosporin Cross-Reactivity Warning: Patient has documented penicillin allergy.".to_string(),
                            clinical_rationale: "3-5% cross-reactivity rate exists between penicillins and cephalosporins. Monitor closely for rash or bronchospasm.".to_string(),
                        });
                    }
                }

                if (allergy.contains("sulfa") || allergy.contains("sulfonamide"))
                    && (d_clean.contains("glimepiride") || d_clean.contains("cotrimoxazole"))
                {
                    report.warnings.push(DrugSafetyWarning {
                        level: SafetyWarningLevel::Moderate,
                        warning_type: "AllergyCrossReactivity".to_string(),
                        drug_name: raw_drug.clone(),
                        message: "Sulfonamide Cross-Reactivity: Glimepiride is a sulfonylurea with potential sulfa cross-reactivity.".to_string(),
                        clinical_rationale: "Monitor for hypersensitivity cutaneous reactions.".to_string(),
                    });
                }

                if (allergy.contains("nsaid") || allergy.contains("aspirin"))
                    && (d_clean.contains("ibuprofen")
                        || d_clean.contains("diclofenac")
                        || d_clean.contains("aspirin")
                        || d_clean.contains("voveran")
                        || d_clean.contains("brufen")
                        || d_clean.contains("combiflam"))
                {
                    report.warnings.push(DrugSafetyWarning {
                        level: SafetyWarningLevel::Critical,
                        warning_type: "AllergyCrossReactivity".to_string(),
                        drug_name: raw_drug.clone(),
                        message: format!(
                            "NSAID / Aspirin Hypersensitivity Alert: Patient has documented '{}' allergy.",
                            allergy
                        ),
                        clinical_rationale: "Risk of acute bronchospasm, urticaria, or anaphylaxis (Samter triad / AERD).".to_string(),
                    });
                    report.has_critical = true;
                }
            }

            // 2. Regulatory Schedule Warnings
            if let Some(e) = entry {
                if e.schedule == "Schedule H1" {
                    report.warnings.push(DrugSafetyWarning {
                        level: SafetyWarningLevel::Info,
                        warning_type: "ScheduleRestriction".to_string(),
                        drug_name: raw_drug.clone(),
                        message: format!("Schedule H1 Restriction: '{}' requires separate Schedule H1 register entry (patient name, prescriber, qty).", e.inn_name),
                        clinical_rationale: "Mandated under Central Drugs Standard Control Organization (CDSCO) Rules 1945.".to_string(),
                    });
                } else if e.schedule == "Schedule X" || e.schedule == "Narcotic" {
                    report.warnings.push(DrugSafetyWarning {
                        level: SafetyWarningLevel::Critical,
                        warning_type: "ScheduleRestriction".to_string(),
                        drug_name: raw_drug.clone(),
                        message: format!("NDPS / Narcotic Restriction: '{}' is a controlled substance requiring duplicate prescription and tamper-evident storage.", e.inn_name),
                        clinical_rationale: "Strict compliance with NDPS Act 1985 required.".to_string(),
                    });
                }

                // Categorize for duplicate therapy
                category_counts
                    .entry(e.therapeutic_category.clone())
                    .or_default()
                    .push(e.inn_name.clone());
            }
        }

        // 3. Duplicate Therapy Detection
        for (category, drug_list) in category_counts {
            if drug_list.len() > 1 {
                // If 2 NSAIDs
                if category.contains("Analgesics, Antipyretics and Anti-inflammatory") {
                    let nsaids: Vec<&String> = drug_list
                        .iter()
                        .filter(|d| !d.to_lowercase().contains("paracetamol"))
                        .collect();
                    if nsaids.len() > 1 {
                        report.warnings.push(DrugSafetyWarning {
                            level: SafetyWarningLevel::Critical,
                            warning_type: "DuplicateTherapy".to_string(),
                            drug_name: nsaids.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(" + "),
                            message: format!("Duplicate Systemic NSAID Therapy Detected: Combining {} significantly increases gastrointestinal ulceration and acute kidney injury risk without added analgesia.", nsaids.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(" and ")),
                            clinical_rationale: "Dual NSAID prescribing is strongly contraindicated across clinical practice guidelines.".to_string(),
                        });
                        report.has_critical = true;
                    }
                } else if category.contains("Gastrointestinal") {
                    // Check duplicate PPI
                    let ppis: Vec<&String> = drug_list
                        .iter()
                        .filter(|d| {
                            d.to_lowercase().contains("pantoprazole")
                                || d.to_lowercase().contains("omeprazole")
                                || d.to_lowercase().contains("rabeprazole")
                        })
                        .collect();
                    if ppis.len() > 1 {
                        report.warnings.push(DrugSafetyWarning {
                            level: SafetyWarningLevel::Moderate,
                            warning_type: "DuplicateTherapy".to_string(),
                            drug_name: ppis.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(" + "),
                            message: "Redundant Proton Pump Inhibitor Therapy: Multiple PPIs prescribed concurrently.".to_string(),
                            clinical_rationale: "Select single PPI at appropriate therapeutic dose.".to_string(),
                        });
                    }
                }
            }
        }

        report
    }

    /// Helper to resolve a free-text drug or brand query into a registered PharmacopoeiaEntry.
    fn resolve_drug(&self, query: &str) -> Option<&PharmacopoeiaEntry> {
        let q = query.to_lowercase();
        // Exact ID match
        if let Some(entry) = self.by_id.get(query) {
            return Some(entry);
        }

        // Substring / prefix search
        for entry in &self.database.entries {
            if entry.inn_name.to_lowercase() == q || entry.ip_title.to_lowercase() == q {
                return Some(entry);
            }
            for b in &entry.common_brands_india {
                if b.to_lowercase() == q || q.starts_with(&b.to_lowercase()) {
                    return Some(entry);
                }
            }
            if q.contains(&entry.inn_name.to_lowercase()) {
                return Some(entry);
            }
        }
        None
    }
}
