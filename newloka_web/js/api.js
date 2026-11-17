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
    if (_cfg.sessionUserRole) h['x-user-role'] = _cfg.sessionUserRole;
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
    return new Promise((resolve) => {
        if (typeof indexedDB === 'undefined') return resolve(null);
        if (_db) return resolve(_db);
        try {
            const req = indexedDB.open('newloka_store', 4);
            req.onerror = () => resolve(null);
            req.onsuccess = () => { _db = req.result; resolve(_db); };
            req.onupgradeneeded = (e) => {
                const db = e.target.result;
                const stores = ['patients','encounters','observations','conditions','medicationRequests','procedures','queue','audit','allergyIntolerances','documentReferences','serviceRequests','carePlans','familyMemberHistories','immunizations','medicationAdministrations','medicationStatements'];
                stores.forEach(s => {
                    if (!db.objectStoreNames.contains(s)) db.createObjectStore(s, { keyPath: 'id' });
                });
            };
        } catch {
            resolve(null);
        }
    });
}

async function dbPut(store, obj) {
    const db = await openDB();
    if (!db) return obj?.id;
    return new Promise((resolve) => {
        try {
            const tx = db.transaction([store], 'readwrite');
            const st = tx.objectStore(store);
            const req = st.put(obj);
            req.onsuccess = () => resolve(req.result);
            req.onerror = () => resolve(null);
        } catch {
            resolve(null);
        }
    });
}

async function dbGetAll(store) {
    const db = await openDB();
    if (!db) return [];
    return new Promise((resolve) => {
        try {
            const tx = db.transaction([store], 'readonly');
            const st = tx.objectStore(store);
            const req = st.getAll();
            req.onsuccess = () => resolve(req.result || []);
            req.onerror = () => resolve([]);
        } catch {
            resolve([]);
        }
    });
}

async function dbGet(store, id) {
    if (!id) return null;
    const db = await openDB();
    if (!db) return null;
    return new Promise((resolve) => {
        try {
            const tx = db.transaction([store], 'readonly');
            const st = tx.objectStore(store);
            const req = st.get(id);
            req.onsuccess = () => resolve(req.result);
            req.onerror = () => resolve(null);
        } catch {
            resolve(null);
        }
    });
}

async function dbDelete(store, id) {
    const db = await openDB();
    if (!db) return;
    return new Promise((resolve) => {
        try {
            const tx = db.transaction([store], 'readwrite');
            const st = tx.objectStore(store);
            const req = st.delete(id);
            req.onsuccess = () => resolve();
            req.onerror = () => resolve();
        } catch {
            resolve();
        }
    });
}

