# Master Specification: Comprehensive Healthcare Role & UI Viewport Matrix
**Authored by Amethyst (`newloka-core`) | Architectural Standard for Modular Presets**

This document establishes the exhaustive clinical and operational taxonomy of every user class, group, and subgroup that interacts with New Loka across deployment tiers **T0 (Solo Clinician / Phone)** to **T4 (Research Federation)**.

For each role, this specification defines the **Primary Viewport Structure**, **Specific UI Workflows**, **Required Backend Data Pipelines**, **ABAC Clearance & Data Isolation Boundaries**, and the **Declarative Preset Blueprint**.

---

## 1. Executive Summary of Viewport Archetypes

New Loka classifies all user interfaces into 6 standardized layout archetypes:
1. **Archetype A: Clinical Consultation Canvas (Outpatient / Ambulatory)**
   - Persistent Storyboard context rail on the left.
   - Central interactive clinical document or prescription pad.
   - Embedded live search & formulary autocomplete assistant.
2. **Archetype B: Inpatient Unit & Bed Acuity Matrix (Ward / Resident / ICU)**
   - Top-level census header with acuity distribution (NEWS2).
   - High-density bed matrix with color-coded acuity flags.
   - Quick drawer for round notes, CPOE orders, and rapid clinical review.
3. **Archetype C: High-Velocity Rapid Triage & Track Board (Emergency / Ambulance)**
   - Emergency severity index (ESI) colored card column layout.
   - Single-tap vitals capture, crash order sets, and disposition timer.
4. **Archetype D: Flowsheet & Barcode Execution Timetable (Nursing / MAR / OT)**
   - Time-matrix grid with scheduled medication administration slots.
   - Barcode scanning verification (BCMA) and fluid balance totals (I/O).
5. **Archetype E: Transaction Ledger & Restricted Cash Terminal (Billing / Reception)**
   - High-throughput invoicing list, payment methods, and receipt printer.
   - Strict ABAC masking of confidential clinical diagnoses and charts.
6. **Archetype F: Operational Telemetry & Alert Watchlist (Pharmacy / Dept Head / Executive)**
   - High-visibility flashing alert bar for stockouts and safety events.
   - Real-time event streams, buffer stock threshold gauges, and audit logs.

---

## 2. Exhaustive Role & UI Case Taxonomy

### GROUP 1: PATIENTS & FAMILIES

#### 1.1 Outpatient / Clinic Patient
- **Persona & Scope:** Ambulatory individual receiving consultation, chronic disease monitoring, or preventive care.
- **Viewport Layout:** Archetype A (Simplified Patient Portal Viewport). Mobile-first, high contrast, clean typography.
- **Specific UI Cases:**
  - View upcoming appointments and check-in status.
  - Review active digital prescriptions with dosage timing icons (morning/afternoon/night/food).
  - Inspect finalized lab and imaging reports with patient-friendly reference guides.
  - Settle co-pays and consultation invoices via integrated UPI / digital payment gateways.
  - Direct message with primary care physician for non-emergency follow-up queries.
- **Backend Data Pipelines:**
  - `GET /fhir/Patient/:id` (own demographics)
  - `GET /api/rxpad/prescriptions?patientId=:id` (finalized prescriptions)
  - `GET /fhir/DiagnosticReport?subject=Patient/:id&status=final` (final lab reports)
  - `GET /api/billing/invoices?patientId=:id` (invoices and receipts)
- **ABAC & Data Isolation:** Read-only access strictly restricted to own patient ID (`subject.id == session.patient_id`). Clinical progress notes and preliminary diagnostic impressions are hidden until signed off by the attending clinician. No access to other patients' records or system telemetry.
- **Preset Blueprint (`preset_patient_portal`):**
  - Modules: `['patientPortalHome', 'myPrescriptions', 'myLabReports', 'myBills', 'teleconsult']`
  - Default Landing View: `myPrescriptions`

#### 1.2 Inpatient Bedside Patient & Family Caregiver
- **Persona & Scope:** Hospitalized patient or legal guardian at bedside tracking daily inpatient care.
- **Viewport Layout:** Archetype A (Bedside Companion Tablet).
- **Specific UI Cases:**
  - Daily Care Schedule: Scheduled doctor rounds, medication administration times, planned diagnostic tests.
  - Care Team Directory: Names and photos of attending physician, resident on duty, and assigned floor nurse.
  - Meal Ordering & Dietary Plan: Prescribed diet (e.g. Diabetic, Salt-restricted, Liquid).
  - Nurse Call & Comfort Requests: Digital request for water, assistance, or pain re-evaluation.
