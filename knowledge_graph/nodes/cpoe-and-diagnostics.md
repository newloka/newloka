# Computerized Physician Order Entry (CPOE) and Diagnostics

## Epistemic Metadata
- **Entity**: CPOE Workflows & Departmental Diagnostics
- **Layer**: Layer 2: Operational & Functional
- **Status**: SHIPPED
- **Domain**: Diagnostic Ordering, Laboratory Silos & Results Review

## Facts
- Order Lifecycle Management: Computerized Physician Order Entry (CPOE) supports ordering of laboratory tests, diagnostic imaging studies, nursing interventions, and procedures across a 5-stage lifecycle (Draft, Requested/Active, In-Progress, Completed with DiagnosticReport/Observation attached, and Cancelled with mandatory clinical reason). [valid_from: 2026-06-20, recorded: 2026-11-13, source: newloka_core/src/cpoe/mod.rs & Project_Scope_and_Requirements.txt §4.2]
- Order Prioritization Levels: Orders carry explicit priority levels including Routine (standard turn-around), Urgent (elevated outpatient or pre-op priority), and Stat (immediate execution with urgent clinical alert signaling). [valid_from: 2026-06-20, recorded: 2026-11-13, source: newloka_core/src/cpoe/mod.rs]
- Departmental Diagnostic Configurations: Configurable diagnostic departments isolate Radiology (ImagingStudy CT/X-ray/MRI metadata and narrative impressions), Biochemistry (quantitative LOINC observations, reference ranges, panic alerts), Pathology (biopsy, cytology, histological staging), and Microbiology (specimen culture, colony counts, antibiotic sensitivity panels). [valid_from: 2026-06-20, recorded: 2026-11-13, source: Project_Scope_and_Requirements.txt §4.3 & newloka_server/src/lib.rs]
- Results Review & Panic Lab Alerts: In `newloka_web`, the `resultsReview` view surfaces pending orders, critical lab flags (panic thresholds via NEWS2 and LOINC reference intervals), and direct clinician sign-off actions. [valid_from: 2026-09-06, recorded: 2026-11-13, source: newloka_web/js/config.js & newloka_web/js/app.js]
- CPOE Order Sets: Clinicians can order standardized sets (such as 'Fever Workup', 'Diabetic Baseline', or 'Chest Pain Protocol') with a single click, automatically generating linked ServiceRequest, Specimen, and Observation records. [valid_from: 2026-09-06, recorded: 2026-11-13, source: newloka_web/js/app.js]

## Superseded
(none)