async function dbClear(store) {
    const db = await openDB();
    if (!db) return;
    return new Promise((resolve) => {
        try {
            const tx = db.transaction([store], 'readwrite');
            const st = tx.objectStore(store);
            const req = st.clear();
            req.onsuccess = () => resolve();
            req.onerror = () => resolve();
        } catch {
            resolve();
        }
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
        const filterLocal = (list) => {
            if (!patientId) return list;
            const target = patientId.replace(/^Patient\//, '');
            return list.filter(o => {
                const subRef = (o.subject?.reference || '').replace(/^Patient\//, '');
                const patRef = (o.patient?.reference || '').replace(/^Patient\//, '');
                return subRef === target || patRef === target || subRef.includes(target) || patRef.includes(target);
            });
        };

        if (!isOnline()) {
            const all = await dbGetAll(storeName);
            const filtered = filterLocal(all);
            return { resourceType: 'Bundle', type: 'searchset', total: filtered.length, entry: filtered.map(r => ({ resource: r })) };
        }
        try {
            const q = patientId ? `?patient=${encodeURIComponent(patientId)}&_count=50` : '?_count=50';
            const res = await fetchWithRetry(url(`/${resourceType}${q}`), { headers: headers() });
            const data = await res.json();
            if (data.entry) {
                for (const e of data.entry) {
                    if (e.resource) await dbPut(storeName, e.resource);
                }
            }
            return data;
        } catch (err) {
            console.warn(`Network search for ${resourceType} failed, falling back to local store:`, err);
            const all = await dbGetAll(storeName);
            const filtered = filterLocal(all);
            return { resourceType: 'Bundle', type: 'searchset', total: filtered.length, entry: filtered.map(r => ({ resource: r })) };
        }
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

/**
 * Retrieve all patients across clinic database and local storage.
 * Normalized to both FHIR and lightweight clinic formats.
 */
export async function getPatients() {
    try {
        const rxPatients = await searchRxPadPatients('');
        if (Array.isArray(rxPatients) && rxPatients.length) {
            return rxPatients.map(p => ({
                id: p.id,
                name: typeof p.name === 'string' ? [{ text: p.name }] : (p.name || [{ text: 'Unnamed' }]),
                displayName: typeof p.name === 'string' ? p.name : (p.name?.[0]?.text || 'Unnamed'),
                gender: (p.age_sex || '').split('/')[1]?.trim() || '',
                birthDate: '',
                telecom: p.contact ? [{ value: p.contact }] : [],
                contact: p.contact || '',
                age_sex: p.age_sex || ''
            }));
        }
    } catch {}
    try {
        const bundle = await searchPatients();
        if (bundle && bundle.entry && bundle.entry.length) {
            return bundle.entry.map(e => e.resource).filter(Boolean);
        }
    } catch {}
    try {
        return await getLocalPatients();
    } catch {
        return [];
    }
}

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

/* ------------------------------------------------------------------ */
/* System & In-App Updater API                                        */
/* ------------------------------------------------------------------ */

export async function getSystemVersion() {
    const res = await fetchWithRetry(url('/api/system/version'));
    if (!res.ok) throw new Error(`Failed to fetch system version: ${res.statusText}`);
    return res.json();
}

export async function checkSystemUpdate() {
    const res = await fetchWithRetry(url('/api/system/update/check'));
    if (!res.ok) throw new Error(`Failed to check for updates: ${res.statusText}`);
    return res.json();
}

export async function applySystemUpdate(assetUrl) {
    const res = await fetchWithRetry(url('/api/system/update/apply'), {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ asset_url: assetUrl || null })
    }, 1, 180000);
    if (!res.ok) {
        const err = await res.json().catch(() => ({ message: res.statusText }));
        throw new Error(err.message || 'Update installation failed');
    }
    return res.json();
}

export async function uploadSystemUpdate(file) {
    const res = await fetch(url('/api/system/update/upload'), {
        method: 'POST',
        headers: {
            'Content-Type': 'application/octet-stream'
        },
        body: file
    });
    if (!res.ok) {
        const err = await res.json().catch(() => ({ message: res.statusText }));
        throw new Error(err.message || 'Offline package upload failed');
    }
    return res.json();
}

export async function restartServer() {
    const res = await fetch(url('/api/system/restart'), {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({})
    });
    if (!res.ok) {
        const err = await res.json().catch(() => ({ message: res.statusText }));
        throw new Error(err.message || 'Server restart failed');
    }
    return res.json();
}

/* ------------------------------------------------------------------ */
/* eRx Pad Pro & Medication Dispensing Endpoints                     */
/* ------------------------------------------------------------------ */

export async function getRxPadPrescriptions() {
    const res = await fetchWithRetry(url('/api/rxpad/prescriptions'), {
        method: 'GET',
        headers: headers()
    }, 2, 10000);
    if (!res.ok) {
        throw new Error(`Failed to fetch prescriptions (${res.status})`);
    }
    return res.json();
}

export async function getRxPadPrescription(id) {
    const res = await fetchWithRetry(url(`/api/rxpad/prescriptions/${id}`), {
        method: 'GET',
        headers: headers()
    }, 2, 10000);
    if (!res.ok) {
        throw new Error(`Failed to fetch prescription details (${res.status})`);
    }
    return res.json();
}

export async function dispenseRxPadPrescription(id, notes = '', dispensedBy = 'Clinician') {
    const res = await fetchWithRetry(url(`/api/rxpad/prescriptions/${id}/dispense`), {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            ...(headers())
        },
        body: JSON.stringify({
            notes: notes || null,
            dispensed_by: dispensedBy || null
        })
    }, 1, 15000);
    if (!res.ok) {
        const err = await res.json().catch(() => ({ message: res.statusText }));
        throw new Error(err.message || 'Medication dispensing failed');
    }
    return res.json();
}

export async function getNextRxPadSerial() {
    const res = await fetchWithRetry(url('/api/rxpad/next-serial'), {
        method: 'GET',
        headers: headers()
    }, 1, 5000);
    if (!res.ok) {
        throw new Error(`Failed to get next serial (${res.status})`);
    }
    const data = await res.json();
    const num = data.nextSerial || data.next_serial || parseInt(data.formatted, 10) || 1;
    return {
        next_serial: num,
        nextSerial: num,
        formatted: data.formatted || String(num).padStart(4, '0')
    };
}

export async function searchRxPadPatients(q = '') {
    const res = await fetchWithRetry(url(`/api/rxpad/patients?q=${encodeURIComponent(q)}`), {
        method: 'GET',
        headers: headers()
    }, 1, 5000);
    if (!res.ok) {
        throw new Error(`Failed to search patients (${res.status})`);
    }
    return res.json();
}

export async function deleteRxPadPrescription(id) {
    const res = await fetchWithRetry(url(`/api/rxpad/prescriptions/${id}`), {
        method: 'DELETE',
        headers: headers()
    }, 1, 10000);
    if (!res.ok) {
        throw new Error(`Failed to delete prescription (${res.status})`);
    }
    return res.json();
}

export async function getRxPadFormulary() {
    const res = await fetchWithRetry(url('/api/rxpad/formulary'), {
        method: 'GET',
        headers: headers()
    }, 2, 10000);
    if (!res.ok) {
        throw new Error(`Failed to fetch formulary (${res.status})`);
    }
    return res.json();
}

export async function updateRxPadFormulary(items) {
    const res = await fetchWithRetry(url('/api/rxpad/formulary'), {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            ...(headers())
        },
        body: JSON.stringify(items)
    }, 1, 10000);
    if (!res.ok) {
        throw new Error(`Failed to update formulary (${res.status})`);
    }
    return res.json();
}

export async function syncRxPadPrescription(payload) {
    const res = await fetchWithRetry(url('/api/rxpad/sync'), {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            ...(headers())
        },
        body: JSON.stringify(payload)
    }, 1, 15000);
    if (!res.ok) {
        const err = await res.json().catch(() => ({}));
        throw new Error(err.error || `Failed to sync prescription (${res.status})`);
    }
    return res.json();
}

export async function getRxPadTemplate() {
    const res = await fetchWithRetry(url('/api/rxpad/template'), {
        method: 'GET',
        headers: headers()
    }, 2, 8000);
    if (!res.ok) throw new Error(`Failed to fetch RxPad template (${res.status})`);
    return res.json();
}

export async function saveRxPadTemplate(template) {
    const res = await fetchWithRetry(url('/api/rxpad/template'), {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            ...(headers())
        },
        body: JSON.stringify(template)
    }, 1, 15000);
    if (!res.ok) throw new Error(`Failed to save RxPad template (${res.status})`);
    return res.json();
}

