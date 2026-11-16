# Web Clinical Interface and Single-Page Application

## Epistemic Metadata
- **Entity**: New Loka Web Single-Page Application (`newloka_web`)
- **Layer**: Layer 2: Operational & Functional
- **Status**: SHIPPED
- **Domain**: Clinical UI/UX, Point-of-Care Workflows & Offline Cache

## Facts
- Zero-Framework Architecture: Written in modular ES6 Vanilla JavaScript and Tailwind CSS without heavy framework dependencies (React, Angular, or Vue). Delivers instant rendering, negligible memory footprint, and high responsiveness on low-end hardware. [valid_from: 2026-06-11, recorded: 2026-11-13, source: newloka_web/js/app.js & docs/ARCHITECTURE.md]
- 35 Verified Clinical Views: Complete point-of-care clinical management across:
  1. `dashboard`: Clinic operational overview, pending reviews, appointment list, panic alerts.
  2. `patients`: Directory, search, demographic registration, photo capture.
  3. `patientChart`: Comprehensive longitudinal chart (vitals, diagnoses, timelines, meds).
  4. `appointmentSchedule`: Outpatient scheduling, queue management, visit status.
  5. `clinicalNotes`: SOAP progress notes, clinical phrases, templates, auto-formatting.
  6. `cpoeOrders`: Diagnostic order sets, priority selection (Routine/Urgent/Stat).
  7. `drugInteraction`: Automated safety checks against allergy and medication profiles.
  8. `resultsReview`: Diagnostic laboratory and imaging reviews with panic thresholds.
  9. `vitalsFlowsheet`: Longitudinal vitals tracking, trends, NEWS2 early warning scoring.
  10. `alerts`: Active patient risk flags, fall risks, infection isolation badges.
  11. `carePlans`: Multi-disciplinary goals, chronic disease care paths.
  12. `familyHistory`: Pedigree condition tracking and familial risk markers.
  13. `encounters`: Visit records, consultation room tracking, episode management.
  14. `observations`: Granular diagnostic metrics, LOINC code mappings.
  15. `conditions`: Active, resolved, and chronic problem lists (ICD-10 / SNOMED).
  16. `medications`: Active and historical medication lists, dosage schedules.
  17. `rxpad`: Full-fidelity embedded eRx Pad Pro engine and Prescriptions Queue with authentic signature image and letterhead preservation.
  18. `inpatientRegister`: Senior Resident ward census, bed list, and NEWS2 acuity tracking.
  19. `billing`: Receptionist and cashier invoicing ledger with ABAC clinical data redaction.
  20. `deptSummary`: Pharmacy head activity telemetry and flashing critical stockout alerts.
  21. `procedures`: Minor surgeries, bedside interventions, procedure notes.
  22. `immunizations`: National immunization schedule tracking, batch records.
  23. `documents`: Medical document attachments, ECGs, scans, discharge summaries.
  24. `ingest`: AI document scanner, handwriting OCR, and voice note review queues.
  25. `mar`: Inpatient 24-hour nursing Medication Administration Record.
  26. `handoffSbar`: Structured Situation-Background-Assessment-Recommendation shift handoffs.
  27: `whiteboard`: Inpatient ward census, bed assignment, patient movement boards.
  28. `audit`: Real-time cryptographic audit log explorer and search.
  29. `mesh`: P2P LAN peer discovery and sync management status.
  30. `settings`: Practice profile presets, 28 feature toggles, in-app updater, offline backup.
  31. `departments`: Hospital departmental census, bed capacities, clinical chiefs, and SBAR inter-department patient transfers.
  32. `reports`: Executive governance dashboard, occupancy rates, ALOS, emergency TAT, and departmental KPI telemetry.
  33. `research`: Privacy-preserving cohort queries, Laplace differential privacy (epsilon-calibrated), and k-anonymity suppressed exports.
  34. `consent`: FHIR R4 Consent directives registry, granular purpose/provision rules, revocation workflow, and real-time verification console.
  35. `federation`: Health data mesh topology, live peer latency telemetry, cross-hospital exchange log, and academic network synchronization.
  [valid_from: 2026-11-14, recorded: 2026-11-14, source: newloka_web/js/config.js & newloka_web/js/app.js]
