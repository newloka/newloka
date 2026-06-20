/**
 * New Loka API Client + Offline Cache
 *
 * Wraps the FHIR R4 endpoints with local IndexedDB caching and an
 * offline mutation queue.  All network calls include a configurable
 * timeout and a small exponential-backoff retry layer.
 */

let _cfg = {};
let _db = null;

export function init(config) { _cfg = config; }

function url(path) {
    const base = (_cfg.apiBase || 'http://127.0.0.1:8080').replace(/\/$/, '');
    return `${base}${path}`;
}

function headers() {
    const h = { 'Content-Type': 'application/fhir+json' };
    if (_cfg.token) h['Authorization'] = `Bearer ${_cfg.token}`;
    return h;
}

/**
 * Fetch with AbortController timeout.
 */
async function fetchWithTimeout(resource, opts = {}, timeoutMs = 15000) {
    const controller = new AbortController();
    const id = setTimeout(() => controller.abort(), timeoutMs);
    try {
        const res = await fetch(resource, { ...opts, signal: controller.signal });
        clearTimeout(id);
        return res;
    } catch (e) {
        clearTimeout(id);
        throw e;
    }
}

/**
 * Retry wrapper with exponential backoff (3 attempts by default).
 */
async function fetchWithRetry(resource, opts = {}, retries = 3, timeoutMs = 15000) {
    let lastErr;
    for (let attempt = 0; attempt < retries; attempt++) {
        try {
            return await fetchWithTimeout(resource, opts, timeoutMs);
        } catch (e) {
            lastErr = e;
            if (attempt < retries - 1) {
                const delay = Math.min(1000 * (2 ** attempt), 5000);
                await new Promise(r => setTimeout(r, delay));
            }
        }
    }
    throw lastErr;
}

/* ------------------------------------------------------------------ */
/* IndexedDB helpers                                                  */
/* ------------------------------------------------------------------ */

function openDB() {
    return new Promise((resolve, reject) => {
        if (_db) return resolve(_db);
        const req = indexedDB.open('newloka_store', 4);
        req.onerror = () => reject(req.error);
        req.onsuccess = () => { _db = req.result; resolve(_db); };
        req.onupgradeneeded = (e) => {
            const db = e.target.result;
            const stores = ['patients','encounters','observations','conditions','medicationRequests','procedures','queue','audit','allergyIntolerances','documentReferences','serviceRequests','carePlans','familyMemberHistories','immunizations','medicationAdministrations','medicationStatements'];
            stores.forEach(s => {
                if (!db.objectStoreNames.contains(s)) db.createObjectStore(s, { keyPath: 'id' });
            });
        };
    });
}

async function dbPut(store, obj) {
    const db = await openDB();
    return new Promise((resolve, reject) => {
        const tx = db.transaction([store], 'readwrite');
        const st = tx.objectStore(store);
        const req = st.put(obj);
        req.onsuccess = () => resolve(req.result);
        req.onerror = () => reject(req.error);
    });
}

async function dbGetAll(store) {
    const db = await openDB();
    return new Promise((resolve, reject) => {
        const tx = db.transaction([store], 'readonly');
        const st = tx.objectStore(store);
        const req = st.getAll();
        req.onsuccess = () => resolve(req.result || []);
        req.onerror = () => reject(req.error);
    });
}

async function dbGet(store, id) {
    if (!id) return null;
    const db = await openDB();
    return new Promise((resolve, reject) => {
        const tx = db.transaction([store], 'readonly');
        const st = tx.objectStore(store);
        const req = st.get(id);
        req.onsuccess = () => resolve(req.result);
        req.onerror = () => reject(req.error);
    });
}

async function dbDelete(store, id) {
    const db = await openDB();
    return new Promise((resolve, reject) => {
        const tx = db.transaction([store], 'readwrite');
        const st = tx.objectStore(store);
        const req = st.delete(id);
        req.onsuccess = () => resolve();
        req.onerror = () => reject(req.error);
    });
}

async function dbClear(store) {
    const db = await openDB();
    return new Promise((resolve, reject) => {
        const tx = db.transaction([store], 'readwrite');
        const st = tx.objectStore(store);
        const req = st.clear();
        req.onsuccess = () => resolve();
        req.onerror = () => reject(req.error);
    });
}

/* ------------------------------------------------------------------ */
/* Offline queue                                                      */
/* ------------------------------------------------------------------ */