- **Backend Data Pipelines:**
  - `GET /fhir/CarePlan?subject=Patient/:id`
  - `GET /api/inpatient/census?patientId=:id` (bed assignment and care team)
  - `GET /fhir/MedicationAdministration?subject=Patient/:id` (administration log)
- **ABAC & Data Isolation:** Scoped to active encounter and own patient ID. Sensitive psychiatric or child protection notes are redacted.
- **Preset Blueprint (`preset_bedside_companion`):**
  - Modules: `['bedsideSchedule', 'careTeam', 'carePlan', 'dietOrders']`
  - Default Landing View: `bedsideSchedule`

---

### GROUP 2: PHYSICIANS & CLINICAL DOCTORS

#### 2.1 Medical Intern / House Surgeon
- **Persona & Scope:** Provisional medical graduate under direct supervision performing ward work, documentation, and preliminary orders.
- **Viewport Layout:** Archetype B (Supervised Ward Resident Canvas) with co-signature indicators.
- **Specific UI Cases:**
  - Inpatient census review and vitals flowsheet monitoring.
  - Drafting SOAP progress notes and admission summaries requiring senior countersignature.
  - Entering provisional diagnostic orders and routine lab requisitions with supervising doctor tagged.
  - Rapid reference to hospital antibiotic stewardship policies and drug dosages.
- **Backend Data Pipelines:**
  - `GET /api/inpatient/census`
  - `GET /fhir/Observation?_sort=-date`
  - `POST /fhir/ClinicalImpression` (draft status)
  - `POST /fhir/ServiceRequest` (with `status: draft`, requiring attending sign-off)
- **ABAC & Data Isolation:** Clinical read/write in assigned department. Prescriptions and controlled substance orders cannot be finalized without attending co-signature (`can_prescribe_unsupervised: false`).
- **Preset Blueprint (`preset_medical_intern`):**
  - Modules: `['inpatientRegister', 'patientChart', 'clinicalNotes', 'cpoeOrders', 'vitalsFlowsheet', 'resultsReview']`
  - Default Landing View: `inpatientRegister`

#### 2.2 Junior Resident (PGY-1/2)
- **Persona & Scope:** Post-graduate resident in training managing day-to-day ward patients, acute admissions, and night duties.
- **Viewport Layout:** Archetype B (Resident Ward Management Viewport).
- **Specific UI Cases:**
  - Morning ward census review with NEWS2 early warning deterioration flags.
  - Inpatient admission intake, clinical examinations, and problem list curation.
  - Updating daily round plans and ordering diagnostic workups.
  - Generating discharge summaries and medication reconciliation drafts.
- **Backend Data Pipelines:**
  - `GET /api/inpatient/census`
  - `POST /api/inpatient/census` (updating resident round notes)
  - `GET /api/rxpad/prescriptions`
  - `POST /api/rxpad/sync`
- **ABAC & Data Isolation:** Full clinical read/write within assigned clinical department. Cannot access financial ledgers or administrative configuration.
- **Preset Blueprint (`preset_junior_resident`):**
  - Modules: `['inpatientRegister', 'patientChart', 'clinicalNotes', 'cpoeOrders', 'rxpad', 'mar', 'resultsReview', 'handoffSbar']`
  - Default Landing View: `inpatientRegister`

#### 2.3 Senior Resident / Chief Resident
- **Persona & Scope:** Senior trainee leading ward rounds, supervising interns, reviewing acute escalations, and approving discharges.
- **Viewport Layout:** Archetype B (Chief Resident Control Viewport with Acuity Triage).
- **Specific UI Cases:**
  - Ward census overview with real-time bed occupancy and critical patient indicators.
  - Reviewing and signing intern progress notes and discharge summaries.
  - Escalating high-acuity patients (NEWS2 ≥ 7) to ICU / Critical Care Team.
  - Managing inter-departmental transfer requests and multi-disciplinary consults.
- **Backend Data Pipelines:**
  - `GET /api/inpatient/census` & `POST /api/inpatient/census`
  - `GET /fhir/ServiceRequest?status=active`
  - `POST /api/rxpad/sync` (full prescribing rights)
- **ABAC & Data Isolation:** Full clinical read/write/prescribe authority across assigned service. Ability to execute emergency break-glass overrides with justification.
- **Preset Blueprint (`preset_senior_resident`):**
  - Modules: `['inpatientRegister', 'patientChart', 'clinicalNotes', 'cpoeOrders', 'rxpad', 'resultsReview', 'handoffSbar', 'whiteboard']`
  - Default Landing View: `inpatientRegister`

