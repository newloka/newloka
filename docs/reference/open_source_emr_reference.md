# Comparative Reference: Open Source EMR Implementations
**Authored by Amethyst (`newloka-core`) | Reference Document**

This document details the architectural frameworks, navigation trees, module divisions, and clinical workflows of the most complete and mature open-source Electronic Medical Record (EMR/EHR) and Hospital Information Systems (HIS): **OpenEMR**, **Bahmni / OpenMRS**, and **Medplum**.

---

## 1. OpenEMR (ONC-Certified Ambulatory & Multi-Specialty EMR)
*Repository: [openemr/openemr](https://github.com/openemr/openemr) | Architecture: Modular Monolith + FHIR R4 REST API*

OpenEMR is the most widely deployed open-source ambulatory EHR in the world, maintaining ONC Complete Ambulatory EHR certification.

### 1.1 UI & Navigation Hierarchy
OpenEMR organizes point-of-care workflows via an adaptive sliding menu, tree view, or top navigation bar:
- **Patient/Client Workspace:**
  - `Patients`: Master patient index with multi-parameter filtering (Name, DOB, MRN, National ID, Phone).
  - `New/Search`: Rapid demographic registration, contact data, insurance guarantors.
  - `Summary`: Central clinical chart (Active issues, Allergies, Meds, Vitals, Clinical notes, Immunizations, Documents).
  - `Visits`: Encounter records, visit history, consultation room tracking.
  - `Visit Forms`: Layout-Based Forms (LBF) including SOAP Notes, Review of Systems (ROS), Physical Exam, Eye Exam, CAM, and Minor Surgery.
- **Fees & Financial Ledger:**
  - `Fee Sheet`: Point-of-care CPT, HCPCS, and ICD-10 coding linked to encounters and providers.
  - `Billing Manager`: Claim batch generation (X12 837P, HCFA-1500), clearinghouse submission, and claim tracking.
  - `Payment / Checkout`: Cash register, co-pay receipts, credit card tokens, patient account ledger.
  - `Batch Payments / EOB`: Electronic Remittance Advice (ERA 835) automated posting.
- **Clinical Procedures & Laboratory Orders:**
  - `Order Entry`: Electronic orders for external and in-house labs (HL7 2.x lab interface).
  - `Electronic Reports`: Structured lab results review with normal reference ranges, abnormal flags, and provider sign-off.
  - `Configuration`: Lab test catalog, specimen types, result templates.
- **Practice Administration:**
  - `Users & Access Control`: Role-based access control (RBAC / php-GACL) governing Physicians, Nurses, Front Desk, Billing, Administrators.
  - `Facilities`: Multi-clinic locations, billing addresses, CLIA licenses, POS codes.
  - `Globals`: Comprehensive feature flags (>400 configuration switches for billing, security, UI theme, patient portal).
- **Messages & Communication (In-Basket):**
  - Clinical reminders, patient portal messages, lab alert callbacks, inter-staff messaging.

### 1.2 OpenEMR Strengths & Architectural Lessons for New Loka
1. **Layout-Based Forms (LBF):** Dynamic, declarative clinical forms defined in database schemas rather than hardcoded markup, enabling clinics to author specialized templates without code deployments.
2. **Comprehensive Fee Sheet:** Tight coupling between clinical diagnostic codes and immediate fee generation without clinician cognitive overhead.
3. **Limitations:** Monolithic legacy codebase, tightly bound to PHP/MySQL, lacks offline-first P2P mesh synchronization and local zero-dependency binary portability.

---

## 2. Bahmni (OpenMRS + OpenELIS + Odoo Hospital HIS)
*Repository: [Bahmni/bahmni-apps-frontend](https://github.com/Bahmni/bahmni-apps-frontend) | Architecture: Microservices + Event-Driven AtomFeed*

Bahmni is an enterprise-grade hospital management system purpose-built for low-resource district hospitals and multi-ward clinical settings.

### 2.1 Component Architecture & Integrated Microservices
```
┌─────────────────────────────────────────────────────────────┐
│                 Bahmni Web UI (bahmni-apps)                 │
└───────────────┬───────────────────────────────┬─────────────┘
                │ AtomFeed / REST               │
    ┌───────────▼───────────┐       ┌───────────▼───────────┐
    │ OpenMRS (Clinical EMR)│       │  OpenELIS (Lab LIS)   │
    │ • Patients & Encounters│      │ • Sample Tracking     │
    │ • Observations/Vitals │       │ • Analyzer Interfaces │
    │ • Diagnosis & Orders  │       │ • Result Validation   │
    └───────────┬───────────┘       └───────────┬───────────┘
                │                               │
                └───────────────┬───────────────┘
                                │
                    ┌───────────▼───────────┐
                    │    Odoo (ERP & LIS)   │
                    │ • Pharmacy & Dispense │
                    │ • Inpatient Billing   │
                    │ • Inventory & Stock   │
                    └───────────────────────┘
```

### 2.2 Bahmni App Ecosystem
- **Registration App:** Patient lookup, biometrics, demographic capture, visit opening, wristband barcode printing.
- **Clinical Consultation App (OPD):**
  - Patient Dashboard: Longitudinal disease progression, active problems, vitals trends.
  - Consultation Workspace:
    - Diagnosis tab: Coded ICD-10 search with primary/secondary and confirmed/presumed status.
    - Medications tab: Electronic prescriptions linked to hospital formulary stock.
    - Orders tab: Radiology, pathology, and microbiology test requisitions.
    - Disposition tab: Admit to ward, discharge, transfer, follow-up scheduling.
- **Inpatient Ward / Bed Management App:**
  - Visual Ward Matrix: Bed allocation grid grouped by ward (Male Medical, Female Surgical, ICU, Maternity).
  - Patient Movement: Transfer beds, bed-side nurse assignment, discharge readiness indicator.
- **Orders & Radiology App:** PACS integration (dcm4chee), digital X-ray and USG review.
- **Pharmacy & Inventory (Odoo):** Prescription fill queues, lot/batch tracking, expiry alerts, dispensary cash collection.

### 2.3 Bahmni Strengths & Architectural Lessons for New Loka
1. **Separation of Concerns:** Distinct clinical, laboratory, and financial engines connected via standardized event feeds.
2. **Inpatient Bed Grid:** Essential visual cueing for nurses and residents managing physical wards.
3. **Limitations:** Heavy resource consumption (>16 GB RAM required for all microservices), complex multi-container orchestration, poor single-laptop air-gapped performance.

---

## 3. Medplum (Modern FHIR R4-Native Headless & Modular EHR)
*Repository: [medplum/medplum](https://github.com/medplum/medplum) | Architecture: Pure FHIR R4 + React/TypeScript Monorepo*

Medplum is a modern, developer-first healthcare platform that models 100% of its data layer as HL7 FHIR R4 JSON resources.

### 3.1 Architectural Paradigm
- **FHIR-First Data Model:** Every entity is a native FHIR resource (`Patient`, `Practitioner`, `Encounter`, `Observation`, `MedicationRequest`, `CarePlan`).
- **Headless API Core:** High-throughput Node.js/PostgreSQL backend with automated schema validation.
- **Componentized Web UI (`@medplum/react`):** Reusable, type-safe clinical components (`ResourceTable`, `ResourceForm`, `Timeline`, `MedicationDialog`, `QuestionnaireForm`).
- **Subscription Bots:** Real-time serverless business logic triggered on FHIR mutations (e.g. panic lab results trigger instant SMS/email alerts).

### 3.2 Key Takeaways for New Loka
- New Loka's Rust backend (`newloka_core`) shares Medplum's pure FHIR R4 philosophy while replacing Node.js/Postgres with zero-dependency compiled Rust and SQLite AES-256-GCM.
- Declarative component pipelines: The frontend should bind directly to FHIR resource streams rather than ad-hoc custom endpoints.
