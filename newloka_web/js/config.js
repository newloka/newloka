// =============================================================================
// New Loka Modular Presets & Clinical Configuration Engine
// =============================================================================

/**
 * Exhaustive Master Preset Registry
 * Maps clinical and administrative healthcare personas to their declarative
 * viewport archetype, active modules, ABAC profile, and backend data pipelines.
 */
const PRESET_REGISTRY = {
    // --- Physicians & Clinical Specialists ---
    doc_attending: {
        id: "doc_attending",
        name: "Attending Physician / Consultant",
        category: "Physicians & Specialists",
        tier: "T0",
        role: "doctor",
        description: "Comprehensive ambulatory & inpatient clinical suite with native prescription pad, formulary autocomplete, and clinical CPOE.",
        defaultLandingView: "rxpad",
        enabledModules: [
            'dashboard', 'rxpad', 'pharmacopoeia', 'patientChart', 'clinicalNotes', 'cpoeOrders',
            'inpatientRegister', 'resultsReview', 'vitalsFlowsheet', 'documents',
            'appointmentSchedule', 'patients', 'conditions', 'medications', 'procedures', 'alerts'
        ],
        abacRules: { canPrescribe: true, clinicalNotesAccess: "full", billingAccess: "summary", redactDiagnoses: false },
        pipelines: ['/api/prescriber/profile', '/api/rxpad/formulary', '/api/rxpad/prescriptions', '/api/inpatient/census'],
        quickAction: { label: "📝 New Rx", hash: "rxpad" }
    },
    doc_senior_res: {
        id: "doc_senior_res",
        name: "Senior Resident / Inpatient Ward",
        category: "Physicians & Specialists",
        tier: "T1",
        role: "resident",
        description: "Inpatient ward census management, bed allocation, NEWS2 clinical acuity stratification, resident round notes, and SBAR handoffs.",
        defaultLandingView: "inpatientRegister",
        enabledModules: [
            'inpatientRegister', 'patientChart', 'clinicalNotes', 'cpoeOrders',
            'rxpad', 'pharmacopoeia', 'vitalsFlowsheet', 'resultsReview', 'handoffSbar', 'whiteboard',
            'mar', 'alerts', 'patients', 'conditions', 'procedures'
        ],
        abacRules: { canPrescribe: true, clinicalNotesAccess: "full", billingAccess: "none", redactDiagnoses: false },
        pipelines: ['/api/inpatient/census', '/api/rxpad/prescriptions', '/api/prescriber/profile'],
        quickAction: { label: "🛏️ Ward Census", hash: "inpatientRegister" }
    },
    doc_junior_res: {
        id: "doc_junior_res",
        name: "Junior Resident (PGY-1/2)",
        category: "Physicians & Specialists",
        tier: "T1",
        role: "resident",
        description: "Acute admissions, routine ward care, SOAP clinical progress notes, diagnostic requisitions, and discharge summaries.",
        defaultLandingView: "inpatientRegister",
        enabledModules: [
            'inpatientRegister', 'patientChart', 'clinicalNotes', 'cpoeOrders',
            'rxpad', 'pharmacopoeia', 'vitalsFlowsheet', 'resultsReview', 'mar', 'alerts', 'patients'
        ],
        abacRules: { canPrescribe: true, clinicalNotesAccess: "full", billingAccess: "none", redactDiagnoses: false },
        pipelines: ['/api/inpatient/census', '/api/rxpad/prescriptions'],
        quickAction: { label: "📝 Round Notes", hash: "inpatientRegister" }
    },
    doc_intern: {
        id: "doc_intern",
        name: "Medical Intern / House Surgeon",
        category: "Physicians & Specialists",
        tier: "T1",
        role: "resident",
        description: "Supervised ward orders, vitals tracking, draft progress notes, and routine lab requisitions with supervising doctor tagged.",
        defaultLandingView: "inpatientRegister",
        enabledModules: [
            'inpatientRegister', 'patientChart', 'clinicalNotes', 'pharmacopoeia', 'vitalsFlowsheet',
            'resultsReview', 'patients', 'alerts'
        ],
        abacRules: { canPrescribe: false, clinicalNotesAccess: "draft", billingAccess: "none", redactDiagnoses: false },
        pipelines: ['/api/inpatient/census', '/fhir/Observation'],
        quickAction: { label: "👥 Patients", hash: "patients" }
    },
    doc_solo_gp: {
        id: "doc_solo_gp",
        name: "Solo General Practitioner (T0)",
        category: "Physicians & Specialists",
        tier: "T0",
        role: "doctor",
        description: "Autonomous single-practitioner workflow: outpatient appointments, native prescription pad with printed branding, and instant cashier billing.",
        defaultLandingView: "rxpad",
        enabledModules: [
            'rxpad', 'pharmacopoeia', 'patients', 'billing', 'appointmentSchedule', 'clinicalNotes',
            'cpoeOrders', 'drugInteraction', 'patientChart', 'documents', 'settings'
        ],
        abacRules: { canPrescribe: true, clinicalNotesAccess: "full", billingAccess: "full", redactDiagnoses: false },
        pipelines: ['/api/prescriber/profile', '/api/rxpad/formulary', '/api/billing/invoices'],
        quickAction: { label: "📝 Prescription Pad", hash: "rxpad" }
    },
    doc_hod: {
        id: "doc_hod",
        name: "Department Head / Service Chief",
        category: "Physicians & Specialists",
        tier: "T2",
        role: "doctor",
        description: "Department command center with live activity metrics, critical stockout alerts, inpatient census overview, and audit explorer.",
        defaultLandingView: "deptSummary",
        enabledModules: [
            'deptSummary', 'inpatientRegister', 'rxpad', 'pharmacopoeia', 'patientChart', 'clinicalNotes',
            'dashboard', 'audit', 'appointmentSchedule', 'settings'
        ],
        abacRules: { canPrescribe: true, clinicalNotesAccess: "full", billingAccess: "summary", redactDiagnoses: false },
        pipelines: ['/api/pharmacy/summary', '/api/inpatient/census', '/api/config'],
        quickAction: { label: "📊 Dept Summary", hash: "deptSummary" }
    },
    doc_surgeon: {
        id: "doc_surgeon",
        name: "Surgical Specialist / Proceduralist",
        category: "Physicians & Specialists",
        tier: "T1",
        role: "doctor",
        description: "Surgical procedure notes, pre-op safety checklists, intra-operative documentation, implant logs, and post-op order sets.",
        defaultLandingView: "procedures",
        enabledModules: [
            'procedures', 'patientChart', 'clinicalNotes', 'cpoeOrders', 'rxpad', 'pharmacopoeia',
            'inpatientRegister', 'resultsReview', 'documents', 'alerts'
        ],
        abacRules: { canPrescribe: true, clinicalNotesAccess: "full", billingAccess: "none", redactDiagnoses: false },
        pipelines: ['/fhir/Procedure', '/api/rxpad/prescriptions'],
        quickAction: { label: "🔧 Procedures", hash: "procedures" }
    },
    doc_emergency: {
        id: "doc_emergency",
        name: "Emergency Physician / Resuscitation",
        category: "Physicians & Specialists",
        tier: "T1",
        role: "doctor",
        description: "Rapid acute care track board, emergency resuscitation orders, high-acuity deterioration flags, and stat diagnostic review.",
        defaultLandingView: "dashboard",
        enabledModules: [
            'dashboard', 'inpatientRegister', 'patientChart', 'cpoeOrders', 'rxpad', 'pharmacopoeia',
            'vitalsFlowsheet', 'resultsReview', 'alerts'
        ],
        abacRules: { canPrescribe: true, clinicalNotesAccess: "full", emergencyOverride: true, redactDiagnoses: false },
        pipelines: ['/api/inpatient/census', '/fhir/Observation'],
        quickAction: { label: "🚨 Emergency Board", hash: "dashboard" }
    },

    // --- Nursing Staff ---
    nurse_ward: {
        id: "nurse_ward",
        name: "Ward Staff Nurse (Bedside)",
        category: "Nursing Staff",
        tier: "T1",
        role: "nurse",
        description: "Inpatient bedside nursing: 24h Medication Administration Record (MAR), vital signs capture with NEWS2 scoring, and shift handoffs.",
        defaultLandingView: "mar",
        enabledModules: [
            'mar', 'vitalsFlowsheet', 'inpatientRegister', 'patientChart', 'handoffSbar',
            'alerts', 'patients'
        ],
        abacRules: { canPrescribe: false, clinicalNotesAccess: "bedside", billingAccess: "none", redactDiagnoses: false },
        pipelines: ['/fhir/MedicationRequest', '/api/inpatient/census'],
        quickAction: { label: "💊 Nursing MAR", hash: "mar" }
    },
    nurse_icu: {
        id: "nurse_icu",
        name: "Intensive Care Nurse (ICU / CCU)",
        category: "Nursing Staff",
        tier: "T2",
        role: "nurse",
        description: "High-density hemodynamic flowsheet, continuous IV infusion titration tracking, ventilator settings, and critical panic alerts.",
        defaultLandingView: "vitalsFlowsheet",
        enabledModules: [
            'vitalsFlowsheet', 'mar', 'patientChart', 'resultsReview', 'alerts',
            'handoffSbar', 'inpatientRegister'
        ],
        abacRules: { canPrescribe: false, clinicalNotesAccess: "icu", billingAccess: "none", redactDiagnoses: false },
        pipelines: ['/fhir/Observation', '/api/inpatient/census'],
        quickAction: { label: "📈 ICU Flowsheet", hash: "vitalsFlowsheet" }
    },
    nurse_ot: {
        id: "nurse_ot",
        name: "Operating Theatre Nurse (Perioperative)",
        category: "Nursing Staff",
        tier: "T1",
        role: "nurse",
        description: "WHO Surgical Safety Checklist, surgical instrument/sponge count verification, specimen chain-of-custody, and recovery notes.",
        defaultLandingView: "procedures",
        enabledModules: [
            'procedures', 'patientChart', 'documents', 'alerts', 'inpatientRegister'
        ],
        abacRules: { canPrescribe: false, clinicalNotesAccess: "surgical", billingAccess: "none", redactDiagnoses: false },
        pipelines: ['/fhir/Procedure'],
        quickAction: { label: "🔧 OT Checklist", hash: "procedures" }
    },
    nurse_charge: {
        id: "nurse_charge",
        name: "Charge Nurse / Ward Sister",
        category: "Nursing Staff",
        tier: "T1",
        role: "nurse",
        description: "Ward-wide bed occupancy matrix, nurse-patient staffing allocation, unadministered dose oversight, and structured SBAR handoff.",
        defaultLandingView: "inpatientRegister",
        enabledModules: [
            'inpatientRegister', 'whiteboard', 'handoffSbar', 'mar', 'vitalsFlowsheet',
            'alerts', 'patients'
        ],
        abacRules: { canPrescribe: false, clinicalNotesAccess: "supervisory", billingAccess: "none", redactDiagnoses: false },
        pipelines: ['/api/inpatient/census', '/fhir/Communication'],
        quickAction: { label: "📋 Ward Board", hash: "inpatientRegister" }
    },

    // --- Pharmacy & Diagnostics ---
    pharm_head: {
        id: "pharm_head",
        name: "Pharmacy Head / Chief Pharmacist",
        category: "Pharmacy & Diagnostics",
        tier: "T1",
        role: "pharmacy",
        description: "Department command center: flashing alert bar for critical stockouts, dispensing volume telemetry, and formulary curation.",
        defaultLandingView: "deptSummary",
        enabledModules: [
            'deptSummary', 'rxpad', 'pharmacopoeia', 'medications', 'drugInteraction', 'patients',
            'audit', 'settings'
        ],
        abacRules: { canPrescribe: false, clinicalNotesAccess: "none", billingAccess: "pharmacy_only", redactDiagnoses: true },
        pipelines: ['/api/pharmacy/summary', '/api/rxpad/prescriptions', '/api/rxpad/formulary'],
        quickAction: { label: "🚨 Stockout Alerts", hash: "deptSummary" }
    },
    pharm_disp: {
        id: "pharm_disp",
        name: "Clinical Pharmacist (Dispensary)",
        category: "Pharmacy & Diagnostics",
        tier: "T0",
        role: "pharmacy",
        description: "Outpatient and ward prescription fill queue, drug interaction safety screening, batch/lot tracking, and dispense sign-off.",
        defaultLandingView: "rxpad",
        enabledModules: [
            'rxpad', 'pharmacopoeia', 'deptSummary', 'drugInteraction', 'patientChart', 'patients'
        ],
        abacRules: { canPrescribe: false, clinicalNotesAccess: "none", billingAccess: "none", redactDiagnoses: false },
        pipelines: ['/api/rxpad/prescriptions', '/api/rxpad/formulary'],
        quickAction: { label: "💊 Dispense Queue", hash: "rxpad" }
    },
    lab_tech: {
        id: "lab_tech",
        name: "Medical Laboratory Technologist",
        category: "Pharmacy & Diagnostics",
        tier: "T1",
        role: "allied",
        description: "Diagnostic specimen accessioning, laboratory test worklists, numerical parameter entry, and panic value escalations.",
        defaultLandingView: "resultsReview",
        enabledModules: [
            'resultsReview', 'cpoeOrders', 'alerts', 'patients'
        ],
        abacRules: { canPrescribe: false, clinicalNotesAccess: "none", billingAccess: "none", redactDiagnoses: true },
        pipelines: ['/fhir/Observation', '/fhir/DiagnosticReport'],
        quickAction: { label: "🧪 Lab Results", hash: "resultsReview" }
    },
    lab_path: {
        id: "lab_path",
        name: "Consultant Pathologist",
        category: "Pharmacy & Diagnostics",
        tier: "T2",
        role: "doctor",
        description: "Histopathology biopsy reviews, synoptic diagnostic reports, abnormal smear verification, and digital sign-off.",
        defaultLandingView: "resultsReview",
        enabledModules: [
            'resultsReview', 'patientChart', 'documents', 'cpoeOrders'
        ],
        abacRules: { canPrescribe: false, clinicalNotesAccess: "pathology", billingAccess: "none", redactDiagnoses: false },
        pipelines: ['/fhir/DiagnosticReport'],
        quickAction: { label: "🔬 Pathology Sign-off", hash: "resultsReview" }
    },
    rad_cons: {
        id: "rad_cons",
        name: "Consultant Radiologist",
        category: "Pharmacy & Diagnostics",
        tier: "T2",
        role: "doctor",
        description: "Diagnostic imaging worklists (CR, CT, MR, US), PACS deep-linking, structured radiological reporting, and urgent alerts.",
        defaultLandingView: "resultsReview",
        enabledModules: [
            'resultsReview', 'documents', 'patientChart', 'cpoeOrders'
        ],
        abacRules: { canPrescribe: false, clinicalNotesAccess: "radiology", billingAccess: "none", redactDiagnoses: false },
        pipelines: ['/fhir/DiagnosticReport', '/fhir/ImagingStudy'],
        quickAction: { label: "🩻 Imaging Worklist", hash: "resultsReview" }
    },

    // --- Administration & Revenue Cycle ---
    admin_front: {
        id: "admin_front",
        name: "Front Desk & Patient Intake",
        category: "Administration & Operations",
        tier: "T0",
        role: "billing",
        description: "Patient registration, appointment scheduling, queue tokens, demographic checks, and outpatient check-in/out.",
        defaultLandingView: "appointmentSchedule",
        enabledModules: [
            'appointmentSchedule', 'patients', 'billing'
        ],
        abacRules: { canPrescribe: false, clinicalNotesAccess: "none", billingAccess: "reception", redactDiagnoses: true },
        pipelines: ['/fhir/Appointment', '/fhir/Patient'],
        quickAction: { label: "📅 Appointments", hash: "appointmentSchedule" }
    },
    admin_cashier: {
        id: "admin_cashier",
        name: "Cashier & Billing Specialist",
        category: "Administration & Operations",
        tier: "T0",
        role: "billing",
        description: "Invoice generation, cashier payments (Cash/UPI/Card), itemized receipts, and strict ABAC redaction of clinical charts.",
        defaultLandingView: "billing",
        enabledModules: [
            'billing', 'patients', 'appointmentSchedule'
        ],
        abacRules: { canPrescribe: false, clinicalNotesAccess: "none", billingAccess: "full", redactDiagnoses: true },
        pipelines: ['/api/billing/invoices'],
        quickAction: { label: "💳 Billing Ledger", hash: "billing" }
    },
    exec_admin: {
        id: "exec_admin",
        name: "Hospital Administrator / Executive",
        category: "Administration & Operations",
        tier: "T2",
        role: "admin",
        description: "Executive governance: hospital bed occupancy, revenue aging, clinical quality indicators, and cryptographic audit trail.",
        defaultLandingView: "dashboard",
        enabledModules: [
            'dashboard', 'inpatientRegister', 'deptSummary', 'billing', 'audit',
            'patients', 'settings'
        ],
        abacRules: { canPrescribe: false, clinicalNotesAccess: "audit", billingAccess: "full", redactDiagnoses: false },
        pipelines: ['/api/inpatient/census', '/api/billing/invoices', '/api/pharmacy/summary', '/fhir/AuditEvent'],
        quickAction: { label: "🏛️ Hospital Dashboard", hash: "dashboard" }
    },
    tech_sysadmin: {
        id: "tech_sysadmin",
        name: "IT Systems Administrator",
        category: "Administration & Operations",
        tier: "T0",
        role: "admin",
        description: "Modular feature toggle manager, 1-click & air-gapped system updater, P2P doctor mesh discovery, and cryptographic keys.",
        defaultLandingView: "settings",
        enabledModules: [
            'settings', 'audit', 'dashboard'
        ],
        abacRules: { canPrescribe: false, clinicalNotesAccess: "none", billingAccess: "none", redactDiagnoses: true },
        pipelines: ['/api/config', '/api/system/version'],
        quickAction: { label: "⚙️ System Settings", hash: "settings" }
    }
};

