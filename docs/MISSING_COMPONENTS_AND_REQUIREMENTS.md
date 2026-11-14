# New Loka: Master Vision & Requirements Gap Analysis
**Living Tracking Document for Missing Components, In-Progress Capabilities, and Future Evolution**
*Maintained by Amethyst (`newloka-core`) | Ecosystem: New Loka Health Data Platform*

---

## 1. Executive Summary & Epistemological Foundation

### Moving from Ideological Debt to Adaptive Engineering
In large-scale, mission-critical healthcare systems engineering, dogmatic "invariants" create ideological debt—rigid constraints established with incomplete foresight that later obstruct essential real-world adaptations and compel fragile workarounds for non-existent problems.

New Loka is grounded in five **Foundational Axioms** (guiding principles, not dogmatic straightjackets):
1. **Local-First Data Sovereignty:** Clinical care must never stop because an internet connection dropped. The patient and local clinician retain primary ownership and custody of health data. Sync is additive, not a precondition for basic clinical operation.
2. **Cryptographic Security at the Data Layer:** Security is enforced where data lives (AES-256-GCM AEAD encryption at rest, Device Master Key hierarchies, Ed25519 tamper-evident audit trails, and core Attribute-Based Access Control), never left solely to client-side UI visibility.
3. **Clinical Prudence & Non-Dangerous Merge:** Machine algorithms must never silently overwrite or auto-resolve life-critical clinical decisions (medications, contraindications, severe allergies). Automated extractions (OCR, speech, AI) strictly require human-in-the-loop clinical review before permanent chart commit.
4. **Single Polymorphic Core:** The foundational engine (`newloka_core`) adapts across deployment scales (T0 solo clinic, T1 local mesh, T2 rural hospital, T3 multi-department enterprise, T4 research federation) through modular configuration, avoiding unmaintainable divergent codebase forks.
5. **Ergonomic Empathy & High-Velocity Usability:** Healthcare software must respect clinician attention under acute stress. Critical information (allergies, emergency notes, active medications, vital trends) must be accessible within three taps or clicks, with zero unnecessary cognitive latency.

Beyond these axioms, New Loka operates under a posture of **continuous learning, real-world adaptation, and active problem solving**.

### Purpose of this Document
This document is the definitive, continuously updated inventory of all components, functional specifications, and architectural requirements defined in the **New Loka Vision** (`Project_Scope_and_Requirements.txt`, the 11-Phase Roadmap in `site/team.html`, and reference architectures) compared against our **current codebase state**.

---

## 2. Global Vision vs. Implementation Status Overview

