//! Unit tests for New Loka Core

#[cfg(test)]
mod tests {
    use newloka_core::*;

    #[test]
    fn test_deployment_tier_checks() {
        assert!(!DeploymentTier::T0_SoloClinician.supports_mesh());
        assert!(DeploymentTier::T1_SmallClinic.supports_mesh());
        assert!(DeploymentTier::T3_MultiDepartmentHospital.supports_abac_silos());
        assert!(DeploymentTier::T4_ResearchFederation.supports_federation());
    }

    #[test]
    fn test_fhir_patient_creation() {
        let meta = fhir::Meta::new("node-1".to_string(), "user-a".to_string());
        let patient = fhir::Patient {
            id: "pat-001".to_string(),
            meta,
            identifier: vec![fhir::Identifier {
                system: "newloka".to_string(),
                value: "123".to_string(),
                use_field: Some("official".to_string()),
            }],
            active: true,
            name: vec![fhir::HumanName {
                use_field: Some("official".to_string()),
                family: "Doe".to_string(),
                given: vec!["Jane".to_string()],
                prefix: vec![],
            }],
            telecom: vec![],
            gender: "female".to_string(),
            birth_date: "1985-03-15".to_string(),
            address: vec![],
            marital_status: None,
            general_practitioner: vec![],
            managing_organization: None,
            deceased_boolean: None,
            deceased_date_time: None,
        };
        assert_eq!(patient.id, "pat-001");
        assert_eq!(patient.gender, "female");
    }

    #[test]
    fn test_fhir_resource_wrapper() {
        let meta = fhir::Meta::new("node-1".to_string(), "user-a".to_string());
        let patient = fhir::Patient {
            id: "pat-002".to_string(),
            meta: meta.clone(),
            identifier: vec![],
            active: true,
            name: vec![],
            telecom: vec![],
            gender: "male".to_string(),
            birth_date: "1990-01-01".to_string(),
            address: vec![],
            marital_status: None,
            general_practitioner: vec![],
            managing_organization: None,
            deceased_boolean: None,
            deceased_date_time: None,
        };
        let resource = fhir::FhirResource::Patient(patient);
        assert_eq!(resource.id(), "pat-002");
        assert_eq!(resource.resource_type(), "Patient");
    }

