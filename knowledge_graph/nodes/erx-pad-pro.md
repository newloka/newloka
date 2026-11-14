# eRx Pad Pro and Prescription Formulary Integration

## Epistemic Metadata
- **Entity**: eRx Pad Pro & Offline Clinical Formulary
- **Layer**: Layer 2: Operational & Functional
- **Status**: SHIPPED
- **Domain**: Electronic Prescribing, Formulary Autocomplete & PDF Generation

## Facts
- Dedicated Prescription Engine: Standalone and embedded e-prescribing tool integrated with New Loka's Axum backend (`newloka_server::rxpad`) and clinical web UI. [valid_from: 2026-09-06, recorded: 2026-11-13, source: newloka_server/src/rxpad.rs & D:\Medical\hermes_kb\clinic-prescription.md]
- Comprehensive Drug Formulary: Preloaded with 61,000+ brand and generic pharmaceutical records (India CDSCO / NLEM / FDA aligned) providing real-time autocomplete on brand names, salts, routes, dosage strengths, and frequencies. [valid_from: 2026-09-06, recorded: 2026-11-13, source: newloka_server/src/rxpad.rs & git commit bd84617]
- Direct PDF Generation Without Print Dialogs: Generates signed clinical prescription PDFs named `DDMMYY_NNNN_PatientName.pdf` directly via offline `html2pdf.bundle.min.js`, uploads base64 to `/api/rxpad/sync`, saves into the clinic archive, and references in the chart. [valid_from: 2026-09-06, recorded: 2026-11-13, source: newloka_server/src/rxpad.rs & git commit 47bc73e]
- Two-Way Database Synchronization: Prescriptions synchronize to `clinic.db` with offline resilience, queuing into `localStorage['rxpad.pendingSync']` when offline, draining via a 2.5-second background retry loop upon reconnection, and broadcasting via Server-Sent Events (`/api/rxpad/events`). [valid_from: 2026-09-06, recorded: 2026-11-13, source: newloka_server/src/rxpad.rs & git commit 47bc73e]
- Privacy & Clean-Slate Mandate: On initial launch or reset, the prescription pad unconditionally wipes all patient fields and purges drafts to prevent accidental patient data crossover on shared clinical computers, allowing chart pre-filling only via explicit query parameters (`?patientName=...`). [valid_from: 2026-09-07, recorded: 2026-11-13, source: D:\Medical\hermes_kb\clinic-prescription.md & newloka_server/src/rxpad.rs]
- Automated Dispensary & Cascade Deletion: Supports one-click status transitions (Active -> Dispensed) and cascade deletion (`DELETE /api/rxpad/prescriptions/:id`) which removes linked FHIR MedicationRequest and MedicationDispense records atomically. [valid_from: 2026-09-07, recorded: 2026-11-13, source: newloka_server/src/rxpad.rs & git commit c2b0933]
- Native Prescription Pad & In-App Letterhead: Native interactive prescription pad canvas (`views.rxpad`) integrated into New Loka's web core, featuring dual-tab operation (`Composer` & `Queue`), customizable clinic letterhead, clinician registration credentials, cryptographic signature blocks, live pharmacopoeia autocomplete over 1,000+ formulary records, and real-time drug-drug interaction warning shields. [valid_from: 2026-11-14, recorded: 2026-11-14, source: newloka_web/js/app.js & newloka_server/src/rxpad.rs]

## Superseded
- Legacy 2020100 ePrescription Pad HTML template: Replaced by eRx Pad Pro with direct PDF generation, 61k drug database, SSE live sync, and clean-slate privacy enforcement. [superseded: 2026-09-07, reason: replaced by standalone eRx Pad Pro integration in newloka_server]
