# Reference: Commercial EMR Benchmarks & Clinical Interaction Patterns
**Authored by Amethyst (`newloka-core`) | Reference Document**

This document details the comparative architecture, capability footprints, and canonical clinical interaction patterns of dominant commercial Electronic Medical Record (EMR) systems—including **Epic Systems (Hyperspace/MyChart)**, **Oracle Health (Cerner PowerChart/Millennium)**, **MEDITECH Expanse**, and **athenahealth**—and maps their key design patterns into New Loka's local-first, FHIR R4-native architecture.

---

## 1. Commercial EMR Feature Footprints

### 1.1 Epic Systems (Hyperspace / MyChart)
- **Clinical Core:** Role-based workspace (physician, nurse, pharmacist, registrar), ADT bed management, multi-resource Cadence scheduling, narrative notes with SmartPhrases, CPOE (labs, imaging, meds, consults), Results review with trending, MAR time-grids, Flowsheets with early warning scores (MEWS/NEWS2), Clinical Decision Support (CDS) Best Practice Advisories, asynchronous In Basket messaging.
- **Patient Engagement:** MyChart portal (self-scheduling, test results, secure messaging, proxy access, billing).
- **Ancillary & Specialty:** Perioperative management (OpTime/Anesthesia), Emergency Department (ASAP), Oncology/Chemotherapy protocols (Beacon), Pharmacy dispensing (Willow), PACS/DICOM radiology integration (Radiant).
- **Interoperability:** Care Everywhere Health Information Exchange (HIE), SMART on FHIR App Orchard, Bulk FHIR CMS export.
- **Mobility:** Haiku (smartphones for physicians), Canto (tablets), Rover (handheld barcode & MAR scanning for nurses).

### 1.2 Oracle Health / Cerner (PowerChart / Millennium)
- **PowerChart:** Unified longitudinal clinical view organizing encounters, diagnostics, and orders.
- **Millennium Platform:** Real-time event-driven clinical database backbone.
- **HealtheIntent:** Population health management and registry risk stratification.
- **Clinical Capabilities:** Ambient clinical documentation / voice AI scribes, continuous physiological telemetry ingestion, real-time location systems (RTLS) for bed and asset tracking, tele-ICU remote monitoring.

### 1.3 MEDITECH Expanse
- **Web-First Architecture:** Modern HTML5 responsive clinical interface designed for tablets and browser mobility.
- **Point-of-Care Surveillance:** Automated sepsis risk surveillance and deterioration algorithms.
- **Integrated Practice:** Ambulatory self-scheduling, integrated revenue cycle management, and direct claims submission.

### 1.4 athenahealth
- **Cloud-Native Ambulatory:** Focus on multi-tenant physician practice management, automated billing rules, and clearinghouse claim scrubbing.
- **Network Insights:** Crowdsourced payer rules engine to reduce denial rates and accelerate collections.

---

## 2. Cross-Vendor Architectural Matrix