#### 2.4 Attending Physician / Consultant
- **Persona & Scope:** Board-certified specialist with ultimate medico-legal responsibility for patient diagnosis, treatment, and outcomes.
- **Viewport Layout:** Archetype A (Comprehensive Clinical Decision & Prescribing Suite).
- **Specific UI Cases:**
  - Rapid longitudinal chart review with Storyboard highlighting key alerts, vitals, and active problems.
  - Composing and issuing electronic prescriptions using the native A4 prescription pad with formulary autocomplete.
  - High-level SOAP note documentation using structured clinical phrases and templates.
  - Final sign-off on CPOE order sets, clinical procedures, and patient discharges.
- **Backend Data Pipelines:**
  - `GET /api/prescriber/profile`
  - `GET /api/rxpad/formulary`
  - `POST /api/rxpad/sync`
  - `GET /api/rxpad/next-serial`
  - `GET /fhir/Patient/:id/everything`
- **ABAC & Data Isolation:** Unrestricted clinical authority across admitted and consulted patients. Legally authorized prescriber. Audit logging active on all chart opens.
- **Preset Blueprint (`preset_attending_physician`):**
  - Modules: `['rxpad', 'patientChart', 'clinicalNotes', 'cpoeOrders', 'inpatientRegister', 'resultsReview', 'vitalsFlowsheet', 'documents', 'appointmentSchedule']`
  - Default Landing View: `rxpad`

#### 2.5 Department Head (HOD) / Service Chief
- **Persona & Scope:** Senior clinician overseeing departmental clinical excellence, morbidity/mortality audits, and resource allocation.
- **Viewport Layout:** Archetype F + Archetype A (Department Command & Clinical Suite).
- **Specific UI Cases:**
  - Departmental activity dashboard: Total OPD visits, IPD census, average length of stay (ALOS), complication rates.
  - Clinical audit explorer: Reviewing compliance with clinical guidelines and antibiotic stewardship.
  - Direct clinical consultation for complex / referred cases.
  - Staff workload distribution and resident duty scheduling.
- **Backend Data Pipelines:**
  - `GET /api/dept/telemetry`
  - `GET /api/inpatient/census`
  - `GET /fhir/AuditEvent`
- **ABAC & Data Isolation:** Department-wide clinical supervision privilege. Access to aggregate clinical statistics and anonymized performance metrics.
- **Preset Blueprint (`preset_department_head`):**
  - Modules: `['deptSummary', 'inpatientRegister', 'rxpad', 'patientChart', 'clinicalNotes', 'audit', 'appointmentSchedule']`
  - Default Landing View: `deptSummary`

#### 2.6 Solo General Practitioner / Rural Clinic Doctor (T0)
- **Persona & Scope:** Independent solo clinician running an outpatient practice on a single laptop/phone without support staff.
- **Viewport Layout:** Archetype A (Unified Solo Practice Canvas).
- **Specific UI Cases:**
  - Direct patient registration, instant vitals entry, and provisional diagnosis.
  - Real-time prescription generation with integrated pharmacopoeia and print-ready letterhead.
  - Automatic billing calculation (Consultation + Pharmacy fees) upon saving prescription.
  - End-of-day revenue reconciliation and encrypted offline local backup.
- **Backend Data Pipelines:**
  - `GET /api/rxpad/formulary`
  - `POST /api/rxpad/sync`
  - `POST /api/billing/invoices`
  - `POST /api/backup/local`
- **ABAC & Data Isolation:** Single-user clinical and financial administrator (DMK encrypted local database). Inpatient hospital clutter completely suppressed.
- **Preset Blueprint (`preset_solo_general_practitioner`):**
  - Modules: `['rxpad', 'patients', 'billing', 'appointmentSchedule', 'documents', 'settings']`
  - Default Landing View: `rxpad`

#### 2.7 Surgical Specialist / Proceduralist
- **Persona & Scope:** Surgeon performing elective and emergency operations, bedside procedures, and perioperative care.
- **Viewport Layout:** Archetype A + OpTime Surgical Matrix.
- **Specific UI Cases:**
  - Pre-operative checklist verification (NPO status, anesthesia clearance, informed consent).
  - Intra-operative documentation: Operative procedure note, surgical findings, implant serial numbers, blood loss, specimen verification.
  - Post-operative orders: Post-anesthesia monitoring, analgesia protocol, antibiotic prophylaxis duration, drain management.