export async function getSystemConfig() {
    const res = await fetchWithRetry(url('/api/config'), {
        method: 'GET',
        headers: headers()
    }, 2, 8000);
    if (!res.ok) throw new Error(`Failed to fetch system config (${res.status})`);
    return res.json();
}

export async function saveSystemConfig(cfg) {
    const res = await fetchWithRetry(url('/api/config'), {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            ...(headers())
        },
        body: JSON.stringify(cfg)
    }, 1, 10000);
    if (!res.ok) throw new Error(`Failed to save system config (${res.status})`);
    return res.json();
}

export async function getPrescriberProfile() {
    const res = await fetchWithRetry(url('/api/prescriber/profile'), {
        method: 'GET',
        headers: headers()
    }, 2, 8000);
    if (!res.ok) throw new Error(`Failed to fetch prescriber profile (${res.status})`);
    return res.json();
}

export async function savePrescriberProfile(profile) {
    const res = await fetchWithRetry(url('/api/prescriber/profile'), {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            ...(headers())
        },
        body: JSON.stringify(profile)
    }, 1, 10000);
    if (!res.ok) throw new Error(`Failed to save prescriber profile (${res.status})`);
    return res.json();
}

export async function getInpatientCensus() {
    const res = await fetchWithRetry(url('/api/inpatient/census'), {
        method: 'GET',
        headers: headers()
    }, 2, 8000);
    if (!res.ok) throw new Error(`Failed to fetch inpatient census (${res.status})`);
    return res.json();
}