    #[test]
    fn test_crypto_encryption_roundtrip() {
        let prk = crypto::PatientRecordKey::generate();
        let plaintext = b"sensitive patient data";
        let (ciphertext, nonce) = prk.encrypt(plaintext).unwrap();
        let decrypted = prk.decrypt(&ciphertext, &nonce).unwrap();
        assert_eq!(plaintext.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn test_crypto_hierarchical_keys() {
        let dmk = crypto::DeviceMasterKey::generate();
        let prk = crypto::PatientRecordKey::generate();
        let (wrapped, nonce) = crypto::wrap_prk(&dmk, &prk).unwrap();
        let unwrapped = crypto::unwrap_prk(&dmk, &wrapped, &nonce).unwrap();
        assert_eq!(prk.key, unwrapped.key);
    }

    #[test]
    fn test_audit_signing() {
        let signer = crypto::AuditSigner::generate();
        let message = b"audit entry content";
        let signature = signer.sign(message);
        let verifying_key = signer.verifying_key();
        let valid = crypto::verify_audit_signature(&verifying_key, message, &signature).unwrap();
        assert!(valid);
    }

    #[test]
    fn test_audit_chain_integrity() {
        let node_id = "node-test".to_string();
        let signer = crypto::AuditSigner::generate();
        let mut engine = audit::AuditEngine::new(node_id.clone(), signer.clone());

        let entry1 = engine
            .log(
                audit::AuditEventType::Create,
                "user-1".to_string(),
                Some("pat-001".to_string()),
                Some("enc-001".to_string()),
                Some("Encounter".to_string()),
                "create encounter".to_string(),
                audit::AuditOutcome::Success,
                None,
            )
            .unwrap();

        let entry2 = engine
            .log(
                audit::AuditEventType::Access,
                "user-1".to_string(),
                Some("pat-001".to_string()),
                Some("enc-001".to_string()),
                Some("Encounter".to_string()),
                "read encounter".to_string(),
                audit::AuditOutcome::Success,
                None,
            )
            .unwrap();

        assert_eq!(engine.entries().len(), 2);
        assert_ne!(entry1.entry_hash, entry2.entry_hash);

        let verifying_key = signer.verifying_key();
        assert!(engine.verify_chain(&verifying_key).unwrap());
    }

    #[test]
    fn test_abac_allow_clinician_create() {
        let req = abac::PolicyRequest {
            subject: abac::Subject {
                user_id: "user-1".to_string(),
                roles: vec![identity::Role::Clinician],
                department_id: Some("dept-a".to_string()),
                team_ids: vec!["team-1".to_string()],
                session_valid: true,
                emergency_override: false,
                lab_affiliations: vec![],
            },
            resource: abac::Resource {
                resource_type: "Patient".to_string(),
                resource_id: "pat-001".to_string(),
                patient_id: Some("pat-001".to_string()),
                department_id: Some("dept-a".to_string()),
                owner_team_ids: vec!["team-1".to_string()],
                sensitivity: abac::SensitivityLevel::Normal,
                lab_department: None,
            },
            action: abac::Action::Create,
            context: abac::Context {
                tier: DeploymentTier::T3_MultiDepartmentHospital,
                offline: false,
                peer_node_id: None,
                time_of_day: "14:00".to_string(),
                lab_config: cpoe::LabConfiguration::default(),
                patient_has_lab_order: false,
            },
        };
        let decision = abac::PolicyEngine::evaluate(&req);
        assert_eq!(decision, abac::PolicyDecision::Allow);
    }

    #[test]
    fn test_abac_deny_nurse_prescribe() {
        let req = abac::PolicyRequest {
            subject: abac::Subject {
                user_id: "user-nurse".to_string(),
                roles: vec![identity::Role::Nurse],
                department_id: Some("dept-a".to_string()),
                team_ids: vec![],
                session_valid: true,
                emergency_override: false,
                lab_affiliations: vec![],
            },
            resource: abac::Resource {
                resource_type: "MedicationRequest".to_string(),
                resource_id: "med-001".to_string(),
                patient_id: Some("pat-001".to_string()),
                department_id: Some("dept-a".to_string()),
                owner_team_ids: vec![],
                sensitivity: abac::SensitivityLevel::Normal,
                lab_department: None,
            },
            action: abac::Action::Override,
            context: abac::Context {
                tier: DeploymentTier::T3_MultiDepartmentHospital,
                offline: false,
                peer_node_id: None,
                time_of_day: "09:00".to_string(),
                lab_config: cpoe::LabConfiguration::default(),
                patient_has_lab_order: false,
            },
        };
        let decision = abac::PolicyEngine::evaluate(&req);
        match decision {
            abac::PolicyDecision::Deny { reason } => {
                assert!(reason.contains("emergency or admin"));
            }
            _ => panic!("Expected deny for nurse override"),
        }
    }

    #[test]
    fn test_abac_deny_department_silo() {
        let req = abac::PolicyRequest {
            subject: abac::Subject {
                user_id: "user-1".to_string(),
                roles: vec![identity::Role::Clinician],
                department_id: Some("dept-a".to_string()),
                team_ids: vec!["team-1".to_string()],
                session_valid: true,
                emergency_override: false,
                lab_affiliations: vec![],
            },
            resource: abac::Resource {
                resource_type: "Patient".to_string(),
                resource_id: "pat-001".to_string(),
                patient_id: Some("pat-001".to_string()),
                department_id: Some("dept-b".to_string()),
                owner_team_ids: vec!["team-2".to_string()],
                sensitivity: abac::SensitivityLevel::Normal,
                lab_department: None,
            },
            action: abac::Action::Read,
            context: abac::Context {
                tier: DeploymentTier::T3_MultiDepartmentHospital,
                offline: false,
                peer_node_id: None,
                time_of_day: "10:00".to_string(),
                lab_config: cpoe::LabConfiguration::default(),
                patient_has_lab_order: false,
            },
        };
        let decision = abac::PolicyEngine::evaluate(&req);
        match decision {
            abac::PolicyDecision::Deny { reason } => {
                assert!(reason.contains("Department silo"));
            }
            _ => panic!("Expected deny for silo violation"),
        }
    }

    #[test]
    fn test_abac_emergency_override() {
        let req = abac::PolicyRequest {
            subject: abac::Subject {
                user_id: "user-1".to_string(),
                roles: vec![identity::Role::Clinician],
                department_id: Some("dept-a".to_string()),
                team_ids: vec![],
                session_valid: true,
                emergency_override: true,
                lab_affiliations: vec![],
            },
            resource: abac::Resource {
                resource_type: "Patient".to_string(),
                resource_id: "pat-001".to_string(),
                patient_id: Some("pat-001".to_string()),
                department_id: Some("dept-b".to_string()),
                owner_team_ids: vec![],
                sensitivity: abac::SensitivityLevel::Critical,
                lab_department: None,
            },
            action: abac::Action::Read,
            context: abac::Context {
                tier: DeploymentTier::T3_MultiDepartmentHospital,
                offline: false,
                peer_node_id: None,
                time_of_day: "02:00".to_string(),
                lab_config: cpoe::LabConfiguration::default(),
                patient_has_lab_order: false,
            },
        };
        let decision = abac::PolicyEngine::evaluate(&req);
        assert_eq!(
            decision,
            abac::PolicyDecision::AllowWithAudit {
                reason: "Emergency override active".to_string()
            }
        );
    }

    #[test]
    fn test_abac_billing_role_clinical_data_control() {
        let req_condition = abac::PolicyRequest {
            subject: abac::Subject {
                user_id: "user-cashier".to_string(),
                roles: vec![identity::Role::Billing],
                department_id: Some("finance".to_string()),
                team_ids: vec![],
                session_valid: true,
                emergency_override: false,
                lab_affiliations: vec![],
            },
            resource: abac::Resource {
                resource_type: "Condition".to_string(),
                resource_id: "cond-101".to_string(),
                patient_id: Some("pat-001".to_string()),
                department_id: Some("dept-a".to_string()),
                owner_team_ids: vec![],
                sensitivity: abac::SensitivityLevel::Normal,
                lab_department: None,
            },
            action: abac::Action::Read,
            context: abac::Context {
                tier: DeploymentTier::T2_RuralHospital,
                offline: false,
                peer_node_id: None,
                time_of_day: "11:00".to_string(),
                lab_config: cpoe::LabConfiguration::default(),
                patient_has_lab_order: false,
            },
        };
        let decision = abac::PolicyEngine::evaluate(&req_condition);
        match decision {
            abac::PolicyDecision::Deny { reason } => {
                assert!(reason.contains("Confidential clinical diagnostic charts are restricted"));
            }
            _ => panic!("Billing staff must not have read access to confidential Condition charts"),
        }
    }

    #[test]
    fn test_abac_billing_cannot_prescribe() {
        let req_med = abac::PolicyRequest {
            subject: abac::Subject {
                user_id: "user-cashier".to_string(),
                roles: vec![identity::Role::Billing],
                department_id: Some("finance".to_string()),
                team_ids: vec![],
                session_valid: true,
                emergency_override: false,
                lab_affiliations: vec![],
            },
            resource: abac::Resource {
                resource_type: "MedicationRequest".to_string(),
                resource_id: "med-rx-001".to_string(),
                patient_id: Some("pat-001".to_string()),
                department_id: Some("finance".to_string()),
                owner_team_ids: vec![],
                sensitivity: abac::SensitivityLevel::Normal,
                lab_department: None,
            },
            action: abac::Action::Create,
            context: abac::Context::default(),
        };
        let decision = abac::PolicyEngine::evaluate(&req_med);
        match decision {
            abac::PolicyDecision::Deny { reason } => {
                assert!(reason.contains("prescribing"));
            }
            _ => panic!("Billing staff must not be permitted to create MedicationRequest"),
        }
    }

    #[test]
    fn test_sync_vector_clock() {
        let mut engine = sync::SyncEngine::new("node-a".to_string());
        let vc1 = engine.tick();
        assert_eq!(vc1.len(), 1);
        let vc2 = engine.tick();
        assert_eq!(vc2[0].1, 2);
    }

    #[test]
    fn test_sync_merge_clocks() {
        let a = vec![(1, 3), (2, 5)];
        let b = vec![(1, 4), (3, 2)];
        let merged = sync::SyncEngine::merge_clocks(&a, &b);
        assert_eq!(merged, vec![(1, 4), (2, 5), (3, 2)]);
    }

    #[test]
    fn test_sync_concurrent_detection() {
        let a = vec![(1, 3), (2, 5)];
        let b = vec![(1, 4), (2, 4)];
        assert!(sync::SyncEngine::is_concurrent(&a, &b));
        assert!(!sync::SyncEngine::dominates(&a, &b));
        assert!(!sync::SyncEngine::dominates(&b, &a));
    }

    #[test]
    fn test_sync_not_concurrent_when_dominates() {
        let a = vec![(1, 5), (2, 5)];
        let b = vec![(1, 3), (2, 4)];
        assert!(!sync::SyncEngine::is_concurrent(&a, &b));
        assert!(sync::SyncEngine::dominates(&a, &b));
    }

    #[test]
    fn test_identity_session() {
        let session = identity::Session::new(
            "user-1".to_string(),
            "node-1".to_string(),
            DeploymentTier::T0_SoloClinician,
        );
        assert!(session.is_valid());
        assert_eq!(session.user_id, "user-1");
        assert!(!session.emergency_override);
    }

    #[test]
    fn test_role_permissions() {
        assert!(identity::Role::Clinician.can_create_patient());
        assert!(identity::Role::Clinician.can_prescribe());
        assert!(!identity::Role::Nurse.can_prescribe());
        assert!(identity::Role::EmergencyOverride.can_override());
        assert!(identity::Role::Administrator.can_admin());
    }

    #[test]
    fn test_authenticator_password_hash() {
        let (salt, hash) = identity::Authenticator::hash_password("secret123").unwrap();
        assert_eq!(salt.len(), 16);
        assert!(!hash.is_empty());
        let valid = identity::Authenticator::verify_password("secret123", &salt, &hash).unwrap();
        assert!(valid);
        let invalid = identity::Authenticator::verify_password("wrong", &salt, &hash).unwrap();
        assert!(!invalid);
    }

    #[test]
    fn test_node_identity_generation() {
        let (identity, _signer) = identity::NodeIdentity::generate(
            "Test Clinic".to_string(),
            DeploymentTier::T1_SmallClinic,
        );
        assert!(!identity.node_id.is_empty());
        assert_eq!(identity.display_name, "Test Clinic");
        assert!(!identity.public_key.is_empty());
    }

    #[test]
    fn test_serialization_roundtrip() {
        let meta = fhir::Meta::new("node-1".to_string(), "user-a".to_string());
        let patient = fhir::Patient {
            id: "pat-003".to_string(),
            meta,
            identifier: vec![],
            active: true,
            name: vec![fhir::HumanName {
                use_field: None,
                family: "Smith".to_string(),
                given: vec!["John".to_string()],
                prefix: vec![],
            }],
            telecom: vec![],
            gender: "male".to_string(),
            birth_date: "1970-05-20".to_string(),
            address: vec![],
            marital_status: None,
            general_practitioner: vec![],
            managing_organization: None,
            deceased_boolean: None,
            deceased_date_time: None,
        };
        let resource = fhir::FhirResource::Patient(patient);
        let json = serde_json::to_string(&resource).unwrap();
        let parsed: fhir::FhirResource = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id(), "pat-003");
    }

    #[test]
    fn test_consent_engine_permit_and_deny() {
        use newloka_core::consent::{ConsentDecision, ConsentDirective, ConsentEngine};
        let now = chrono::Utc::now();
        let directive = ConsentDirective {
            id: "consent-001".into(),
            patient_id: "pat-100".into(),
            status: "active".into(),
            scope: "research".into(),
            purpose: "RESEARCH".into(),
            recipient_node_id: Some("node-research-federation".into()),
            recipient_name: Some("National Medical Research Grid".into()),
            data_categories: vec!["Observation".into(), "Condition".into()],
            valid_from: now - chrono::Duration::days(10),
            valid_to: now + chrono::Duration::days(355),
            revoked: false,
            revocation_reason: None,
            revoked_at: None,
            digital_signature: Some("sig-ed25519-valid".into()),
            consent_notes: Some("Patient agreed to oncology study data sharing".into()),
        };

        let directives = vec![directive];

        // 1. Matching purpose & recipient -> Permit
        let decision = ConsentEngine::check_consent(
            &directives,
            "pat-100",
            "node-research-federation",
            "RESEARCH",
            "Observation",
            now,
        );
        assert!(matches!(decision, ConsentDecision::Permit { .. }));

        // 2. Mismatched purpose -> Deny
        let deny_purpose = ConsentEngine::check_consent(
            &directives,
            "pat-100",
            "node-research-federation",
            "COMMERCIAL_MARKETING",
            "Observation",
            now,
        );
        assert!(matches!(deny_purpose, ConsentDecision::Deny { .. }));

        // 3. Mismatched recipient -> Deny
        let deny_recipient = ConsentEngine::check_consent(
            &directives,
            "pat-100",
            "unauthorized-node",
            "RESEARCH",
            "Observation",
            now,
        );
        assert!(matches!(deny_recipient, ConsentDecision::Deny { .. }));

        // 4. Data category not consented (e.g. MedicationRequest) -> Deny
        let deny_category = ConsentEngine::check_consent(
            &directives,
            "pat-100",
            "node-research-federation",
            "RESEARCH",
            "MedicationRequest",
            now,
        );
        assert!(matches!(deny_category, ConsentDecision::Deny { .. }));
    }

    #[test]
    fn test_differential_privacy_and_deidentification() {
        use newloka_core::federation::DifferentialPrivacyEngine;

        // k-anonymity: < 5 suppressed
        assert_eq!(DifferentialPrivacyEngine::apply_k_anonymity(3, 5), None);
        assert_eq!(DifferentialPrivacyEngine::apply_k_anonymity(5, 5), Some(5));
        assert_eq!(
            DifferentialPrivacyEngine::apply_k_anonymity(120, 5),
            Some(120)
        );

        // Laplace perturbation
        let raw_val = 50.0;
        let perturbed = DifferentialPrivacyEngine::add_laplace_noise(raw_val, 1.0, 0.5, Some(0.1));
        assert!(perturbed > 0.0);
        assert_ne!(perturbed, raw_val);

        // De-identification
        let meta = fhir::Meta::new("node-1".to_string(), "user-a".to_string());
        let patient = fhir::Patient {
            id: "pat-secret-123".to_string(),
            meta,
            identifier: vec![],
            active: true,
            name: vec![fhir::HumanName {
                use_field: None,
                family: "Sharma".to_string(),
                given: vec!["Ramesh".to_string()],
                prefix: vec![],
            }],
            telecom: vec![],
            gender: "male".to_string(),
            birth_date: "1968-07-15".to_string(),
            address: vec![],
            marital_status: None,
            general_practitioner: vec![],
            managing_organization: None,
            deceased_boolean: None,
            deceased_date_time: None,
        };

        let deid = DifferentialPrivacyEngine::deidentify_patient(
            &patient,
            vec!["E11 - Type 2 Diabetes".into()],
            vec!["Metformin 500mg".into()],
            vec!["HbA1c 7.8%".into()],
            "institutional-salt-42",
        );

        // Verify direct identifiers are absent and age bracket is computed
        assert!(deid.pseudo_id.starts_with("SUBJ-"));
        assert!(!deid.pseudo_id.contains("pat-secret-123"));
        assert_eq!(deid.gender, "male");
        assert_eq!(deid.conditions.len(), 1);
        assert_eq!(deid.medications.len(), 1);
    }

    #[test]
    fn test_department_defaults_and_transfers() {
        use newloka_core::department::{
            default_departments, DepartmentTransferRequest, TransferStatus,
        };

        let depts = default_departments();
        assert!(depts.len() >= 8);
        assert!(depts.iter().any(|d| d.code == "CARD"));
        assert!(depts.iter().any(|d| d.code == "EMERG"));
        assert!(depts.iter().any(|d| d.code == "ICU"));

        let transfer = DepartmentTransferRequest {
            id: "tr-001".into(),
            patient_id: "pat-500".into(),
            patient_name: "Anita Deshmukh".into(),
            mrn: Some("MRN-7821".into()),
            source_department: "dept-emerg".into(),
            target_department: "dept-cardio".into(),
            requesting_clinician_id: "user-er-doc".into(),
            requesting_clinician_name: "Dr. ER Attending".into(),
            accepting_clinician_id: None,
            accepting_clinician_name: None,
            assigned_bed: None,
            urgency: "urgent".into(),
            reason: "Non-ST Elevation Myocardial Infarction".into(),
            clinical_handover_notes:
                "SBAR: 54F presenting with crushing substernal chest pain. Trop I positive.".into(),
            active_medications_summary: vec![
                "Aspirin 325mg stat".into(),
                "Clopidogrel 300mg stat".into(),
            ],
            pending_orders_summary: vec!["Urgent Echo".into()],
            allergies_summary: vec!["Penicillin (Severe)".into()],
            status: TransferStatus::Requested,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            handover_signed_by: Some("Dr. ER Attending".into()),
        };

        assert_eq!(transfer.status, TransferStatus::Requested);
        assert_eq!(transfer.status.as_str(), "requested");
    }

    #[test]
    fn test_enterprise_directory_ldap_auth() {
        use newloka_core::identity::EnterpriseDirectoryConfig;

        let mut config = EnterpriseDirectoryConfig::default();
        config.enabled = true;

        // Valid doctor authentication
        let auth_doc = config
            .authenticate("dr.sharma@hospital.local", "securepassword123")
            .unwrap();
        assert!(auth_doc.authenticated);
        assert!(auth_doc
            .roles
            .contains(&newloka_core::identity::Role::Clinician));

        // Valid admin authentication
        let auth_admin = config
            .authenticate("admin-sys@hospital.local", "adminpass")
            .unwrap();
        assert!(auth_admin
            .roles
            .contains(&newloka_core::identity::Role::Administrator));

        // Valid cardio specialist authentication
        let auth_cardio = config
            .authenticate("cardio.lead@hospital.local", "docpass")
            .unwrap();
        assert_eq!(auth_cardio.department_id, Some("dept-cardio".into()));

        // Empty password should fail
        assert!(config
            .authenticate("dr.sharma@hospital.local", "")
            .is_none());
    }

    #[test]
    fn test_pure_clinical_scoring_invariants() {
        use newloka_core::clinical::{
            calculate_egfr_ckd_epi, calculate_news2, calculate_pediatric_dose, evaluate_dka_risk,
            AvpuScore, BiologicalSex, ClinicalRiskTier, DkaSeverity,
        };

        // Strict deterministic NEWS2 calculation
        let normal = calculate_news2(16, 99, false, 120, 70, AvpuScore::Alert, 36.6);
        assert_eq!(normal.total_score, 0);
        assert_eq!(normal.risk_tier, ClinicalRiskTier::Low);
        assert!(!normal.triggers_stat_alert);

        let acute = calculate_news2(25, 91, true, 88, 125, AvpuScore::Voice, 39.2);
        // RR 25 (3), SpO2 91 (3), O2 (2), SBP 88 (3), HR 125 (2), AVPU Voice (3), Temp 39.2 (2) = 18
        assert_eq!(acute.total_score, 18);
        assert_eq!(acute.risk_tier, ClinicalRiskTier::High);
        assert!(acute.triggers_stat_alert);

        // eGFR calculations
        let normal_male = calculate_egfr_ckd_epi(0.9, 40, BiologicalSex::Male).unwrap();
        assert!(normal_male.egfr_value >= 90.0);
        assert!(!normal_male.requires_dose_adjustment);

        let renal_failure = calculate_egfr_ckd_epi(4.0, 70, BiologicalSex::Female).unwrap();
        assert!(renal_failure.egfr_value < 15.0);
        assert!(renal_failure.requires_dose_adjustment);

        // Pediatric dosing
        assert_eq!(calculate_pediatric_dose(10.0, 15.0, 500.0), 150.0);
        assert_eq!(calculate_pediatric_dose(40.0, 15.0, 500.0), 500.0); // capped at 500

        // DKA triage
        assert_eq!(
            evaluate_dka_risk(400.0, 6.90, 6.0, true),
            DkaSeverity::Severe
        );
        assert_eq!(
            evaluate_dka_risk(120.0, 7.40, 24.0, false),
            DkaSeverity::None
        );
    }
}