- **Backend Data Pipelines:**
  - `GET /fhir/Procedure` & `POST /fhir/Procedure`
  - `GET /fhir/Consent?patient=:id`
  - `POST /api/rxpad/sync`
- **ABAC & Data Isolation:** Full surgical operative notes and procedure privileges. Access to surgical theater scheduling.
- **Preset Blueprint (`preset_surgical_specialist`):**
  - Modules: `['procedures', 'patientChart', 'clinicalNotes', 'cpoeOrders', 'rxpad', 'documents']`
  - Default Landing View: `procedures`

#### 2.8 Emergency Physician / Resuscitation Lead (ASAP)
- **Persona & Scope:** Acute care clinician managing resuscitation bays, rapid triage, trauma codes, and emergency stabilization.
- **Viewport Layout:** Archetype C (Emergency Track Board & Resuscitation Terminal).
- **Specific UI Cases:**
  - Emergency Track Board displaying bed status, triage category (Red/Yellow/Green), chief complaint, and length of stay in ED.
  - One-tap CPOE Order Sets: "Acute Coronary Syndrome", "Sepsis Bundle", "Acute Stroke Protocol", "Polytrauma".
  - Rapid bedside ultrasound and ECG review.
  - Disposition determination: Admit to ICU/Ward, Emergency OT transfer, or Discharge home.
- **Backend Data Pipelines:**
  - `GET /api/emergency/trackboard`
  - `POST /fhir/ServiceRequest` (priority: `stat`)
  - `POST /fhir/Encounter` (admit/transfer)
- **ABAC & Data Isolation:** Emergency break-glass privileges enabled by default. Immediate read/write access across all un-registered or trauma patients.
- **Preset Blueprint (`preset_emergency_physician`):**
  - Modules: `['emergencyTrackboard', 'patientChart', 'cpoeOrders', 'rxpad', 'vitalsFlowsheet', 'resultsReview']`
  - Default Landing View: `emergencyTrackboard`

---

### GROUP 3: NURSING PROFESSIONALS

#### 3.1 Ward Staff Nurse (General / Step-down)
- **Persona & Scope:** Registered nurse providing continuous bedside care, vital signs monitoring, and medication administration in medical/surgical wards.
- **Viewport Layout:** Archetype D (Inpatient Nursing Station & MAR Timetable).
- **Specific UI Cases:**
  - Vitals signs capture with automated NEWS2 early warning score calculation.
  - Executing scheduled medication doses on the Medication Administration Record (MAR).
  - Shift nursing assessment notes (skin integrity, fall risk assessment, IV cannula sites).
  - Recording fluid balance (fluid intake: oral/IV vs fluid output: urine/drains).
- **Backend Data Pipelines:**
  - `GET /fhir/MedicationRequest?status=active`
  - `POST /fhir/MedicationAdministration`
  - `POST /fhir/Observation` (vitals & fluid balance)
- **ABAC & Data Isolation:** Clinical bedside documentation rights. Cannot create or edit prescriptions (`can_prescribe: false`). Cannot alter doctor orders.
- **Preset Blueprint (`preset_staff_nurse_ward`):**
  - Modules: `['mar', 'vitalsFlowsheet', 'inpatientRegister', 'patientChart', 'handoffSbar', 'alerts']`
  - Default Landing View: `mar`

#### 3.2 Triage & Emergency Nurse
- **Persona & Scope:** Frontline emergency nurse evaluating arriving patients, assigning acuity scores, and initiating initial protocols.
- **Viewport Layout:** Archetype C (Rapid Triage Terminal).
- **Specific UI Cases:**
  - Rapid patient intake: Chief complaint, initial vital signs, Glasgow Coma Scale (GCS), pain score.
  - Triage category assignment: Emergency Severity Index (ESI Level 1–5).
  - Initiating nurse standing orders (e.g. ECG for chest pain, blood glucose for altered sensorium, IV access).
  - Directing patient to appropriate bay (Resuscitation, Acute Care, Ambulatory Triage).
- **Backend Data Pipelines:**
  - `POST /fhir/Observation` (triage vitals)
  - `POST /fhir/Encounter` (triage entry)
  - `GET /api/emergency/trackboard`
- **ABAC & Data Isolation:** Scoped to Emergency Department. Standing nurse triage order entry authorized.
- **Preset Blueprint (`preset_emergency_triage_nurse`):**
  - Modules: `['emergencyTrackboard', 'vitalsFlowsheet', 'patientChart', 'alerts']`
  - Default Landing View: `emergencyTrackboard`

