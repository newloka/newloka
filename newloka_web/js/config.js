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
    theme: 'dark',
    pageSize: 20,
    defaultEncounterStatus: 'in-progress',
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
    T0: ['patients','encounters','observations','ingest','chart','offline_auth','local_storage','pin','conditions','medications','procedures'],
    T1: ['patients','encounters','observations','conditions','medications','procedures','ingest','chart','offline_auth','mesh','sync','audit'],
    T2: ['patients','encounters','observations','conditions','medications','procedures','ingest','chart','offline_auth','mesh','sync','audit','server'],
    T3: ['patients','encounters','observations','conditions','medications','procedures','reports','ingest','chart','offline_auth','mesh','sync','audit','server','departments','ldap','handoff'],
    T4: ['patients','encounters','observations','conditions','medications','procedures','reports','ingest','chart','offline_auth','mesh','sync','audit','server','departments','ldap','handoff','research','consent','federation'],
};
function allowed(tier, feature) {
    return FEATURES[tier]?.includes(feature) ?? false;
}
export { load, save, reset, allowed };