export async function saveInpatientCensus(census) {
    const res = await fetchWithRetry(url('/api/inpatient/census'), {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            ...(headers())
        },
        body: JSON.stringify(census)
    }, 1, 10000);
    if (!res.ok) throw new Error(`Failed to save inpatient census (${res.status})`);
    return res.json();
}

export async function getBillingInvoices() {
    const res = await fetchWithRetry(url('/api/billing/invoices'), {
        method: 'GET',
        headers: headers()
    }, 2, 8000);
    if (!res.ok) throw new Error(`Failed to fetch billing invoices (${res.status})`);
    return res.json();
}

export async function createBillingInvoice(inv) {
    const res = await fetchWithRetry(url('/api/billing/invoices'), {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            ...(headers())
        },
        body: JSON.stringify(inv)
    }, 1, 10000);
    if (!res.ok) throw new Error(`Failed to create invoice (${res.status})`);
    return res.json();
}

export async function payBillingInvoice(id, paymentInfo = {}) {
    const res = await fetchWithRetry(url(`/api/billing/invoices/${encodeURIComponent(id)}/pay`), {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            ...(headers())
        },
        body: JSON.stringify(paymentInfo)
    }, 1, 10000);
    if (!res.ok) throw new Error(`Failed to record payment (${res.status})`);
    return res.json();
}

export async function getPharmacySummary() {
    const res = await fetchWithRetry(url('/api/pharmacy/summary'), {
        method: 'GET',
        headers: headers()
    }, 2, 8000);
    if (!res.ok) throw new Error(`Failed to fetch pharmacy summary (${res.status})`);
    return res.json();
}

/* ------------------------------------------------------------------ */
/* Tier T3: Departments & Inter-Department Patient Transfers          */
/* ------------------------------------------------------------------ */

export async function getDepartments() {
    const res = await fetchWithRetry(url('/api/departments'), { method: 'GET', headers: headers() }, 2, 8000);
    if (!res.ok) throw new Error(`Failed to fetch departments (${res.status})`);
    return res.json();
}

export async function saveDepartment(dept) {
    const res = await fetchWithRetry(url('/api/departments'), {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', ...headers() },
        body: JSON.stringify(dept)
    }, 1, 10000);
    if (!res.ok) throw new Error(`Failed to save department (${res.status})`);
    return res.json();
}

export async function getTransfers() {
    const res = await fetchWithRetry(url('/api/departments/transfers'), { method: 'GET', headers: headers() }, 2, 8000);
    if (!res.ok) throw new Error(`Failed to fetch transfers (${res.status})`);
    return res.json();
}

export async function createTransfer(transfer) {
    const res = await fetchWithRetry(url('/api/departments/transfers'), {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', ...headers() },
        body: JSON.stringify(transfer)
    }, 1, 10000);
    if (!res.ok) throw new Error(`Failed to create transfer (${res.status})`);
    return res.json();
}

export async function acceptTransfer(id, payload) {
    const res = await fetchWithRetry(url(`/api/departments/transfers/${encodeURIComponent(id)}/accept`), {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', ...headers() },
        body: JSON.stringify(payload)
    }, 1, 10000);
    if (!res.ok) throw new Error(`Failed to accept transfer (${res.status})`);
    return res.json();
}

export async function rejectTransfer(id, payload) {
    const res = await fetchWithRetry(url(`/api/departments/transfers/${encodeURIComponent(id)}/reject`), {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', ...headers() },
        body: JSON.stringify(payload)
    }, 1, 10000);
    if (!res.ok) throw new Error(`Failed to reject transfer (${res.status})`);
    return res.json();
}

