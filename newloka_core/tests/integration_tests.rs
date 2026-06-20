//! Integration tests for New Loka
//!
//! These tests require a running SQLite database and test end-to-end flows.

#[cfg(test)]
mod integration {
    use newloka_core::*;

    async fn setup_storage() -> (storage::StorageEngine, String) {
        let node_id = "test-node".to_string();
        let dmk = crypto::DeviceMasterKey::generate();
        let db_path = format!(
            "file:test_{}?mode=memory&cache=shared",
            uuid::Uuid::new_v4()
        );
        let storage = storage::StorageEngine::open(&db_path, node_id.clone(), dmk)
            .await
            .unwrap();
        (storage, node_id)
    }

    fn create_test_patient(node_id: &str, user_id: &str) -> fhir::Patient {
        let meta = fhir::Meta::new(node_id.to_string(), user_id.to_string());
        fhir::Patient {
            id: uuid::Uuid::new_v4().to_string(),
            meta,
            identifier: vec![fhir::Identifier {
                system: "newloka".to_string(),
                value: "MRN-12345".to_string(),
                use_field: Some("official".to_string()),
            }],
            active: true,
            name: vec![fhir::HumanName {
                use_field: Some("official".to_string()),
                family: "TestPatient".to_string(),
                given: vec!["Alice".to_string()],
                prefix: vec![],
            }],
            telecom: vec![fhir::ContactPoint {
                system: "phone".to_string(),
                value: "+91-9876543210".to_string(),
                use_field: Some("mobile".to_string()),
            }],
            gender: "female".to_string(),
            birth_date: "1990-07-12".to_string(),
            address: vec![fhir::Address {
                use_field: Some("home".to_string()),
                line: vec!["123 Health Street".to_string()],
                city: Some("Mumbai".to_string()),
                district: Some("Mumbai Suburban".to_string()),
                state: Some("Maharashtra".to_string()),
                postal_code: Some("400001".to_string()),
                country: Some("India".to_string()),
            }],
            marital_status: Some(fhir::CodeableConcept {
                text: Some("Married".to_string()),
                coding: vec![fhir::Coding {
                    system: "http://terminology.hl7.org/CodeSystem/v3-MaritalStatus".to_string(),
                    code: "M".to_string(),
                    display: Some("Married".to_string()),
                }],
            }),
            general_practitioner: vec![],
            managing_organization: None,
            deceased_boolean: None,
            deceased_date_time: None,
        }
    }

    fn create_test_encounter(node_id: &str, user_id: &str, patient_id: &str) -> fhir::Encounter {
        let meta = fhir::Meta::new(node_id.to_string(), user_id.to_string());
        fhir::Encounter {
            id: uuid::Uuid::new_v4().to_string(),
            meta,
            status: "in-progress".to_string(),
            class: fhir::Coding {
                system: "http://terminology.hl7.org/CodeSystem/v3-ActCode".to_string(),
                code: "AMB".to_string(),
                display: Some("ambulatory".to_string()),
            },
            type_: vec![fhir::CodeableConcept {
                text: Some("General Checkup".to_string()),
                coding: vec![fhir::Coding {
                    system: "http://snomed.info/sct".to_string(),
                    code: "185345009".to_string(),
                    display: Some("Encounter for check up".to_string()),
                }],
            }],
            subject: fhir::Reference {
                reference: format!("Patient/{}", patient_id),
                display: Some("Alice TestPatient".to_string()),
            },
            participant: vec![fhir::EncounterParticipant {
                type_: vec![fhir::CodeableConcept {
                    text: Some("Primary performer".to_string()),
                    coding: vec![],
                }],
                period: None,
                individual: fhir::Reference {
                    reference: "Practitioner/dr-001".to_string(),
                    display: Some("Dr. Sharma".to_string()),
                },
            }],
            period: fhir::Period {
                start: Some(chrono::Utc::now()),
                end: None,
            },
            location: vec![],
            reason_code: vec![fhir::CodeableConcept {
                text: Some("Routine examination".to_string()),
                coding: vec![],
            }],
            diagnosis: vec![],
            service_provider: Some(fhir::Reference {
                reference: "Organization/clinic-001".to_string(),
                display: Some("City Health Clinic".to_string()),
            }),
            part_of: None,
        }
    }

