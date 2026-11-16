//! Pharmacopoeia Module for New Loka
//!
//! Provides comprehensive, local-first integration with the Indian Pharmacopoeia
//! (IP 2022, 9th Edition), National Formulary of India (NFI), and NLEM 2022.
//! Powers electronic prescribing (RxPad), inpatient CPOE, standardized evidence-based
//! order sets, and pharmacovigilance safety shielding.

pub mod index;
pub mod models;

pub use index::PharmacopoeiaIndex;
pub use models::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_pharmacopoeia_loads_successfully() {
        let index = PharmacopoeiaIndex::load_embedded()
            .expect("Embedded pharmacopoeia JSON must deserialize without error");
        let (version, date, standard, monographs_count, ordersets_count) = index.metadata();
        assert_eq!(version, "2026.11");
        assert!(!date.is_empty());
        assert!(standard.contains("Indian Pharmacopoeia"));
        assert!(monographs_count >= 50, "Expected at least 50 monographs, found {}", monographs_count);
        assert!(ordersets_count >= 10, "Expected at least 10 order sets, found {}", ordersets_count);
    }

    #[test]
    fn test_search_by_generic_name() {
        let index = PharmacopoeiaIndex::default();
        let results = index.search("paracetamol", None, 5);
        assert!(!results.is_empty());
        assert_eq!(results[0].entry.id, "ip-paracetamol");
        assert_eq!(results[0].entry.inn_name, "Paracetamol");
        assert_eq!(results[0].matched_via, "generic");
    }

    #[test]
    fn test_search_by_indian_brand_equivalent() {
        let index = PharmacopoeiaIndex::default();

        // 1. Search "Dolo" -> resolves to Paracetamol
        let dolo_res = index.search("Dolo", None, 5);
        assert!(!dolo_res.is_empty());
        assert_eq!(dolo_res[0].entry.inn_name, "Paracetamol");
        assert_eq!(dolo_res[0].matched_via, "brand");
        assert!(dolo_res[0].matched_brand.as_ref().unwrap().contains("Dolo"));

        // 2. Search "Augmentin" -> resolves to Amoxicillin and Potassium Clavulanate
        let aug_res = index.search("Augmentin", None, 5);
        assert!(!aug_res.is_empty());
        assert_eq!(aug_res[0].entry.inn_name, "Amoxicillin and Potassium Clavulanate");
        assert_eq!(aug_res[0].matched_via, "brand");

        // 3. Search "Monocef" -> resolves to Ceftriaxone
        let mono_res = index.search("Monocef", None, 5);
        assert!(!mono_res.is_empty());
        assert_eq!(mono_res[0].entry.inn_name, "Ceftriaxone");

        // 4. Search "Pan 40" -> resolves to Pantoprazole
        let pan_res = index.search("Pan 40", None, 5);
        assert!(!pan_res.is_empty());
        assert_eq!(pan_res[0].entry.inn_name, "Pantoprazole");
    }

    #[test]
    fn test_category_filtering_and_listing() {
        let index = PharmacopoeiaIndex::default();
        let categories = index.list_categories();
        assert!(!categories.is_empty());

        let has_cardio = categories.iter().any(|c| c.name.contains("Cardiovascular"));
        assert!(has_cardio, "Expected Cardiovascular category");

        let antibacterials = index.search("", Some("Antibacterials"), 50);
        assert!(!antibacterials.is_empty());
        for res in antibacterials {
            assert!(res.entry.therapeutic_category.contains("Antibacterials"));
        }
    }

    #[test]
    fn test_clinical_order_sets_integrity() {
        let index = PharmacopoeiaIndex::default();
        let order_sets = index.list_order_sets();
        assert!(order_sets.len() >= 10);

        // Verify Acute Coronary Syndrome protocol
        let acs = index.get_order_set("orderset-acs-chestpain").expect("ACS order set must exist");
        assert!(acs.name.contains("Acute Coronary Syndrome"));
        assert!(acs.items.iter().any(|it| it.title.contains("Aspirin 325 mg")));
        assert!(acs.items.iter().any(|it| it.title.contains("Clopidogrel 300 mg")));
        assert!(acs.items.iter().any(|it| it.title.contains("Atorvastatin 80 mg")));

        // Verify Sepsis Hour-1 bundle
        let sepsis = index.get_order_set("orderset-sepsis-hour1").expect("Sepsis order set must exist");
        assert!(sepsis.items.iter().any(|it| it.title.contains("Piperacillin / Tazobactam")));
        assert!(sepsis.items.iter().any(|it| it.title.contains("Blood Cultures")));
        assert!(sepsis.items.iter().any(|it| it.title.contains("0.9% Normal Saline 30 mL/kg")));

        // Verify Diabetic Ketoacidosis protocol
        let dka = index.get_order_set("orderset-dka-glycemic").expect("DKA order set must exist");
        assert!(dka.items.iter().any(|it| it.title.contains("Human Regular Insulin")));
    }

    #[test]
    fn test_clinical_safety_penicillin_allergy_shield() {
        let index = PharmacopoeiaIndex::default();
        let drugs = vec!["Augmentin 625 Duo".to_string(), "Paracetamol 650mg".to_string()];
        let allergies = vec!["Penicillin".to_string()];

        let report = index.check_safety(&drugs, &allergies);
        assert!(report.has_critical);
        let pen_warning = report.warnings.iter().find(|w| w.warning_type == "AllergyCrossReactivity");
        assert!(pen_warning.is_some());
        assert_eq!(pen_warning.unwrap().level, SafetyWarningLevel::Critical);
    }

    #[test]
    fn test_clinical_safety_duplicate_nsaid_detection() {
        let index = PharmacopoeiaIndex::default();
        let drugs = vec!["Ibuprofen 400mg".to_string(), "Diclofenac 50mg".to_string()];
        let allergies = vec![];

        let report = index.check_safety(&drugs, &allergies);
        assert!(report.has_critical);
        let dup_warning = report.warnings.iter().find(|w| w.warning_type == "DuplicateTherapy");
        assert!(dup_warning.is_some());
        assert!(dup_warning.unwrap().message.contains("Duplicate Systemic NSAID"));
    }

    #[test]
    fn test_schedule_h1_regulatory_flagging() {
        let index = PharmacopoeiaIndex::default();
        let drugs = vec!["Tramadol 50mg".to_string()];
        let allergies = vec![];

        let report = index.check_safety(&drugs, &allergies);
        let h1_warning = report.warnings.iter().find(|w| w.warning_type == "ScheduleRestriction");
        assert!(h1_warning.is_some());
        assert_eq!(h1_warning.unwrap().level, SafetyWarningLevel::Info);
        assert!(h1_warning.unwrap().message.contains("Schedule H1"));
    }
}