| Phase / Dimension | Vision Scope | Codebase Status | Status Classification | Key Missing Component |
|---|---|---|---|---|
| **Phase 1: Core Offline Clinician Workflow** | Local encrypted storage, FHIR R4 models, patient CRUD, timeline, offline auth, audit ledger | `newloka_core` (23+ FHIR resources, AES-256-GCM, SQLite, Ed25519 audit), `newloka_server`, `newloka_web`, `newloka_cli` | **SHIPPED (100%)** | None for core workflow |
| **Phase 2: Local Sync and Mesh** | LAN peer discovery, delta sync with vector clocks, conflict detection | Core CRDT vector clocks & delta engine in `sync/`, server `/sync/*` endpoints | **PARTIAL (75%)** | Background mDNS daemon & BLE peer-to-peer sync |
| **Phase 3: Institutional Access Control** | ABAC policy engine, department silos, break-glass override, LDAP integration | Core ABAC engine, emergency override logging, LDAP directory auth in `newloka_core` & `newloka_server` | **SHIPPED (100%)** | None for basic institutional ABAC |
| **Phase 4: Local Intelligence & Document Ingestion** | Local OCR for handwritten prescriptions, speech-to-text dictation, structured FHIR extraction, human review queues | `#ingest` web view with dropzones, stubs, and review queue interface | **PARTIAL (30%)** | Real local OCR engine, local Whisper STT, structured prescription parser |
| **Phase 5: Clinical Calculators & Knowledge Banks** | Built-in clinical decision support calculators, reference libraries, drug interaction & dosage databases | 61k+ offline formulary in `rxpad.rs`, interaction checker, NEWS2 scoring | **PARTIAL (40%)** | Interactive clinical calculators suite (eGFR, Wells, CHA2DS2-VASc, pediatric dosage) |
| **Phase 6: Distributed Deployments & Large Organisations** | Multi-department hospital hierarchy, bed allocation, SBAR handovers, executive telemetry, tiered packaging | 9-department hierarchy, SBAR transfer workflows, hospital reports, release packaging script | **SHIPPED (90%)** | Multi-campus health network dashboard & Docker/K8s production manifests |
| **Phase 7: Internal Secure Communications** | E2EE messaging, VoIP/video between clinicians & departments, zero third-party cloud relays | Shift handoff notes and SBAR transfer notes | **PENDING (10%)** | Real-time LAN clinician chat (Double Ratchet), on-call paging alerts |
| **Phase 8: Imaging & Lab Integration** | DICOM PACS bridge, HL7/FHIR lab bridges, automatic analyzer result ingestion | FHIR `ImagingStudy`, `DiagnosticReport`, `Specimen`, `ServiceRequest`, lab department silos | **PARTIAL (40%)** | Native web DICOM viewer, HL7 v2.x MLLP analyzer listener, specimen barcoding |
| **Phase 9: Federation and Research** | Privacy-preserving query exchange, differential privacy, $k$-anonymity, consent directives, ABDM/ABHA | Laplace DP engine ($\text{scale} = \frac{1}{\epsilon}$), $k$-anonymity ($k \ge 5$), FHIR Consent engine, `#research`, `#consent`, `#federation` views | **SHIPPED (75%)** | Official ABDM M1/M2/M3 National Gateway bridge, IHE MHD/XDS.b profiles |
| **Phase 10: Complete Hospital Information System (HIS)** | Ward bed census, billing, scheduling, dispensary inventory, surgical OT management | Ward whiteboard, 9-department bed tracking, outpatient scheduling, cashier billing with ABAC masking | **PARTIAL (50%)** | Dispensary batch/expiry inventory tracking, surgical OT scheduler, insurance/TPA claims |
| **Phase 11: Experimental Feature Sets** | Ambient consultation listening, predictive triage alerts, synthetic EHR data generators | 20-patient synthetic Indian demographic seed generator (`demo.rs`) | **PARTIAL (25%)** | Ambient consultation audio stream parsing, automated sepsis early-warning triggers |

---

## 3. Granular Subsystem Gap Analysis

### Subsystem A: Local Intelligence & Document Ingestion (Phase 4)
*Goal: Bridge physical paper-based clinical reality (handwritten prescriptions, external paper lab sheets) into structured, secure electronic records without cloud dependencies.*

- [x] **Shipped:**
  - `#ingest` web view with document upload interface and confidence badge rendering.
  - Human-in-the-loop review workflow layout (side-by-side document image vs. extracted form).
  - Storage for `DocumentReference` FHIR resources with MIME type attachments.
- [ ] **Missing & Required:**
  1. **Native Local OCR Execution Engine:**
     - Integrate a local, offline OCR engine (e.g. lightweight Tesseract C API bindings or an ONNX runtime / `candle` vision transformer) capable of running on modest CPU hardware.
     - Support for low-contrast smartphone photos, variable angles, and clinical handwriting.
  2. **Offline Speech-to-Text (STT) Dictation Pipeline:**
     - Local audio capture and transcription engine (e.g., `whisper.cpp` or `candle-whisper` quantized models) for ambulatory consultation rooms.
     - Clinical vocabulary customization (Indian brand names, medical abbreviations, Latin SIG codes like *b.i.d.*, *t.d.s.*).
  3. **Structured Prescription & Lab Value Entity Extraction:**
     - Heuristic or local LLM parsing pipeline converting unstructured transcription/OCR text into validated draft FHIR resources (`MedicationRequest`, `Observation`, `DiagnosticReport`).
     - Automated detection of missing critical data (missing dosage, ambiguous frequency) surfacing red flags in the review queue.