export async function getHospitalReports() {
    const res = await fetchWithRetry(url('/api/reports/overview'), { method: 'GET', headers: headers() }, 2, 8000);
    if (!res.ok) throw new Error(`Failed to fetch hospital reports (${res.status})`);
    return res.json();
}

/* ------------------------------------------------------------------ */
/* Tier T4: Research Federation & Differential Privacy                */
/* ------------------------------------------------------------------ */

export async function getFederationNodes() {
    const res = await fetchWithRetry(url('/api/federation/nodes'), { method: 'GET', headers: headers() }, 2, 8000);
    if (!res.ok) throw new Error(`Failed to fetch federation nodes (${res.status})`);
    return res.json();
}

export async function registerFederationNode(node) {
    const res = await fetchWithRetry(url('/api/federation/nodes'), {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', ...headers() },
        body: JSON.stringify(node)
    }, 1, 10000);
    if (!res.ok) throw new Error(`Failed to register federation node (${res.status})`);
    return res.json();
}

export async function pingFederationNode(peerUrl) {
    const res = await fetchWithRetry(url('/api/federation/ping'), {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', ...headers() },
        body: JSON.stringify({ peer_url: peerUrl })
    }, 1, 6000);
    if (!res.ok) throw new Error(`Failed to ping node (${res.status})`);
    return res.json();
}

export async function executeResearchQuery(query) {
    const res = await fetchWithRetry(url('/api/research/query'), {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', ...headers() },
        body: JSON.stringify(query)
    }, 1, 10000);
    if (!res.ok) throw new Error(`Failed to execute research query (${res.status})`);
    return res.json();
}

export async function getResearchCohorts() {
    const res = await fetchWithRetry(url('/api/research/cohorts'), { method: 'GET', headers: headers() }, 2, 8000);
    if (!res.ok) throw new Error(`Failed to fetch research cohorts (${res.status})`);
    return res.json();
}

export async function createResearchCohort(cohort) {
    const res = await fetchWithRetry(url('/api/research/cohorts'), {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', ...headers() },
        body: JSON.stringify(cohort)
    }, 1, 10000);
    if (!res.ok) throw new Error(`Failed to create research cohort (${res.status})`);
    return res.json();
}

export async function exportResearchBundle(query = {}) {
    const res = await fetchWithRetry(url('/api/research/export'), {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', ...headers() },
        body: JSON.stringify(query)
    }, 1, 15000);
    if (!res.ok) throw new Error(`Failed to export research bundle (${res.status})`);
    return res.json();
}

/* ------------------------------------------------------------------ */
/* Tier T4: Patient Consent Directives & Dynamic Verification         */
/* ------------------------------------------------------------------ */

export async function getConsentDirectives(patientId = null) {
    const path = patientId ? `/api/consent/directives?patient_id=${encodeURIComponent(patientId)}` : '/api/consent/directives';
    const res = await fetchWithRetry(url(path), { method: 'GET', headers: headers() }, 2, 8000);
    if (!res.ok) throw new Error(`Failed to fetch consent directives (${res.status})`);
    return res.json();
}

export async function createConsentDirective(directive) {
    const res = await fetchWithRetry(url('/api/consent/directives'), {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', ...headers() },
        body: JSON.stringify(directive)
    }, 1, 10000);
    if (!res.ok) throw new Error(`Failed to create consent directive (${res.status})`);
    return res.json();
}

export async function revokeConsentDirective(id, reason = "Patient requested opt-out") {
    const res = await fetchWithRetry(url(`/api/consent/directives/${encodeURIComponent(id)}/revoke`), {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', ...headers() },
        body: JSON.stringify({ reason })
    }, 1, 10000);
    if (!res.ok) throw new Error(`Failed to revoke consent directive (${res.status})`);
    return res.json();
}