/**
 * Backward-compatible profile aliases mapping legacy profile keys to new presets
 */
const CLINIC_PROFILES = {
    solo: PRESET_REGISTRY.doc_solo_gp,
    specialist: PRESET_REGISTRY.doc_surgeon,
    hospital: PRESET_REGISTRY.doc_attending,
    resident: PRESET_REGISTRY.doc_senior_res,
    billing_clerk: PRESET_REGISTRY.admin_cashier,
    pharmacy_head: PRESET_REGISTRY.pharm_head
};

const ALL_MODULES = [
    { key: 'dashboard', label: 'Clinical Dashboard', category: 'Core' },
    { key: 'rxpad', label: 'Prescription Pad & Dispensing (eRx)', category: 'Pharmacy' },
    { key: 'pharmacopoeia', label: 'Indian Pharmacopoeia & Order Sets', category: 'Pharmacy' },
    { key: 'inpatientRegister', label: 'Inpatient Register & Ward Census', category: 'Inpatient Ward' },
    { key: 'billing', label: 'Billing, Invoicing & Cashier Desk', category: 'Operations & Finance' },
    { key: 'deptSummary', label: 'Department Activity & Alert Center', category: 'Operations' },
    { key: 'patients', label: 'Patient Directory & Intake', category: 'Core' },
    { key: 'patientChart', label: 'Comprehensive Patient Chart', category: 'Core' },
    { key: 'appointmentSchedule', label: 'Appointment Scheduling', category: 'Outpatient' },
    { key: 'clinicalNotes', label: 'Clinical SOAP Notes & Phrases', category: 'Clinical Docs' },
    { key: 'cpoeOrders', label: 'CPOE Orders & Order Sets', category: 'Clinical Docs' },
    { key: 'drugInteraction', label: 'Drug Interaction Safety Alerts', category: 'Safety' },
    { key: 'resultsReview', label: 'Diagnostic Results & Panic Labs', category: 'Diagnostics' },
    { key: 'vitalsFlowsheet', label: 'Vitals Flowsheet & NEWS2', category: 'Diagnostics' },
    { key: 'alerts', label: 'Allergy & Risk Profiles', category: 'Safety' },
    { key: 'medications', label: 'Prescriptions & Medication Records', category: 'Pharmacy' },
    { key: 'immunizations', label: 'Vaccines & Immunization Registry', category: 'Preventive' },
    { key: 'documents', label: 'Clinical Media & Document Attachments', category: 'Media' },
    { key: 'encounters', label: 'Outpatient Encounters & Visits', category: 'Operations' },
    { key: 'observations', label: 'Observations & Measurements', category: 'Diagnostics' },
    { key: 'conditions', label: 'Problem List & Diagnoses', category: 'Core' },
    { key: 'procedures', label: 'Procedures & Minor Surgeries', category: 'Clinical Docs' },
    { key: 'familyHistory', label: 'Family Member History', category: 'Preventive' },
    { key: 'carePlans', label: 'Care Plans & Interventions', category: 'Inpatient / Chronic' },
    { key: 'mar', label: 'Inpatient Hourly Nursing MAR', category: 'Inpatient Ward' },
    { key: 'handoffSbar', label: 'Shift-to-Shift SBAR Handoff', category: 'Inpatient Ward' },
    { key: 'whiteboard', label: 'Ward Census & Bed Board', category: 'Inpatient Ward' },
    { key: 'departments', label: 'Departments & Inpatient Transfers', category: 'Inpatient Ward' },
    { key: 'reports', label: 'Hospital Reports & Quality Governance', category: 'Operations & Finance' },
    { key: 'research', label: 'Research Cohorts & Differential Privacy', category: 'Research' },
    { key: 'consent', label: 'Patient Consent Directives & Exchange', category: 'Research' },
    { key: 'federation', label: 'Research Federation Mesh Network', category: 'System' },
    { key: 'ingest', label: 'AI Document Intake Pipeline', category: 'Intake' },
    { key: 'audit', label: 'Compliance & Safety Audit Trail', category: 'Compliance' },
    { key: 'settings', label: 'System Configuration', category: 'System' }
];