function isOnline() { return navigator.onLine; }

async function enqueue(method, resourceType, payload, id) {
    const item = {
        method, resourceType, payload,
        id: id || (crypto.randomUUID?.() || `${Date.now()}`),
        ts: Date.now()
    };
    await dbPut('queue', item);
    return item.id;
}

async function flushQueue() {
    const items = await dbGetAll('queue');
    for (const item of items) {
        try {
            if (item.method === 'POST') {
                await fetchWithRetry(url(`/${item.resourceType}`), { method: 'POST', headers: headers(), body: JSON.stringify(item.payload) });
            } else if (item.method === 'PUT') {
                await fetchWithRetry(url(`/${item.resourceType}/${item.id}`), { method: 'PUT', headers: headers(), body: JSON.stringify(item.payload) });
            } else if (item.method === 'DELETE') {
                await fetchWithRetry(url(`/${item.resourceType}/${item.id}`), { method: 'DELETE', headers: headers() });
            }
            await dbDelete('queue', item.id);
        } catch (err) {
            console.warn('Flush queue item failed, will retry later:', err);
        }
    }
}

/* ------------------------------------------------------------------ */
/* Generic resource API factory                                       */
/* ------------------------------------------------------------------ */

/**
 * Create a set of CRUD functions for a single FHIR resource type.
 *
 * @param {string} resourceType   FHIR resource type name (e.g. "Patient")
 * @param {string} storeName      IndexedDB object-store name (e.g. "patients")
 */
function makeResourceApi(resourceType, storeName) {
    const lc = resourceType.toLowerCase();

    async function search(patientId = '') {
        if (!isOnline()) {
            const all = await dbGetAll(storeName);
            const filtered = patientId ? all.filter(o => o.subject?.reference?.includes(patientId)) : all;
            return { resourceType: 'Bundle', type: 'searchset', total: filtered.length, entry: filtered.map(r => ({ resource: r })) };
        }
        const q = patientId ? `?patient=${encodeURIComponent(patientId)}&_count=50` : '?_count=50';
        const res = await fetchWithRetry(url(`/${resourceType}${q}`), { headers: headers() });
        const data = await res.json();
        if (data.entry) {
            for (const e of data.entry) {
                if (e.resource) await dbPut(storeName, e.resource);
            }
        }
        return data;
    }

    async function create(payload) {
        if (!isOnline()) {
            await enqueue('POST', resourceType, payload);
            return { id: payload.id, queued: true };
        }
        const res = await fetchWithRetry(url(`/${resourceType}`), { method: 'POST', headers: headers(), body: JSON.stringify(payload) });
        const data = await res.json();
        if (data && data.id) await dbPut(storeName, data);
        return data;
    }

    async function get(id) {
        if (!id) return null;
        const cached = await dbGet(storeName, id);
        if (!isOnline()) return cached || null;
        const res = await fetchWithRetry(url(`/${resourceType}/${id}`), { headers: headers() });
        if (res.status === 404) return null;
        const data = await res.json();
        if (data && data.id) await dbPut(storeName, data);
        return data;
    }

    async function update(id, payload) {
        if (!isOnline()) {
            await enqueue('PUT', resourceType, payload, id);
            return { id, queued: true };
        }
        const res = await fetchWithRetry(url(`/${resourceType}/${id}`), { method: 'PUT', headers: headers(), body: JSON.stringify(payload) });
        const data = await res.json();
        if (data && data.id) await dbPut(storeName, data);
        return data;
    }

    async function remove(id) {
        if (!isOnline()) {
            await enqueue('DELETE', resourceType, {}, id);
            return { id, queued: true };
        }
        await fetchWithRetry(url(`/${resourceType}/${id}`), { method: 'DELETE', headers: headers() });
        await dbDelete(storeName, id);
        return { id, deleted: true };
    }

    return { search, create, get, update, remove };
}

/* ------------------------------------------------------------------ */
/* Resource-specific exports                                          */
/* ------------------------------------------------------------------ */

const patientApi     = makeResourceApi('Patient', 'patients');
const encounterApi   = makeResourceApi('Encounter', 'encounters');
const observationApi = makeResourceApi('Observation', 'observations');
const conditionApi   = makeResourceApi('Condition', 'conditions');
const medReqApi      = makeResourceApi('MedicationRequest', 'medicationRequests');
const procedureApi   = makeResourceApi('Procedure', 'procedures');