export async function verifyConsent(patientId, recipientNodeId, purpose, resourceType) {
    const res = await fetchWithRetry(url('/api/consent/verify'), {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', ...headers() },
        body: JSON.stringify({
            patient_id: patientId,
            recipient_node_id: recipientNodeId,
            purpose,
            resource_type: resourceType
        })
    }, 1, 6000);
    if (!res.ok) throw new Error(`Failed to verify consent (${res.status})`);
    return res.json();
}

/* ------------------------------------------------------------------ */
/* Indian Pharmacopoeia & Standardized Order Sets                     */
/* ------------------------------------------------------------------ */

export async function getPharmacopoeiaInfo() {
    const res = await fetchWithRetry(url('/api/pharmacopoeia/info'), {
        method: 'GET',
        headers: headers()
    }, 2, 8000);
    if (!res.ok) throw new Error(`Failed to fetch pharmacopoeia info (${res.status})`);
    return res.json();
}

export async function searchPharmacopoeia(q = '', category = null, limit = 25) {
    let endpoint = `/api/pharmacopoeia/search?q=${encodeURIComponent(q)}&limit=${limit}`;
    if (category) endpoint += `&category=${encodeURIComponent(category)}`;
    const res = await fetchWithRetry(url(endpoint), {
        method: 'GET',
        headers: headers()
    }, 2, 8000);
    if (!res.ok) throw new Error(`Failed to search pharmacopoeia (${res.status})`);
    return res.json();
}

export async function getPharmacopoeiaEntry(id) {
    const res = await fetchWithRetry(url(`/api/pharmacopoeia/entry/${encodeURIComponent(id)}`), {
        method: 'GET',
        headers: headers()
    }, 2, 8000);
    if (!res.ok) throw new Error(`Failed to fetch pharmacopoeia entry (${res.status})`);
    return res.json();
}

export async function getPharmacopoeiaCategories() {
    const res = await fetchWithRetry(url('/api/pharmacopoeia/categories'), {
        method: 'GET',
        headers: headers()
    }, 2, 8000);
    if (!res.ok) throw new Error(`Failed to fetch pharmacopoeia categories (${res.status})`);
    return res.json();
}

export async function getPharmacopoeiaOrderSets() {
    const res = await fetchWithRetry(url('/api/pharmacopoeia/order-sets'), {
        method: 'GET',
        headers: headers()
    }, 2, 8000);
    if (!res.ok) throw new Error(`Failed to fetch pharmacopoeia order sets (${res.status})`);
    return res.json();
}

export async function getPharmacopoeiaOrderSet(id) {
    const res = await fetchWithRetry(url(`/api/pharmacopoeia/order-sets/${encodeURIComponent(id)}`), {
        method: 'GET',
        headers: headers()
    }, 2, 8000);
    if (!res.ok) throw new Error(`Failed to fetch order set (${res.status})`);
    return res.json();
}

export async function checkPharmacopoeiaSafety(drugs = [], allergies = []) {
    const res = await fetchWithRetry(url('/api/pharmacopoeia/check-safety'), {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', ...headers() },
        body: JSON.stringify({ drugs, allergies })
    }, 1, 8000);
    if (!res.ok) throw new Error(`Failed to check safety (${res.status})`);
    return res.json();
}

export async function getIngestSamples() {
    const res = await fetchWithRetry(url('/api/ingest/samples'), {
        method: 'GET',
        headers: headers()
    }, 2, 8000);
    if (!res.ok) throw new Error(`Failed to fetch ingestion sample cases (${res.status})`);
    return res.json();
}

export async function extractIngest(text, patientId = null) {
    const res = await fetchWithRetry(url('/api/ingest/extract'), {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', ...headers() },
        body: JSON.stringify({ text, patient_id: patientId })
    }, 2, 12000);
    if (!res.ok) throw new Error(`Failed to extract clinical entities (${res.status})`);
    return res.json();
}

export async function commitIngest(payload) {
    const res = await fetchWithRetry(url('/api/ingest/commit'), {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', ...headers() },
        body: JSON.stringify(payload)
    }, 1, 15000);
    if (!res.ok) throw new Error(`Failed to commit approved entities (${res.status})`);
    return res.json();
}