const DEFAULTS = {
    apiBase: 'http://127.0.0.1:8080',
    tier: 'T0',
    activePreset: 'doc_attending',
    clinicProfile: 'hospital',
    userRole: 'doctor',
    featureOverrides: {},
    nodeId: 'solo-clinic-node',
    syncEnabled: false,
    meshEnabled: false,
    department: '',
    offlineAuth: 'pin',
    sessionPin: '1234',
    language: 'en',
    emergencyAccess: false,
    sessionUser: '',
    sessionUserRole: 'doctor',
    theme: 'newloka',
    customTheme: {},
    pageSize: 20,
    defaultEncounterStatus: 'in-progress',
    activePatientId: null,
    defaultLandingView: 'rxpad',
    roleLandingViews: {
        doctor: 'rxpad',
        clinician: 'rxpad',
        resident: 'inpatientRegister',
        resident_doctor: 'inpatientRegister',
        nurse: 'mar',
        billing: 'billing',
        cashier: 'billing',
        reception: 'appointmentSchedule',
        pharmacy: 'deptSummary',
        pharmacist: 'deptSummary',
        department_head: 'deptSummary',
        admin: 'dashboard',
        administrator: 'dashboard',
        allied: 'resultsReview'
    },
    userLandingView: '',
    prescriber: {
        clinicianName: '',
        qualifications: '',
        regNo: '',
        specialty: '',
        isLegalPrescriber: true,
        signatureBase64: ''
    },
    clinic: {
        name: '',
        subtitle: '',
        regNo: '',
        address: '',
        phone: '',
        email: '',
        logoBase64: ''
    }
};