---

### Subsystem B: Clinical Calculators & Point-of-Care Decision Support (Phase 5)
*Goal: Provide instant, evidence-based clinical calculators and safety guidance directly at the point of care without requiring external internet searches.*

- [x] **Shipped:**
  - National NEWS2 (National Early Warning Score 2) calculation in the Vitals Flowsheet (`vitalsFlowsheet`).
  - Real-time Drug-Drug and Drug-Allergy interaction safety shield against active patient records.
  - 61,000+ record offline CDSCO / NLEM / FDA pharmacopoeia database.
- [ ] **Missing & Required:**
  1. **Core Clinical Calculators Suite:**
     - Renal: Cockcroft-Gault & CKD-EPI eGFR calculation with automatic warning on nephrotoxic drugs.
     - Cardiovascular: CHA2DS2-VASc score for atrial fibrillation stroke risk, ASCVD 10-year risk estimator.
     - Emergency / Critical Care: Wells Score for DVT & Pulmonary Embolism, Glasgow Coma Scale (GCS), CURB-65 for pneumonia severity.
     - Hepatic: Child-Pugh Score and MELD / MELD-Na score with hepatic dosing warnings.
     - Pediatrics: Weight-based and Body Surface Area (Mosteller formula) pediatric dosing calculator with safe maximum dose clamps.
  2. **Offline Standard Treatment Guidelines (STGs) & Clinical Reference Bank:**
     - Integrated, fast-search offline medical knowledge repository (e.g., Indian ICMR treatment guidelines, WHO essential protocols) accessible within the clinical notes canvas without leaving patient context.

---

### Subsystem C: Diagnostics, PACS & Laboratory Instrumentation (Phase 8)
*Goal: Unify hospital diagnostic equipment (radiology modalities, hematology & biochemistry analyzers) with patient longitudinal records.*

- [x] **Shipped:**
  - Typed FHIR models: `ImagingStudy`, `DiagnosticReport`, `Specimen`, `ServiceRequest`.
  - Computerized Physician Order Entry (CPOE) with 5-stage lifecycle and priority tiers (Routine / Urgent / Stat).
  - Departmental ABAC silos for Radiology, Biochemistry, Pathology, and Microbiology.
  - Results review view (`#resultsReview`) with panic value thresholds and LOINC coding.
- [ ] **Missing & Required:**
  1. **Embedded Web DICOM Viewer:**
     - Zero-footprint web DICOM viewer component (using Cornerstone.js / OHIF or Rust-compiled WebAssembly DICOM parser).
     - Capabilities: Window/Level (presets for lung, bone, brain, soft tissue), pan, zoom, length/angle measurement, cine playback for multi-frame studies (CT/MRI/Angio).
  2. **Laboratory Instrument MLLP / ASTM Interface:**
     - Native background listener in `newloka_server` supporting HL7 v2.x over Minimal Lower Layer Protocol (MLLP) and ASTM E1381/E1394 serial/TCP protocols.
     - Automatic ingestion of analyzer output directly into pending `Specimen` and `Observation` records.
  3. **Specimen Accessioning & Barcode Label Generation:**
     - Automated generation and printing of standardized barcode labels (Code128 / DataMatrix) containing accession numbers, patient MRN, and department routing codes upon CPOE order placement.

---

### Subsystem D: Mobile Primacy & Physical Deployment Adaptation (T0 & T1)
*Goal: Deliver full-featured, zero-compromise clinical capability on smartphones and field hardware for solo practitioners and community health workers.*

- [x] **Shipped:**
  - Responsive, touch-friendly mobile layout across all 35 clinical views.
  - Offline IndexedDB caching with automatic local write queue in `newloka_web/js/api.js`.
  - Zero-configuration single-command desktop/laptop server launcher (`newloka-server --tier T0 --db clinic.db`).
  - Automatic parent directory creation on disk.