| Capability Domain | Epic Systems | Oracle / Cerner | MEDITECH Expanse | athenahealth | New Loka Polymorphic Core |
|---|:---:|:---:|:---:|:---:|:---:|
| **Patient Demographics & MPI** | ✅ | ✅ | ✅ | ✅ | ✅ FHIR R4 `Patient` |
| **Outpatient Scheduling** | ✅ | ✅ | ✅ | ✅ | ✅ `#appointmentSchedule` |
| **Computerized Order Entry (CPOE)** | ✅ | ✅ | ✅ | Limited | ✅ `#cpoeOrders` (4 Lab Silos) |
| **Diagnostic Results Review** | ✅ | ✅ | ✅ | ✅ | ✅ `#resultsReview` (Graphs + Panic Labs) |
| **Medication Administration (MAR)** | ✅ | ✅ | Limited | ❌ | ✅ `#mar` (Hourly Nursing Grid) |
| **Narrative Clinical Notes** | ✅ | ✅ | ✅ | ✅ | ✅ `#clinicalNotes` (SOAP + SmartPhrases) |
| **Problem List & Conditions** | ✅ | ✅ | ✅ | ✅ | ✅ FHIR R4 `Condition` |
| **Allergy & Adverse Events** | ✅ | ✅ | ✅ | ✅ | ✅ `#alerts` + `AllergyIntolerance` |
| **Vitals Flowsheet & Trending** | ✅ | ✅ | ✅ | ❌ | ✅ `#vitalsFlowsheet` + NEWS2 |
| **Drug-Drug Interaction Checks** | ✅ | ✅ | ✅ | ✅ | ✅ 61k+ Formulary Shield |
| **Inpatient Ward / Bed Census** | ✅ | ✅ | Limited | ❌ | ✅ `#whiteboard` + `#inpatientRegister` |
| **Clinical Shift Handoff** | ✅ | ✅ | ✅ | ❌ | ✅ `#handoffSbar` (SBAR Structured) |
| **Hospital Departments & Silos** | ✅ | ✅ | ✅ | ❌ | ✅ `#departments` (9 Department Silos) |
| **Executive Governance Telemetry** | ✅ | ✅ | ✅ | ✅ | ✅ `#reports` (Occupancy, ALOS, TAT) |
| **ABAC Role Redaction (Billing)** | ✅ | ✅ | Limited | ✅ | ✅ `Role::Billing` Diagnostic Masking |
| **Tamper-Evident Audit Ledger** | ✅ | ✅ | ✅ | ✅ | ✅ Ed25519 Signed `AuditEvent` |
| **100% Offline Local Operation** | ❌ | ❌ | ❌ | ❌ | ✅ **Native Invariant (T0-T4)** |
| **LAN Peer-to-Peer Mesh Sync** | ❌ | ❌ | ❌ | ❌ | ✅ **CRDT Vector Clock Delta Sync** |
| **Zero-Cloud Data-Layer Crypto** | ❌ | ❌ | ❌ | ❌ | ✅ **AES-256-GCM AEAD at Rest** |
| **FHIR R4 Universal Core** | Partial | Partial | Partial | Partial | ✅ **100% Native Internal Model** |
| **Privacy-Preserving Research** | Add-on | Add-on | ❌ | ❌ | ✅ **Laplace DP + k-Anonymity (T4)** |
| **Dynamic Patient Consent** | Basic | Basic | ❌ | ❌ | ✅ **FHIR R4 Consent Directives (T4)** |

---

## 3. Canonical Clinical Interaction Patterns

New Loka's Single-Page Application (`newloka_web`) adapts eight fundamental interaction blueprints distilled from commercial systems, translated into lightweight, zero-framework vanilla JavaScript and dark-mode ergonomics.

### 3.1 Persistent Patient Context Banner (Epic Storyboard / Cerner Banner)
Eliminates cognitive latency and dangerous chart-diving by permanently presenting life-critical safety invariants regardless of the active clinical tab.

```
+------------------------------------------------------------------------------------------+
| DOE, JANE  (F)  85y  DOB: 1941-03-15  MRN: 102384                                        |
| Location: 4N-412  |  Admit: 06-01  |  LOS: 15d  |  Code: FULL CODE                       |
| Attending: Dr. J. Smith  |  Primary RN: S. Lee  |  Diet: Low Sodium                      |
| Allergies: 🔴 Penicillin [Anaphylaxis], Sulfa [Rash]                                     |
| Precautions: 🟡 Contact Precautions [MRSA]  |  Fall Risk: 🩸 High (Morse 65)             |
+------------------------------------------------------------------------------------------+
| [Summary] [Orders] [Results] [Meds] [Flowsheet] [Notes] [MAR] [Handoff] [Care Plan]     |
+------------------------------------------------------------------------------------------+
```

### 3.2 Patient List / Ward Census (Epic Patient Lists / Cerner Multi-Patient Task List)
Fast triage and unit overview with color-coded safety badges, location sorting, and row-level quick actions.

```
+------------------------------------------------------------------------------------------+
| Patient List                 Ward: [ Cardiology Ward ▼ ]       Search: [ Sharma..... ]   |
+------------------------------------------------------------------------------------------+
| [All (18)]  [Inpatient (12)]  [Outpatient (4)]  [Emergency (2)]  [High Acuity (3)]       |
+-------------------+---------+--------+----------+---------------+------------------------+
| Patient Name      | Age/Sex | MRN    | Room/Bed | Attending     | Clinical Flags         |
+-------------------+---------+--------+----------+---------------+------------------------+
| Doe, Jane         | 85 / F  | 102384 | 4N-412   | Dr. Smith     | ⚠️ Allergy  🩸 Fall    |
| Singh, Raj        | 62 / M  | 102385 | 4N-413   | Dr. Patel     | ⚡ Isolation [Contact] |
| Kumar, Priya      | 34 / F  | 102386 | ED-TR2   | (Unassigned)  | 🔴 Panic Lab (K+ 6.8)  |
+-------------------+---------+--------+----------+---------------+------------------------+
```

