# Reference: Epic Systems Architecture, Viewports, and UI Trees
**Authored by Amethyst (`newloka-core`) | Reference Document**

This document details the software architecture, viewport paradigms, UI hierarchy, and modular capabilities of **Epic Systems** (Epic Hyperspace), the preeminent enterprise Electronic Health Record (EHR) system powering over 50% of US hospital beds and major international academic medical centers.

---

## 1. Epic Hyperspace Viewport Architecture

Epic Hyperspace structures the user interface into four concentric, context-preserving layers:
1. **Global Shell & Chrome** (Persistent Header, Top Bar, In Basket, Epic Button)
2. **Workspaces** (Independent top-level task containers: Schedules, In Basket, Patient Charts)
3. **Storyboard** (Persistent contextual patient rail on the left screen margin)
4. **Activities & Navigators** (Task-specific workflows and step-by-step clinical protocols)
5. **Sidebar / Utility Tray** (Collapsible drawer for Order Composer, References, SmartPhrases)

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│ [Epic ▼] [In Basket (4)] [Schedule: Dr. Mercer] [Pt: Ramesh Sharma] [Pt: Anita D.]   │
├──────────────┬─────────────────────────────────────────────────────────────────────────┤
│ STORYBOARD   │ ACTIVITY TABS: [Chart Review] [SnapShot] [Flowsheets] [Notes] [Orders]  │
│ (220-250px)  ├─────────────────────────────────────────────────────────────────────────┤
│              │ MAIN WORKING CANVAS                                                     │
│ • Photo      │                                                                         │
│ • Name, Age  │                                                                         │
│ • MRN, DOB   │                                                                         │
│ • Code Status│                                                                         │
│ • Team       │                                                                         │
│ • Alerts     │                                                                         │
│ • Vitals     │                                                                         │
│ • Allergies  │                                                                         │
│ • Problems   │                                                                         │
│ • Plan / Bed │                                                                         │
└──────────────┴─────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Core Viewport Components

### 2.1 Global Shell & In Basket
- **The Epic Button:** System menu giving quick access to patient lookup, open workspaces, tools, preferences, and session lock/logout.
- **Multi-Workspace Tabs:** Clinicians can open multiple patients simultaneously without losing draft notes or uncommitted orders. Tab switches preserve scroll position and uncommitted state.
- **In Basket:** The asynchronous operational nerve center of Epic:
  - `Results`: Abnormal and normal diagnostic lab/imaging results requiring review and sign-off.
  - `Rx Refills`: Pharmacy renewal requests with 1-click approve, modify, or deny actions.
  - `Staff Messages`: Nurse triage queries, phone encounters, clinical callbacks.
  - `Patient Advice Requests`: Asynchronous secure messages from MyChart patient portal.
  - `Consults / Orders`: Inpatient referral consult requests.

### 2.2 The Storyboard (Left Context Rail)
The Storyboard is a fixed vertical rail (typically 220–250px wide) that remains permanently visible on the left side of any patient workspace. It eliminates "chart diving" for basic context:
- **Demographics & Identification:** Patient photo, preferred name, legal name, age, biological sex/gender identity, DOB, MRN, national identifier.
- **Code Status & Critical Invariants:** High-contrast badge for Resuscitation Status (Full Code, DNR, DNI, Modified).
- **Care Team:** Attending Physician, Resident Physician, Primary Inpatient Nurse, Case Manager.
- **Safety Alerts & Isolation:** Red/amber badges for Infection Isolation (Airborne, Contact, Droplet), Fall Risk (Morse score), Aspiration Risk, Violent Patient flag.
- **Vital Signs Snapshot:** Most recent BP, Heart Rate, Respiratory Rate, Temp, SpO2, and Pain Score with time-elapsed tag (e.g. *35m ago*).
- **Allergies Banner:** Clickable pills (e.g. `Penicillin [Anaphylaxis]`, `Latex`).
- **Active Problem List:** Top 3-5 active diagnoses.
- **Encounter / Room Context:** Inpatient unit, room, bed number, admission day count (LOS Day 3), estimated discharge date.

### 2.3 Navigators
A **Navigator** is a vertical table-of-contents driving step-by-step clinical workflows:
- **Clinic Visit (Outpatient) Navigator:**
  1. Reason for Visit / Chief Complaint
  2. Vitals & Nurse Intake
  3. Allergies & Medication Reconciliation
  4. History (Past Medical, Surgical, Family, Social)
  5. Review of Systems (ROS) & Physical Exam
  6. Clinical Impressions & Diagnoses
  7. Order Entry (CPOE - Labs, Imaging, Prescriptions)
  8. Patient Instructions & After Visit Summary (AVS)
  9. Wrap-Up / Follow-Up Scheduling & Sign Encounter
- **Inpatient Admission Navigator:** Code status, home medication rec, initial orders, nursing risk assessments, VTE prophylaxis protocols.
- **Inpatient Discharge Navigator:** Medication reconciliation at discharge, follow-up appointments, pending lab tracking, discharge summary, transport orders.