export const searchPatients            = patientApi.search;
export const createPatient             = patientApi.create;
export const getPatient                = patientApi.get;
export const updatePatient             = patientApi.update;
export const deletePatient             = patientApi.remove;

export const searchEncounters          = encounterApi.search;
export const createEncounter           = encounterApi.create;
export const getEncounter              = encounterApi.get;
export const updateEncounter           = encounterApi.update;
export const deleteEncounter           = encounterApi.remove;

export const searchObservations        = observationApi.search;
export const createObservation         = observationApi.create;
export const getObservation            = observationApi.get;
export const updateObservation         = observationApi.update;
export const deleteObservation         = observationApi.remove;

export const searchConditions          = conditionApi.search;
export const createCondition           = conditionApi.create;
export const getCondition              = conditionApi.get;
export const updateCondition           = conditionApi.update;
export const deleteCondition           = conditionApi.remove;

export const searchMedicationRequests  = medReqApi.search;
export const createMedicationRequest   = medReqApi.create;
export const getMedicationRequest      = medReqApi.get;
export const updateMedicationRequest   = medReqApi.update;
export const deleteMedicationRequest   = medReqApi.remove;

export const searchProcedures          = procedureApi.search;
export const createProcedure           = procedureApi.create;
export const getProcedure              = procedureApi.get;
export const updateProcedure           = procedureApi.update;
export const deleteProcedure           = procedureApi.remove;

/* ------------------------------------------------------------------ */
/* Misc endpoints                                                       */
/* ------------------------------------------------------------------ */

export async function getHealth() {
    const res = await fetchWithRetry(url('/health'), { headers: headers() });
    return res.json();
}

export async function getManifest() {
    const res = await fetchWithRetry(url('/sync/manifest'), { headers: headers() });
    return res.json();
}

export async function seedServer(count = 20) {
    console.warn('seedServer is deprecated: the /seed endpoint does not exist');
    return { ok: false, reason: 'Endpoint not implemented' };
}

/* Audit */

export async function searchAudit(params = {}) {
    if (!isOnline()) {
        let all = await dbGetAll('audit');
        const p = params;
        all = all.filter(a => {
            if (p.actor && !(a.agent?.[0]?.who?.reference || '').toLowerCase().includes(p.actor.toLowerCase())) return false;
            if (p.event_type && !(a.type?.coding?.[0]?.code || '').toLowerCase().includes(p.event_type.toLowerCase())) return false;
            if (p.outcome && !(a.outcome || '').toLowerCase().includes(p.outcome.toLowerCase())) return false;
            if (p.patient && (a.patient?.reference || '').replace('Patient/','') !== p.patient) return false;
            return true;
        });
        return { resourceType: 'Bundle', type: 'searchset', total: all.length, entry: all.map(a => ({ resource: a })) };
    }
    const qs = new URLSearchParams();
    qs.set('_count', String(params._count || 500));
    if (params.actor) qs.set('actor', params.actor);
    if (params.event_type) qs.set('event_type', params.event_type);
    if (params.outcome) qs.set('outcome', params.outcome);
    if (params.patient) qs.set('patient', params.patient);
    if (params.since != null) qs.set('since', String(params.since));
    if (params.until != null) qs.set('until', String(params.until));
    const res = await fetchWithRetry(url('/AuditEvent?' + qs.toString()), { headers: headers() });
    const data = await res.json();
    if (data.entry) {
        for (const e of data.entry) {
            if (e.resource) await dbPut('audit', e.resource);
        }
    }
    return data;
}

export async function createAudit(payload) {
    if (!isOnline()) { await dbPut('audit', payload); return { queued: true }; }
    const res = await fetchWithRetry(url('/AuditEvent'), { method: 'POST', headers: headers(), body: JSON.stringify(payload) });
    return res.status >= 200 && res.status < 300;
}

export async function deltaSync(body) {
    const res = await fetchWithRetry(url('/sync/delta'), { method: 'POST', headers: headers(), body: JSON.stringify(body) });
    return res.json();
}

/* Low-level helpers (re-exported for advanced consumers) */

export {
    openDB, dbGetAll, dbGet, dbPut, dbDelete, dbClear,
    isOnline, flushQueue, enqueue,
};