function load() {
    try {
        const raw = localStorage.getItem('nl_config');
        if (!raw) return { ...DEFAULTS };
        const parsed = JSON.parse(raw);
        return {
            ...DEFAULTS,
            ...parsed,
            roleLandingViews: { ...DEFAULTS.roleLandingViews, ...(parsed.roleLandingViews || {}) },
            prescriber: { ...DEFAULTS.prescriber, ...(parsed.prescriber || {}) },
            clinic: { ...DEFAULTS.clinic, ...(parsed.clinic || {}) }
        };
    } catch { return { ...DEFAULTS }; }
}

function save(cfg) { localStorage.setItem('nl_config', JSON.stringify(cfg)); }
function reset() { localStorage.removeItem('nl_config'); }

const FEATURES = {
    T0: [
        'dashboard','rxpad','pharmacopoeia','billing','patients','encounters','observations','conditions','medications',
        'procedures','ingest','audit','settings','patientChart','clinicalNotes',
        'cpoeOrders','resultsReview','vitalsFlowsheet','alerts','carePlans','familyHistory',
        'immunizations','documents','appointmentSchedule','drugInteraction'
    ],
    T1: [
        'dashboard','rxpad','pharmacopoeia','inpatientRegister','billing','deptSummary',
        'patients','encounters','observations','conditions','medications',
        'procedures','ingest','audit','settings','patientChart','clinicalNotes',
        'cpoeOrders','resultsReview','mar','vitalsFlowsheet','alerts','carePlans',
        'familyHistory','immunizations','documents','appointmentSchedule','drugInteraction',
        'handoffSbar','whiteboard','mesh','sync'
    ],
    T2: [
        'dashboard','rxpad','pharmacopoeia','inpatientRegister','billing','deptSummary',
        'patients','encounters','observations','conditions','medications',
        'procedures','ingest','audit','settings','patientChart','clinicalNotes',
        'cpoeOrders','resultsReview','mar','vitalsFlowsheet','alerts','carePlans',
        'familyHistory','immunizations','documents','appointmentSchedule','drugInteraction',
        'handoffSbar','whiteboard','mesh','sync','server'
    ],
    T3: [
        'dashboard','rxpad','pharmacopoeia','inpatientRegister','billing','deptSummary',
        'patients','encounters','observations','conditions','medications',
        'procedures','ingest','audit','settings','patientChart','clinicalNotes',
        'cpoeOrders','resultsReview','mar','vitalsFlowsheet','alerts','carePlans',
        'familyHistory','immunizations','documents','appointmentSchedule','drugInteraction',
        'handoffSbar','whiteboard','mesh','sync','server','departments','reports'
    ],
    T4: [
        'dashboard','rxpad','pharmacopoeia','inpatientRegister','billing','deptSummary',
        'patients','encounters','observations','conditions','medications',
        'procedures','ingest','audit','settings','patientChart','clinicalNotes',
        'cpoeOrders','resultsReview','mar','vitalsFlowsheet','alerts','carePlans',
        'familyHistory','immunizations','documents','appointmentSchedule','drugInteraction',
        'handoffSbar','whiteboard','mesh','sync','server','departments','reports',
        'research','consent','federation'
    ],
};