### 2.4 Activities
Activities are modular workspaces selected from top or left tabs:
- **SnapShot:** Holistic multi-widget overview of the patient.
- **Chart Review:** Filterable longitudinal archive with tabs for Encounters, Labs, Imaging, Notes, Media, Procedures, Episodes.
- **Results Review:** Grid flowsheet displaying laboratory parameters longitudinally over time with abnormal values highlighted in red/bold.
- **Flowsheets & Doc Flowsheets:** High-density time-matrix grids for nursing vitals, I/O (fluid intake/output), wound care, and neuro checks.
- **Notes (SmartTools Engine):** Clinical documentation editor utilizing:
  - `SmartText`: Standardized disease templates.
  - `SmartPhrases` (dot-phrases): e.g. `.vitals` inserts latest vitals, `.meds` inserts active medications, `.labs` inserts latest CBC/BMP.
  - `SmartLists`: Dropdown picklists embedded directly inside prose.
- **MAR (Medication Administration Record):** 24-hour barcoded medication administration timetable with barcode scanning validation (BCMA), scheduled doses, PRN medications, and continuous IV infusions.
- **Order Entry (CPOE Composer):** Search catalog, order sets (smart groups), duplicate therapy alerts, dose-range checking, and co-signature routing.

---

## 3. Epic Enterprise Module Breakdown

| Module Name | Clinical Scope | Target Roles | Primary Viewports & Functions |
|---|---|---|---|
| **Cadence** | Enterprise Scheduling | Front Desk, Call Center | Master appointment grid, overbook controls, provider availability templates, waitlist queue. |
| **Prelude** | Patient Registration & ADT | Registration Clerks | Demographic capture, identity proofing, insurance verification, guarantor linking. |
| **Grand Central** | Bed Tracking & ADT | Bed Managers, Nurses | Visual hospital bed census, room occupancy, transfer requests, environmental cleaning tracking. |
| **ClinDoc** | Inpatient Clinical Doc | Inpatient Nurses, Clinicians | Nursing flowsheets, shift assessments, plan of care, clinical pathways, intake/output tracking. |
| **ASAP** | Emergency Department | ED Physicians, Triage Nurses | ED Track Board (colored acuity bed cards), rapid triage scoring (ESI), trauma flowsheets. |
| **OpTime & Anesthesia** | Operating Room & Surgical | Surgeons, Scrub Nurses, Anesthetists | OR schedule, surgical case booking, perioperative nursing documentation, intra-op anesthesia telemetry. |
| **Willow Inpatient** | Inpatient Pharmacy | Hospital Pharmacists | Medication order verification queue, IV compounding, automated dispensing cabinet (Pyxis/Omnicell) sync. |
| **Willow Ambulatory** | Retail / Outpatient Pharmacy | Outpatient Pharmacists | Prescription dispensing, insurance adjudication, patient drug counseling records. |
| **Beaker Clinical (CP)** | Clinical Pathology / Labs | Lab Technologists, Phlebotomists | Specimen accessioning, analyzer telemetry, batch resulting, reflex testing rules. |
| **Beaker Anatomic (AP)** | Anatomic Pathology | Pathologists, Histotechnicians | Grossing, cassette tracking, histology slide processing, synoptic cancer reporting. |
| **Radiant** | Radiology Information System | Radiologists, Technologists | Modality worklists, PACS image linking, peer review, structured radiological reports. |
| **Cupid** | Cardiology Information System | Cardiologists, Cath Lab Nurses | Echo reporting, hemodynamic monitoring, cath lab procedure logs, ECG management. |
| **Stork** | Obstetrics & Maternity | Obstetricians, Midwives | Antenatal tracking, labor progression partogram, fetal heart rate telemetry, newborn birth records. |
| **Beacon** | Medical & Pediatric Oncology | Medical Oncologists, Chemo Nurses | Staging, multi-day chemotherapy regimen ordering, BSA dose calculation, extravasation tracking. |
| **Resolute PB** | Professional Billing | Ambulatory Billing Clerks | Physician claim generation, encounter superbills, co-pay receipts, patient self-pay portal. |
| **Resolute HB** | Hospital Billing | Hospital Revenue Cycle Team | UB-04 institutional claims, DRG grouping, inpatient charge capture, denial management. |
| **MyChart** | Patient & Family Portal | Patients, Guardians | View lab results, request appointments, direct messaging with clinician, pay bills, fill health intake. |
| **Rover** | Mobile Nursing Application | Floor Nurses (Handhelds) | Barcode medication administration (BCMA), bedside vitals capture, specimen barcode collection. |

---

## 4. Key Architectural Takeaways for New Loka

1. **Persistent Context (Storyboard):** Clinicians should never have to re-read or search for basic patient identity, alerts, and vitals when switching between activities (Prescription Pad, Notes, Orders, Labs).
2. **Multi-Workspace Concurrency:** Clinicians often treat multiple patients in parallel; New Loka must allow keeping multiple patient tabs open with uncommitted drafts preserved in local memory.
3. **Role-Driven Presets:** An ED triage nurse requires an immediate track board; an outpatient doctor requires a prescription letterhead and pharmacopoeia; a billing cashier requires a financial ledger with masked clinical details. New Loka must declaratively bind layout, pipelines, and ABAC permissions based on active preset.