/* ------------------------------------------------------------------ */
/* New Resource APIs for v0.2.0                                        */
/* ------------------------------------------------------------------ */

const allergyApi      = makeResourceApi('AllergyIntolerance', 'allergyIntolerances');
const docRefApi       = makeResourceApi('DocumentReference', 'documentReferences');
const serviceReqApi   = makeResourceApi('ServiceRequest', 'serviceRequests');
const carePlanApi     = makeResourceApi('CarePlan', 'carePlans');
const famHistApi      = makeResourceApi('FamilyMemberHistory', 'familyMemberHistories');
const immunizationApi = makeResourceApi('Immunization', 'immunizations');
const medAdminApi     = makeResourceApi('MedicationAdministration', 'medicationAdministrations');
const medStmtApi      = makeResourceApi('MedicationStatement', 'medicationStatements');

export const searchAllergyIntolerances = allergyApi.search;
export const createAllergyIntolerance  = allergyApi.create;
export const getAllergyIntolerance     = allergyApi.get;
export const updateAllergyIntolerance  = allergyApi.update;
export const deleteAllergyIntolerance  = allergyApi.remove;

export const searchDocumentReferences  = docRefApi.search;
export const createDocumentReference   = docRefApi.create;
export const getDocumentReference      = docRefApi.get;
export const updateDocumentReference   = docRefApi.update;
export const deleteDocumentReference   = docRefApi.remove;

export const searchServiceRequests     = serviceReqApi.search;
export const createServiceRequest      = serviceReqApi.create;
export const getServiceRequest         = serviceReqApi.get;
export const updateServiceRequest      = serviceReqApi.update;
export const deleteServiceRequest      = serviceReqApi.remove;

export const searchCarePlans           = carePlanApi.search;
export const createCarePlan            = carePlanApi.create;
export const getCarePlan               = carePlanApi.get;
export const updateCarePlan            = carePlanApi.update;
export const deleteCarePlan            = carePlanApi.remove;

export const searchFamilyMemberHistories = famHistApi.search;
export const createFamilyMemberHistory   = famHistApi.create;
export const getFamilyMemberHistory      = famHistApi.get;
export const updateFamilyMemberHistory   = famHistApi.update;
export const deleteFamilyMemberHistory   = famHistApi.remove;

export const searchImmunizations       = immunizationApi.search;
export const createImmunization        = immunizationApi.create;
export const getImmunization             = immunizationApi.get;
export const updateImmunization        = immunizationApi.update;
export const deleteImmunization          = immunizationApi.remove;

export const searchMedicationAdministrations = medAdminApi.search;
export const createMedicationAdministration  = medAdminApi.create;
export const getMedicationAdministration     = medAdminApi.get;
export const updateMedicationAdministration  = medAdminApi.update;
export const deleteMedicationAdministration  = medAdminApi.remove;

export const searchMedicationStatements  = medStmtApi.search;
export const createMedicationStatement   = medStmtApi.create;
export const getMedicationStatement      = medStmtApi.get;
export const updateMedicationStatement   = medStmtApi.update;
export const deleteMedicationStatement   = medStmtApi.remove;

/* Local helpers for filtering / sorting */

export async function getLocalPatients() {
    return dbGetAll('patients');
}
export async function getLocalEncounters() {
    return dbGetAll('encounters');
}
export async function getLocalObservations() {
    return dbGetAll('observations');
}
export async function getLocalConditions() {
    return dbGetAll('conditions');
}
export async function getLocalMedicationRequests() {
    return dbGetAll('medicationRequests');
}
export async function getLocalProcedures() {
    return dbGetAll('procedures');
}
export async function getLocalAllergyIntolerances() {
    return dbGetAll('allergyIntolerances');
}
export async function getLocalDocumentReferences() {
    return dbGetAll('documentReferences');
}
export async function getLocalServiceRequests() {
    return dbGetAll('serviceRequests');
}
export async function getLocalCarePlans() {
    return dbGetAll('carePlans');
}
export async function getLocalFamilyMemberHistories() {
    return dbGetAll('familyMemberHistories');
}
export async function getLocalImmunizations() {
    return dbGetAll('immunizations');
}
export async function getLocalMedicationAdministrations() {
    return dbGetAll('medicationAdministrations');
}
export async function getLocalMedicationStatements() {
    return dbGetAll('medicationStatements');
}