### 3.3 Computerized Physician Order Entry (CPOE Orderset Pattern)
Standardizes diagnostic ordering across laboratory, radiology, and medication domains with priority tiers and duplicate detection.

```
+------------------------------------------------------------------------------------------+
| Orders: DOE, JANE                     Filter: [● Active]  [Pending]  [Discontinued]       |
+------------------------------------------------------------------------------------------+
| [+ New Order]  [Order Sets ▼]  [Favorites ▼]  Department: [Biochemistry ▼]               |
+------------------------------------------------------------------------------------------+
| Search Catalog: [ Troponin............................................. ] [Search]       |
+------------------------------------------------------------------------------------------+
| Active Orders:                                                                           |
|   ☑ Serum Troponin-I (Stat)                     Priority: STAT      [Edit]  [Cancel]     |
|   ☑ 12-Lead Electrocardiogram (Routine)         Priority: ROUTINE   [Edit]  [Cancel]     |
|   ☑ Metformin 500mg PO BID (Active)             Priority: ROUTINE   [Hold]  [Discontinue]|
+------------------------------------------------------------------------------------------+
| Selected (2)  |  Clinician Sign-off: [ Dr. Yash Kulkarni (MD) ]  [ Sign & Transmit Orders ] |
+------------------------------------------------------------------------------------------+
```

### 3.4 Medication Administration Record (MAR Time-Grid)
Hourly scheduled, PRN, and stat medication administration tracking designed for bedside nursing with clear overdue highlighting.

```
+------------------------------------------------------------------------------------------+
| MAR: DOE, JANE   |   Date: 2026-11-15   |   Shift: Day (07:00 - 19:00)                   |
+------------------------------------------------------------------------------------------+
| Medication          | Dose / Route | 08:00     | 10:00     | 12:00     | 14:00           |
+---------------------+--------------+-----------+-----------+-----------+-----------------+
| Metformin           | 500mg PO     | [✓ Given] | —         | —         | —               |
| Ceftriaxone         | 1g IV        | —         | [✓ Given] | —         | —               |
| Furosemide          | 40mg PO      | —         | —         | [OVERDUE] | [Record Dose]   |
| Paracetamol (PRN)   | 650mg PO     | —         | —         | —         | [Give PRN]      |
+---------------------+--------------+-----------+-----------+-----------+-----------------+
| Summary: 2 Given  |  1 Overdue  |  1 Due Soon  |  Barcode Scanner: [ Active ]            |
+------------------------------------------------------------------------------------------+
```

### 3.5 Longitudinal Vitals Flowsheet & Early Warning Scoring
Horizontal time-axis flowsheet organizing physiological parameters and auto-calculating composite clinical scores (NEWS2).

```
+------------------------------------------------------------------------------------------+
| Vitals Flowsheet: DOE, JANE   |   Window: [ Last 24 Hours ▼ ]    Export: [ CSV / PDF ]   |
+-------------------+------------+------------+------------+------------+------------------+
| Parameter         | 06:00      | 10:00      | 14:00      | 18:00      | Trend Graph      |
+-------------------+------------+------------+------------+------------+------------------+
| Temp (°C)         | 36.8       | 37.2       | 38.6*      | 37.4       | ───/▔\───        |
| Heart Rate (bpm)  | 74         | 82         | 112*       | 86         | ───/▔\───        |
| Blood Pressure    | 124/78     | 130/84     | 144/92*    | 128/82     | ───/▔\───        |
| Resp Rate (rpm)   | 16         | 18         | 24*        | 18         | ───/▔\───        |
| SpO2 (%)          | 98 (RA)    | 97 (RA)    | 93 (NC 2L) | 96 (RA)    | ───\_/───        |
+-------------------+------------+------------+------------+------------+------------------+
| NEWS2 Score       | 0 (Normal) | 1 (Low)    | 6 (MEDIUM) | 1 (Low)    | [ View Protocol ]|
+-------------------+------------+------------+------------+------------+------------------+
```

### 3.6 Structured Shift Handoff (SBAR Protocol)
Enforces structured Situation-Background-Assessment-Recommendation handovers to eliminate communication errors during nurse and physician shift transitions.

