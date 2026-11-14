# FHIR R4 Clinical Data Models

## Epistemic Metadata
- **Entity**: FHIR R4 Clinical Resource Models
- **Layer**: Layer 1: Structural & Architectural
- **Status**: SHIPPED
- **Domain**: Clinical Data Architecture & Standardization

## Facts
- Core Standard: All clinical data inside New Loka is natively modeled as HL7 FHIR Release 4 (R4) resources, eliminating proprietary vendor schemas and ensuring seamless interoperability with global healthcare standards. [valid_from: 2026-06-11, recorded: 2026-11-13, source: newloka_core/src/fhir/mod.rs]
- Supported Resource Types: The Rust core engine (`newloka_core::fhir`) and Axum server implement typed structs, validation, and JSON serialization for 23+ clinical resources:
  1. Patient (identifiers, telecom, address, demographics)
  2. Encounter (ambulatory, emergency, inpatient, period, reason, service provider)
  3. Observation (vitals, laboratory values, LOINC codes, reference ranges)
  4. Condition (clinicalStatus, verificationStatus, severity, ICD-10/SNOMED CT)
  5. MedicationRequest (dosage, route, frequency, refills, priority)
  6. MedicationDispense (status, quantity, handed-over timestamps)
  7. MedicationAdministration (24h nursing MAR records, dosage given)
  8. MedicationStatement (patient-reported or historical medications)
  9. Procedure (performed period, outcome, performer, SNOMED coding)
  10. AllergyIntolerance (substance, criticality, verification, manifestation)
  11. Flag (critical clinical alerts, fall risk, infection isolation)
  12. DetectedIssue (drug-drug interactions, contraindications, duplicate therapy)
  13. Composition (structured clinical notes, discharge summaries, SOAP notes)
  14. CarePlan (goals, activities, care team, chronic condition management)
  15. FamilyMemberHistory (relationship, conditions, genetic risk factors)
  16. Immunization (vaccine code, lot number, site, route, administration date)
  17. DocumentReference (clinical media, lab PDFs, scan attachments, MIME types)
  18. ServiceRequest (CPOE orders for lab, imaging, and specialist consultations)
  19. DiagnosticReport (laboratory and imaging narrative reports and conclusions)
  20. ImagingStudy (modality, series, DICOM instance UID references)
  21. Specimen (collection timing, source site, container, accession identifier)
  22. AuditEvent (cryptographic tamper-evident operational logs)
  23. Consent (privacy provisions, purpose constraints, research access policies)
  [valid_from: 2026-11-14, recorded: 2026-11-14, source: newloka_core/src/fhir/mod.rs & newloka_server/src/lib.rs]
- Unified Resource Wrapper (`FhirResource`): Polymorphic enum encapsulating all concrete resources, enabling generic storage, encryption, CRDT syncing, and ABAC evaluation without dynamic dispatch overhead. [valid_from: 2026-06-11, recorded: 2026-11-13, source: newloka_core/src/fhir/mod.rs]
- Soft Delete Invariant: FHIR resources in New Loka are never hard-deleted from storage; updates set `deleted_at` timestamps to preserve clinical audit trails, legal defense veracity, and CRDT synchronization tombstone propagation. [valid_from: 2026-06-11, recorded: 2026-11-13, source: newloka_core/src/storage/mod.rs & Project_Scope_and_Requirements.txt §4.1]
- Provenance Integration: Every clinical mutation records origin node ID, author identity, timestamp, and signature, linking to `Provenance` and `AuditEvent` resources. [valid_from: 2026-06-11, recorded: 2026-11-13, source: newloka_core/src/fhir/mod.rs]

## Superseded
(none)