- [ ] **Missing & Required:**
  1. **Native Mobile App Packaging (Android & iOS):**
     - Mobile container build (e.g. Tauri Mobile or Capacitor packaging the embedded frontend and core library).
     - Integration with native hardware security: Android KeyStore and iOS Secure Enclave for hardware-backed Device Master Key (DMK) wrapping and biometric unlock.
  2. **Zero-Configuration LAN Peer Discovery Daemon (Phase 2):**
     - Built-in mDNS / DNS-SD broadcast and listener service in `newloka_server` (e.g. `mdns-sd` crate) allowing clinic tablets to instantly find the primary clinic server without manual IP entry.
  3. **Offline Field Peer-to-Peer Sync (ASHA / Field Worker to Clinic):**
     - Bluetooth Low Energy (BLE) or Wi-Fi Direct sync protocol allowing field workers with mobile devices to exchange delta bundles with visiting clinicians in remote environments without a local Wi-Fi router.

---

### Subsystem E: Hospital Information System (HIS) & Supply Chain (Phase 10)
*Goal: Provide end-to-end operational, logistical, and financial tooling required for high-volume hospital operations.*

- [x] **Shipped:**
  - Inpatient ward census board (`#whiteboard`) and 9-department bed allocations.
  - Outpatient appointment scheduling (`#appointmentSchedule`).
  - Receptionist and cashier billing ledger (`#billing`) with core-enforced ABAC diagnosis masking.
  - Pharmacy head activity telemetry and emergency stockout alerts (`#deptSummary`).
  - Executive hospital reports (`#reports`) tracking occupancy rate, ALOS, and emergency triage TAT.
- [ ] **Missing & Required:**
  1. **Dispensary Batch & Expiry Inventory Management:**
     - Granular stock tracking: Batch/Lot numbers, manufacturing and expiry dates, supplier master, purchase orders, and goods received notes (GRN).
     - First-Expiry-First-Out (FEFO) dispensing logic with automatic warnings on near-expiry stock (< 90 days).
     - Automated real-time inventory deduction upon prescription dispensing (`MedicationDispense`).
  2. **Interactive Visual Bed Management (ADT):**
     - Floor-plan or grid-based visual bed status matrix (Occupied, Available, Cleaning Required, Maintenance, Isolation).
     - Admission, Discharge, and Transfer (ADT) event generation linking encounters to bed stays.
  3. **Operating Theater (OT) / Surgical Scheduling:**
     - Multi-theater surgical scheduling grid.
     - WHO Surgical Safety Checklist enforcement (Sign In, Time Out, Sign Out).
     - Anesthesia record tracking and post-anesthesia care unit (PACU) recovery flowsheet.
  4. **Insurance Claims & Third-Party Administrator (TPA) Workflow:**
     - Pre-authorization request generation, policy details capture, claim status tracker, and co-pay calculation engine.

---

### Subsystem F: National Health Stacks, Standards & Interoperability (India & Global)
*Goal: Seamless compliance with Indian national digital health architecture (ABDM) and international exchange profiles without violating local-first privacy.*

- [x] **Shipped:**
  - Complete HL7 FHIR Release 4 internal standard representation across 23+ clinical resources.
  - Privacy-preserving Differential Privacy engine with Laplace perturbation and $k$-anonymity suppression ($k \ge 5$).
  - FHIR R4 `Consent` directives engine with purpose and category rules.
  - Cryptographic Ed25519 tamper-evident audit ledger conforming to HIPAA / DISHA principles.
- [ ] **Missing & Required:**
  1. **Ayushman Bharat Digital Mission (ABDM) National Gateway Bridge:**
     - **Milestone 1 (M1):** ABHA (Ayushman Bharat Health Account) creation, verification, linking via mobile/Aadhaar OTP, and ABHA QR-code registration at clinic reception.
     - **Milestone 2 (M2):** Health Information Provider (HIP) integration: Responding to ABDM discovery requests, linking patient records to ABHA, and pushing encrypted FHIR health data bundles upon consent approval.
     - **Milestone 3 (M3):** Health Information User (HIU) integration: Initiating consent requests to external hospitals via ABDM Consent Manager, fetching, and decrypting remote records into the patient timeline.
  2. **IHE Interoperability Profile Bridges:**
     - Support for IHE Mobile access to Health Documents (MHD) and Cross-Enterprise Document Sharing (XDS.b) for standards-based health information exchanges.
  3. **Digital Personal Data Protection (DPDP) Act 2023 Compliance Tooling:**
     - Explicit consent notice generators, patient data export packages in open format, and Data Protection Officer (DPO) verification reports.