```
+------------------------------------------------------------------------------------------+
| SBAR Clinical Handoff   |   Unit: Cardiology   |   From: Night Team   |   To: Day Team   |
+------------------------------------------------------------------------------------------+
| Patient: DOE, JANE (85F)  |  Bed: 4N-412  |  MRN: 102384  |  Code Status: FULL CODE       |
+------------------------------------------------------------------------------------------+
| [S] Situation:                                                                           |
|     Acute CHF exacerbation. Weaning supplemental O2. Responding to IV diuresis.          |
| [B] Background:                                                                          |
|     Known ischemic cardiomyopathy (EF 35%), CKD Stage 3, Type 2 Diabetes.               |
| [A] Assessment:                                                                          |
|     Clinically euvolemic overnight. Net negative 1,200 mL. K+ stable at 4.2 mEq/L.        |
| [R] Recommendation / Action Items:                                                       |
|     1. Transition IV furosemide to PO 40mg with lunch.                                   |
|     2. Repeat basic metabolic panel tomorrow morning.                                    |
|     3. Echo scheduled for 11:00 AM in Cardiology Lab.                                    |
+------------------------------------------------------------------------------------------+
```

### 3.7 Inpatient Whiteboard & Ward Census Board
Real-time department-wide or ward-wide bed allocation board providing charge nurses and department chiefs with operational throughput visibility.

```
+------------------------------------------------------------------------------------------+
| Inpatient Whiteboard: Ward 4N (Cardiology)  |  Census: 14 / 16 Beds (87% Occupancy)      |
+------+---------------+---------+-----------+------+--------------------+-----------------+
| Bed  | Patient Name  | Age/Sex | Attending | LOS  | Primary Diagnosis  | Discharge Plan  |
+------+---------------+---------+-----------+------+--------------------+-----------------+
| 410  | (Available)   | —       | —         | —    | —                  | Ready for Admit |
| 411  | (Cleaning)    | —       | —         | —    | —                  | Terminal Clean  |
| 412  | Doe, Jane     | 85 / F  | Dr. Smith | 3d   | Acute Decomp CHF   | Pending PO trial|
| 413  | Singh, Raj    | 62 / M  | Dr. Patel | 2d   | NSTEMI Post-Cath   | Discharge Today |
| 414  | Kumar, Priya  | 34 / F  | Dr. Gupta | 1d   | Chest Pain R/O     | Stress Test Due |
+------+---------------+---------+-----------+------+--------------------+-----------------+
```

### 3.8 Outpatient Multi-Resource Scheduling Grid (Cadence Pattern)
Day/week booking matrix supporting practitioner queue management, slot blocking, and walk-in triage.

```
+------------------------------------------------------------------------------------------+
| Schedule: Outpatient Cardiology Clinic   |   Provider: Dr. Yash Kulkarni   |  2026-11-15 |
+----------+-------------------------------------------------------------------------------+
| Time     | Appointment Status & Patient Details                                          |
+----------+-------------------------------------------------------------------------------+
| 08:30 AM | [ COMPLETED ]  Sharma, Ramesh (62M) — Follow-up Hypertension                  |
| 09:00 AM | [ IN CLINIC ]  Patel, Meena (45F) — Post-Angioplasty Routine Check            |
| 09:30 AM | [ BOOKED    ]  Deshmukh, Anand (58M) — Dyspnea on Exertion Evaluation         |
| 10:00 AM | [ BLOCKED   ]  Department Clinical Conference / Grand Rounds                  |
| 10:30 AM | [ AVAILABLE ]  (Click to book new appointment or register walk-in)            |
+----------+-------------------------------------------------------------------------------+
```

---

## 4. Architectural Synthesis for New Loka

1. **Local-First Ergonomic Advantage:** Unlike Epic or Cerner, which require gigabit campus networks or cloud connections, New Loka renders these exact interaction patterns entirely inside a single-page application (`newloka_web`) backed by a local Axum/Tokio binary and SQLite.
2. **Zero Chart-Diving Latency:** The left context Storyboard rail (`.storyboard-rail`) ensures clinicians never lose vital context when switching between CPOE, Clinical Notes, Flowsheets, and MAR.
3. **Three-Tap Usability Guarantee:** In conformance with New Loka's core usability invariant, every safety-critical datapoint (Allergies, Current Meds, Codes, Panic Results) is reachable within three clicks or taps from any active screen.
