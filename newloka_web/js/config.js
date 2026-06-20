const DEFAULTS = {
    apiBase: 'http://127.0.0.1:8080',
    tier: 'T1',
    nodeId: 'browser-node',
    syncEnabled: true,
    meshEnabled: false,
    department: 'default',
    offlineAuth: 'pin',
    language: 'en',
    emergencyAccess: false,
    sessionUser: 'clinician',
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
    T0: ['dashboard','patients','encounters','observations','conditions','medications','procedures','ingest','audit','settings','patientChart','clinicalNotes','cpoeOrders','resultsReview','mar','vitalsFlowsheet','alerts','carePlans','familyHistory','immunizations','documents'],
    T1: ['dashboard','patients','encounters','observations','conditions','medications','procedures','ingest','audit','settings','patientChart','clinicalNotes','cpoeOrders','resultsReview','mar','vitalsFlowsheet','alerts','carePlans','familyHistory','immunizations','documents','handoffSbar','whiteboard','mesh','sync'],
    T2: ['dashboard','patients','encounters','observations','conditions','medications','procedures','ingest','audit','settings','patientChart','clinicalNotes','cpoeOrders','resultsReview','mar','vitalsFlowsheet','alerts','carePlans','familyHistory','immunizations','documents','handoffSbar','whiteboard','mesh','sync','server'],
    T3: ['dashboard','patients','encounters','observations','conditions','medications','procedures','ingest','audit','settings','patientChart','clinicalNotes','cpoeOrders','resultsReview','mar','vitalsFlowsheet','alerts','carePlans','familyHistory','immunizations','documents','handoffSbar','whiteboard','mesh','sync','server','departments','reports'],
    T4: ['dashboard','patients','encounters','observations','conditions','medications','procedures','ingest','audit','settings','patientChart','clinicalNotes','cpoeOrders','resultsReview','mar','vitalsFlowsheet','alerts','carePlans','familyHistory','immunizations','documents','handoffSbar','whiteboard','mesh','sync','server','departments','reports','research','consent','federation'],
};
function allowed(tier, feature) {
    return FEATURES[tier]?.includes(feature) ?? false;
}
export { load, save, reset, allowed };