#### 3.3 Intensive Care / Critical Care Nurse (ICU)
- **Persona & Scope:** Specialized critical care nurse managing ventilated, hemodynamically unstable, or multi-organ failure patients.
- **Viewport Layout:** Archetype D (High-Density ICU Flowsheet & Continuous Telemetry).
- **Specific UI Cases:**
  - Hourly vital signs and hemodynamic telemetry recording (CVP, arterial line BP, ICP).
  - Continuous IV vasoactive infusion titration (Noradrenaline, Vasopressin) with rate change logs.
  - Ventilator parameter logging (FiO2, PEEP, Tidal Volume, PIP) and arterial blood gas (ABG) tracking.
  - Strict hourly urine output and fluid balance monitoring.
- **Backend Data Pipelines:**
  - `POST /fhir/Observation` (high-density hemodynamic telemetry)
  - `POST /fhir/MedicationAdministration` (infusion titration rates)
- **ABAC & Data Isolation:** ICU departmental silo access. High-frequency telemetry entry enabled.
- **Preset Blueprint (`preset_icu_nurse`):**
  - Modules: `['vitalsFlowsheet', 'mar', 'patientChart', 'resultsReview', 'alerts', 'handoffSbar']`
  - Default Landing View: `vitalsFlowsheet`

#### 3.4 Operation Theatre (OT) Scrub & Circulating Nurse
- **Persona & Scope:** Perioperative nursing professional managing sterile fields, surgical counts, and operative logs.
- **Viewport Layout:** Archetype D (Perioperative Safety & Surgical Flowsheet).
- **Specific UI Cases:**
  - WHO Surgical Safety Checklist execution: Sign In (before anesthesia), Time Out (before incision), Sign Out (before closure).
  - Surgical instrument, sponge, and needle count logging with zero-discrepancy verification.
  - Specimen labeling and chain-of-custody handoff to pathology.
  - Documenting skin prep, electrosurgical grounding pad placement, and tourniquet times.
- **Backend Data Pipelines:**
  - `GET /fhir/Procedure`
  - `POST /fhir/Procedure` (surgical nursing checklist)
  - `POST /fhir/Specimen` (pathology handoff)
- **ABAC & Data Isolation:** Perioperative and OT suite clearance. Checklists require multi-party digital sign-offs.
- **Preset Blueprint (`preset_ot_nurse`):**
  - Modules: `['procedures', 'patientChart', 'documents', 'alerts']`
  - Default Landing View: `procedures`

#### 3.5 Charge Nurse / Ward Sister / Floor Supervisor
- **Persona & Scope:** Senior nurse managing ward bed occupancy, nurse-patient staffing ratios, and shift handoffs.
- **Viewport Layout:** Archetype B (Ward Command Board & Staffing Matrix).
- **Specific UI Cases:**
  - Inpatient census management: reviewing bed occupancy, pending admissions, and pending discharges.
  - Nurse patient assignments: allocating patients to floor nurses based on acuity weighting.
  - Structured shift handoff execution using SBAR (Situation-Background-Assessment-Recommendation).
  - Inspecting ward-wide medication administration compliance and un-administered dose flags.
- **Backend Data Pipelines:**
  - `GET /api/inpatient/census`
  - `GET /fhir/MedicationAdministration?status=not-done`
  - `POST /fhir/Communication` (SBAR handoff)
- **ABAC & Data Isolation:** Ward-wide nursing supervision access.
- **Preset Blueprint (`preset_charge_nurse`):**
  - Modules: `['inpatientRegister', 'whiteboard', 'handoffSbar', 'mar', 'alerts']`
  - Default Landing View: `inpatientRegister`

---

### GROUP 4: ALLIED HEALTH, PHARMACY & DIAGNOSTICS

#### 4.1 Clinical Pharmacist / Outpatient Dispensary
- **Persona & Scope:** Licensed pharmacist verifying outpatient and inpatient prescriptions, checking contraindications, and dispensing medications.
- **Viewport Layout:** Archetype F + Pharmacy Dispensing Counter.
- **Specific UI Cases:**
  - Live prescription verification queue: Reviewing doctor's electronic prescriptions.
  - Drug interaction verification: Reviewing automated alerts for drug-drug interactions, food interactions, and dose adjustments.
  - Dispensing execution: Scanning medicine barcodes, entering batch/lot numbers and expiry dates, marking order as `dispensed`.
  - Patient counseling documentation: Recording instructions provided to patient on food/timing.
- **Backend Data Pipelines:**
  - `GET /api/rxpad/prescriptions?status=prescribed`
  - `POST /api/rxpad/dispense`
  - `GET /api/rxpad/formulary`