---

### Subsystem G: Internal Secure Communications Stack (Phase 7)
*Goal: Provide zero-cloud, encrypted clinical messaging and coordination between care teams inside the hospital LAN.*

- [x] **Shipped:**
  - Structured SBAR clinical transfer notes and shift handoff progress logs.
- [ ] **Missing & Required:**
  1. **Local-Network Encrypted Clinician Messaging:**
     - End-to-end encrypted messaging engine operating over the local hospital server (using Double Ratchet / Signal protocol principles or TLS-authenticated WebSockets).
     - Individual and departmental channels (e.g., `#on-call-cardiology`, `#icu-nurses`, `#code-blue-team`).
  2. **Critical Result Paging & Alert Notification Bus:**
     - High-priority push alerts to clinician workstations and tablets when panic laboratory values or stat imaging reports are finalized.
     - Mandatory acknowledgment tracking with automated escalation if unacknowledged within 15 minutes.

---

## 4. Priority Roadmap & Active Problem-Solving Horizons

To prevent building theoretical solutions for unvalidated problems, remaining capabilities are structured into three pragmatic engineering horizons:

```
┌────────────────────────────────────────────────────────────────────────┐
│ Horizon 1: Immediate Clinical Velocity & Offline AI Ingestion          │
│ ────────────────────────────────────────────────────────────────────── │
│ • Standalone Clinical Calculators Suite (eGFR, Wells, CHA2DS2, Peds)   │
│ • Local OCR Ingestion Pipeline & Structured Prescription Extractor     │
│ • Dispensary Inventory Tracking (Batch, Expiry, FEFO Auto-Deduction)   │
└──────────────────────────────────┬─────────────────────────────────────┘
                                   │
┌──────────────────────────────────▼─────────────────────────────────────┐
│ Horizon 2: Equipment Interfacing & National Digital Health Stack       │
│ ────────────────────────────────────────────────────────────────────── │
│ • In-Browser Web DICOM PACS Viewer (Cornerstone.js / WASM parser)      │
│ • ABDM India Gateway Bridge (M1 ABHA Onboarding, M2 HIP, M3 HIU)      │
│ • Background mDNS Zero-Config LAN Peer Discovery Daemon                │
│ • HL7 v2 / ASTM Lab Analyzer Listener for Automated Blood Panels       │
└──────────────────────────────────┬─────────────────────────────────────┘
                                   │
┌──────────────────────────────────▼─────────────────────────────────────┐
│ Horizon 3: Enterprise Hospital Operations & Physical Mobility          │
│ ────────────────────────────────────────────────────────────────────── │
│ • Native Mobile Packaging (Android APK / iOS) with Hardware Keystore   │
│ • Local-LAN E2EE Care Team Messaging & Panic Alert Notification Bus    │
│ • Surgical Operating Theater (OT) Scheduling & WHO Safety Checklist    │
│ • Offline Field Worker Bluetooth / Wi-Fi Direct Mesh Sync              │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 5. Living Document Maintenance Protocol

To ensure this document remains an accurate, living compass rather than static documentation:
1. **Turn-by-Turn Updating:** Whenever a gap component transitions from *Missing* to *In-Progress* or *Shipped*, this document must be updated in the same sitting.
2. **Bi-Temporal Knowledge Graph Sync:** State transitions must be reflected across `knowledge_graph/nodes/roadmap-and-status.md` and indexed in `knowledge_graph/index.md` and `graph.json`.
3. **Pragmatic Problem Solving:** New requirements uncovered during clinical testing or user feedback are added to Section 3 under their appropriate subsystem, accompanied by concrete clinical rationales rather than speculative assumptions.