function allowed(tier, feature, cfg = null) {
    if (cfg?.featureOverrides && cfg.featureOverrides[feature] !== undefined) {
        return !!cfg.featureOverrides[feature];
    }
    if (cfg?.activePreset && PRESET_REGISTRY[cfg.activePreset]) {
        return PRESET_REGISTRY[cfg.activePreset].enabledModules.includes(feature);
    }
    if (cfg?.clinicProfile && CLINIC_PROFILES[cfg.clinicProfile]) {
        return CLINIC_PROFILES[cfg.clinicProfile].enabledModules ? 
            CLINIC_PROFILES[cfg.clinicProfile].enabledModules.includes(feature) : 
            CLINIC_PROFILES[cfg.clinicProfile].features.includes(feature);
    }
    return FEATURES[tier]?.includes(feature) ?? false;
}

function getPreset(presetId) {
    return PRESET_REGISTRY[presetId] || CLINIC_PROFILES[presetId] || PRESET_REGISTRY.doc_attending;
}

function getPresetsByCategory() {
    const categories = {};
    for (const key of Object.keys(PRESET_REGISTRY)) {
        const p = PRESET_REGISTRY[key];
        if (!categories[p.category]) categories[p.category] = [];
        categories[p.category].push(p);
    }
    return categories;
}

export { 
    load, 
    save, 
    reset, 
    allowed, 
    PRESET_REGISTRY, 
    CLINIC_PROFILES, 
    ALL_MODULES, 
    FEATURES, 
    getPreset, 
    getPresetsByCategory 
};