- **ABAC & Data Isolation:** Pharmacy department silo. Authorized to verify and dispense medications; cannot author primary prescriptions or access non-pharmacological clinical progress notes.
- **Preset Blueprint (`preset_clinical_pharmacist`):**
  - Modules: `['rxpad', 'deptSummary', 'drugInteraction', 'patientChart']`
  - Default Landing View: `rxpad`

#### 4.2 Pharmacy Head / Chief Pharmacist (Willow)
- **Persona & Scope:** Departmental head overseeing drug purchasing, formulary curation, buffer stock maintenance, and narcotic audits.
- **Viewport Layout:** Archetype F (Pharmacy Department Command & Alert Telemetry).
- **Specific UI Cases:**
  - Flashing alert bar monitoring: Immediate notification of critical stockouts (buffer stock < minimum safety threshold).
  - Departmental dispensing metrics: Total dispenses today, pending queues, average turnaround time.
  - High-risk safety intervention tracking: Co-prescription overrides, narcotic drug sign-out logs.
  - Pharmacopoeia management: Adding new drugs, setting standard doses, updating generic equivalents.
- **Backend Data Pipelines:**
  - `GET /api/pharmacy/summary`
  - `GET /api/rxpad/formulary`
  - `POST /api/rxpad/formulary`
  - `GET /fhir/AuditEvent?entity=MedicationDispense`
- **ABAC & Data Isolation:** Full administrative control over hospital pharmacy inventory and formulary master data.
- **Preset Blueprint (`preset_pharmacy_head`):**
  - Modules: `['deptSummary', 'rxpad', 'drugInteraction', 'audit', 'settings']`
  - Default Landing View: `deptSummary`

#### 4.3 Medical Laboratory Technologist (MLT)
- **Persona & Scope:** Diagnostic laboratory technician processing blood, urine, and body fluid specimens and validating analyzer outputs.
- **Viewport Layout:** Laboratory Accession & Batch Entry Terminal.
- **Specific UI Cases:**
  - Specimen accessioning: Scanning tube barcodes, confirming specimen arrival and viability.
  - Worklist review: Grouping tests by analyzer bench (Hematology, Biochemistry, Coagulation, Serology).
  - Quantitative result entry: Entering numerical parameters with automatic calculation (e.g. eGFR, Anion Gap).
  - Panic value escalation: Immediate trigger when values cross life-threatening thresholds (e.g. Potassium > 6.5 mmol/L).
- **Backend Data Pipelines:**
  - `GET /fhir/ServiceRequest?category=laboratory`
  - `POST /fhir/Observation` (diagnostic results)
  - `POST /fhir/DiagnosticReport` (preliminary status)
- **ABAC & Data Isolation:** Laboratory departmental silo. Restricted strictly to lab test orders and observations; cannot view patient clinical SOAP notes or billing details.
- **Preset Blueprint (`preset_lab_technologist`):**
  - Modules: `['resultsReview', 'cpoeOrders', 'alerts']`
  - Default Landing View: `resultsReview`

#### 4.4 Consultant Pathologist (Beaker)
- **Persona & Scope:** Diagnostic medical specialist reviewing abnormal blood films, biopsy tissues, and authorizing finalized diagnostic reports.
- **Viewport Layout:** Diagnostic Review & Synoptic Pathology Suite.
- **Specific UI Cases:**
  - Reviewing abnormal analyzer flags and differential counts.
  - Synoptic reporting for biopsies: Macroscopic gross description, microscopic examination, immunohistochemistry, and final histological diagnosis.
  - Digital sign-off: Cryptographically signing diagnostic reports to transition status from `preliminary` to `final`.
- **Backend Data Pipelines:**
  - `GET /fhir/DiagnosticReport?status=preliminary`
  - `POST /fhir/DiagnosticReport` (final signature)
- **ABAC & Data Isolation:** Pathology specialist clearance. Full authorization to sign and release diagnostic laboratory reports.
- **Preset Blueprint (`preset_pathologist`):**
  - Modules: `['resultsReview', 'patientChart', 'documents']`
  - Default Landing View: `resultsReview`

#### 4.5 Consultant Radiologist (Radiant)
- **Persona & Scope:** Medical imaging specialist interpreting X-rays, CT scans, MRIs, and ultrasounds and authoring radiological reports.
- **Viewport Layout:** Radiant Imaging & PACS Reporting Console.
- **Specific UI Cases:**
  - Radiology worklist: Filter by modality (CR, CT, MR, US), priority (Routine vs Stat Emergency).
  - DICOM viewer launch: Contextual deep-linking to PACS web viewer (dcm4chee).
  - Structured radiological report authoring: Clinical indication, technique, findings, impression, and follow-up recommendation.
  - Urgent finding peer alert: Directly notifying attending clinician of critical findings (e.g. tension pneumothorax, intracranial hemorrhage).
