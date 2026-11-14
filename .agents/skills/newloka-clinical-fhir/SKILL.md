---
name: newloka-clinical-fhir
version: 1.0.0
description: "Author, model, and validate FHIR R4 clinical resources, CPOE workflows, eRx formulary integration, and medical safety invariants in New Loka."
author: Amethyst Agent
license: GPL-3.0
platforms: [windows, linux, macos]
metadata:
  tags: [FHIR, R4, Clinical, CPOE, ePrescription, Healthcare]
---

# New Loka FHIR R4 & Clinical Systems Engineering

Guides the design, extension, validation, and maintenance of clinical resources and workflows in New Loka.

## 1. FHIR R4 Clinical Standards Mapping

All medical records in New Loka are modeled according to HL7 FHIR R4:
- **Demographics & Identification:** `Patient` (with national identifiers, ABHA ID, contact, gender, birthDate).
- **Episodes of Care:** `Encounter` (status, class: ambulatory/inpatient/emergency, serviceProvider, period).
- **Diagnostics & Metrics:** `Observation` (vital signs, laboratory quantitative values, LOINC codes, reference intervals).
- **Problems & Diagnoses:** `Condition` (clinicalStatus, verificationStatus, ICD-10 / SNOMED CT terminology).
- **Medications & Pharmacy:** `MedicationRequest` (dosage, route, frequency), `MedicationDispense`, `MedicationAdministration` (24h MAR), `MedicationStatement`.
- **Procedures & Interventions:** `Procedure` (status, code, performedPeriod, performer).
- **Allergies & Risk:** `AllergyIntolerance` (criticality, category, manifestation), `Flag` (NEWS2 panic alerts, fall risk).
- **Clinical Documentation:** `Composition` (structured SOAP notes, discharge summaries), `DocumentReference` (PDFs, ECGs, scans).
- **CPOE Orders & Results:** `ServiceRequest` (lab/imaging orders), `DiagnosticReport` (lab/imaging results), `Specimen` (sample tracking), `ImagingStudy` (DICOM references).

## 2. CPOE Order Lifecycle

Maintain the 5-stage order state machine in `newloka_core::cpoe`:
1. `Draft`: Clinician drafting order set.
2. `Requested`: Transmitted to target laboratory queue.
3. `InProgress`: Specimen collected/accessioned or imaging study initiated.
4. `Completed`: DiagnosticReport or Observation attached and signed off.
5. `Cancelled`: Annulled with mandatory clinical reason.

### Priorities:
- `routine`: Standard ward/outpatient pacing.
- `urgent`: Expedited outpatient or surgical workup.
- `stat`: Immediate emergency execution with panic notification.

## 3. Departmental Silos & ABAC Rules
In multi-department hospital deployments (T3+):
- Biochemistry staff only have access to biochemistry orders and results.
- Radiologists have Read/Create/Update permissions exclusively within Radiology imaging studies and reports.
- Pathologists access tissue and cytology specimens and reports.
- Microbiologists access culture, sensitivity, and organism identification data.
- Cross-department access requires explicit **Emergency Override ('Break Glass')** with mandatory clinical reason logging and high-severity audit logging.

## 4. Clinical Safety Invariants
- **Non-Dangerous Merge:** MedicationRequest, AllergyIntolerance, and critical ServiceRequest resources must never be auto-resolved upon concurrent sync conflicts. They must generate a flagged clinical conflict requiring clinician resolution.
- **Zero Auto-Commit:** Any AI, OCR, or speech-to-text extraction must be staged in a review interface (`/static/#ingest`) and requires human clinician approval before chart commit.
