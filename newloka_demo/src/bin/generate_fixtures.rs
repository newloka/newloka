use chrono::Utc;
use newloka_core::crypto::DeviceMasterKey;
use newloka_storage::{EncryptedRecord, StorageEngine};
use std::fs;
use std::path::Path;

pub const DEMO_MASTER_KEY_BYTES: [u8; 32] = *b"DEMOMASTERKEY1234567890123456789";

fn main() -> anyhow::Result<()> {
    println!("Generating encrypted demo fixtures using Demo Master Key...");
    let dmk = DeviceMasterKey {
        key: DEMO_MASTER_KEY_BYTES,
    };

    let fixtures_dir = Path::new("newloka_demo/fixtures/records");
    fs::create_dir_all(fixtures_dir)?;

    let mut records: Vec<EncryptedRecord> = Vec::new();

    // 1. Instance Config & Config Default
    let config = serde_json::json!({
        "tier": "T0",
        "clinicProfile": "solo",
        "defaultLandingView": "rxpad",
        "roleLandingViews": {
            "doctor": "rxpad",
            "clinician": "rxpad",
            "resident": "inpatientRegister",
            "resident_doctor": "inpatientRegister",
            "nurse": "mar",
            "billing": "billing",
            "cashier": "billing",
            "reception": "appointmentSchedule",
            "pharmacy": "deptSummary",
            "pharmacist": "deptSummary",
            "department_head": "deptSummary",
            "administrator": "dashboard"
        },
        "installedModules": [
            "dashboard", "rxpad", "inpatientRegister", "billing", "deptSummary",
            "patients", "patientChart", "clinicalNotes", "cpoeOrders", "vitalsFlowsheet",
            "resultsReview", "drugInteraction", "medications", "encounters", "observations",
            "conditions", "procedures", "alerts", "carePlans", "familyHistory", "immunizations",
            "documents", "appointmentSchedule", "mar", "handoffSbar", "whiteboard", "ingest",
            "departments", "reports", "research", "consent", "federation",
            "audit", "settings"
        ],
        "clinic": {
            "name": "New Loka Clinic",
            "subtitle": "Open-Source Community Health",
            "regNo": "NL-CLINIC-DEMO-01",
            "address": "100 Innovation Way, Suite 400",
            "phone": "+1 (555) 019-2834",
            "email": "clinic@newloka.org"
        },
        "sync": {
            "meshPeers": [],
            "centralHospitalUrl": "",
            "centralHospitalToken": "",
            "centralSyncEnabled": false
        }
    });

    records.push(StorageEngine::encrypt_record(
        &dmk,
        "Config",
        "instance-config",
        &config,
        None,
        None,
    )?);
    records.push(StorageEngine::encrypt_record(
        &dmk,
        "Config",
        "config-default",
        &config,
        None,
        None,
    )?);

    // 2. Embedded Assets (Logo & Signature)
    let assets_txt = include_str!("../../../newloka_web/assets/embedded_assets.txt");
    let mut lines = assets_txt.lines().filter(|l| !l.trim().is_empty());
    let logo_b64 = lines.next().unwrap_or("").trim().to_string();
    let sig_b64 = lines.next().unwrap_or("").trim().to_string();

    // 3. Prescriber Profile
    let prescriber = serde_json::json!({
        "clinicianName": "Dr. Alex Mercer",
        "clinician_name": "Dr. Alex Mercer",
        "qualifications": "MBBS, MD",
        "regNo": "NL-DOC-2026-001",
        "registration_no": "NL-DOC-2026-001",
        "specialty": "General Medicine",
        "department": "",
        "isLegalPrescriber": true,
        "is_legal_prescriber": true,
        "signatureBase64": &sig_b64,
        "signature_data_url": &sig_b64,
        "signature": "Dr. Alex Mercer, MBBS, MD",
        "logoBase64": &logo_b64,
        "logo_data_url": &logo_b64,
        "clinicHeader": {
            "clinicName": "New Loka Clinic",
            "tagline": "Open-Source Community Health",
            "address": "100 Innovation Way, Suite 400",
            "phone": "+1 (555) 019-2834",
            "email": "clinic@newloka.org",
            "regNo": "NL-DOC-2026-001",
            "logoBase64": &logo_b64
        }
    });

    records.push(StorageEngine::encrypt_record(
        &dmk,
        "Practitioner",
        "prescriber-profile",
        &prescriber,
        None,
        None,
    )?);
    records.push(StorageEngine::encrypt_record(
        &dmk,
        "Practitioner",
        "prescriber-profile-default",
        &prescriber,
        None,
        None,
    )?);

    // 4. RxPad Template
    let template = serde_json::json!({
        "resourceType": "DocumentReference",
        "id": "rxpad-template-active",
        "name": "New Loka Clinic Demo",
        "isLockedByAdmin": false,
        "userEditable": true,
        "header": {
            "showLogo": true,
            "logoBase64": &logo_b64,
            "showEntityName": true,
            "entityName": "New Loka Clinic",
            "showDoctorName": true,
            "doctorName": "Dr. Alex Mercer",
            "showQualifications": true,
            "qualifications": "MBBS, MD",
            "showRegNo": true,
            "regNo": "NL-DOC-2026-001",
            "showDepartment": false,
            "department": "",
            "showSubtitle": true,
            "subtitle": "Open-Source Community Health",
            "showAddressContact": true,
            "address": "100 Innovation Way, Suite 400",
            "phone": "+1 (555) 019-2834",
            "email": "clinic@newloka.org",
            "signatureBase64": &sig_b64
        },
        "bindings": {
            "entityName": "persist",
            "doctorName": "persist",
            "doctorQual": "persist",
            "doctorReg": "persist",
            "doctorSig": "persist",
            "clinicContact": "persist",
            "currentDate": "fetch",
            "ageSex": "fetch",
            "patientSerial": "db_sync",
            "patientName": "db_sync",
            "patientContact": "db_sync",
            "weight": "fetch",
            "bp": "fetch",
            "diagnosis": "db_sync",
            "advisedLabs": "db_sync",
            "medications": "db_sync",
            "followUp": "db_sync"
        },
        "visibility": {
            "logo": true,
            "entityName": true,
            "doctorName": true,
            "qualifications": true,
            "regNo": true,
            "department": false,
            "subtitle": false,
            "addressContact": true,
            "serial": true,
            "patientName": true,
            "ageSex": true,
            "date": true,
            "patientContact": true,
            "weight": false,
            "bp": false,
            "diagnosis": true,
            "advisedLabs": true,
            "medications": true,
            "followUp": true,
            "signature": true
        }
    });

    records.push(StorageEngine::encrypt_record(
        &dmk,
        "RxPadTemplate",
        "rxpad-template-default",
        &template,
        None,
        None,
    )?);
    records.push(StorageEngine::encrypt_record(
        &dmk,
        "RxPadTemplate",
        "rxpad-template-active",
        &template,
        None,
        None,
    )?);

    // 5. Inpatient Census
    let census = serde_json::json!([
        {
            "id": "inp-001",
            "patientName": "Ramesh Patel",
            "ageSex": "54y / M",
            "mrn": "MRN100000",
            "ward": "Ward 4N (Medical)",
            "bed": "Bed 12",
            "admitDate": "15/11/2026",
            "daysAdmitted": 2,
            "diagnosis": "Type 2 Diabetes Mellitus with Hyperglycemia",
            "acuity": "Guarded",
            "news2": 3,
            "attending": "Dr. Alex Mercer (Attending Physician)",
            "diet": "Diabetic diet (1800 kcal)",
            "status": "Admitted"
        },
        {
            "id": "inp-002",
            "patientName": "Sunita Sharma",
            "ageSex": "48y / F",
            "mrn": "MRN100001",
            "ward": "Ward 4N (Medical)",
            "bed": "Bed 14",
            "admitDate": "15/11/2026",
            "daysAdmitted": 1,
            "diagnosis": "Acute Bronchitis with Bronchospasm",
            "acuity": "Stable",
            "news2": 1,
            "attending": "Dr. Alex Mercer (Attending Physician)",
            "diet": "Regular soft diet",
            "status": "Admitted"
        },
        {
            "id": "inp-003",
            "patientName": "Vikram Patil",
            "ageSex": "62y / M",
            "mrn": "MRN100002",
            "ward": "ICU (Intensive Care)",
            "bed": "Bed 02",
            "admitDate": "15/11/2026",
            "daysAdmitted": 1,
            "diagnosis": "Acute Coronary Syndrome (NSTEMI)",
            "acuity": "Critical",
            "news2": 7,
            "attending": "Dr. Alex Mercer (Attending Physician)",
            "diet": "Low sodium cardiac diet",
            "status": "Admitted"
        }
    ]);

    records.push(StorageEngine::encrypt_record(
        &dmk,
        "InpatientCensus",
        "inpatient-census",
        &census,
        None,
        None,
    )?);

    // 6. Billing Invoices
    let invoices = serde_json::json!([
        {
            "id": "INV-2026-0001",
            "invoice_number": "INV-2026-0001",
            "date": "15/11/2026",
            "patientName": "Ramesh Patel",
            "patient_name": "Ramesh Patel",
            "ageSex": "54y / M",
            "contact": "+1 (555) 019-2834",
            "diagnosis": "Type 2 Diabetes Mellitus & Essential Hypertension",
            "clinical_diagnosis": "Type 2 Diabetes Mellitus & Essential Hypertension",
            "items": [
                { "name": "General Physician Consultation", "description": "General Physician Consultation", "qty": 1, "rate": 500.0, "amount": 500.0 },
                { "name": "Prescription Pharmacy (Metformin 500mg, Telmisartan 40mg)", "description": "Prescription Pharmacy (Metformin 500mg, Telmisartan 40mg)", "qty": 1, "rate": 350.0, "amount": 350.0 },
                { "name": "Complete Blood Count (CBC) & HbA1c", "description": "Complete Blood Count (CBC) & HbA1c", "qty": 1, "rate": 650.0, "amount": 650.0 }
            ],
            "totalAmount": 1500.0,
            "total_amount": 1500.0,
            "discount": 0.0,
            "netAmount": 1500.0,
            "paidAmount": 1500.0,
            "paid_amount": 1500.0,
            "status": "Paid",
            "paymentMethod": "UPI / Digital",
            "payment_method": "UPI / Digital"
        }
    ]);

    records.push(StorageEngine::encrypt_record(
        &dmk,
        "BillingInvoice",
        "billing-invoices",
        &invoices,
        None,
        None,
    )?);

    // 7. Consent Directives
    let consent = serde_json::json!({
        "resourceType": "Consent",
        "id": "consent-demo-001",
        "status": "active",
        "scope": { "coding": [{ "system": "http://terminology.hl7.org/CodeSystem/consentscope", "code": "patient-privacy" }] },
        "category": [{ "coding": [{ "system": "http://terminology.hl7.org/CodeSystem/consentcategorycodes", "code": "npp" }] }],
        "patient": { "reference": "Patient/pat-demo-000", "display": "Ramesh Patel" },
        "dateTime": Utc::now().to_rfc3339(),
        "policyRule": { "text": "New Loka Local-First Patient Confidentiality and ABAC Consent Policy" },
        "provision": {
            "type": "permit",
            "period": { "start": "2026-01-01", "end": "2030-12-31" },
            "purpose": [
                { "code": "TREAT", "display": "Clinical Treatment and Point of Care" },
                { "code": "CLINTRL", "display": "Clinical Trial and De-Identified Observational Research" }
            ]
        }
    });
    records.push(StorageEngine::encrypt_record(
        &dmk,
        "Consent",
        "consent-demo-001",
        &consent,
        Some("pat-demo-000".to_string()),
        None,
    )?);

    // 7b. Enterprise Hospital Departments
    let depts = serde_json::to_value(&newloka_core::department::default_departments())?;
    records.push(StorageEngine::encrypt_record(
        &dmk,
        "Department",
        "hospital-departments",
        &depts,
        None,
        None,
    )?);

    // 7c. Department Transfers
    let transfers = serde_json::json!([
        {
            "id": "tr-001",
            "patient_id": "pat-demo-002",
            "patient_name": "Vikram Patil",
            "mrn": "MRN100002",
            "source_department": "Emergency & Trauma",
            "target_department": "Cardiology & Cath Lab",
            "requesting_clinician_id": "doc_emergency",
            "requesting_clinician_name": "Dr. Acute Resuscitation",
            "accepting_clinician_id": "doc_attending",
            "accepting_clinician_name": "Dr. Cardio Attending",
            "assigned_bed": "CCU Bed 02",
            "urgency": "urgent",
            "reason": "Acute Coronary Syndrome / NSTEMI with elevated Troponin-I",
            "clinical_handover_notes": "SBAR: 62M presented with 2h severe substernal crushing pain. ECG shows ST depression in V4-V6. Given loading dose Aspirin 325mg + Ticagrelor 180mg. Urgent coronary angiography requested.",
            "active_medications_summary": ["Aspirin 325mg stat", "Ticagrelor 180mg stat", "Atorvastatin 80mg stat", "Sublingual Nitroglycerin"],
            "pending_orders_summary": ["Emergency Echocardiogram", "Repeat Troponin-I in 3h", "Serum Potassium"],
            "allergies_summary": ["No known drug allergies"],
            "status": "accepted",
            "created_at": "2026-11-13T10:30:00Z",
            "updated_at": "2026-11-13T11:00:00Z",
            "handover_signed_by": "Dr. Cardio Attending"
        }
    ]);
    records.push(StorageEngine::encrypt_record(
        &dmk,
        "Transfer",
        "department-transfers",
        &transfers,
        None,
        None,
    )?);

    // 7d. Federation Nodes
    let now_ts = Utc::now().timestamp();
    let fed_nodes = serde_json::json!([
        {
            "node_id": "node-aiims-delhi",
            "institution_name": "All India Institute of Medical Sciences (AIIMS Delhi)",
            "endpoint_url": "https://federation.aiims.edu/newloka",
            "public_key": [1, 2, 3, 4],
            "trust_tier": "T4_Federation_Hub",
            "status": "active",
            "last_heartbeat": now_ts,
            "supported_studies": ["STUDY-ONCO-GERO-2026", "STUDY-DIAB-CARDIAC-01"]
        },
        {
            "node_id": "node-tmh-mumbai",
            "institution_name": "Tata Memorial Centre & ACTREC Cancer Research",
            "endpoint_url": "https://mesh.actrec.gov.in/newloka",
            "public_key": [5, 6, 7, 8],
            "trust_tier": "Academic_Medical_Center",
            "status": "active",
            "last_heartbeat": now_ts - 120,
            "supported_studies": ["STUDY-ONCO-GERO-2026"]
        },
        {
            "node_id": "node-pgimer-chd",
            "institution_name": "PGIMER Department of Clinical Pharmacology",
            "endpoint_url": "https://research.pgimer.edu.in/newloka",
            "public_key": [9, 10, 11, 12],
            "trust_tier": "T3_Partner",
            "status": "active",
            "last_heartbeat": now_ts - 360,
            "supported_studies": ["STUDY-DIAB-CARDIAC-01"]
        }
    ]);
    records.push(StorageEngine::encrypt_record(
        &dmk,
        "FederationNode",
        "federation-nodes",
        &fed_nodes,
        None,
        None,
    )?);

    // 7e. Consent Directives Store
    let now = Utc::now();
    let directives = serde_json::json!([
        {
            "id": "consent-dir-001",
            "patient_id": "pat-demo-000",
            "status": "active",
            "scope": "research",
            "purpose": "RESEARCH",
            "recipient_node_id": "node-aiims-delhi",
            "recipient_name": "AIIMS Oncology & Geroscience Collaborative Grid",
            "data_categories": ["Observation", "Condition", "DiagnosticReport"],
            "valid_from": (now - chrono::Duration::days(15)).to_rfc3339(),
            "valid_to": (now + chrono::Duration::days(350)).to_rfc3339(),
            "revoked": false,
            "revocation_reason": null,
            "revoked_at": null,
            "digital_signature": "ed25519-sig-valid-9981",
            "consent_notes": "Patient provided signed digital consent for multi-center diabetes research"
        },
        {
            "id": "consent-dir-002",
            "patient_id": "pat-demo-002",
            "status": "active",
            "scope": "treatment-transfer",
            "purpose": "TREATMENT_TRANSFER",
            "recipient_node_id": "node-tmh-mumbai",
            "recipient_name": "Tata Memorial Centre",
            "data_categories": ["ALL"],
            "valid_from": (now - chrono::Duration::days(2)).to_rfc3339(),
            "valid_to": (now + chrono::Duration::days(90)).to_rfc3339(),
            "revoked": false,
            "revocation_reason": null,
            "revoked_at": null,
            "digital_signature": "ed25519-sig-valid-4412",
            "consent_notes": "Transfer consent for cardiology consultation"
        }
    ]);
    records.push(StorageEngine::encrypt_record(
        &dmk,
        "ConsentDirective",
        "consent-directives",
        &directives,
        None,
        None,
    )?);

    // 7f. Research Cohorts
    let cohorts = serde_json::json!([
        {
            "id": "cohort-001",
            "title": "T2D Inpatients with HbA1c > 8.0% on Metformin",
            "description": "Cross-sectional cohort examining glycemic control and macrovascular events in elderly diabetics",
            "inclusion_conditions": ["E11", "Type 2 Diabetes"],
            "min_age": 45,
            "max_age": 85,
            "gender": null,
            "sample_size": 18,
            "created_at": "2026-11-10T12:00:00Z"
        },
        {
            "id": "cohort-002",
            "title": "Acute NSTEMI Cardiac Registry 2026",
            "description": "Longitudinal registry of acute coronary syndrome admissions receiving dual antiplatelet therapy",
            "inclusion_conditions": ["I21.4", "NSTEMI", "Acute Coronary Syndrome"],
            "min_age": 40,
            "max_age": 90,
            "gender": null,
            "sample_size": 24,
            "created_at": "2026-11-12T14:30:00Z"
        }
    ]);
    records.push(StorageEngine::encrypt_record(
        &dmk,
        "Cohort",
        "research-cohorts",
        &cohorts,
        None,
        None,
    )?);

    // 8. 10 Demo Patients with Clinical Charts
    let male_names = [
        "Ramesh", "Suresh", "Rajesh", "Amit", "Vikram", "Arjun", "Ravi", "Kiran", "Anil", "Sunil",
    ];
    let female_names = [
        "Priya",
        "Sunita",
        "Anjali",
        "Pooja",
        "Neha",
        "Meera",
        "Lakshmi",
        "Saraswati",
        "Kavita",
        "Rekha",
    ];
    let families = [
        "Patel", "Sharma", "Gupta", "Singh", "Kumar", "Rao", "Nair", "Shah", "Mehta", "Iyer",
    ];
    let cities = [
        "Mumbai",
        "Delhi",
        "Bangalore",
        "Chennai",
        "Kolkata",
        "Hyderabad",
        "Pune",
        "Ahmedabad",
        "Jaipur",
        "Lucknow",
    ];
    let conditions_list = [
        ("Type 2 Diabetes Mellitus", "E11.9"),
        ("Essential Hypertension", "I10"),
        ("Asthma", "J45.9"),
        ("Acute Bronchitis", "J20.9"),
        ("Coronary Artery Disease", "I25.10"),
        ("Chronic Kidney Disease", "N18.9"),
        ("Osteoarthritis", "M19.90"),
        ("Major Depressive Disorder", "F32.9"),
        ("Migraine", "G43.909"),
        ("Appendicitis", "K35.80"),
    ];

    for i in 0..10 {
        let pat_id = format!("pat-demo-{:03}", i);
        let gender = if i % 2 == 0 { "male" } else { "female" };
        let given = if i % 2 == 0 {
            male_names[i % male_names.len()]
        } else {
            female_names[i % female_names.len()]
        };
        let family = families[i % families.len()];
        let birth_year = 1950 + (i * 3) % 60;
        let birth_date = format!("{}-{:02}-{:02}", birth_year, (i % 12) + 1, (i % 28) + 1);
        let city = cities[i % cities.len()];

        let patient = serde_json::json!({
            "resourceType": "Patient",
            "id": &pat_id,
            "meta": {
                "versionId": "1",
                "lastUpdated": Utc::now().to_rfc3339()
            },
            "identifier": [{ "system": "http://newloka.org/mrn", "value": format!("MRN{:06}", 100000 + i) }],
            "active": true,
            "name": [{ "family": family, "given": [given] }],
            "gender": gender,
            "birthDate": birth_date,
            "telecom": [{ "system": "phone", "value": format!("+91-{}{}", 7000000000u64 + i as u64, i) }],
            "address": [{ "city": city, "country": "India" }]
        });

        records.push(StorageEngine::encrypt_record(
            &dmk,
            "Patient",
            &pat_id,
            &patient,
            Some(pat_id.clone()),
            None,
        )?);

        // Encounter
        let enc_id = format!("enc-demo-{:03}", i);
        let encounter = serde_json::json!({
            "resourceType": "Encounter",
            "id": &enc_id,
            "meta": { "versionId": "1", "lastUpdated": Utc::now().to_rfc3339() },
            "status": "in-progress",
            "class": { "system": "http://hl7.org/fhir/v3/ActCode", "code": "AMB", "display": "Ambulatory" },
            "serviceType": { "text": "Outpatient Consultation" },
            "subject": { "reference": format!("Patient/{}", pat_id) },
            "participant": [{ "individual": { "display": "Dr. Alex Mercer" } }],
            "period": { "start": Utc::now().to_rfc3339() }
        });
        records.push(StorageEngine::encrypt_record(
            &dmk,
            "Encounter",
            &enc_id,
            &encounter,
            Some(pat_id.clone()),
            None,
        )?);

        // Condition
        let (cond_text, cond_code) = conditions_list[i % conditions_list.len()];
        let cond_id = format!("cond-demo-{:03}", i);
        let condition = serde_json::json!({
            "resourceType": "Condition",
            "id": &cond_id,
            "meta": { "versionId": "1", "lastUpdated": Utc::now().to_rfc3339() },
            "clinicalStatus": { "coding": [{ "code": "active", "display": "Active" }] },
            "verificationStatus": { "coding": [{ "code": "confirmed", "display": "Confirmed" }] },
            "code": {
                "coding": [{ "system": "http://hl7.org/fhir/sid/icd-10", "code": cond_code, "display": cond_text }],
                "text": cond_text
            },
            "subject": { "reference": format!("Patient/{}", pat_id) },
            "encounter": { "reference": format!("Encounter/{}", enc_id) },
            "recordedDate": Utc::now().to_rfc3339()
        });
        records.push(StorageEngine::encrypt_record(
            &dmk,
            "Condition",
            &cond_id,
            &condition,
            Some(pat_id.clone()),
            None,
        )?);

        // Observations (Blood Pressure & Heart Rate)
        let obs_bp_id = format!("obs-bp-{:03}", i);
        let obs_bp = serde_json::json!({
            "resourceType": "Observation",
            "id": &obs_bp_id,
            "meta": { "versionId": "1", "lastUpdated": Utc::now().to_rfc3339() },
            "status": "final",
            "category": [{ "coding": [{ "code": "vital-signs", "display": "Vital Signs" }] }],
            "code": { "coding": [{ "system": "http://loinc.org", "code": "85354-9", "display": "Blood Pressure" }] },
            "subject": { "reference": format!("Patient/{}", pat_id) },
            "effectiveDateTime": Utc::now().to_rfc3339(),
            "component": [
                {
                    "code": { "coding": [{ "code": "8480-6", "display": "Systolic" }] },
                    "valueQuantity": { "value": 120 + (i as i64 * 3) % 25, "unit": "mmHg" }
                },
                {
                    "code": { "coding": [{ "code": "8462-4", "display": "Diastolic" }] },
                    "valueQuantity": { "value": 78 + (i as i64 * 2) % 15, "unit": "mmHg" }
                }
            ]
        });
        records.push(StorageEngine::encrypt_record(
            &dmk,
            "Observation",
            &obs_bp_id,
            &obs_bp,
            Some(pat_id.clone()),
            None,
        )?);

        // Medication Request
        let med_id = format!("med-demo-{:03}", i);
        let med = serde_json::json!({
            "resourceType": "MedicationRequest",
            "id": &med_id,
            "meta": { "versionId": "1", "lastUpdated": Utc::now().to_rfc3339() },
            "status": "active",
            "intent": "order",
            "medicationCodeableConcept": { "text": "Metformin 500mg" },
            "subject": { "reference": format!("Patient/{}", pat_id) },
            "authoredOn": Utc::now().to_rfc3339(),
            "requester": { "display": "Dr. Alex Mercer" },
            "dosageInstruction": [{
                "text": "1 tablet twice daily with food",
                "timing": { "repeat": { "frequency": 2, "period": 1, "periodUnit": "d" } }
            }]
        });
        records.push(StorageEngine::encrypt_record(
            &dmk,
            "MedicationRequest",
            &med_id,
            &med,
            Some(pat_id.clone()),
            None,
        )?);

        // Immunization
        let imm_id = format!("imm-demo-{:03}", i);
        let imm = serde_json::json!({
            "resourceType": "Immunization",
            "id": &imm_id,
            "meta": { "versionId": "1", "lastUpdated": Utc::now().to_rfc3339() },
            "status": "completed",
            "vaccineCode": { "text": "COVID-19 mRNA Vaccine" },
            "patient": { "reference": format!("Patient/{}", pat_id) },
            "occurrenceDateTime": "2024-03-15",
            "lotNumber": "LOT-2024-03-A",
            "primarySource": true
        });
        records.push(StorageEngine::encrypt_record(
            &dmk,
            "Immunization",
            &imm_id,
            &imm,
            Some(pat_id.clone()),
            None,
        )?);

        // AllergyIntolerance
        let alg_id = format!("alg-demo-{:03}", i);
        let allergy = serde_json::json!({
            "resourceType": "AllergyIntolerance",
            "id": &alg_id,
            "meta": { "versionId": "1", "lastUpdated": Utc::now().to_rfc3339() },
            "clinicalStatus": { "coding": [{ "system": "http://terminology.hl7.org/CodeSystem/allergyintolerance-clinical", "code": "active" }] },
            "code": { "text": "Penicillin", "coding": [{ "system": "http://snomed.info/sct", "code": "91936005" }] },
            "patient": { "reference": format!("Patient/{}", pat_id) },
            "reaction": [{ "manifestation": [{ "text": "Rash" }] }]
        });
        records.push(StorageEngine::encrypt_record(
            &dmk,
            "AllergyIntolerance",
            &alg_id,
            &allergy,
            Some(pat_id.clone()),
            None,
        )?);

        // CarePlan
        let cp_id = format!("cp-demo-{:03}", i);
        let careplan = serde_json::json!({
            "resourceType": "CarePlan",
            "id": &cp_id,
            "meta": { "versionId": "1", "lastUpdated": Utc::now().to_rfc3339() },
            "status": "active",
            "intent": "plan",
            "subject": { "reference": format!("Patient/{}", pat_id) },
            "title": format!("Care plan for {}", given),
            "category": [{ "coding": [{ "system": "http://hl7.org/fhir/us/core/CodeSystem/careplan-category", "code": "assess-plan" }] }],
            "addresses": [{ "display": "General health management" }]
        });
        records.push(StorageEngine::encrypt_record(
            &dmk,
            "CarePlan",
            &cp_id,
            &careplan,
            Some(pat_id.clone()),
            None,
        )?);

        // Procedure
        let proc_id = format!("proc-demo-{:03}", i);
        let procedure = serde_json::json!({
            "resourceType": "Procedure",
            "id": &proc_id,
            "meta": { "versionId": "1", "lastUpdated": Utc::now().to_rfc3339() },
            "status": "completed",
            "code": { "text": "Laparoscopic Appendectomy", "coding": [{ "system": "http://www.cms.gov/Medicare/Coding/ICD10", "code": "0DBJ4ZZ" }] },
            "subject": { "reference": format!("Patient/{}", pat_id) },
            "encounter": { "reference": format!("Encounter/{}", enc_id) },
            "performedDateTime": Utc::now().to_rfc3339(),
            "performer": [{ "actor": { "display": "Dr. Alex Mercer" } }]
        });
        records.push(StorageEngine::encrypt_record(
            &dmk,
            "Procedure",
            &proc_id,
            &procedure,
            Some(pat_id.clone()),
            None,
        )?);

        // ServiceRequest
        let sr_id = format!("sr-demo-{:03}", i);
        let order_names = [
            "Basic Metabolic Panel (BMP)",
            "Complete Blood Count (CBC)",
            "Chest X-Ray PA/Lateral",
            "12-Lead ECG",
            "Lipid Profile",
        ];
        let order_name = order_names[i % order_names.len()];
        let service_request = serde_json::json!({
            "resourceType": "ServiceRequest",
            "id": &sr_id,
            "meta": { "versionId": "1", "lastUpdated": Utc::now().to_rfc3339() },
            "status": "active",
            "intent": "order",
            "category": [{ "text": if order_name.contains("X-Ray") { "radiology" } else { "laboratory" } }],
            "code": { "text": order_name },
            "subject": { "reference": format!("Patient/{}", pat_id) },
            "authoredOn": Utc::now().to_rfc3339(),
            "requester": { "display": "Dr. Alex Mercer" }
        });
        records.push(StorageEngine::encrypt_record(
            &dmk,
            "ServiceRequest",
            &sr_id,
            &service_request,
            Some(pat_id.clone()),
            None,
        )?);

        // FamilyMemberHistory
        let fmh_id = format!("fmh-demo-{:03}", i);
        let relationships = ["Father", "Mother", "Sibling", "Maternal Grandfather"];
        let conditions_fh = [
            "Coronary Artery Disease",
            "Essential Hypertension",
            "Type 2 Diabetes",
            "Asthma",
        ];
        let fmh = serde_json::json!({
            "resourceType": "FamilyMemberHistory",
            "id": &fmh_id,
            "meta": { "versionId": "1", "lastUpdated": Utc::now().to_rfc3339() },
            "status": "completed",
            "patient": { "reference": format!("Patient/{}", pat_id) },
            "relationship": { "text": relationships[i % relationships.len()] },
            "condition": [{ "code": { "text": conditions_fh[i % conditions_fh.len()] } }]
        });
        records.push(StorageEngine::encrypt_record(
            &dmk,
            "FamilyMemberHistory",
            &fmh_id,
            &fmh,
            Some(pat_id.clone()),
            None,
        )?);

        // Prescription DocumentReference for patient #0 (Ramesh Patel)
        if i == 0 {
            let rx_id = "rxpad-demo-0001";
            let rx_doc = serde_json::json!({
                "resourceType": "DocumentReference",
                "id": rx_id,
                "meta": { "versionId": "1", "lastUpdated": Utc::now().to_rfc3339() },
                "status": "current",
                "type": {
                    "coding": [{
                        "system": "urn:newloka:doc-type",
                        "code": "rxpad-prescription",
                        "display": "e-Prescription Pad Record"
                    }]
                },
                "subject": { "reference": format!("Patient/pat-demo-000") },
                "date": Utc::now().to_rfc3339(),
                "description": "Prescription #0001 for Ramesh Patel",
                "content": [{
                    "attachment": {
                        "title": "151126_0001_RameshPatel.pdf",
                        "url": "/rxpad?serial=1"
                    }
                }],
                "_rxpad": {
                    "serial": 1,
                    "date": "15/11/2026",
                    "patientName": "Ramesh Patel",
                    "ageSex": "54 / M",
                    "contact": "+1 (555) 019-2834",
                    "weight": "72 kg",
                    "bp": "130/84",
                    "diagnosis": "Type 2 Diabetes Mellitus & Essential Hypertension",
                    "doctor": "Dr. Alex Mercer",
                    "regNo": "NL-DOC-2026-001",
                    "meds": [
                        { "drug": "METFORMIN 500MG", "dose": "1 tab", "freq": "BD", "dur": "30d", "instr": "After food" },
                        { "drug": "TELMISARTAN 40MG", "dose": "1 tab", "freq": "OD", "dur": "30d", "instr": "Morning" }
                    ],
                    "labs": ["HbA1c", "Serum Creatinine", "Lipid Profile"],
                    "followUp": "After 1 month",
                    "dispenseStatus": "active",
                    "filename": "151126_0001_RameshPatel.pdf",
                    "savedAt": Utc::now().timestamp_millis()
                }
            });
            records.push(StorageEngine::encrypt_record(
                &dmk,
                "DocumentReference",
                rx_id,
                &rx_doc,
                Some(pat_id.clone()),
                None,
            )?);
        }
    }

    println!("Writing {} encrypted records to disk...", records.len());
    let mut manifest_items = Vec::new();

    for r in &records {
        let filename = format!("{}.enc", r.id);
        let filepath = fixtures_dir.join(&filename);
        let serialized = serde_json::to_string(r)?;
        fs::write(&filepath, serialized)?;

        manifest_items.push(serde_json::json!({
            "id": r.id,
            "resource_type": r.resource_type,
            "patient_id": r.patient_id,
            "file": filename
        }));
    }

    let manifest_path = Path::new("newloka_demo/fixtures/manifest.json");
    fs::write(
        manifest_path,
        serde_json::to_string_pretty(&manifest_items)?,
    )?;

    println!(
        "SUCCESS: Wrote {} encrypted records to newloka_demo/fixtures/records and manifest.json",
        records.len()
    );
    Ok(())
}