- **Backend Data Pipelines:**
  - `GET /fhir/ImagingStudy`
  - `POST /fhir/DiagnosticReport` (radiological impression)
- **ABAC & Data Isolation:** Radiology departmental silo. Imaging reporting authorization.
- **Preset Blueprint (`preset_radiologist`):**
  - Modules: `['resultsReview', 'documents', 'patientChart']`
  - Default Landing View: `resultsReview`

---

### GROUP 5: ADMINISTRATIVE, REVENUE & OPERATIONS

#### 5.1 Front Desk / Appointment Registrar (Cadence)
- **Persona & Scope:** Administrative receptionist greeting patients, scheduling visits, verifying contact details, and directing patient queues.
- **Viewport Layout:** Cadence Scheduling & Appointment Desk.
- **Specific UI Cases:**
  - Outpatient appointment booking: Viewing doctor slot availability calendars.
  - Walk-in queue registration: Assigning token numbers and consultation rooms.
  - Patient demographic updates: Address, phone number, emergency contacts.
  - Patient check-in and check-out status transitions.
- **Backend Data Pipelines:**
  - `GET /fhir/Appointment` & `POST /fhir/Appointment`
  - `GET /fhir/Patient` & `POST /fhir/Patient`
- **ABAC & Data Isolation:** Administrative role. Strictly barred from viewing clinical diagnoses, progress notes, or prescriptions.
- **Preset Blueprint (`preset_front_desk`):**
  - Modules: `['appointmentSchedule', 'patients']`
  - Default Landing View: `appointmentSchedule`

#### 5.2 Cashier / Billing Specialist (Resolute PB/HB)
- **Persona & Scope:** Revenue cycle clerk generating patient bills, accepting cash/card/UPI payments, issuing receipts, and reconciling accounts.
- **Viewport Layout:** Archetype E (Billing Ledger & Cash Register Terminal).
- **Specific UI Cases:**
  - Invoice generation: Combining doctor consultation fees, pharmacy dispense costs, and lab investigation charges.
  - Collecting payments: Logging payment method (Cash, UPI, Card, Insurance) and amount collected.
  - Printable receipt generation: Printing clear itemized financial receipts with clinic tax headers.
  - Reconciling daily cash balances against electronic receipts.
- **Backend Data Pipelines:**
  - `GET /api/billing/invoices`
  - `POST /api/billing/invoices`
  - `POST /api/billing/invoices/:id/pay`
- **ABAC & Data Isolation:** Core ABAC Data Isolation enforced (`Role::Billing`). Patient clinical diagnoses and ICD-10 notes are masked to `🔒 [CLINICAL DATA REDACTED - ABAC CONFIDENTIALITY]`. Cannot alter clinical charts or view medical notes.
- **Preset Blueprint (`preset_cashier_billing`):**
  - Modules: `['billing', 'patients']`
  - Default Landing View: `billing`

#### 5.3 Hospital Administrator / Medical Director
- **Persona & Scope:** Executive leader responsible for overall hospital operations, regulatory compliance, quality metrics, and clinical governance.
- **Viewport Layout:** Archetype F (Executive Governance & Hospital Operational Control).
- **Specific UI Cases:**
  - Hospital-wide operational metrics: Bed occupancy percentage, emergency door-to-doctor time, average length of stay (ALOS).
  - Financial health indicators: Daily gross billings, net collections, insurance receivables aging.
  - Compliance and quality dashboards: Medication error reports, hospital-acquired infection (HAI) rates, patient satisfaction scores.
  - Ed25519 cryptographic audit explorer: Reviewing access logs, emergency break-glass overrides, and policy denials.
- **Backend Data Pipelines:**
  - `GET /api/config`
  - `GET /api/dept/telemetry`
  - `GET /fhir/AuditEvent`
  - `GET /api/inpatient/census`
- **ABAC & Data Isolation:** Executive oversight role. Access to institutional audit logs, system configuration, and aggregate statistics. Direct access to individual identifiable medical charts is logged under strict executive audit scrutiny.
- **Preset Blueprint (`preset_hospital_executive`):**
  - Modules: `['dashboard', 'inpatientRegister', 'deptSummary', 'billing', 'audit', 'settings']`
  - Default Landing View: `dashboard`

