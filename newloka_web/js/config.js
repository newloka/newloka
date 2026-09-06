const CLINIC_PROFILES = {
    solo: {
        name: "Single-Person Clinic (Solo Practice)",
        tier: "T0",
        description: "Tailored for an autonomous solo practitioner: appointments, outpatient encounters, SOAP notes, prescriptions, drug interaction alerts, vitals, and diagnostics without inpatient ward clutter.",
        features: [
            'dashboard', 'patients', 'patientChart', 'appointmentSchedule',
            'clinicalNotes', 'cpoeOrders', 'drugInteraction', 'resultsReview',
            'vitalsFlowsheet', 'alerts', 'carePlans', 'familyHistory',
            'encounters', 'observations', 'conditions', 'medications', 'rxpad',
            'procedures', 'immunizations', 'documents', 'ingest', 'audit', 'settings'
        ],
        hidden: ['whiteboard', 'handoffSbar', 'mar']
    },
    specialist: {
        name: "Solo Specialist Practice (Cardiology, Derm, Surgery)",
        tier: "T0",
        description: "Tailored for solo specialists with advanced procedure documentation, imaging reviews, ECG/media attachments, and CPOE protocols.",
        features: [
            'dashboard', 'patients', 'patientChart', 'appointmentSchedule',
            'clinicalNotes', 'cpoeOrders', 'drugInteraction', 'resultsReview',
            'vitalsFlowsheet', 'alerts', 'carePlans', 'familyHistory',
            'encounters', 'observations', 'conditions', 'medications', 'rxpad',
            'procedures', 'documents', 'ingest', 'audit', 'settings'
        ],
        hidden: ['whiteboard', 'handoffSbar', 'mar']
    },
    hospital: {
        name: "Inpatient Ward / Hospital (T1–T4)",
        tier: "T1",
        description: "Full clinical suite including inpatient ward census whiteboard, shift-to-shift SBAR handoffs, and 24h nursing MAR.",
        features: [
            'dashboard', 'patients', 'patientChart', 'appointmentSchedule',
            'clinicalNotes', 'cpoeOrders', 'drugInteraction', 'resultsReview',
            'mar', 'vitalsFlowsheet', 'alerts', 'carePlans', 'familyHistory',
            'immunizations', 'documents', 'encounters', 'observations',
            'conditions', 'medications', 'rxpad', 'procedures', 'handoffSbar',
            'whiteboard', 'ingest', 'audit', 'settings'
        ],
        hidden: []
    }
};

const ALL_MODULES = [
    { key: 'dashboard', label: 'Clinical Dashboard', category: 'Core' },
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
    { key: 'rxpad', label: 'Prescription Pad & Dispensing (eRx)', category: 'Pharmacy' },
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
    { key: 'ingest', label: 'AI Document Intake Pipeline', category: 'Intake' },
    { key: 'audit', label: 'Compliance & Safety Audit Trail', category: 'Compliance' },
    { key: 'settings', label: 'System Configuration', category: 'System' }
];

const DEFAULTS = {
    apiBase: 'http://127.0.0.1:8080',
    tier: 'T0',
    clinicProfile: 'solo',
    featureOverrides: {},
    nodeId: 'solo-clinic-node',
    syncEnabled: false,
    meshEnabled: false,
    department: 'Solo Practice',
    offlineAuth: 'pin',
    sessionPin: '1234',
    language: 'en',
    emergencyAccess: false,
    sessionUser: 'Dr. Clinician',
    theme: 'newloka',
    customTheme: {},
    pageSize: 20,
    defaultEncounterStatus: 'in-progress',
    activePatientId: null,
};

function load() {
    try {
        const raw = localStorage.getItem('nl_config');
        if (!raw) return { ...DEFAULTS };
        const parsed = JSON.parse(raw);
        return { ...DEFAULTS, ...parsed };
    } catch { return { ...DEFAULTS }; }
}

function save(cfg) { localStorage.setItem('nl_config', JSON.stringify(cfg)); }
function reset() { localStorage.removeItem('nl_config'); }

const FEATURES = {
    T0: [
        'dashboard','patients','encounters','observations','conditions','medications','rxpad',
        'procedures','ingest','audit','settings','patientChart','clinicalNotes',
        'cpoeOrders','resultsReview','vitalsFlowsheet','alerts','carePlans','familyHistory',
        'immunizations','documents','appointmentSchedule','drugInteraction'
    ],
    T1: [
        'dashboard','patients','encounters','observations','conditions','medications','rxpad',
        'procedures','ingest','audit','settings','patientChart','clinicalNotes',
        'cpoeOrders','resultsReview','mar','vitalsFlowsheet','alerts','carePlans',
        'familyHistory','immunizations','documents','appointmentSchedule','drugInteraction',
        'handoffSbar','whiteboard','mesh','sync'
    ],
    T2: [
        'dashboard','patients','encounters','observations','conditions','medications','rxpad',
        'procedures','ingest','audit','settings','patientChart','clinicalNotes',
        'cpoeOrders','resultsReview','mar','vitalsFlowsheet','alerts','carePlans',
        'familyHistory','immunizations','documents','appointmentSchedule','drugInteraction',
        'handoffSbar','whiteboard','mesh','sync','server'
    ],
    T3: [
        'dashboard','patients','encounters','observations','conditions','medications','rxpad',
        'procedures','ingest','audit','settings','patientChart','clinicalNotes',
        'cpoeOrders','resultsReview','mar','vitalsFlowsheet','alerts','carePlans',
        'familyHistory','immunizations','documents','appointmentSchedule','drugInteraction',
        'handoffSbar','whiteboard','mesh','sync','server','departments','reports'
    ],
    T4: [
        'dashboard','patients','encounters','observations','conditions','medications','rxpad',
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
    if (cfg?.clinicProfile && CLINIC_PROFILES[cfg.clinicProfile]) {
        return CLINIC_PROFILES[cfg.clinicProfile].features.includes(feature);
    }
    return FEATURES[tier]?.includes(feature) ?? false;
}

export { load, save, reset, allowed, CLINIC_PROFILES, ALL_MODULES, FEATURES };