    #[tokio::test]
    async fn test_storage_patient_roundtrip() {
        let (storage, node_id) = setup_storage().await;
        let patient = create_test_patient(&node_id, "user-1");
        let patient_id = patient.id.clone();
        let resource = fhir::FhirResource::Patient(patient);

        let id = storage
            .store(
                &resource,
                Some(patient_id.clone()),
                Some("dept-general".to_string()),
            )
            .await
            .unwrap();
        assert!(!id.is_empty());

        let retrieved = storage.get(&id).await.unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id(), patient_id);
        assert_eq!(retrieved.resource_type(), "Patient");
    }

    #[tokio::test]
    async fn test_storage_search_by_patient() {
        let (storage, node_id) = setup_storage().await;
        let patient = create_test_patient(&node_id, "user-1");
        let patient_id = patient.id.clone();
        let resource = fhir::FhirResource::Patient(patient);
        storage
            .store(
                &resource,
                Some(patient_id.clone()),
                Some("dept-general".to_string()),
            )
            .await
            .unwrap();

        let encounter = create_test_encounter(&node_id, "user-1", &patient_id);
        let enc_id = encounter.id.clone();
        let enc_resource = fhir::FhirResource::Encounter(encounter);
        storage
            .store(
                &enc_resource,
                Some(patient_id.clone()),
                Some("dept-general".to_string()),
            )
            .await
            .unwrap();

        let results = storage
            .search("Encounter", Some(&patient_id))
            .await
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id(), enc_id);
    }

    #[tokio::test]
    async fn test_storage_soft_delete() {
        let (storage, node_id) = setup_storage().await;
        let patient = create_test_patient(&node_id, "user-1");
        let patient_id = patient.id.clone();
        let resource = fhir::FhirResource::Patient(patient);
        storage
            .store(&resource, Some(patient_id.clone()), None)
            .await
            .unwrap();

        let deleted = storage.soft_delete(&patient_id).await.unwrap();
        assert!(deleted);

        let retrieved = storage.get(&patient_id).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_audit_integration_with_storage() {
        let node_id = "audit-node".to_string();
        let signer = crypto::AuditSigner::generate();
        let mut engine = audit::AuditEngine::new(node_id.clone(), signer);

        let entry = engine
            .log(
                audit::AuditEventType::Create,
                "user-1".to_string(),
                Some("pat-001".to_string()),
                Some("enc-001".to_string()),
                Some("Encounter".to_string()),
                "create encounter during integration test".to_string(),
                audit::AuditOutcome::Success,
                None,
            )
            .unwrap();

        assert_eq!(entry.node_id, node_id);
        assert_eq!(entry.event_type, audit::AuditEventType::Create);
        assert_eq!(entry.outcome, audit::AuditOutcome::Success);
    }

    #[tokio::test]
    async fn test_sync_delta_build() {
        let (storage, node_id) = setup_storage().await;
        let patient = create_test_patient(&node_id, "user-1");
        let patient_id = patient.id.clone();
        let resource = fhir::FhirResource::Patient(patient);
        storage
            .store(&resource, Some(patient_id.clone()), None)
            .await
            .unwrap();

        let engine = sync::SyncEngine::new(node_id.clone());
        let since = 0i64;
        let delta = engine.build_manifest(&storage, since).await.unwrap();
        assert_eq!(delta.from_node, node_id);
        assert_eq!(delta.records.len(), 1);
        assert_eq!(delta.records[0].resource_type, "Patient");
    }

    #[tokio::test]
    async fn test_abac_integration_with_session() {
        let user = identity::User {
            id: "dr-sharma".to_string(),
            username: "sharma".to_string(),
            display_name: "Dr. Sharma".to_string(),
            roles: vec![identity::Role::Clinician],
            department_id: Some("dept-cardiology".to_string()),
            team_ids: vec!["team-cardio-1".to_string()],
            active: true,
            created_at: chrono::Utc::now(),
            password_hash: "hash".to_string(),
            salt: vec![],
            totp_secret: None,
            last_login: None,
            lab_affiliations: vec![],
        };

        let session = identity::Session::new(
            user.id.clone(),
            "node-1".to_string(),
            DeploymentTier::T3_MultiDepartmentHospital,
        );

        assert!(session.is_valid());
        assert_eq!(session.user_id, "dr-sharma");

        let req = abac::PolicyRequest {
            subject: abac::Subject {
                user_id: user.id,
                roles: user.roles,
                department_id: user.department_id,
                team_ids: user.team_ids,
                session_valid: session.is_valid(),
                emergency_override: session.emergency_override,
                lab_affiliations: vec![],
            },
            resource: abac::Resource {
                resource_type: "Patient".to_string(),
                resource_id: "pat-cardio-001".to_string(),
                patient_id: Some("pat-cardio-001".to_string()),
                department_id: Some("dept-cardiology".to_string()),
                owner_team_ids: vec!["team-cardio-1".to_string()],
                sensitivity: abac::SensitivityLevel::Normal,
                lab_department: None,
            },
            action: abac::Action::Create,
            context: abac::Context {
                tier: session.tier,
                offline: false,
                peer_node_id: None,
                time_of_day: "10:30".to_string(),
                lab_config: cpoe::LabConfiguration::default(),
                patient_has_lab_order: false,
            },
        };

        let decision = abac::PolicyEngine::evaluate(&req);
        assert_eq!(decision, abac::PolicyDecision::Allow);
    }

    #[tokio::test]
    async fn test_end_to_end_patient_workflow() {
        let (storage, node_id) = setup_storage().await;
        let _dmk = crypto::DeviceMasterKey::generate();
        let signer = crypto::AuditSigner::generate();
        let mut audit = audit::AuditEngine::new(node_id.clone(), signer);

        // 1. Create patient
        let patient = create_test_patient(&node_id, "dr-sharma");
        let patient_id = patient.id.clone();
        let resource = fhir::FhirResource::Patient(patient);
        storage
            .store(
                &resource,
                Some(patient_id.clone()),
                Some("dept-general".to_string()),
            )
            .await
            .unwrap();

        audit
            .log(
                audit::AuditEventType::Create,
                "dr-sharma".to_string(),
                Some(patient_id.clone()),
                Some(patient_id.clone()),
                Some("Patient".to_string()),
                "Register new patient".to_string(),
                audit::AuditOutcome::Success,
                None,
            )
            .unwrap();

        // 2. Create encounter
        let encounter = create_test_encounter(&node_id, "dr-sharma", &patient_id);
        let enc_id = encounter.id.clone();
        let enc_resource = fhir::FhirResource::Encounter(encounter);
        storage
            .store(
                &enc_resource,
                Some(patient_id.clone()),
                Some("dept-general".to_string()),
            )
            .await
            .unwrap();

        audit
            .log(
                audit::AuditEventType::Create,
                "dr-sharma".to_string(),
                Some(patient_id.clone()),
                Some(enc_id.clone()),
                Some("Encounter".to_string()),
                "Start consultation".to_string(),
                audit::AuditOutcome::Success,
                None,
            )
            .unwrap();

        // 3. Add observation
        let meta = fhir::Meta::new(node_id.clone(), "dr-sharma".to_string());
        let observation = fhir::Observation {
            id: uuid::Uuid::new_v4().to_string(),
            meta,
            status: "final".to_string(),
            category: vec![fhir::CodeableConcept {
                text: Some("Vital Signs".to_string()),
                coding: vec![fhir::Coding {
                    system: "http://terminology.hl7.org/CodeSystem/observation-category"
                        .to_string(),
                    code: "vital-signs".to_string(),
                    display: Some("Vital Signs".to_string()),
                }],
            }],
            code: fhir::CodeableConcept {
                text: Some("Body temperature".to_string()),
                coding: vec![fhir::Coding {
                    system: "http://loinc.org".to_string(),
                    code: "8310-5".to_string(),
                    display: Some("Body temperature".to_string()),
                }],
            },
            subject: fhir::Reference {
                reference: format!("Patient/{}", patient_id),
                display: Some("Alice TestPatient".to_string()),
            },
            encounter: Some(fhir::Reference {
                reference: format!("Encounter/{}", enc_id),
                display: None,
            }),
            effective_date_time: Some(chrono::Utc::now()),
            issued: Some(chrono::Utc::now()),
            performer: vec![fhir::Reference {
                reference: "Practitioner/dr-sharma".to_string(),
                display: Some("Dr. Sharma".to_string()),
            }],
            value_quantity: Some(fhir::Quantity {
                value: 37.2,
                unit: "Cel".to_string(),
                system: Some("http://unitsofmeasure.org".to_string()),
                code: Some("Cel".to_string()),
            }),
            value_string: None,
            value_codeable_concept: None,
            interpretation: vec![],
            note: vec![],
            reference_range: vec![fhir::ObservationReferenceRange {
                low: Some(fhir::Quantity {
                    value: 36.5,
                    unit: "Cel".to_string(),
                    system: None,
                    code: None,
                }),
                high: Some(fhir::Quantity {
                    value: 37.5,
                    unit: "Cel".to_string(),
                    system: None,
                    code: None,
                }),
                type_: None,
                text: Some("Normal range".to_string()),
            }],
            component: vec![],
        };
        let obs_resource = fhir::FhirResource::Observation(observation);
        storage
            .store(
                &obs_resource,
                Some(patient_id.clone()),
                Some("dept-general".to_string()),
            )
            .await
            .unwrap();

        // 4. Verify patient chart
        let encounters = storage
            .search("Encounter", Some(&patient_id))
            .await
            .unwrap();
        assert_eq!(encounters.len(), 1);

        let observations = storage
            .search("Observation", Some(&patient_id))
            .await
            .unwrap();
        assert_eq!(observations.len(), 1);

        // 5. Verify audit trail
        assert_eq!(audit.entries().len(), 2);
    }
}
