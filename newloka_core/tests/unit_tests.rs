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
        let valid = crypto::verify_audit_signature(&verifying_key, message, &signature,
        ).unwrap();
        assert!(valid);
    }

    #[test]
    fn test_audit_chain_integrity() {
        let node_id = "node-test".to_string();
        let signer = crypto::AuditSigner::generate();
        let mut engine = audit::AuditEngine::new(node_id.clone(), signer.clone());

        let entry1 = engine.log(
            audit::AuditEventType::Create,
            "user-1".to_string(),
            Some("pat-001".to_string()),
            Some("enc-001".to_string()),
            Some("Encounter".to_string()),
            "create encounter".to_string(),
            audit::AuditOutcome::Success,
            None,
        ).unwrap();

        let entry2 = engine.log(
            audit::AuditEventType::Access,
            "user-1".to_string(),
            Some("pat-001".to_string()),
            Some("enc-001".to_string()),
            Some("Encounter".to_string()),
            "read encounter".to_string(),
            audit::AuditOutcome::Success,
            None,
        ).unwrap();

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
            },
            resource: abac::Resource {
                resource_type: "Patient".to_string(),
                resource_id: "pat-001".to_string(),
                patient_id: Some("pat-001".to_string()),
                department_id: Some("dept-a".to_string()),
                owner_team_ids: vec!["team-1".to_string()],
                sensitivity: abac::SensitivityLevel::Normal,
            },
            action: abac::Action::Create,
            context: abac::Context {
                tier: DeploymentTier::T3_MultiDepartmentHospital,
                offline: false,
                peer_node_id: None,
                time_of_day: "14:00".to_string(),
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
            },
            resource: abac::Resource {
                resource_type: "MedicationRequest".to_string(),
                resource_id: "med-001".to_string(),
                patient_id: Some("pat-001".to_string()),
                department_id: Some("dept-a".to_string()),
                owner_team_ids: vec![],
                sensitivity: abac::SensitivityLevel::Normal,
            },
            action: abac::Action::Override,
            context: abac::Context {
                tier: DeploymentTier::T3_MultiDepartmentHospital,
                offline: false,
                peer_node_id: None,
                time_of_day: "09:00".to_string(),
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
            },
            resource: abac::Resource {
                resource_type: "Patient".to_string(),
                resource_id: "pat-001".to_string(),
                patient_id: Some("pat-001".to_string()),
                department_id: Some("dept-b".to_string()),
                owner_team_ids: vec!["team-2".to_string()],
                sensitivity: abac::SensitivityLevel::Normal,
            },
            action: abac::Action::Read,
            context: abac::Context {
                tier: DeploymentTier::T3_MultiDepartmentHospital,
                offline: false,
                peer_node_id: None,
                time_of_day: "10:00".to_string(),
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
            },
            resource: abac::Resource {
                resource_type: "Patient".to_string(),
                resource_id: "pat-001".to_string(),
                patient_id: Some("pat-001".to_string()),
                department_id: Some("dept-b".to_string()),
                owner_team_ids: vec![],
                sensitivity: abac::SensitivityLevel::Critical,
            },
            action: abac::Action::Read,
            context: abac::Context {
                tier: DeploymentTier::T3_MultiDepartmentHospital,
                offline: false,
                peer_node_id: None,
                time_of_day: "02:00".to_string(),
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
}