- Role-Based Startup Views & Polymorphic Config: Supports install-time tier constraints (T0–T4) and granular instance-level user/admin visibility toggles. Configures automated landing view routing on launch: Doctors open to native Prescription Pad with loaded pharmacopoeia, Senior Residents to Inpatient Register with ward census & NEWS2 acuity, Reception/Cashier to Billing with ABAC-redacted diagnoses, and Pharmacy Heads to Department Summary with a pulsing alert banner. [valid_from: 2026-11-14, recorded: 2026-11-14, source: newloka_web/js/config.js & newloka_web/js/app.js]
- Persistent Storyboard Context Rail: Anchored left-hand clinical context rail (`.storyboard-rail`, 260px) presenting instant glanceable patient identification (photo avatar, name, age/gender, MRN, code status badge, attending physician and care team, allergy pills, and recent vitals) to eliminate chart-diving latency across point-of-care clinical screens. [valid_from: 2026-11-14, recorded: 2026-11-14, source: newloka_web/js/app.js & docs/reference/epic_systems_architecture_and_ui_trees.md]
- Enterprise Role Presets Engine (`PRESET_REGISTRY`): Exhaustive declarative presets mapping 22 clinical and operational roles across Physicians, Nursing, Diagnostics & Pharmacy, and Administration into tailored viewports, active module sets, default landing routes, and ABAC constraints with on-demand admin provisioning. [valid_from: 2026-11-14, recorded: 2026-11-14, source: newloka_web/js/config.js & docs/role_ui_matrix.md]
- Three-Tap Usability Invariant: The clinical UI guarantees that critical patient information (allergies, emergency notes, active medications, vital trends) is accessible within 3 taps or clicks from any screen. [valid_from: 2026-06-11, recorded: 2026-11-13, source: Project_Scope_and_Requirements.txt §4.9]
- Offline-First UI State Machine: Uses IndexedDB to cache active patients, charts, and formulary data. Offline write queues automatically record changes locally when disconnected from the LAN server and sync upon reconnection. [valid_from: 2026-06-11, recorded: 2026-11-13, source: newloka_web/js/api.js]
- Commercial EMR Benchmark & Interaction Patterns: Codifies comparative capability matrix against Epic Hyperspace, Oracle Health/Cerner, MEDITECH, and athenahealth, alongside 8 canonical clinical interaction patterns (Storyboard banner, Navigator census, CPOE ordersets, MAR grid, NEWS2 flowsheet, SBAR handoff, Cadence schedule, Whiteboard census) integrated into docs/reference/commercial_emr_benchmarks.md. [valid_from: 2026-06-16, recorded: 2026-11-15, source: docs/reference/commercial_emr_benchmarks.md]
- Full FHIR Bridge and Interactive Bedside Workflows: Completed elimination of all placeholder stubs and localStorage shims in carePlans, familyHistory, immunizations, documents, MAR, whiteboard, and schedule. Replaced with asynchronous FHIR R4 resource creation and query engines (CarePlan, FamilyMemberHistory, Immunization, DocumentReference, MedicationAdministration, ServiceRequest, Encounter, AuditEvent), a functional WebRTC / mobile camera diagnostic capture pipeline with live video snapshot and environmental sensor fallback, a bedside 5-rights barcode/QR scanner verification modal, a dual-attestation controlled substance waste log, and synchronized inpatient whiteboard-to-census bidirectional updates. [valid_from: 2026-11-15, recorded: 2026-11-15, source: newloka_web/js/app.js & newloka_web/js/api.js]
- Graceful Unconfigured State & Zero Hardcoded Fallbacks: Client views across `rxpad`, `inpatientRegister`, `billing`, and `settings` contain zero hardcoded clinician or clinic fallback strings. When system metadata or profiles are unconfigured, optional header elements (clinic name, subtitle, doctor name, qualifications, reg no, department, signature, contact info) are gracefully hidden from the UI canvas and print outputs rather than injecting synthetic placeholders, while settings inputs provide helpful placeholder guidance and persist clean empty strings on demand. [valid_from: 2026-11-15, recorded: 2026-11-15, source: newloka_web/js/app.js & newloka_web/js/config.js]
- Decoupled ES6 Core Infrastructure Modules: Modularizes core frontend utilities into dedicated ES6 modules under `newloka_web/js/core/`: `utils.js` (DOM-less regex-based `escapeHtml` preventing garbage collection thrashing in high-frequency rendering, age/date formatting, and localStorage helpers), `scoring.js` (pure, deterministic implementations of NEWS2, CKD-EPI 2021, drug interaction engine, panic lab threshold detection, and SVG sparkline/timeline renderers), and `modals.js` (centralized accessible modal lifecycle management and Tier 1 Class C hazard modals requiring structured acknowledgment reasons). [valid_from: 2026-11-16, recorded: 2026-11-16, source: newloka_web/js/core/ & docs/STYLE_GUIDE.md]

## Superseded
(none)