#### 5.4 IT Systems Administrator / Node Operator
- **Persona & Scope:** Technical administrator deploying New Loka instances, managing offline updates, configuring P2P mesh discovery, and rotating cryptographic keys.
- **Viewport Layout:** System Maintenance & Node Engineering Console.
- **Specific UI Cases:**
  - Modular feature configuration: Enabling/disabling modules across the 28-module matrix based on deployment tier (T0–T4).
  - Software updates: Checking GitHub releases, executing 1-click updates, or uploading air-gapped release packages (.zip / .exe).
  - Cryptographic key management: Generating Device Master Keys (DMK), rotating Ed25519 audit signing pairs.
  - Mesh discovery and hospital server synchronization setup.
- **Backend Data Pipelines:**
  - `GET /api/config` & `POST /api/config`
  - `GET /api/system/version`
  - `POST /api/system/update/check`
  - `POST /api/system/update/apply`
- **ABAC & Data Isolation:** Technical system administration role. Full control over system binaries, configuration toggles, and network parameters; strictly barred from browsing patient health records without explicit Break-Glass authorization.
- **Preset Blueprint (`preset_system_admin`):**
  - Modules: `['settings', 'audit', 'dashboard']`
  - Default Landing View: `settings`

---

## 3. Summary Matrix of Roles & Modular Presets

| Role ID | Role Title | Archetype | ABAC Identity | Default Landing | Active Preset ID |
|---|---|---|---|---|---|
| `pt_portal` | Outpatient Patient | A | `patient` | `myPrescriptions` | `preset_patient_portal` |
| `doc_intern` | Medical Intern | B | `resident` (supervised) | `inpatientRegister` | `preset_medical_intern` |
| `doc_junior_res` | Junior Resident | B | `resident` | `inpatientRegister` | `preset_junior_resident` |
| `doc_senior_res` | Senior Resident | B | `resident` | `inpatientRegister` | `preset_senior_resident` |
| `doc_attending` | Attending Physician | A | `doctor` (prescriber) | `rxpad` | `preset_attending_physician` |
| `doc_hod` | Department Head | F + A | `doctor` (service chief)| `deptSummary` | `preset_department_head` |
| `doc_solo_gp` | Solo General Practitioner| A | `doctor` (solo admin) | `rxpad` | `preset_solo_general_practitioner`|
| `doc_surgeon` | Surgical Specialist | A | `doctor` (proceduralist) | `procedures` | `preset_surgical_specialist` |
| `doc_emergency` | Emergency Physician | C | `doctor` (acute care) | `emergencyTrackboard` | `preset_emergency_physician` |
| `nurse_ward` | Ward Staff Nurse | D | `nurse` (bedside) | `mar` | `preset_staff_nurse_ward` |
| `nurse_triage` | Triage / ER Nurse | C | `nurse` (acute triage) | `emergencyTrackboard` | `preset_emergency_triage_nurse` |
| `nurse_icu` | ICU Nurse | D | `nurse` (critical care) | `vitalsFlowsheet` | `preset_icu_nurse` |
| `nurse_ot` | Operating Theatre Nurse| D | `nurse` (perioperative) | `procedures` | `preset_ot_nurse` |
| `nurse_charge` | Charge Nurse / Sister | B | `nurse` (ward supervisor)| `inpatientRegister` | `preset_charge_nurse` |
| `pharm_disp` | Clinical Pharmacist | F | `pharmacy` | `rxpad` | `preset_clinical_pharmacist` |
| `pharm_head` | Pharmacy Head | F | `pharmacy` (director) | `deptSummary` | `preset_pharmacy_head` |
| `lab_tech` | Lab Technologist | A | `allied` (lab tech) | `resultsReview` | `preset_lab_technologist` |
| `lab_path` | Consultant Pathologist | A | `doctor` (pathologist) | `resultsReview` | `preset_pathologist` |
| `rad_cons` | Consultant Radiologist | A | `doctor` (radiologist) | `resultsReview` | `preset_radiologist` |
| `admin_front` | Front Desk Clerk | A | `reception` | `appointmentSchedule` | `preset_front_desk` |
| `admin_cashier`| Cashier / Billing Clerk| E | `billing` | `billing` | `preset_cashier_billing` |
| `exec_admin` | Hospital Administrator | F | `admin` (executive) | `dashboard` | `preset_hospital_executive` |
| `tech_sysadmin`| IT Systems Administrator| A | `admin` (technical) | `settings` | `preset_system_admin` |
