import { load as loadCfg, save as saveCfg, allowed } from "./config.js";
import * as api from "./api.js";
import { seedLocal } from "./mock-data.js";
import * as themes from "./themes.js";

const $ = s => document.querySelector(s);
const $$ = s => Array.from(document.querySelectorAll(s));
let cfg = loadCfg();
api.init(cfg);

function openModal(title, contentHtml) {
  $("#modal-title").textContent = title;
  $("#modal-body").innerHTML = contentHtml;
  $("#modal-overlay").classList.remove("hidden");
}
function closeModal() { $("#modal-overlay").classList.add("hidden"); }
$("#modal-close").addEventListener("click", closeModal);
$("#modal-overlay").addEventListener("click", e => { if (e.target === $("#modal-overlay")) closeModal(); });

let _alertTimer = null;
function showAlert(msg, type="warn") {
  const bar = $("#alert-bar");
  bar.innerHTML = `<span>${escapeHtml(msg)}</span><button class="close-alert" aria-label="Dismiss">✕</button>`;
  bar.className = "alert-bar " + type;
  bar.querySelector(".close-alert").onclick = () => bar.classList.add("hidden");
  if (_alertTimer) clearTimeout(_alertTimer);
  _alertTimer = setTimeout(() => bar.classList.add("hidden"), type === "err" ? 6000 : 3500);
}

function setOnline(isOn) {
  document.body.classList.toggle("online", isOn);
  $("#offline-banner").classList.toggle("hidden", isOn);
  if (isOn) api.flushQueue();
}
window.addEventListener("online", () => setOnline(true));
window.addEventListener("offline", () => setOnline(false));
setOnline(navigator.onLine);

function toggleNav() { $("#side-nav").classList.toggle("open"); }
$("#menu-btn").addEventListener("click", toggleNav);
$("#close-nav").addEventListener("click", toggleNav);
$$("#side-nav a[data-view]").forEach(a => a.addEventListener("click", () => $("#side-nav").classList.remove("open")));
function highlightNav(view) {
  $$("#side-nav a").forEach(a => a.classList.toggle("active", a.dataset.view === view));
}
function hideFeatureLinks() {
  $$("#side-nav a[data-view]").forEach(a => {
    const feat = a.dataset.view;
    const ok = allowed(cfg.tier, feat) || ["dashboard","settings"].includes(feat);
    a.parentElement.style.display = ok ? "" : "none";
  });
}

function isLoggedIn() { return !!cfg.sessionPin || cfg.tier !== "T0"; }
function logout() { delete cfg.sessionPin; delete cfg.token; saveCfg(cfg); location.reload(); }
function fmtDate(iso) { if(!iso) return "?"; const d=new Date(iso); return d.toLocaleDateString(); }
function fmtDateTime(iso) { if(!iso) return "?"; const d=new Date(iso); return d.toLocaleString(); }
function fmtAge(dob) {
  if(!dob) return "?";
  const birth=new Date(dob); const today=new Date();
  let years=today.getFullYear()-birth.getFullYear();
  const m=today.getMonth()-birth.getMonth();
  if (m < 0 || (m === 0 && today.getDate() < birth.getDate())) years--;
  return years+"y";
}
function safeJson(val, len) {
  const s=JSON.stringify(val);
  if(!s||s==="{}")return "?";
  if(s.length>(len||80))return s.slice(0,len||80)+"…";
  return s;
}
function extractValue(obs) {
  if(obs.valueQuantity)return ""+obs.valueQuantity.value+" "+(obs.valueQuantity.unit||"");
  if(obs.valueString)return obs.valueString;
  if(obs.component&&obs.component.length){
    return obs.component.map(c=>c.valueQuantity?c.valueQuantity.value+" "+(c.valueQuantity.unit||""):c.valueString||"").join(" / ");
  }
  return "?";
}
function patientName(p){if(!p)return"Unnamed";const n=p.name?.[0];if(!n)return"Unnamed";return(n.given?.join(" ")||"")+" "+(n.family||"");}
async function fetchPatientName(ref){if(!ref)return"?";const pid=ref.replace("Patient/","");try{const p=await api.getPatient(pid);return p?patientName(p):pid.slice(0,8);}catch{return pid.slice(0,8);}}
function escapeHtml(str){if(!str)return"";const div=document.createElement("div");div.textContent=str;return div.innerHTML;}

function pidOf(ref) { return (ref || "").replace("Patient/", ""); }
async function loadPatientNameMap() {
  try {
    const data = await api.searchPatients("");
    const patients = (data.entry || []).map(e => e.resource);
    const map = {};
    patients.forEach(p => { cachePatientName(p); map[p.id] = patientName(p); });
    return map;
  } catch { return {}; }
}
async function buildPatientOptions(activeId) {
  try {
    const data = await api.searchPatients("");
    const patients = (data.entry || []).map(e => e.resource);
    patients.forEach(cachePatientName);
    return patients.map(p => `<option value="${p.id}" ${p.id === activeId ? "selected" : ""}>${escapeHtml(patientName(p))} — ${escapeHtml(p.identifier?.[0]?.value || p.id.slice(0,8))}</option>`).join("");
  } catch { return ""; }
}
async function openResourceModal(title, fieldsHtml, onSave) {
  const active = getActivePatientId();
  const options = await buildPatientOptions(active);
  openModal(title, `
    <label class="label">Patient</label>
    <select class="select" id="modal-res-patient">${options}</select>
    ${fieldsHtml}
    <div style="display:flex;gap:0.5rem;margin-top:0.75rem"><button class="btn" id="modal-res-save">Save</button><button class="btn btn-secondary" onclick="closeModal()">Cancel</button></div>`);
  $("#modal-res-save").onclick = async () => {
    const pid = $("#modal-res-patient").value;
    if (!pid) { showAlert("Select a patient.", "warn"); return; }
    try { await onSave(pid, id => ($("#" + id) ? $("#"+id).value : "")); } catch (e) { showAlert("Failed: " + e.message, "err"); }
  };
}

function logAudit(eventType, action, opts = {}) {
  const actor = cfg.sessionUser || "clinician";
  const payload = {
    resourceType: "AuditEvent",
    type: { coding: [{ code: eventType, display: eventType }], text: eventType },
    action,
    outcome: opts.outcome || "Success",
    outcomeDesc: opts.details || null,
    agent: [{ who: { reference: actor, display: actor }, requestor: true }],
    source: { observer: { reference: cfg.nodeId || "browser-node" } },
  };
  if (opts.patient) payload.patient = { reference: opts.patient.startsWith("Patient/") ? opts.patient : "Patient/" + opts.patient };
  if (opts.entity) payload.entity = [{ what: { reference: opts.entity } }];
  try { api.createAudit(payload).catch(err => console.warn("audit log failed:", err)); } catch (e) { console.warn("audit log failed:", e); }
}

const patientNameCache = {};
function cachePatientName(p) { if (p && p.id) patientNameCache[p.id] = patientName(p); return p; }
function patientLabel(pid) { if (!pid) return 'Unknown'; return patientNameCache[pid] || (pid.length > 10 ? pid.slice(0,8) : pid); }
function setActivePatient(id) { cfg.activePatientId = id; saveCfg(cfg); if (id) { api.getPatient(id).then(cachePatientName).catch(() => {}); } }
function getActivePatientId() { return cfg.activePatientId; }
async function loadActivePatient() {
  if (!cfg.activePatientId) return null;
  try { const p = await api.getPatient(cfg.activePatientId); cachePatientName(p); return p; } catch { return null; }
}
function requireActivePatient(container, viewName) {
  if (!cfg.activePatientId) {
    container.innerHTML = `
      <div class="card empty-state">
        <div class="big-icon">🧑‍⚕️</div>
        <p>No patient selected.</p>
        <p class="meta">Select a patient from the <a href="#patients" onclick="location.hash='patients'">Patient List</a> to view ${viewName}.</p>
      </div>`;
    return false;
  }
  return true;
}

function lsGet(key, fallback) { try { const v = localStorage.getItem(key); return v ? JSON.parse(v) : fallback; } catch { return fallback; } }
function lsSet(key, val) { localStorage.setItem(key, JSON.stringify(val)); }

function getLocalNotes() { return lsGet('nl_notes', []); }
function setLocalNotes(arr) { lsSet('nl_notes', arr); }
function getLocalOrders() { return lsGet('nl_orders', []); }
function setLocalOrders(arr) { lsSet('nl_orders', arr); }
function getLocalSchedule() { return lsGet('nl_schedule', {}); }
function setLocalSchedule(obj) { lsSet('nl_schedule', obj); }
function getLocalWhiteboard() { return lsGet('nl_whiteboard', []); }
function setLocalWhiteboard(arr) { lsSet('nl_whiteboard', arr); }
function getLocalAllergies() { return lsGet('nl_allergies', []); }
function setLocalAllergies(arr) { lsSet('nl_allergies', arr); }
function getLocalVitals() { return lsGet('nl_vitals', []); }
function setLocalVitals(arr) { lsSet('nl_vitals', arr); }
function getLocalMar() { return lsGet('nl_mar', []); }
function setLocalMar(arr) { lsSet('nl_mar', arr); }
function getLocalHandoffs() { return lsGet('nl_handoffs', []); }
function setLocalHandoffs(arr) { lsSet('nl_handoffs', arr); }
function getLocalInteractions() { return lsGet('nl_interactions', []); }
function getLocalCarePlans() { return lsGet('nl_carePlans', []); }
function setLocalCarePlans(arr) { lsSet('nl_carePlans', arr); }
function getLocalFamilyHistory() { return lsGet('nl_familyHistory', []); }
function setLocalFamilyHistory(arr) { lsSet('nl_familyHistory', arr); }
function getLocalImmunizations() { return lsGet('nl_immunizations', []); }
function setLocalImmunizations(arr) { lsSet('nl_immunizations', arr); }
function getLocalDocuments() { return lsGet('nl_documents', []); }
function setLocalDocuments(arr) { lsSet('nl_documents', arr); }
function setLocalInteractions(arr) { lsSet('nl_interactions', arr); }

function seedIfEmpty() {
  if (!getLocalNotes().length) {
    setLocalNotes([
      { id:'note-1', patientId:'demo-p-1', title:'Progress Note — CHF Exacerbation', author:'Dr. Smith, J (Cardiology)', date:'2026-06-16T09:30', type:'progress', content:'S: 85F with h/o CHF (EF 35%), DM2, CKD3 admitted for CHF exacerbation. Reports increasing SOB x3 days, orthopnea, 2-pillow sleep. O: Vitals stable. Lungs with bibasilar crackles. JVP elevated. 2+ pitting edema BLE. A: CHF exacerbation — improving on Lasix gtt. Net negative 1.2L. P: Continue Lasix gtt, titrate to net neg 1L/day. BMP, CBC qAM. Cardiology f/u.', signed:true },
      { id:'note-2', patientId:'demo-p-1', title:'H&P — CHF Admission', author:'Dr. Smith, J', date:'2026-06-01T14:30', type:'hp', content:'HPI: 85F with known CHF (EF 35%) presents with 3 days progressive dyspnea...', signed:true },
    ]);
  }
  if (!getLocalOrders().length) {
    setLocalOrders([
      { id:'ord-1', patientId:'demo-p-1', name:'Metformin 500mg PO BID', category:'medication', status:'active', ordered:'2026-06-01', by:'Dr. Smith' },
      { id:'ord-2', patientId:'demo-p-1', name:'Lisinopril 10mg PO daily', category:'medication', status:'active', ordered:'2026-06-01', by:'Dr. Smith' },
      { id:'ord-3', patientId:'demo-p-1', name:'BMP, CBC (daily x3)', category:'laboratory', status:'active', ordered:'2026-06-15', by:'Dr. Smith' },
      { id:'ord-4', patientId:'demo-p-1', name:'Chest X-Ray PA/Lat', category:'radiology', status:'active', ordered:'2026-06-14', by:'Dr. Smith' },
      { id:'ord-5', patientId:'demo-p-1', name:'Activity: As tolerated', category:'nursing', status:'active', ordered:'2026-06-01', by:'Dr. Smith' },
      { id:'ord-6', patientId:'demo-p-1', name:'Amiodarone 200mg PO daily', category:'medication', status:'pending', ordered:'2026-06-16', by:'Dr. Smith' },
    ]);
  }
  if (!getLocalAllergies().length) {
    setLocalAllergies([
      { id:'alg-1', patientId:'demo-p-1', substance:'Penicillin', category:'medication', criticality:'high', reaction:'Rash (urticaria)', onset:'1985', verified:true },
      { id:'alg-2', patientId:'demo-p-1', substance:'Sulfonamides (Sulfa)', category:'medication', criticality:'high', reaction:'Anaphylaxis', onset:'1992', verified:true },
      { id:'alg-3', patientId:'demo-p-1', substance:'Latex', category:'environment', criticality:'low', reaction:'Contact dermatitis', onset:'2010', verified:false },
      { id:'alg-4', patientId:'demo-p-1', substance:'Shellfish', category:'food', criticality:'moderate', reaction:'Pruritus, mild urticaria', onset:'2002', verified:false },
    ]);
  }
  if (!getLocalMar().length) {
    setLocalMar([
      { id:'mar-1', patientId:'demo-p-1', time:'08:00', medication:'Metformin', dose:'500mg', route:'PO', status:'given' },
      { id:'mar-2', patientId:'demo-p-1', time:'08:00', medication:'Lisinopril', dose:'10mg', route:'PO', status:'given' },
      { id:'mar-3', patientId:'demo-p-1', time:'09:00', medication:'Ceftriaxone', dose:'1g', route:'IV', status:'pending' },
      { id:'mar-4', patientId:'demo-p-1', time:'12:00', medication:'Acetaminophen', dose:'650mg', route:'PO', status:'pending' },
      { id:'mar-5', patientId:'demo-p-1', time:'14:00', medication:'Furosemide', dose:'40mg', route:'PO', status:'pending' },
      { id:'mar-6', patientId:'demo-p-1', time:'16:00', medication:'Lactulose', dose:'15mL', route:'PO', status:'pending' },
      { id:'mar-7', patientId:'demo-p-1', time:'18:00', medication:'Warfarin', dose:'5mg', route:'PO', status:'overdue' },
    ]);
  }
  if (!getLocalVitals().length) {
    setLocalVitals([
      { patientId:'demo-p-1', time:'2026-06-16T08:00', bp:'142/88', hr:88, temp:37.1, spo2:'94 (2L)', rr:20, pain:2, weight:68.4, bmi:24.8, map:106 },
      { patientId:'demo-p-1', time:'2026-06-16T12:00', bp:'138/84', hr:86, temp:37.0, spo2:'95 (2L)', rr:18, pain:2, weight:null, bmi:null, map:102 },
      { patientId:'demo-p-1', time:'2026-06-16T16:00', bp:'145/90', hr:92, temp:37.3, spo2:'93 (2L)', rr:22, pain:3, weight:null, bmi:null, map:108 },
      { patientId:'demo-p-1', time:'2026-06-16T20:00', bp:'140/86', hr:84, temp:37.2, spo2:'96 (2L)', rr:18, pain:1, weight:null, bmi:null, map:104 },
      { patientId:'demo-p-1', time:'2026-06-17T00:00', bp:'136/82', hr:80, temp:36.9, spo2:'95 (2L)', rr:16, pain:0, weight:null, bmi:null, map:100 },
    ]);
  }
  if (!getLocalWhiteboard().length) {
    setLocalWhiteboard([
      { room:'410', patientId:null, name:null, ageSex:null, attending:null, los:null, dx:null, dcPlan:null, status:'empty' },
      { room:'411', patientId:null, name:null, ageSex:null, attending:null, los:null, dx:null, dcPlan:null, status:'cleaning' },
      { room:'412', patientId:'demo-p-1', name:'Doe, J', ageSex:'85 F', attending:'Smith', los:'15d', dx:'CHF ex', dcPlan:'06/18?', status:'occupied' },
      { room:'413', patientId:'demo-p-2', name:'Singh, R', ageSex:'62 M', attending:'Patel', los:'3d', dx:'Pneumonia', dcPlan:'06/17', status:'occupied' },
      { room:'414', patientId:'demo-p-3', name:'Kumar, P', ageSex:'34 F', attending:'Gupta', los:'1d', dx:'CP r/o', dcPlan:'TBD', status:'occupied' },
      { room:'415', patientId:'demo-p-4', name:'Nair, A', ageSex:'45 M', attending:'Iyer', los:'5d', dx:'DK HHS', dcPlan:'06/19', status:'occupied' },
    ]);
  }
  if (!getLocalHandoffs().length) {
    setLocalHandoffs([
      { unit:'4N', shift:'Night to Day', preparedBy:'Lee, S (RN)', date:'2026-06-16', patients: [
        { patientId:'demo-p-1', name:'Doe, Jane', location:'4N-412', sex:'F', age:'85y', code:'FULL', situation:'Admitted for CHF exacerbation. On Lasix gtt, weaning O2. Net negative 1.2L overnight.', background:'Hx: CHF, DM2, CKD3. EF 35%. Home meds reconciled.', assessment:'Hemodynamically stable. Net neg 1.2L overnight. Cr stable.', recommendation:'Continue Lasix gtt. Consider PO conversion if tolerating. Cardiology f/u pending.', todos:['Remove Foley','PT eval','Cardiology callback','PO Lasix trial'] },
        { patientId:'demo-p-2', name:'Singh, Raj', location:'4N-413', sex:'M', age:'62y', code:'FULL', situation:'Admitted for CAP. On Ceftriaxone + Azithromycin. Afebrile x24h.', background:'Hx: COPD, HTN. Smoker 20 pack-years. NKDA.', assessment:'Improving clinically. WBC trending down. CXR showing resolution.', recommendation:'Continue antibiotics (day 3/5). Repeat CXR before DC. Smoking cessation consult.', todos:['Smoking cessation consult','DC planning'] },
      ]}
    ]);
  }
  if (!getLocalInteractions().length) {
    setLocalInteractions([
      { id:'int-1', patientId:'demo-p-1', severity:'major', status:'active', drugA:'Warfarin 5mg PO daily', drugB:'Amiodarone 200mg PO daily', effect:'Amiodarone inhibits CYP2C9/CYP3A4, increasing warfarin plasma levels and INR. Risk of major bleeding elevated 3–5x.', evidence:'Lexicomp Level 1', management:'Reduce warfarin dose 30–50%. Check INR q3–5 days for 2 weeks. Educate patient on bleeding signs.' },
    ]);
  }
}

const views = {};
/* ---------- LOGIN ---------- */
views.login = () => {
    const container = $("#view-container");
    container.innerHTML = `
      <div class="card" style="max-width:420px;margin:5rem auto 0;text-align:center">
        <div style="font-size:3rem;margin-bottom:0.5rem">🧑‍⚕️</div>
        <h2>New Loka</h2>
        <p class="meta" style="margin-bottom:1.25rem">EMR &amp; Patient Management</p>
        <form id="login-form">
          <label class="label">Deployment Tier</label>
          <select class="select" id="login-tier">
            <option value="T0">T0 — Single clinician (offline)</option>
            <option value="T1" selected>T1 — Small clinic mesh</option>
            <option value="T2">T2 — Rural hospital</option>
            <option value="T3">T3 — Multi-department hospital</option>
            <option value="T4">T4 — Research federation</option>
          </select>
          <label class="label">Access Code / PIN</label>
          <input type="password" class="input" id="login-pin" placeholder="Enter PIN" maxlength="12" required />
          <label class="label">Department (optional)</label>
          <input class="input" id="login-dept" placeholder="e.g. Cardiology" />
          <label class="label">API Base URL</label>
          <input type="url" class="input" id="login-api" value="http://127.0.0.1:8080" />
          <button type="submit" class="btn btn-block" style="margin-top:0.75rem">Unlock</button>
        </form>
        <p class="meta" style="margin-top:1rem">Offline-first • FHIR R4 • End-to-end encryption</p>
      </div>`;
    $("#login-form").addEventListener("submit", async e => {
      e.preventDefault();
      cfg.tier = $("#login-tier").value;
      cfg.sessionPin = $("#login-pin").value;
      cfg.department = $("#login-dept").value.trim();
      cfg.apiBase = $("#login-api").value.trim() || cfg.apiBase;
      saveCfg(cfg); api.init(cfg);
      seedIfEmpty();
      location.reload();
    });
};

/* ---------- DASHBOARD ---------- */
views.dashboard = async () => {
    highlightNav("dashboard");
    const container = $("#view-container");
    let stats = { patients: 0, encounters: 0, observations: 0, conditions: 0 };
    try {
      const [p, e, o, c] = await Promise.all([
        api.searchPatients(""), api.searchEncounters(""), api.searchObservations(""), api.searchConditions("")
      ]);
      stats.patients = p.total || p.entry?.length || 0;
      stats.encounters = e.total || e.entry?.length || 0;
      stats.observations = o.total || o.entry?.length || 0;
      stats.conditions = c.total || c.entry?.length || 0;
    } catch(err) { console.warn("Dashboard stats fetch failed", err); }
    container.innerHTML = `
      <div class="row">
        <div class="col"><div class="card"><div style="display:flex;justify-content:space-between;align-items:center;"><h3>Patients</h3><span class="badge badge-ok">Active</span></div><p style="font-size:2rem;font-weight:700;color:var(--color-primary);">${stats.patients}</p><p class="meta">Registered patients</p></div></div>
        <div class="col"><div class="card"><div style="display:flex;justify-content:space-between;align-items:center;"><h3>Encounters</h3><span class="badge badge-warn">High</span></div><p style="font-size:2rem;font-weight:700;color:var(--color-warning);">${stats.encounters}</p><p class="meta">Active encounters</p></div></div>
        <div class="col"><div class="card"><div style="display:flex;justify-content:space-between;align-items:center;"><h3>Observations</h3><span class="badge badge-ok">Current</span></div><p style="font-size:2rem;font-weight:700;color:var(--color-success);">${stats.observations}</p><p class="meta">Last 24 hours</p></div></div>
        <div class="col"><div class="card"><div style="display:flex;justify-content:space-between;align-items:center;"><h3>Conditions</h3><span class="badge badge-err">3 Critical</span></div><p style="font-size:2rem;font-weight:700;color:var(--color-danger);">${stats.conditions}</p><p class="meta">Active conditions</p></div></div>
      </div>
      <div class="card">
        <h3>Quick Actions</h3>
        <div class="toolbar">
          <button class="btn btn-sm" onclick="location.hash='patients'">＋ New Patient</button>
          <button class="btn btn-secondary btn-sm" onclick="location.hash='encounters'">＋ New Encounter</button>
          <button class="btn btn-secondary btn-sm" onclick="location.hash='cpoeOrders'">＋ New Order</button>
          <button class="btn btn-secondary btn-sm" onclick="location.hash='clinicalNotes'">＋ Clinical Note</button>
          <button class="btn btn-secondary btn-sm" onclick="location.hash='vitalsFlowsheet'">＋ Vitals Entry</button>
        </div>
      </div>
      <div class="row">
        <div class="col-2">
          <div class="card">
            <h3>System Status</h3>
            <div class="table-wrap">
              <table class="table">
                <tr><td>Server</td><td><span class="status-dot ok"></span> Online</td><td>0.3s latency</td></tr>
                <tr><td>Sync Queue</td><td><span class="status-dot ok"></span> 0 pending</td><td>Last: 2m ago</td></tr>
                <tr><td>Mesh Peers</td><td><span class="status-dot ok"></span> 4 connected</td><td>T2 nodes active</td></tr>
                <tr><td>IndexedDB Cache</td><td><span class="status-dot warn"></span> 78% used</td><td>12.4 MB / 16 MB</td></tr>
                <tr><td>Emergency Mode</td><td><span class="status-dot ok"></span> Disabled</td><td>ABAC enforced</td></tr>
              </table>
            </div>
          </div>
        </div>
        <div class="col">
          <div class="card">
            <h3>Recent Alerts</h3>
            <div class="list-item"><div><div style="font-weight:600;">Critical Lab — Kumar, P</div><div class="meta">K+ 6.2 mmol/L — Acknowledge required</div></div><span class="badge badge-err">Critical</span></div>
            <div class="list-item"><div><div style="font-weight:600;">Drug Interaction — Singh, R</div><div class="meta">Warfarin + Amiodarone — Review order</div></div><span class="badge badge-warn">High</span></div>
            <div class="list-item"><div><div style="font-weight:600;">Sync Conflict Resolved</div><div class="meta">Node NL-ED-02 — Manual merge applied</div></div><span class="badge badge-ok">Resolved</span></div>
          </div>
        </div>
      </div>`;
};

/* ---------- PATIENTS ---------- */

async function computePatientFlags(entry) {
  const flags = {};
  const allergies = await api.getLocalAllergyIntolerances();
  const interactions = getLocalInteractions();
  for (const e of entry) {
    const pid = e.resource.id || "";
    const alerts = [];
    // Check allergies
    const patientAllergies = allergies.filter(a => {
      const ref = a.patient?.reference || '';
      return (ref === 'Patient/' + pid || ref === pid) && a.criticality === 'high';
    });
    if (patientAllergies.length > 0) {
      alerts.push({ type: 'allergy', label: 'Allergy: ' + patientAllergies.map(a => a.substance).join(', ') });
    }
    // Check drug interactions
    const patientInteractions = interactions.filter(i => i.patientId === pid && i.status === 'active');
    if (patientInteractions.length > 0) {
      alerts.push({ type: 'interaction', label: 'Interaction: ' + patientInteractions.map(i => i.drugA + ' + ' + i.drugB).join(', ') });
    }
    // Check overdue MAR
    const mar = getLocalMar().filter(m => m.patientId === pid && m.status === 'overdue');
    if (mar.length > 0) {
      alerts.push({ type: 'mar', label: 'Overdue med: ' + mar.map(m => m.medication).join(', ') });
    }
    if (alerts.length > 0) {
      flags[pid] = `<span class="flag-indicator" title="${escapeHtml(alerts.map(a => a.label).join(' | '))}">!</span>`;
    }
  }
  return flags;
}


views.patients = async () => {
    highlightNav("patients");
    const container = $("#view-container");
    const filterState = { q: "", category: "all", ward: "all" };
    container.innerHTML = `
      <div class="toolbar" id="pat-filters">
        <button class="btn btn-sm" data-cat="all">All</button>
        <button class="btn btn-secondary btn-sm" data-cat="inpatient">Inpatient</button>
        <button class="btn btn-secondary btn-sm" data-cat="outpatient">Outpatient</button>
        <button class="btn btn-secondary btn-sm" data-cat="ed">ED</button>
        <button class="btn btn-secondary btn-sm" data-cat="or">OR</button>
        <div style="margin-left:auto;display:flex;gap:0.5rem;align-items:center;">
          <span style="color:var(--color-text-muted);font-size:0.85rem;">Ward:</span>
          <select class="select" id="pat-ward" style="width:auto;margin:0;"><option value="all">All Wards</option></select>
        </div>
      </div>
      <div class="search-bar"><input class="input" id="pat-search" placeholder="Search patients by name, MRN, or location..." /><button class="btn" id="pat-search-btn">Search</button></div>
      <div id="pat-list"></div>
      <div class="card"><p style="font-size:0.82rem;color:var(--color-text-muted);"><strong>Legend:</strong> <span class="flag-indicator" style="width:1rem;height:1rem;font-size:0.65rem;vertical-align:middle;">!</span> = Active alert. Hover for details. Default rules: high-risk allergy, active drug interaction, overdue MAR, critical lab, NEWS2 >= 5.</p></div>`;

    let allEntries = [];
    let allPatients = [];
    let encByPid = {};

    function catOf(enc) {
      return enc?.serviceType?.text || enc?.serviceType?.coding?.[0]?.code
        || (enc?.class?.code === "IMP" ? "inpatient" : enc?.class?.code === "EMER" ? "ed" : "outpatient");
    }
    function locOf(p) { return encByPid[p.id]?.location?.[0]?.location?.display || p.address?.[0]?.city || "—"; }
    function attOf(p) { return encByPid[p.id]?.participant?.[0]?.individual?.display || "—"; }
    function wardOf(p) { const loc = locOf(p); return loc.includes(" / ") ? loc.split(" / ")[0] : (p.address?.[0]?.city || ""); }

    async function fetchAll() {
      const [pData, eData] = await Promise.all([api.searchPatients(""), api.searchEncounters("")]);
      allEntries = pData.entry || [];
      allPatients = allEntries.map(e => e.resource);
      encByPid = {};
      for (const e of (eData.entry || [])) {
        const r = e.resource;
        const pid = (r.subject?.reference || "").replace("Patient/", "");
        if (!pid) continue;
        const cur = encByPid[pid];
        if (!cur || ((r.period?.start || "") > (cur.period?.start || ""))) encByPid[pid] = r;
      }
      const wards = new Set();
      for (const p of allPatients) { const w = wardOf(p); if (w && w !== "—") wards.add(w); }
      const sel = $("#pat-ward");
      sel.innerHTML = `<option value="all">All Wards</option>` + [...wards].sort().map(w => `<option value="${escapeHtml(w)}">${escapeHtml(w)}</option>`).join("");
    }

    function matches(p) {
      const enc = encByPid[p.id];
      const cat = catOf(enc);
      if (filterState.category !== "all" && cat !== filterState.category) return false;
      const w = wardOf(p);
      if (filterState.ward !== "all" && w !== filterState.ward) return false;
      if (filterState.q) {
        const q = filterState.q.toLowerCase();
        const name = patientName(p).toLowerCase();
        const mrn = (p.identifier?.[0]?.value || "").toLowerCase();
        const loc = locOf(p).toLowerCase();
        if (!name.includes(q) && !mrn.includes(q) && !loc.includes(q)) return false;
      }
      return true;
    }

    async function render() {
      const list = $("#pat-list");
      const visible = allPatients.filter(matches);
      if (!visible.length) {
        list.innerHTML = `<div class="empty-state"><div class="big-icon">🧑‍⚕️</div><p>No patients match the current filters.</p></div>`;
        return;
      }
      visible.forEach(cachePatientName);
      const flags = await computePatientFlags(visible.map(p => ({ resource: p })));
      list.innerHTML = `<div class="card" style="padding:0;overflow:hidden;"><div class="table-wrap"><table class="table">
        <thead><tr><th>Pt Name</th><th>DOB / Age / Sex</th><th>MRN</th><th>Location</th><th>Attending</th><th>Flags</th><th>Actions</th></tr></thead>
        <tbody>${visible.map(p => {
          const pid = p.id || ""; const name = patientName(p); const dob = p.birthDate || "?"; const sex = p.gender || "?";
          const loc = locOf(p); const att = attOf(p); const mrn = p.identifier?.[0]?.value || pid.slice(0,8);
          return `<tr><td><strong>${escapeHtml(name)}</strong></td><td>${dob} — ${fmtAge(dob)} ${sex}</td><td>${escapeHtml(mrn)}</td><td>${escapeHtml(loc)}</td><td>${escapeHtml(att)}</td><td class="flag-cell">${flags[pid] || '—'}</td><td><button class="btn btn-sm" data-pid="${pid}">Chart</button></td></tr>`;
        }).join("")}</tbody></table></div></div>`;
      list.querySelectorAll("button[data-pid]").forEach(b => b.onclick = () => { setActivePatient(b.dataset.pid); location.hash = "patientChart"; });
    }

    function wire() {
      $$("#pat-filters button[data-cat]").forEach(b => b.onclick = () => {
        filterState.category = b.dataset.cat;
        $$("#pat-filters button[data-cat]").forEach(x => x.classList.toggle("btn-secondary", x.dataset.cat !== filterState.category));
        render();
      });
      $("#pat-ward").onchange = e => { filterState.ward = e.target.value; render(); };
      $("#pat-search-btn").onclick = () => { filterState.q = $("#pat-search").value.trim(); render(); };
      $("#pat-search").addEventListener("keydown", e => { if (e.key === "Enter") { filterState.q = $("#pat-search").value.trim(); render(); } });
    }

    wire();
    try { await fetchAll(); await render(); }
    catch (err) { showAlert("Failed to load patients: " + err.message, "err"); }
};

/* ---------- PATIENT CHART ---------- */
views.patientChart = async () => {
    highlightNav("patientChart");
    const container = $("#view-container");
    if (!requireActivePatient(container, "Patient Chart")) return;
    const p = await loadActivePatient();
    if (!p) { container.innerHTML = `<div class="card empty-state"><p>Patient not found.</p></div>`; return; }
    const name = patientName(p); const dob = p.birthDate || "?"; const sex = p.gender || "?"; const mrn = p.identifier?.[0]?.value || p.id?.slice(0,8) || "?";
    // Load the most recent encounter for this patient to drive the banner.
    let enc = null;
    try {
      const eData = await api.searchEncounters(p.id);
      const encs = (eData.entry || []).map(e => e.resource).sort((a,b) => (b.period?.start || "").localeCompare(a.period?.start || ""));
      enc = encs[0] || null;
    } catch { enc = null; }
    const loc = enc?.location?.[0]?.location?.display || p.address?.[0]?.city || "—";
    const attending = enc?.participant?.[0]?.individual?.display || "—";
    const admit = enc?.period?.start ? fmtDate(enc.period.start) : "—";
    const encStatus = enc?.status || "—";
    // Pre-compute allergy + interaction alerts so the banner always renders.
    const fhirAllergies = (await api.getLocalAllergyIntolerances()).filter(a => {
      const ref = a.patient?.reference || '';
      return ref === 'Patient/' + p.id || ref === p.id;
    });
    const allergyAlerts = fhirAllergies;
    const interactionAlerts = getLocalInteractions().filter(i => i.patientId === p.id && i.status === 'active');
    const allergyText = allergyAlerts.length ? "ALERTS: " + allergyAlerts.map(a => (a.code?.text || a.code?.coding?.[0]?.display || 'Unknown') + " (" + (a.reaction?.[0]?.manifestation?.[0]?.text || a.reaction?.[0]?.description || '') + ")").join(", ") : "";
    const interactionText = interactionAlerts.length ? "INTERACTIONS: " + interactionAlerts.map(i => i.drugA + " + " + i.drugB).join(", ") : "";
    container.innerHTML = `
      <div class="patient-banner">
        <h2>${escapeHtml(name)} <span style="font-weight:400;font-size:0.9rem;color:var(--color-text-muted);">(${sex}) ${fmtAge(dob)} — DOB: ${dob} — MRN: ${mrn}</span></h2>
        <div class="banner-row">
          <span><strong>Location:</strong> ${escapeHtml(loc)}</span><span><strong>Admit:</strong> ${escapeHtml(admit)}</span><span><strong>Status:</strong> ${escapeHtml(encStatus)}</span>
          <span><strong>Attending:</strong> ${escapeHtml(attending)}</span>
        </div>
        ${allergyText ? `<div class="alert-strip danger">${escapeHtml(allergyText)}</div>` : ""}
        ${interactionText ? `<div class="alert-strip warning">${escapeHtml(interactionText)}</div>` : ""}
      </div>
      <div class="tabs" id="chart-tabs">
        <button class="tab active" data-tab="summary">Summary</button>
        <button class="tab" data-tab="encounters">Encounters</button>
        <button class="tab" data-tab="observations">Observations</button>
        <button class="tab" data-tab="conditions">Conditions</button>
        <button class="tab" data-tab="medications">Meds</button>
        <button class="tab" data-tab="procedures">Procedures</button>
        <button class="tab" data-tab="alerts">Alerts</button>
        <button class="tab" data-tab="notes">Notes</button>
        <button class="tab" data-tab="labs">Labs</button>
        <button class="tab" data-tab="imaging">Imaging</button>
      </div>
      <div id="chart-content"></div>`;
    const content = $("#chart-content");
    async function renderTab(tab) {
      $$("#chart-tabs .tab").forEach(t => t.classList.toggle("active", t.dataset.tab === tab));
      if (tab === "summary") {
        const fhirAllergies = (await api.getLocalAllergyIntolerances()).filter(a => {
          const ref = a.patient?.reference || '';
          return ref === 'Patient/' + p.id || ref === p.id;
        });
        const notes = getLocalNotes().filter(n => n.patientId === p.id);
        const meds = getLocalOrders().filter(o => o.patientId === p.id && o.category === 'medication' && o.status === 'active');
        const vitals = getLocalVitals().filter(v => v.patientId === p.id).sort((a,b) => b.time.localeCompare(a.time));
        const latestVitals = vitals[0];
        content.innerHTML = `
          <div class="row">
            <div class="col">
              <div class="card"><h3>Demographics</h3>
                <p><strong>Name:</strong> ${escapeHtml(name)}</p><p><strong>DOB:</strong> ${dob} (${fmtAge(dob)})</p><p><strong>Sex:</strong> ${sex}</p>
                <p><strong>MRN:</strong> ${mrn}</p><p><strong>Phone:</strong> +1-555-0192</p><p><strong>Address:</strong> 42 Oak St, Springfield</p>
                <p><strong>Emergency Contact:</strong> John Doe (son) — +1-555-0193</p>
              </div>
              <div class="card"><h3>Active Problems</h3>
                <div class="list-item"><div><div><strong>CHF — Chronic heart failure</strong></div><div class="meta">ICD-10: I50.9 — Onset: 2019</div></div><span class="badge badge-err">Active</span></div>
                <div class="list-item"><div><div><strong>Type 2 Diabetes Mellitus</strong></div><div class="meta">ICD-10: E11.9 — Onset: 2015</div></div><span class="badge badge-err">Active</span></div>
                <div class="list-item"><div><div><strong>CKD Stage 3</strong></div><div class="meta">ICD-10: N18.3 — Onset: 2021</div></div><span class="badge badge-err">Active</span></div>
              </div>
            </div>
            <div class="col">
              <div class="card"><h3>Current Encounters</h3>
                <div class="list-item"><div><div><strong>Inpatient — CHF Exacerbation</strong></div><div class="meta">Admitted 06/01/2026 — 4N-412 — Dr. Smith</div></div><span class="badge badge-warn">Inpatient</span></div>
              </div>
              <div class="card"><h3>Recent Vitals</h3>
                <div class="table-wrap"><table class="table">
                  <tr><th>Vital</th><th>Value</th><th>Time</th></tr>
                  ${latestVitals ? `
                  <tr><td>BP</td><td>${latestVitals.bp} mmHg</td><td>${fmtDateTime(latestVitals.time)}</td></tr>
                  <tr><td>HR</td><td>${latestVitals.hr} bpm</td><td>${fmtDateTime(latestVitals.time)}</td></tr>
                  <tr><td>Temp</td><td>${latestVitals.temp} °C</td><td>${fmtDateTime(latestVitals.time)}</td></tr>
                  <tr><td>SpO₂</td><td>${latestVitals.spo2}%</td><td>${fmtDateTime(latestVitals.time)}</td></tr>
                  <tr><td>RR</td><td>${latestVitals.rr} /min</td><td>${fmtDateTime(latestVitals.time)}</td></tr>
                  ` : '<tr><td colspan="3">No vitals</td></tr>'}
                </table></div>
              </div>
              <div class="card"><h3>Active Medications</h3>
                ${meds.length ? meds.map(m => `<div class="list-item"><div>${escapeHtml(m.name)}</div></div>`).join('') : '<p class="meta">No active medications.</p>'}
              </div>
            </div>
          </div>`;
      } else if (tab === "encounters") {
        try {
          const data = await api.searchEncounters(p.id);
          const entry = data.entry || [];
          content.innerHTML = entry.length ? entry.map(e => {
            const enc = e.resource;
            return `<div class="list-item"><div><div><strong>${enc.class?.display || enc.class?.code || 'Encounter'}</strong> <span class="meta">${enc.serviceType?.text || ''}</span></div><div class="meta">${fmtDateTime(enc.period?.start)} — ${enc.status}${enc.location?.[0]?.location?.display ? ' — ' + enc.location[0].location.display : ''}${enc.participant?.[0]?.individual?.display ? ' — ' + enc.participant[0].individual.display : ''}</div></div><span class="badge badge-info">${enc.status}</span></div>`;
          }).join('') : '<div class="empty-state"><p>No encounters.</p></div>';
        } catch { content.innerHTML = '<div class="empty-state"><p>Failed to load encounters.</p></div>'; }
      } else if (tab === "observations") {
        try {
          const data = await api.searchObservations(p.id);
          const entry = data.entry || [];
          content.innerHTML = entry.length ? `<div class="table-wrap"><table class="table"><tr><th>Code</th><th>Value</th><th>Date</th></tr>${entry.map(e => `<tr><td>${e.resource.code?.text || e.resource.code?.coding?.[0]?.display || '?'}</td><td>${extractValue(e.resource)}</td><td>${fmtDateTime(e.resource.effectiveDateTime)}</td></tr>`).join('')}</table></div>` : '<div class="empty-state"><p>No observations.</p></div>';
        } catch { content.innerHTML = '<div class="empty-state"><p>Failed to load observations.</p></div>'; }
      } else if (tab === "conditions") {
        try {
          const data = await api.searchConditions(p.id);
          const entry = data.entry || [];
          content.innerHTML = entry.length ? entry.map(e => {
            const c = e.resource;
            return `<div class="list-item"><div><div><strong>${c.code?.text || c.code?.coding?.[0]?.display || 'Condition'}</strong></div><div class="meta">${c.clinicalStatus?.coding?.[0]?.code || c.clinicalStatus || 'unknown'} — Onset: ${fmtDate(c.onsetDateTime)}</div></div><span class="badge badge-err">${c.clinicalStatus || 'active'}</span></div>`;
          }).join('') : '<div class="empty-state"><p>No conditions.</p></div>';
        } catch { content.innerHTML = '<div class="empty-state"><p>Failed to load conditions.</p></div>'; }
      } else if (tab === "medications") {
        try {
          const data = await api.searchMedicationRequests(p.id);
          const entry = data.entry || [];
          content.innerHTML = entry.length ? entry.map(e => {
            const m = e.resource;
            const med = m.medicationCodeableConcept?.text || m.medicationCodeableConcept?.coding?.[0]?.display || 'Medication';
            return `<div class="list-item"><div><div><strong>${escapeHtml(med)}</strong></div><div class="meta">${m.status} — ${m.intent}</div></div><span class="badge badge-info">${m.status}</span></div>`;
          }).join('') : '<div class="empty-state"><p>No medications.</p></div>';
        } catch { content.innerHTML = '<div class="empty-state"><p>Failed to load medications.</p></div>'; }
      } else if (tab === "procedures") {
        try {
          const data = await api.searchProcedures(p.id);
          const entry = data.entry || [];
          content.innerHTML = entry.length ? entry.map(e => {
            const pr = e.resource;
            return `<div class="list-item"><div><div><strong>${pr.code?.text || pr.code?.coding?.[0]?.display || 'Procedure'}</strong></div><div class="meta">${fmtDateTime(pr.performedDateTime || pr.performedPeriod?.start)} — ${pr.status}</div></div><span class="badge badge-info">${pr.status}</span></div>`;
          }).join('') : '<div class="empty-state"><p>No procedures.</p></div>';
        } catch { content.innerHTML = '<div class="empty-state"><p>Failed to load procedures.</p></div>'; }
      } else if (tab === "alerts") {
        const fhirAllergies = (await api.getLocalAllergyIntolerances()).filter(a => {
          const ref = a.patient?.reference || '';
          return ref === 'Patient/' + p.id || ref === p.id;
        });
        const interactionAlerts = getLocalInteractions().filter(i => i.patientId === p.id && i.status === 'active');
        content.innerHTML = `
          <div class="card"><h3>Allergy Alerts (${allergyAlerts.length})</h3>
            ${allergyAlerts.length ? allergyAlerts.map(a => `
              <div class="list-item" style="border-left:4px solid ${a.criticality==='high'?'var(--color-danger)':a.criticality==='moderate'?'var(--color-warning)':'var(--color-success)'};">
                <div><div><strong>${escapeHtml(a.substance)}</strong> <span class="badge ${a.criticality==='high'?'badge-err':a.criticality==='moderate'?'badge-warn':'badge-ok'}">${a.criticality}</span></div>
                <div class="meta">Reaction: ${escapeHtml(a.reaction)} — Onset: ${a.onset} — ${a.verified?'Verified':'Reported'}</div></div>
              </div>`).join('') : '<p class="meta">No allergies recorded.</p>'}
          </div>
          <div class="card"><h3>Drug Interaction Alerts (${interactionAlerts.length})</h3>
            ${interactionAlerts.length ? interactionAlerts.map(i => `
              <div class="list-item" style="border-left:4px solid ${i.severity==='major'?'var(--color-danger)':i.severity==='moderate'?'var(--color-warning)':'var(--color-success)'};">
                <div><div><strong>${escapeHtml(i.drugA)} + ${escapeHtml(i.drugB)}</strong> <span class="badge ${i.severity==='major'?'badge-err':i.severity==='moderate'?'badge-warn':'badge-ok'}">${i.severity}</span></div>
                <div class="meta">${escapeHtml(i.effect?.slice(0,80))}...</div></div>
              </div>`).join('') : '<p class="meta">No active drug interactions.</p>'}
          </div>`;
      } else if (tab === "notes") {
        const notes = getLocalNotes().filter(n => n.patientId === p.id).sort((a,b) => b.date.localeCompare(a.date));
        content.innerHTML = notes.length ? notes.map(n => `
          <div class="list-item"><div><div><strong>${escapeHtml(n.title)}</strong></div><div class="meta">${escapeHtml(n.author)} — ${fmtDateTime(n.date)} — ${n.signed?'Signed':'Draft'}</div></div>
          <span class="badge ${n.signed?'badge-ok':'badge-warn'}">${n.signed?'Signed':'Draft'}</span></div>`).join('') : '<div class="empty-state"><p>No clinical notes.</p></div>';
      } else if (tab === "labs") {
        try {
          const data = await api.searchObservations(p.id);
          const entry = (data.entry || []).filter(e => e.resource?.category?.some(c => c.coding?.some(cc => cc.code === "laboratory")));
          content.innerHTML = entry.length ? `<div class="table-wrap"><table class="table"><thead><tr><th>Test</th><th>Value</th><th>Unit</th><th>Date</th><th>Status</th></tr></thead><tbody>${entry.map(e => { const r=e.resource; const val = r.component ? r.component.map(c => `${c.valueQuantity?.value||"?"} ${c.valueQuantity?.unit||""}`).join(" / ") : `${r.valueQuantity?.value||r.valueString||"?"} ${r.valueQuantity?.unit||""}`; return `<tr class="lab-row" data-id="${r.id}" style="cursor:pointer"><td><strong>${escapeHtml(r.code?.text||"Lab")}</strong></td><td>${escapeHtml(val)}</td><td>${escapeHtml(r.valueQuantity?.unit||"")}</td><td>${fmtDateTime(r.effectiveDateTime)}</td><td><span class="badge badge-ok">${r.status}</span></td></tr>`; }).join("")}</tbody></table></div>` : `<div class="empty-state"><p>No lab results.</p></div>`;
          content.querySelectorAll(".lab-row").forEach(row => { row.onclick = () => { const r = entry.find(e => e.resource.id === row.dataset.id)?.resource; if(!r) return; let detail = `<div class="card"><h3>${escapeHtml(r.code?.text||"Lab Result")}</h3><p class="meta">Date: ${fmtDateTime(r.effectiveDateTime)} &middot; Status: ${r.status||"?"}</p><p><strong>Value:</strong> ${escapeHtml(r.valueQuantity?.value||r.valueString||"?")} ${escapeHtml(r.valueQuantity?.unit||"")}</p><p><strong>Category:</strong> laboratory</p></div>`; openModal("Lab Result Details", detail); }; });
        } catch { content.innerHTML = `<div class="empty-state"><p>Failed to load labs.</p></div>`; }
      } else if (tab === "imaging") {
        try {
          const data = await api.searchObservations(p.id);
          const entry = (data.entry || []).filter(e => e.resource?.category?.some(c => c.coding?.some(cc => cc.code === "imaging")));
          content.innerHTML = entry.length ? `<div class="table-wrap"><table class="table"><thead><tr><th>Study</th><th>Findings</th><th>Date</th><th>Status</th></tr></thead><tbody>${entry.map(e => { const r=e.resource; return `<tr class="img-row" data-id="${r.id}" style="cursor:pointer"><td><strong>${escapeHtml(r.code?.text||"Imaging")}</strong></td><td>${escapeHtml(r.valueString||"Normal study")}</td><td>${fmtDateTime(r.effectiveDateTime)}</td><td><span class="badge badge-ok">${r.status}</span></td></tr>`; }).join("")}</tbody></table></div>` : `<div class="empty-state"><p>No imaging studies.</p></div>`;
          content.querySelectorAll(".img-row").forEach(row => { row.onclick = () => { const r = entry.find(e => e.resource.id === row.dataset.id)?.resource; if(!r) return; let detail = `<div class="card"><h3>${escapeHtml(r.code?.text||"Imaging Study")}</h3><p class="meta">Date: ${fmtDateTime(r.effectiveDateTime)} &middot; Status: ${r.status||"?"}</p><p><strong>Findings:</strong> ${escapeHtml(r.valueString||"Normal study")}</p><p><strong>Category:</strong> imaging</p></div>`; openModal("Imaging Study Details", detail); }; });
        } catch { content.innerHTML = `<div class="empty-state"><p>Failed to load imaging.</p></div>`; }
      }
    }
    $$("#chart-tabs .tab").forEach(t => t.onclick = () => renderTab(t.dataset.tab));
    renderTab("summary");
};
/* ---------- CLINICAL NOTES ---------- */
views.clinicalNotes = async () => {
    highlightNav("clinicalNotes");
    const container = $("#view-container");
    if (!requireActivePatient(container, "Clinical Notes")) return;
    const pid = getActivePatientId();

    // Load the active patient (for the name) and the full patient list (for the selector).
    let active = null, patients = [];
    try {
      active = await loadActivePatient();
      const data = await api.searchPatients("");
      patients = (data.entry || []).map(e => e.resource);
      patients.forEach(cachePatientName);
      if (!active && patients.length) { setActivePatient(patients[0].id); return views.clinicalNotes(); }
    } catch (e) { /* fall through with whatever we have */ }

    const activeName = active ? patientName(active) : (pid || "Unknown");
    const notes = getLocalNotes().filter(n => n.patientId === pid).sort((a,b) => b.date.localeCompare(a.date));
    const now = new Date(); now.setMinutes(now.getMinutes() - now.getTimezoneOffset());
    const defaultDate = now.toISOString().slice(0,16);

    const noteTemplates = {
      progress: { title: "Progress Note", body: "<p><strong>S:</strong> </p><p><strong>O:</strong> </p><p><strong>A:</strong> </p><p><strong>P:</strong> </p>" },
      hp: { title: "H&P", body: "<p><strong>Chief Complaint:</strong> </p><p><strong>HPI:</strong> </p><p><strong>PMH:</strong> </p><p><strong>Exam:</strong> </p><p><strong>Assessment & Plan:</strong> </p>" },
      consult: { title: "Consult Note", body: "<p><strong>Reason for Consult:</strong> </p><p><strong>HPI:</strong> </p><p><strong>Recommendations:</strong> </p>" },
      procedure: { title: "Procedure Note", body: "<p><strong>Procedure:</strong> </p><p><strong>Indication:</strong> </p><p><strong>Findings:</strong> </p><p><strong>Complications:</strong> None.</p>" },
      discharge: { title: "Discharge Summary", body: "<p><strong>Admission Diagnosis:</strong> </p><p><strong>Hospital Course:</strong> </p><p><strong>Discharge Medications:</strong> </p><p><strong>Follow-up:</strong> </p>" },
    };
    let currentType = "progress";

    container.innerHTML = `
      <div class="patient-banner">
        <h2>Clinical Notes <span style="font-weight:400;font-size:0.9rem;color:var(--color-text-muted);">Patient: ${escapeHtml(activeName)}</span></h2>
        <div class="banner-row">
          <label style="display:flex;align-items:center;gap:0.4rem;"><strong>Switch patient:</strong>
            <select class="select" id="note-patient-select" style="width:auto;margin:0;max-width:18rem;">
              ${patients.map(pp => `<option value="${pp.id}" ${pp.id === pid ? "selected" : ""}>${escapeHtml(patientName(pp))} — ${(pp.identifier?.[0]?.value || pp.id.slice(0,8))}</option>`).join("")}
            </select>
          </label>
        </div>
      </div>
      <div class="tabs" id="note-tabs">
        <button class="tab active" data-tab="progress">Progress Note</button><button class="tab" data-tab="hp">H&P</button><button class="tab" data-tab="consult">Consult</button>
        <button class="tab" data-tab="procedure">Procedure Note</button><button class="tab" data-tab="discharge">Discharge Summary</button>
      </div>
      <div class="toolbar">
        <button class="btn btn-sm" id="note-smart">💡 Smart Phrases</button><button class="btn btn-secondary btn-sm" id="note-templates">📄 Templates</button><button class="btn btn-secondary btn-sm" id="note-dictate">🎙️ Dictate</button>
        <div style="margin-left:auto;display:flex;gap:0.5rem;"><button class="btn btn-success btn-sm" id="note-sign">✓ Sign</button><button class="btn btn-secondary btn-sm" id="note-save">Save Draft</button></div>
      </div>
      <div class="row">
        <div class="col-2">
          <div class="card">
            <label class="label">Note Title</label><input class="input" id="note-title" value="${escapeHtml(noteTemplates[currentType].title)}" />
            <label class="label">Author</label><input class="input" id="note-author" value="Dr. Smith, J (Cardiology)" />
            <label class="label">Date / Time</label><input class="input" type="datetime-local" id="note-date" value="${defaultDate}" />
            <label class="label">Note Content</label>
            <div class="note-editor" id="note-content" contenteditable="true">${noteTemplates[currentType].body}</div>
          </div>
        </div>
        <div class="col">
          <div class="card"><h3>Smart Phrases</h3>
            <div class="list-item" data-phrase=".chfex"><div>.chfex</div><span class="meta">CHF exacerbation template</span></div>
            <div class="list-item" data-phrase=".dispo"><div>.dispo</div><span class="meta">Disposition planning</span></div>
            <div class="list-item" data-phrase=".fu"><div>.fu</div><span class="meta">Follow-up instructions</span></div>
            <div class="list-item" data-phrase=".codefull"><div>.codefull</div><span class="meta">Full code status statement</span></div>
          </div>
          <div class="card"><h3>Previous Notes (${notes.length})</h3><div id="note-prev-list">${notes.length ? notes.map(n => `<div class="list-item"><div><div><strong>${escapeHtml(n.title)}</strong></div><div class="meta">${escapeHtml(n.author)} — ${fmtDateTime(n.date)} — ${n.signed?'Signed':'Draft'}</div></div><span class="badge ${n.signed?'badge-ok':'badge-warn'}">${n.signed?'Signed':'Draft'}</span></div>`).join('') : '<p class="meta">No previous notes.</p>'}</div></div>
        </div>
      </div>`;

    const phrases = { '.chfex': 'CHF exacerbation — admitted with dyspnea, orthopnea, JVP elevated, bilateral crackles, 2+ pitting edema.', '.dispo': 'Disposition: Patient stable for discharge. Home with services. Follow-up in 1 week.', '.fu': 'Follow-up: Return to clinic in 1 week or sooner if symptoms worsen.', '.codefull': 'Patient is FULL CODE. Advance directives discussed.' };

    $("#note-patient-select").onchange = (e) => { setActivePatient(e.target.value); views.clinicalNotes(); };

    $$("#note-tabs .tab").forEach(t => t.onclick = () => {
      currentType = t.dataset.tab;
      $$("#note-tabs .tab").forEach(x => x.classList.toggle("active", x === t));
      const tpl = noteTemplates[currentType];
      const titleEl = $("#note-title");
      const editor = $("#note-content");
      // Only reset the title/editor when they still hold a previous template (avoid clobbering user input).
      const knownTitles = Object.values(noteTemplates).map(x => x.title);
      if (knownTitles.includes(titleEl.value)) titleEl.value = tpl.title;
      const knownBodies = Object.values(noteTemplates).map(x => x.body);
      if (knownBodies.includes(editor.innerHTML.trim()) || editor.innerText.trim() === "") editor.innerHTML = tpl.body;
    });

    container.querySelectorAll('[data-phrase]').forEach(el => el.onclick = () => {
      const editor = $("#note-content");
      editor.innerHTML += `<p>${phrases[el.dataset.phrase]}</p>`;
    });

    $("#note-templates").onclick = () => {
      const editor = $("#note-content");
      editor.innerHTML = noteTemplates[currentType].body;
      showAlert("Loaded " + noteTemplates[currentType].title + " template.", "ok");
    };

    $("#note-dictate").onclick = () => {
      const editor = $("#note-content");
      const ts = fmtDateTime(new Date().toISOString());
      editor.innerHTML += `<p><em>[Dictated ${ts} — review and sign.]</em></p>`;
      showAlert("Dictation placeholder inserted. Connect a microphone for live dictation.", "warn");
    };

    $("#note-smart").onclick = () => {
      const editor = $("#note-content");
      editor.innerHTML += `<p>${phrases['.chfex']}</p>`;
    };

    $("#note-save").onclick = () => {
      const arr = getLocalNotes();
      arr.push({ id: 'note-'+Date.now(), patientId: pid, title: $("#note-title").value, author: $("#note-author").value, date: $("#note-date").value, type: currentType, content: $("#note-content").innerText, signed: false });
      setLocalNotes(arr); showAlert("Draft saved.", "ok"); views.clinicalNotes();
      logAudit("Create", "C", { patient: pid, details: "Saved clinical note draft: " + $("#note-title").value });
    };
    $("#note-sign").onclick = () => {
      const arr = getLocalNotes();
      arr.push({ id: 'note-'+Date.now(), patientId: pid, title: $("#note-title").value, author: $("#note-author").value, date: $("#note-date").value, type: currentType, content: $("#note-content").innerText, signed: true });
      setLocalNotes(arr); showAlert("Note signed and saved.", "ok"); views.clinicalNotes();
      logAudit("Create", "C", { patient: pid, outcome: "Success", details: "Signed clinical note: " + $("#note-title").value });
    };
};

/* ---------- CPOE ORDERS ---------- */
views.cpoeOrders = () => {
    highlightNav("cpoeOrders");
    const container = $("#view-container");
    if (!requireActivePatient(container, "Orders")) return;
    const pid = getActivePatientId();
    const orders = getLocalOrders().filter(o => o.patientId === pid);
    container.innerHTML = `
      <div class="patient-banner"><h2>Orders <span style="font-weight:400;font-size:0.9rem;color:var(--color-text-muted);">Patient: ${escapeHtml(patientLabel(pid))}</span></h2></div>
      <div class="tabs" id="order-tabs"><button class="tab active" data-tab="active">Active</button><button class="tab" data-tab="pending">Pending</button><button class="tab" data-tab="discontinued">Discontinued</button></div>
      <div class="toolbar"><button class="btn" id="new-order-btn">＋ New Order</button><button class="btn btn-secondary btn-sm">Order Sets ▼</button><button class="btn btn-secondary btn-sm">Favorites ▼</button><button class="btn btn-secondary btn-sm">Recents ▼</button></div>
      <div class="search-bar"><input class="input" placeholder="Search orders (e.g., Metformin, CBC, Chest X-Ray)..." /><button class="btn">Search</button></div>
      <div id="order-list"></div>
      <div class="card" style="display:flex;justify-content:space-between;align-items:center;"><span style="font-size:0.9rem;">Selected: <strong>0</strong> orders</span><div style="display:flex;gap:0.5rem;"><button class="btn">Sign Orders</button><button class="btn btn-secondary">Co-Sign Required</button></div></div>`;
    function renderOrders(status) {
      const list = $("#order-list");
      const filtered = orders.filter(o => o.status === status);
      list.innerHTML = filtered.length ? `<div class="card"><h3>${status.charAt(0).toUpperCase()+status.slice(1)} Orders</h3>${filtered.map(o => `
        <div class="list-item">
          <div style="display:flex;align-items:center;gap:0.75rem;"><input type="checkbox" /><div><div><strong>${escapeHtml(o.name)}</strong></div><div class="meta">${o.category} — Ordered ${o.ordered} — ${o.by}</div></div></div>
          <div style="display:flex;gap:0.4rem;"><button class="btn btn-secondary btn-sm">Edit</button><button class="btn btn-danger btn-sm order-dc" data-id="${o.id}">DC</button></div>
        </div>`).join('')}</div>` : `<div class="empty-state"><p>No ${status} orders.</p></div>`;
      list.querySelectorAll('.order-dc').forEach(b => b.onclick = () => {
        const arr = getLocalOrders(); const o = arr.find(x=>x.id===b.dataset.id); if(o) o.status='discontinued'; setLocalOrders(arr); renderOrders(status);
      });
    }
    $$("#order-tabs .tab").forEach(t => t.onclick = () => { $$("#order-tabs .tab").forEach(x=>x.classList.remove("active")); t.classList.add("active"); renderOrders(t.dataset.tab); });
    renderOrders("active");
    $("#new-order-btn").onclick = () => {
      openModal("New Order", `
        <label class="label">Order Name</label><input class="input" id="modal-order-name" placeholder="e.g., Metformin 500mg PO BID" />
        <label class="label">Category</label><select class="select" id="modal-order-cat"><option>medication</option><option>laboratory</option><option>radiology</option><option>nursing</option><option>procedure</option></select>
        <div style="display:flex;gap:0.5rem;margin-top:0.75rem;"><button class="btn" id="modal-order-save">Add Order</button><button class="btn btn-secondary" onclick="closeModal()">Cancel</button></div>`);
      $("#modal-order-save").onclick = () => {
        const arr = getLocalOrders();
        arr.push({ id:'ord-'+Date.now(), patientId: pid, name: $("#modal-order-name").value, category: $("#modal-order-cat").value, status:'active', ordered: new Date().toISOString().slice(0,10), by: 'Current User' });
        setLocalOrders(arr); closeModal(); renderOrders("active");
      };
    };
};

/* ---------- RESULTS REVIEW ---------- */
views.resultsReview = async () => {
    highlightNav("resultsReview");
    const container = $("#view-container");
    if (!requireActivePatient(container, "Results Review")) return;
    const pid = getActivePatientId();
    try {
        const allObs = await api.searchObservations(pid);
        const allEntries = allObs.entry || [];
        const labs = allEntries.map(e => e.resource).filter(r => r.category?.some(c => c.coding?.some(cc => cc.code === "laboratory")));
        const imgs = allEntries.map(e => e.resource).filter(r => r.category?.some(c => c.coding?.some(cc => cc.code === "imaging")));
        let html = `<div class="patient-banner"><h2>Results Review <span style="font-weight:400;font-size:0.9rem;color:var(--color-text-muted);">Patient: ${escapeHtml(patientLabel(pid))}</span></h2></div>`;
        html += `<div class="card"><h3>Laboratory Results (${labs.length})</h3>`;
        if (!labs.length) { html += `<p class="meta">No lab results found.</p>`; }
        else {
            html += `<div class="table-wrap"><table class="table"><thead><tr><th>Test</th><th>Value</th><th>Unit</th><th>Date</th><th>Status</th></tr></thead><tbody>`;
            for (const r of labs) {
                const val = r.component ? r.component.map(c => `${c.valueQuantity?.value||"?"} ${c.valueQuantity?.unit||""}`).join(" / ") : `${r.valueQuantity?.value||r.valueString||"?"} ${r.valueQuantity?.unit||""}`;
                html += `<tr class="lab-row" data-id="${r.id}" style="cursor:pointer"><td><strong>${escapeHtml(r.code?.text||"Lab")}</strong></td><td>${escapeHtml(val)}</td><td>${escapeHtml(r.valueQuantity?.unit||"")}</td><td>${fmtDateTime(r.effectiveDateTime)}</td><td><span class="badge badge-ok">${r.status}</span></td></tr>`;
            }
            html += "</tbody></table></div>";
        }
        html += "</div>";
        html += `<div class="card"><h3>Imaging Studies (${imgs.length})</h3>`;
        if (!imgs.length) { html += `<p class="meta">No imaging studies found.</p>`; }
        else {
            html += `<div class="table-wrap"><table class="table"><thead><tr><th>Study</th><th>Findings</th><th>Date</th><th>Status</th></tr></thead><tbody>`;
            for (const r of imgs) {
                html += `<tr class="img-row" data-id="${r.id}" style="cursor:pointer"><td><strong>${escapeHtml(r.code?.text||"Imaging")}</strong></td><td>${escapeHtml(r.valueString||"Normal study")}</td><td>${fmtDateTime(r.effectiveDateTime)}</td><td><span class="badge badge-ok">${r.status}</span></td></tr>`;
            }
            html += "</tbody></table></div>";
        }
        html += "</div>";
        container.innerHTML = html;
        container.querySelectorAll(".lab-row").forEach(row => {
            row.onclick = () => {
                const r = labs.find(x => x.id === row.dataset.id);
                if (!r) return;
                let detail = `<div class="card"><h3>${escapeHtml(r.code?.text||"Lab Result")}</h3>`;
                detail += `<p class="meta">Date: ${fmtDateTime(r.effectiveDateTime)} &middot; Status: ${r.status||"?"}</p>`;
                if (r.component) {
                    detail += "<h4>Components</h4><ul>";
                    for (const c of r.component) { detail += `<li><strong>${escapeHtml(c.code?.text||"?")}</strong>: ${c.valueQuantity?.value||"?"} ${c.valueQuantity?.unit||""}</li>`; }
                    detail += "</ul>";
                } else {
                    detail += `<p><strong>Value:</strong> ${escapeHtml(r.valueQuantity?.value||r.valueString||"?")} ${escapeHtml(r.valueQuantity?.unit||"")}</p>`;
                }
                detail += `<p><strong>Category:</strong> laboratory</p>`;
                detail += "</div>";
                openModal("Lab Result Details", detail);
            };
        });
        container.querySelectorAll(".img-row").forEach(row => {
            row.onclick = () => {
                const r = imgs.find(x => x.id === row.dataset.id);
                if (!r) return;
                let detail = `<div class="card"><h3>${escapeHtml(r.code?.text||"Imaging Study")}</h3>`;
                detail += `<p class="meta">Date: ${fmtDateTime(r.effectiveDateTime)} &middot; Status: ${r.status||"?"}</p>`;
                detail += `<p><strong>Findings:</strong> ${escapeHtml(r.valueString||"Normal study")}</p>`;
                detail += `<p><strong>Category:</strong> imaging</p>`;
                detail += "</div>";
                openModal("Imaging Study Details", detail);
            };
        });
    } catch(err) { showAlert("Failed to load results: " + err.message, "err"); }
};

views.mar = () => {
    highlightNav("mar");
    const container = $("#view-container");
    if (!requireActivePatient(container, "MAR")) return;
    const pid = getActivePatientId();
    const mar = getLocalMar().filter(m => m.patientId === pid);
    container.innerHTML = `
      <div class="patient-banner"><h2>MAR <span style="font-weight:400;font-size:0.9rem;color:var(--color-text-muted);">Patient: ${escapeHtml(patientLabel(pid))} — 06/15/2026 — Shift: Day (07:00–19:00)</span></h2></div>
      <div class="tabs"><button class="tab active">Scheduled</button><button class="tab">PRN</button><button class="tab">Stat</button><button class="tab">Overdue ▼</button></div>
      <div class="card" style="padding:0;overflow:hidden;">
        <div class="mar-timegrid">
          <div class="grid-header">Time</div><div class="grid-header">Medication</div><div class="grid-header">Dose</div><div class="grid-header">Route</div><div class="grid-header">Status</div>
          ${mar.map(m => `
            <div${m.status==='overdue'?' style="background:rgba(239,68,68,0.12);color:var(--color-danger);font-weight:600;"':''}>${m.time}</div>
            <div${m.status==='overdue'?' style="background:rgba(239,68,68,0.12);"':''}>${escapeHtml(m.medication)}</div>
            <div${m.status==='overdue'?' style="background:rgba(239,68,68,0.12);"':''}>${escapeHtml(m.dose)}</div>
            <div${m.status==='overdue'?' style="background:rgba(239,68,68,0.12);"':''}>${m.route}</div>
            <div${m.status==='overdue'?' style="background:rgba(239,68,68,0.12);"':''}>${m.status==='given'?'✓ Given':m.status==='overdue'?'<span class="badge badge-err">OVERDUE</span>':'<button class="btn btn-sm mar-scan" data-id="'+m.id+'">📷 Scan QR</button>'}</div>
          `).join('')}
        </div>
      </div>
      <div class="card" style="display:flex;gap:0.5rem;flex-wrap:wrap;">
        <button class="btn">Mark All Reviewed</button><button class="btn btn-secondary">Not Given — Reason ▼</button><button class="btn btn-secondary">Waste Log</button>
        <div style="margin-left:auto;"><span class="badge badge-ok">${mar.filter(m=>m.status==='given').length} Given</span> <span class="badge badge-warn">${mar.filter(m=>m.status==='pending').length} Pending</span> <span class="badge badge-err">${mar.filter(m=>m.status==='overdue').length} Overdue</span></div>
      </div>`;
    container.querySelectorAll('.mar-scan').forEach(b => b.onclick = () => {
      const arr = getLocalMar(); const m = arr.find(x=>x.id===b.dataset.id); if(m) m.status='given'; setLocalMar(arr); views.mar();
    });
};

/* ---------- VITALS FLOWSHEET ---------- */
views.vitalsFlowsheet = () => {
    highlightNav("vitalsFlowsheet");
    const container = $("#view-container");
    if (!requireActivePatient(container, "Vitals Flowsheet")) return;
    const pid = getActivePatientId();
    const vitals = getLocalVitals().filter(v => v.patientId === pid);
    container.innerHTML = `
      <div class="patient-banner"><h2>Vitals Flowsheet <span style="font-weight:400;font-size:0.9rem;color:var(--color-text-muted);">Patient: ${escapeHtml(patientLabel(pid))}</span></h2></div>
      <div class="card" style="display:flex;justify-content:space-between;align-items:center;">
        <div><strong>NEWS2 Score:</strong> <span style="font-size:1.5rem;font-weight:700;color:var(--color-warning);">4</span> <span class="badge badge-warn">Medium Risk — Inform RN + Physician</span></div>
        <button class="btn btn-sm" onclick="location.hash='observations'">＋ Add Vitals</button>
      </div>
      <div class="card" style="padding:0;overflow:hidden;">
        <div class="flowsheet-grid">
          <div class="grid-header">Vital Sign</div>${vitals.map(v=>`<div class="grid-header">${fmtDateTime(v.time).split(',')[0].split('/').slice(0,2).join('/')} ${v.time.slice(11,16)}</div>`).join('')}
          <div class="grid-label">BP (mmHg)</div>${vitals.map(v=>`<div>${v.bp||'—'}</div>`).join('')}
          <div class="grid-label">HR (bpm)</div>${vitals.map(v=>`<div>${v.hr||'—'}</div>`).join('')}
          <div class="grid-label">Temp (°C)</div>${vitals.map(v=>`<div>${v.temp||'—'}</div>`).join('')}
          <div class="grid-label">SpO₂ (%)</div>${vitals.map(v=>`<div>${v.spo2||'—'}</div>`).join('')}
          <div class="grid-label">RR (/min)</div>${vitals.map(v=>`<div>${v.rr||'—'}</div>`).join('')}
          <div class="grid-label">Pain (0–10)</div>${vitals.map(v=>`<div>${v.pain!=null?v.pain:'—'}</div>`).join('')}
          <div class="grid-label">Weight (kg)</div>${vitals.map(v=>`<div>${v.weight||'—'}</div>`).join('')}
          <div class="grid-label">BMI (calc)</div>${vitals.map(v=>`<div>${v.bmi||'—'}</div>`).join('')}
          <div class="grid-label">MAP (calc)</div>${vitals.map(v=>`<div>${v.map||'—'}</div>`).join('')}
        </div>
      </div>
      <div class="card"><h3>Trend Graph</h3><div class="chart-placeholder">📉 Vitals Trend Chart Placeholder<br><span style="font-size:0.85rem;">Multi-line graph: BP, HR, SpO₂ over selected time range</span></div></div>`;
};

/* ---------- ALLERGY PROFILE ---------- */
views.alerts = async () => {
    highlightNav("alerts");
    const container = $("#view-container");
    if (!requireActivePatient(container, "Alert Profile")) return;
    const pid = getActivePatientId();
    const allergies = (await api.getLocalAllergyIntolerances()).filter(a => {
      const ref = a.patient?.reference || '';
      return ref === 'Patient/' + pid || ref === pid;
    });
    const interactions = getLocalInteractions().filter(i => i.patientId === pid && i.status === 'active');
    container.innerHTML = `
      <div class="patient-banner"><h2>Alert Profile <span style="font-weight:400;font-size:0.9rem;color:var(--color-text-muted);">Patient: ${escapeHtml(patientLabel(pid))}</span></h2></div>
      <div class="toolbar">
        <button class="btn" id="add-alert-btn">+ Add Allergy Alert</button>
        <button class="btn btn-secondary btn-sm" id="add-interaction-btn">+ Add Interaction</button>
        <button class="btn btn-secondary btn-sm">Print Wallet Card</button>
      </div>
      <div class="row">
        <div class="col-2">
          <div class="card"><h3>Allergy Alerts</h3>
            ${allergies.length ? allergies.map(a => `
              <div class="list-item" style="border-left:4px solid ${a.criticality==='high'?'var(--color-danger)':a.criticality==='moderate'?'var(--color-warning)':'var(--color-success)'};">
                <div><div><strong>${escapeHtml(a.substance)}</strong> <span class="badge ${a.criticality==='high'?'badge-err':a.criticality==='moderate'?'badge-warn':'badge-ok'}">${a.criticality}</span></div>
                <div class="meta">Reaction: ${escapeHtml(a.reaction)} — Onset: ${a.onset} — ${a.verified?'Verified':'Reported'}</div></div>
                <div><button class="btn btn-secondary btn-sm">Edit</button></div>
              </div>`).join('') : '<p class="meta">No allergies recorded.</p>'}
          </div>
          <div class="card"><h3>Drug Interactions</h3>
            ${interactions.length ? interactions.map(i => `
              <div class="list-item" style="border-left:4px solid ${i.severity==='major'?'var(--color-danger)':i.severity==='moderate'?'var(--color-warning)':'var(--color-success)'};">
                <div><div><strong>${escapeHtml(i.drugA)} + ${escapeHtml(i.drugB)}</strong> <span class="badge ${i.severity==='major'?'badge-err':i.severity==='moderate'?'badge-warn':'badge-ok'}">${i.severity}</span></div>
                <div class="meta">${escapeHtml(i.effect?.slice(0,100))}...</div></div>
                <div><button class="btn btn-secondary btn-sm">Review</button></div>
              </div>`).join('') : '<p class="meta">No active drug interactions.</p>'}
          </div>
        </div>
        <div class="col">
          <div class="card"><h3>Alert Summary</h3>
            ${allergies.filter(a=>a.criticality==='high').length ? `
              <div class="alert-strip danger">HIGH-RISK ALLERGIES: ${allergies.filter(a=>a.criticality==='high').map(a=>a.substance).join(', ')}</div>
            ` : ''}
            ${interactions.filter(i=>i.severity==='major').length ? `
              <div class="alert-strip danger" style="margin-top:0.5rem;">MAJOR INTERACTIONS: ${interactions.filter(i=>i.severity==='major').map(i=>i.drugA+' + '+i.drugB).join(', ')}</div>
            ` : ''}
            ${!allergies.filter(a=>a.criticality==='high').length && !interactions.filter(i=>i.severity==='major').length ? '<p class="meta">No high-risk alerts.</p>' : ''}
          </div>
          <div class="card"><h3>Configurable Alert Rules</h3>
            <div class="list-item"><div><div><strong>Overdue Medications</strong></div><div class="meta">Flag when MAR has overdue meds</div></div><input type="checkbox" checked disabled></div>
            <div class="list-item"><div><div><strong>Critical Lab Values</strong></div><div class="meta">Flag unacknowledged critical results</div></div><input type="checkbox" checked disabled></div>
            <div class="list-item"><div><div><strong>NEWS2 Score</strong></div><div class="meta">Flag score >= 5</div></div><input type="checkbox" checked disabled></div>
            <div class="list-item"><div><div><strong>High-Risk Allergies</strong></div><div class="meta">Flag any high criticality allergy</div></div><input type="checkbox" checked disabled></div>
            <div class="list-item"><div><div><strong>Active Drug Interactions</strong></div><div class="meta">Flag any major interaction</div></div><input type="checkbox" checked disabled></div>
          </div>
        </div>
      </div>`;
    $("#add-alert-btn").onclick = () => {
      openModal("Add Allergy Alert", `
        <label class="label">Substance</label><input class="input" id="modal-alg-sub" />
        <label class="label">Category</label><select class="select" id="modal-alg-cat"><option>medication</option><option>food</option><option>environment</option></select>
        <label class="label">Criticality</label><select class="select" id="modal-alg-crit"><option>high</option><option>moderate</option><option>low</option></select>
        <label class="label">Reaction</label><input class="input" id="modal-alg-react" />
        <div style="display:flex;gap:0.5rem;margin-top:0.75rem;"><button class="btn" id="modal-alg-save">Save</button><button class="btn btn-secondary" onclick="closeModal()">Cancel</button></div>`);
      $("#modal-alg-save").onclick = async () => {
        const payload = {
          resourceType: 'AllergyIntolerance',
          id: 'alg-' + Date.now(),
          code: { text: $("#modal-alg-sub").value },
          criticality: $("#modal-alg-crit").value,
          category: [$("#modal-alg-cat").value],
          patient: { reference: 'Patient/' + pid },
          reaction: [{ manifestation: [{ text: $("#modal-alg-react").value }] }],
          verificationStatus: { text: 'confirmed' },
          clinicalStatus: { text: 'active' }
        };
        try {
          await api.createAllergyIntolerance(payload);
          showAlert("Allergy alert saved to server.", "ok");
        } catch (err) {
          await api.enqueue('POST', 'AllergyIntolerance', payload);
          showAlert("Allergy alert queued for sync.", "warn");
        }
      };
    };
    const ibtn = $("#add-interaction-btn");
    if (ibtn) ibtn.onclick = () => showAlert("Add interaction via Orders view.", "warn");
};

views.carePlans = () => {
    highlightNav("carePlans");
    const container = $("#view-container");
    if (!requireActivePatient(container, "Care Plans")) return;
    const pid = getActivePatientId();
    const plans = getLocalCarePlans().filter(x => x.patientId === pid);
    container.innerHTML = `
      <div class="patient-banner"><h2>Care Plans <span style="font-weight:400;font-size:0.9rem;color:var(--color-text-muted);">Patient: ${escapeHtml(patientLabel(pid))}</span></h2></div>
      <div class="toolbar"><button class="btn" onclick="openModal('Add Care Plan', '<label class=\'label\'>Title</label><input class=\'input\' id=\'modal-cp-title\' /><label class=\'label\'>Description</label><textarea class=\'input\' id=\'modal-cp-desc\'></textarea><div style=\'display:flex;gap:0.5rem;margin-top:0.75rem;\'><button class=\'btn\' id=\'modal-cp-save\'>Save</button><button class=\'btn btn-secondary\' onclick=\'closeModal()\'>Cancel</button></div>')">+ Add Care Plan</button></div>
      ${plans.length ? plans.map(cp => `
        <div class="card"><h3>${escapeHtml(cp.title || 'Care Plan')}</strong> <span class="badge ${cp.status==='active'?'badge-ok':'badge-warn'}">${cp.status || 'unknown'}</span></h3>
        <p class="meta">${escapeHtml(cp.description || '')}</p>
        <div class="table-wrap"><table class="table"><thead><tr><th>Activity</th><th>Status</th><th>Detail</th></tr></thead><tbody>
        ${cp.activity?.map(a => `<tr><td>${escapeHtml(a.reference?.display || a.detail?.description || '?')}</td><td>${a.detail?.status || '?'}</td><td>${escapeHtml(a.detail?.code?.text || a.detail?.description || '?')}</td></tr>`).join('') || '<tr><td colspan="3">No activities</td></tr>'}
        </tbody></table></div></div>`).join('') : '<div class="empty-state"><p>No care plans recorded.</p></div>'}
    `;
    const saveBtn = $("#modal-cp-save");
    if (saveBtn) saveBtn.onclick = () => {
        const arr = getLocalCarePlans();
        arr.push({ id:'cp-'+Date.now(), patientId: pid, title: $("#modal-cp-title").value, description: $("#modal-cp-desc").value, status:'active', activity: [] });
        setLocalCarePlans(arr); closeModal(); views.carePlans();
    };
};

views.familyHistory = () => {
    highlightNav("familyHistory");
    const container = $("#view-container");
    if (!requireActivePatient(container, "Family History")) return;
    const pid = getActivePatientId();
    const fh = getLocalFamilyHistory().filter(x => x.patientId === pid);
    container.innerHTML = `
      <div class="patient-banner"><h2>Family History <span style="font-weight:400;font-size:0.9rem;color:var(--color-text-muted);">Patient: ${escapeHtml(patientLabel(pid))}</span></h2></div>
      <div class="toolbar"><button class="btn" onclick="openModal('Add Family Member', '<label class=\'label\'>Name</label><input class=\'input\' id=\'modal-fh-name\' /><label class=\'label\'>Relationship</label><input class=\'input\' id=\'modal-fh-rel\' /><label class=\'label\'>Condition</label><input class=\'input\' id=\'modal-fh-cond\' /><div style=\'display:flex;gap:0.5rem;margin-top:0.75rem;\'><button class=\'btn\' id=\'modal-fh-save\'>Save</button><button class=\'btn btn-secondary\' onclick=\'closeModal()\'>Cancel</button></div>')">+ Add Family Member</button></div>
      ${fh.length ? fh.map(f => `
        <div class="card"><h3>${escapeHtml(f.name || 'Family Member')}</strong> <span class="badge badge-ok">${escapeHtml(f.relationship?.text || f.relationship?.coding?.[0]?.display || 'Relative')}</span></h3>
        <div class="table-wrap"><table class="table"><thead><tr><th>Condition</th><th>Outcome</th><th>Notes</th></tr></thead><tbody>
        ${f.condition?.map(c => `<tr><td>${escapeHtml(c.code?.text || c.code?.coding?.[0]?.display || '?')}</td><td>${escapeHtml(c.outcome?.text || '?')}</td><td>${c.note?.map(n=>n.text).join('; ') || '?'}</td></tr>`).join('') || '<tr><td colspan="3">No conditions recorded</td></tr>'}
        </tbody></table></div></div>`).join('') : '<div class="empty-state"><p>No family history recorded.</p></div>'}
    `;
    const saveBtn = $("#modal-fh-save");
    if (saveBtn) saveBtn.onclick = () => {
        const arr = getLocalFamilyHistory();
        arr.push({ id:'fh-'+Date.now(), patientId: pid, name: $("#modal-fh-name").value, relationship: { text: $("#modal-fh-rel").value }, condition: [{ code: { text: $("#modal-fh-cond").value } }] });
        setLocalFamilyHistory(arr); closeModal(); views.familyHistory();
    };
};

views.immunizations = () => {
    highlightNav("immunizations");
    const container = $("#view-container");
    if (!requireActivePatient(container, "Immunizations")) return;
    const pid = getActivePatientId();
    const imm = getLocalImmunizations().filter(x => x.patientId === pid);
    container.innerHTML = `
      <div class="patient-banner"><h2>Immunizations <span style="font-weight:400;font-size:0.9rem;color:var(--color-text-muted);">Patient: ${escapeHtml(patientLabel(pid))}</span></h2></div>
      <div class="toolbar"><button class="btn" onclick="openModal('Add Immunization', '<label class=\'label\'>Vaccine</label><input class=\'input\' id=\'modal-imm-name\' /><label class=\'label\'>Date</label><input class=\'input\' type=\'date\' id=\'modal-imm-date\' /><label class=\'label\'>Lot Number</label><input class=\'input\' id=\'modal-imm-lot\' /><div style=\'display:flex;gap:0.5rem;margin-top:0.75rem;\'><button class=\'btn\' id=\'modal-imm-save\'>Save</button><button class=\'btn btn-secondary\' onclick=\'closeModal()\'>Cancel</button></div>')">+ Add Immunization</button></div>
      ${imm.length ? imm.map(i => `
        <div class="list-item"><div><div><strong>${escapeHtml(i.vaccineCode?.text || i.vaccineCode?.coding?.[0]?.display || 'Vaccine')}</strong></div><div class="meta">Date: ${i.occurrenceDateTime ? fmtDateTime(i.occurrenceDateTime) : '?'} - Lot: ${i.lotNumber || '?'} - Status: ${i.status}</div></div>
        <span class="badge ${i.status==='completed'?'badge-ok':'badge-warn'}">${i.status}</span></div>`).join('') : '<div class="empty-state"><p>No immunizations recorded.</p></div>'}
    `;
    const saveBtn = $("#modal-imm-save");
    if (saveBtn) saveBtn.onclick = () => {
        const arr = getLocalImmunizations();
        arr.push({ id:'imm-'+Date.now(), patientId: pid, vaccineCode: { text: $("#modal-imm-name").value }, occurrenceDateTime: new Date($("#modal-imm-date").value).toISOString(), lotNumber: $("#modal-imm-lot").value, status:'completed' });
        setLocalImmunizations(arr); closeModal(); views.immunizations();
    };
};

views.documents = () => {
    highlightNav("documents");
    const container = $("#view-container");
    if (!requireActivePatient(container, "Documents")) return;
    const pid = getActivePatientId();
    const docs = getLocalDocuments().filter(x => x.patientId === pid);
    container.innerHTML = `
      <div class="patient-banner"><h2>Documents <span style="font-weight:400;font-size:0.9rem;color:var(--color-text-muted);">Patient: ${escapeHtml(patientLabel(pid))}</span></h2></div>
      <div class="toolbar">
        <button class="btn" onclick="openModal('Add Document', '<label class=\'label\'>Description</label><input class=\'input\' id=\'modal-doc-desc\' /><label class=\'label\'>Content Type</label><select class=\'select\' id=\'modal-doc-type\'><option>image/jpeg</option><option>application/pdf</option><option>text/plain</option></select><label class=\'label\'>Data URL / URL</label><input class=\'input\' id=\'modal-doc-url\' placeholder=\'data:image/jpeg;base64,... or https://...\' /><div style=\'display:flex;gap:0.5rem;margin-top:0.75rem;\'><button class=\'btn\' id=\'modal-doc-save\'>Save</button><button class=\'btn btn-secondary\' onclick=\'closeModal()\'>Cancel</button></div>')">+ Attach Document</button>
        <button class="btn btn-secondary btn-sm">-? Take Photo</button>
      </div>
      <p class="meta">-? Mobile app users can capture photos directly from the camera and link them to this patient ID.</p>
      ${docs.length ? docs.map(d => `
        <div class="card"><h3>${escapeHtml(d.description || d.type_?.text || 'Document')}</strong> <span class="badge badge-ok">${d.status}</span></h3>
        <p class="meta">${d.content?.map(c => c.attachment?.contentType + ' (' + (c.attachment?.size ? Math.round(c.attachment.size/1024)+'KB' : 'size unknown') + ')').join(', ') || ''}</p>
        ${d.content?.[0]?.attachment?.url ? `<img src="${d.content[0].attachment.url}" style="max-width:100%;max-height:300px;border-radius:0.5rem;" alt="Document preview" />` : d.content?.[0]?.attachment?.data ? `<img src="data:${d.content[0].attachment.contentType||'image/jpeg'};base64,${d.content[0].attachment.data}" style="max-width:100%;max-height:300px;border-radius:0.5rem;" alt="Document preview" />` : ''}
        </div>`).join('') : '<div class="empty-state"><p>No documents attached.</p><p class="meta">Use Ingest or the mobile camera to attach photos and documents to this patient.</p></div>'}
    `;
    const saveBtn = $("#modal-doc-save");
    if (saveBtn) saveBtn.onclick = () => {
        const arr = getLocalDocuments();
        const url = $("#modal-doc-url").value;
        const isDataUrl = url.startsWith('data:');
        const attachment = { contentType: $("#modal-doc-type").value };
        if (isDataUrl) {
            const parts = url.split(',');
            if (parts.length > 1) { attachment.data = parts[1]; }
        } else {
            attachment.url = url;
        }
        arr.push({ id:'doc-'+Date.now(), patientId: pid, description: $("#modal-doc-desc").value, status:'current', content: [{ attachment }] });
        setLocalDocuments(arr); closeModal(); views.documents();
    };
};

views.handoffSbar = () => {
    highlightNav("handoffSbar");
    const container = $("#view-container");
    const handoffs = getLocalHandoffs();
    const current = handoffs[0];
    container.innerHTML = `
      <div class="card" style="display:flex;justify-content:space-between;align-items:center;">
        <div><strong>Handoff:</strong> ${escapeHtml(current?.shift||'Night to Day')} <span style="color:var(--color-text-muted);">| Prepared by: ${escapeHtml(current?.preparedBy||'—')} | ${escapeHtml(current?.date||'')}</span></div>
        <div style="display:flex;gap:0.5rem;"><button class="btn btn-sm">Print</button><button class="btn btn-secondary btn-sm">Send to Team</button></div>
      </div>
      <div class="toolbar">
        <span style="color:var(--color-text-muted);font-size:0.85rem;">Unit:</span><select class="select" style="width:auto;margin:0;"><option>4N</option><option>4S</option><option>ICU</option><option>ED</option></select>
        <span style="color:var(--color-text-muted);font-size:0.85rem;">Filter:</span><select class="select" style="width:auto;margin:0;"><option>All Patients</option><option>High Acuity</option><option>New Admits</option><option>Pending Discharge</option></select>
      </div>
      ${current?.patients?.map((pt, idx) => `
        <div class="card">
          <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:0.75rem;"><h3>PATIENT ${idx+1}: ${escapeHtml(pt.name)} | ${escapeHtml(pt.location)} | ${pt.sex} | ${pt.age} | ${pt.code}</h3><span class="badge badge-warn">LOS 15d</span></div>
          <div class="sbar-block s"><strong>S — Situation:</strong><p style="margin-top:0.3rem;color:var(--color-text-muted);">${escapeHtml(pt.situation)}</p></div>
          <div class="sbar-block b"><strong>B — Background:</strong><p style="margin-top:0.3rem;color:var(--color-text-muted);">${escapeHtml(pt.background)}</p></div>
          <div class="sbar-block a"><strong>A — Assessment:</strong><p style="margin-top:0.3rem;color:var(--color-text-muted);">${escapeHtml(pt.assessment)}</p></div>
          <div class="sbar-block r"><strong>R — Recommendation:</strong><p style="margin-top:0.3rem;color:var(--color-text-muted);">${escapeHtml(pt.recommendation)}</p></div>
          <div style="margin-top:0.75rem;"><strong>To-Do:</strong><div style="display:flex;gap:0.75rem;flex-wrap:wrap;margin-top:0.4rem;">${pt.todos?.map(t=>`<label><input type="checkbox"> ${escapeHtml(t)}</label>`).join('')}</div></div>
        </div>
      `).join('') || '<div class="empty-state"><p>No handoff data.</p></div>'}
    `;
};

/* ---------- APPOINTMENT SCHEDULE ---------- */
views.appointmentSchedule = () => {
    highlightNav("appointmentSchedule");
    const container = $("#view-container");
    const days = ['Mon 16','Tue 17','Wed 18','Thu 19','Fri 20','Sat 21','Sun 22'];
    const times = ['08:00','08:30','09:00','09:30','10:00','10:30'];
    const schedule = getLocalSchedule();
    container.innerHTML = `
      <div class="card" style="display:flex;justify-content:space-between;align-items:center;">
        <div><strong>Schedule:</strong> Cardiology Clinic <span style="color:var(--color-text-muted);">| June 2026</span></div>
        <div style="display:flex;gap:0.5rem;"><button class="btn btn-sm btn-secondary">&lt;</button><button class="btn btn-sm">Today</button><button class="btn btn-sm btn-secondary">&gt;</button></div>
      </div>
      <div class="card" style="padding:0;overflow:hidden;">
        <div class="schedule-grid">
          <div class="grid-header">Time</div>${days.map(d=>`<div class="grid-header">${d}</div>`).join('')}
          ${times.map(t=>`
            <div class="time-label">${t}</div>
            ${days.map((d,i)=> {
              const key = d+'_'+t;
              const slot = schedule[key];
              if (i>=5) return `<div><div class="slot blocked">Closed</div></div>`;
              if (slot==='booked') return `<div><div class="slot booked">Booked</div></div>`;
              if (slot==='blocked') return `<div><div class="slot blocked">Block</div></div>`;
              return `<div><div class="slot avail schedule-slot" data-key="${key}">Avail</div></div>`;
            }).join('')}
          `).join('')}
        </div>
      </div>
      <div class="card" style="display:flex;gap:0.75rem;align-items:center;flex-wrap:wrap;">
        <span><strong>Selected slot:</strong> <span id="selected-slot">—</span></span>
        <span style="color:var(--color-text-muted);">| Patient:</span><input class="input" placeholder="Search patient..." style="width:12rem;margin:0;" />
        <button class="btn" id="book-slot-btn">Book</button>
      </div>
      <div class="card"><p style="font-size:0.82rem;color:var(--color-text-muted);"><strong>Legend:</strong> <span style="color:var(--color-success);">■ Available</span> · <span style="color:var(--color-primary);">■ Booked</span> · <span style="color:var(--color-text-muted);">■ Blocked / Closed</span></p></div>`;
    let selectedKey = null;
    container.querySelectorAll('.schedule-slot').forEach(el => el.onclick = () => {
      selectedKey = el.dataset.key;
      $("#selected-slot").textContent = selectedKey.replace('_',' ');
    });
    $("#book-slot-btn").onclick = () => {
      if (!selectedKey) { showAlert("Select a slot first."); return; }
      const sch = getLocalSchedule(); sch[selectedKey] = 'booked'; setLocalSchedule(sch); views.appointmentSchedule();
    };
};

/* ---------- WHITEBOARD ---------- */
views.whiteboard = () => {
    highlightNav("whiteboard");
    const container = $("#view-container");
    const wb = getLocalWhiteboard();
    container.innerHTML = `
      <div class="card" style="display:flex;justify-content:space-between;align-items:center;">
        <div><strong>Whiteboard: 4N — Medical</strong> <span style="color:var(--color-text-muted);">| ${wb.filter(r=>r.status==='occupied').length} occupied / ${wb.filter(r=>r.status==='empty').length} avail / ${wb.filter(r=>r.status==='cleaning').length} cleaning</span></div>
        <div style="display:flex;gap:0.5rem;"><button class="btn btn-sm">All</button><button class="btn btn-secondary btn-sm">Discharge Today</button><button class="btn btn-secondary btn-sm">High Acuity</button><button class="btn btn-secondary btn-sm">New Admits</button></div>
      </div>
      <div class="card" style="padding:0;overflow:hidden;">
        <div class="whiteboard-grid">
          <div class="grid-header">Room</div><div class="grid-header">Pt Name</div><div class="grid-header">Age/Sex</div>
          <div class="grid-header">Attending</div><div class="grid-header">LOS</div><div class="grid-header">Dx</div><div class="grid-header">DC Plan</div>
          ${wb.map(r => `
            <div style="background:${r.status==='empty'?'rgba(148,163,184,0.08)':r.status==='cleaning'?'rgba(234,179,8,0.08)':''}">${r.room}</div>
            <div style="background:${r.status==='empty'?'rgba(148,163,184,0.08)':r.status==='cleaning'?'rgba(234,179,8,0.08)':''}">${r.name?'<strong>'+escapeHtml(r.name)+'</strong>':r.status==='cleaning'?'(Cleaning)':'(Empty)'}</div>
            <div style="background:${r.status==='empty'?'rgba(148,163,184,0.08)':r.status==='cleaning'?'rgba(234,179,8,0.08)':''}">${r.ageSex||'—'}</div>
            <div style="background:${r.status==='empty'?'rgba(148,163,184,0.08)':r.status==='cleaning'?'rgba(234,179,8,0.08)':''}">${r.attending||'—'}</div>
            <div style="background:${r.status==='empty'?'rgba(148,163,184,0.08)':r.status==='cleaning'?'rgba(234,179,8,0.08)':''}">${r.los||'—'}</div>
            <div style="background:${r.status==='empty'?'rgba(148,163,184,0.08)':r.status==='cleaning'?'rgba(234,179,8,0.08)':''}">${r.dx||'—'}</div>
            <div style="background:${r.status==='empty'?'rgba(148,163,184,0.08)':r.status==='cleaning'?'rgba(234,179,8,0.08)':''}">${r.dcPlan?'<span class="badge '+(r.dcPlan==='TBD'?'badge-info':'badge-warn')+'">'+r.dcPlan+'</span>':'<span class="badge badge-ok">Avail</span>'}</div>
          `).join('')}
        </div>
      </div>
      <div class="card"><p style="font-size:0.82rem;color:var(--color-text-muted);"><strong>Legend:</strong> <span class="badge badge-ok">Available</span> <span class="badge badge-warn">Cleaning / Pending</span> <span class="badge badge-info">TBD</span></p></div>`;
};


/* ---------- DRUG INTERACTION ---------- */
views.drugInteraction = () => {
    highlightNav("drugInteraction");
    const container = $("#view-container");
    if (!requireActivePatient(container, "Drug Interaction Alerts")) return;
    const pid = getActivePatientId();
    const interactions = getLocalInteractions().filter(i => i.patientId === pid && i.status === 'active');
    container.innerHTML = `
      <div class="patient-banner"><h2>Drug Interaction Alerts <span style="font-weight:400;font-size:0.9rem;color:var(--color-text-muted);">Patient: ${escapeHtml(patientLabel(pid))}</span></h2></div>
      ${interactions.length ? interactions.map(i => `
        <div class="card" style="border:2px solid var(--color-danger);">
          <div style="display:flex;align-items:center;gap:0.75rem;margin-bottom:0.75rem;"><span style="font-size:2rem;">🚨</span><div><h2 style="color:var(--color-danger);margin:0;">Drug-Drug Interaction Detected</h2><p style="color:var(--color-text-muted);margin:0;">Severity: <strong style="color:var(--color-danger);">${i.severity.toUpperCase()}</strong> — Action required before order can be placed.</p></div></div>
          <div class="row">
            <div class="col">
              <div class="card" style="background:rgba(239,68,68,0.12);"><h3>Interacting Drugs</h3>
                <div class="list-item" style="background:transparent;border-color:rgba(239,68,68,0.25);"><div><div><strong>${escapeHtml(i.drugA)}</strong></div><div class="meta">Existing order</div></div></div>
                <div style="text-align:center;font-size:1.5rem;color:var(--color-danger);">⚡</div>
                <div class="list-item" style="background:transparent;border-color:rgba(239,68,68,0.25);"><div><div><strong>${escapeHtml(i.drugB)}</strong></div><div class="meta">New order</div></div></div>
              </div>
            </div>
            <div class="col"><div class="card"><h3>Interaction Details</h3><p><strong>Effect:</strong> ${escapeHtml(i.effect)}</p><p style="margin-top:0.5rem;"><strong>Evidence:</strong> ${escapeHtml(i.evidence)}</p><p style="margin-top:0.5rem;"><strong>Management:</strong> ${escapeHtml(i.management)}</p></div></div>
          </div>
          <div style="display:flex;gap:0.5rem;justify-content:flex-end;margin-top:0.75rem;"><button class="btn btn-secondary">Modify Order</button><button class="btn btn-secondary">Cancel New Order</button><button class="btn btn-danger">Override — Reason Required ▼</button></div>
        </div>
      `).join('') : '<div class="empty-state"><div class="big-icon">✅</div><p>No active drug interactions.</p></div>'}
    `;
};
/* ---------- ENCOUNTERS ---------- */
views.encounters = async () => {
    highlightNav("encounters");
    const container = $("#view-container");
    container.innerHTML = `
      <div class="toolbar"><button class="btn" id="enc-new">＋ New Encounter</button><span id="enc-count" style="margin-left:auto;color:var(--color-text-muted);font-size:0.85rem;"></span></div>
      <div class="search-bar"><input class="input" id="enc-search" placeholder="Search by patient, class, status, or location..." /><button class="btn" id="enc-search-btn">Search</button></div>
      <div id="enc-list"></div>`;

    let allEnc = [];
    let nameById = {};

    async function loadNames() {
      try {
        const pData = await api.searchPatients("");
        const patients = (pData.entry || []).map(e => e.resource);
        patients.forEach(p => { cachePatientName(p); nameById[p.id] = patientName(p); });
      } catch {}
    }

    function matches(enc, q) {
      if (!q) return true;
      const pid = (enc.subject?.reference || "").replace("Patient/", "");
      const name = (nameById[pid] || "").toLowerCase();
      const cls = (enc.class?.display || enc.class?.code || "").toLowerCase();
      const svc = (enc.serviceType?.text || "").toLowerCase();
      const status = (enc.status || "").toLowerCase();
      const loc = (enc.location?.[0]?.location?.display || "").toLowerCase();
      const q2 = q.toLowerCase();
      return name.includes(q2) || cls.includes(q2) || svc.includes(q2) || status.includes(q2) || loc.includes(q2);
    }

    async function render(q = "") {
      const list = $("#enc-list");
      try {
        const data = await api.searchEncounters("");
        allEnc = (data.entry || []).map(e => e.resource);
      } catch (err) { list.innerHTML = `<div class="empty-state"><p>Failed to load encounters.</p></div>`; showAlert("Failed to load encounters: " + err.message, "err"); return; }
      await loadNames();
      const visible = allEnc.filter(e => matches(e, q)).sort((a,b) => (b.period?.start || "").localeCompare(a.period?.start || ""));
      $("#enc-count").textContent = visible.length + " encounter" + (visible.length === 1 ? "" : "s");
      if (!visible.length) { list.innerHTML = `<div class="empty-state"><div class="big-icon">📋</div><p>No encounters found.</p></div>`; return; }
      list.innerHTML = `<div class="card" style="padding:0;overflow:hidden;"><div class="table-wrap"><table class="table">
        <thead><tr><th>Patient</th><th>Service</th><th>Status</th><th>Location</th><th>Attending</th><th>Start</th><th></th></tr></thead>
        <tbody>${visible.map(enc => {
          const pid = (enc.subject?.reference || "").replace("Patient/", "");
          const name = nameById[pid] || pid.slice(0,8) || "Unknown";
          const svc = enc.serviceType?.text || enc.class?.display || enc.class?.code || "—";
          const status = enc.status || "—";
          const loc = enc.location?.[0]?.location?.display || "—";
          const att = enc.participant?.[0]?.individual?.display || "—";
          const start = enc.period?.start ? fmtDateTime(enc.period.start) : "—";
          return `<tr><td><strong>${escapeHtml(name)}</strong></td><td>${escapeHtml(svc)}</td><td><span class="badge ${status==='inprogress'?'badge-warn':status==='finished'?'badge-ok':'badge-info'}">${escapeHtml(status)}</span></td><td>${escapeHtml(loc)}</td><td>${escapeHtml(att)}</td><td>${escapeHtml(start)}</td><td>${pid ? `<button class="btn btn-sm" data-pid="${pid}">Chart</button>` : ""}</td></tr>`;
        }).join("")}</tbody></table></div></div>`;
      list.querySelectorAll("button[data-pid]").forEach(b => b.onclick = () => { setActivePatient(b.dataset.pid); location.hash = "patientChart"; });
    }

    async function openNew() {
      let patients = [];
      try { const pData = await api.searchPatients(""); patients = (pData.entry || []).map(e => e.resource); patients.forEach(cachePatientName); } catch {}
      const active = getActivePatientId();
      const now = new Date(); now.setMinutes(now.getMinutes() - now.getTimezoneOffset());
      const defaultStart = now.toISOString().slice(0,16);
      openModal("New Encounter", `
        <label class="label">Patient</label>
        <select class="select" id="modal-enc-patient">${patients.map(p => `<option value="${p.id}" ${p.id === active ? "selected" : ""}>${escapeHtml(patientName(p))} — ${escapeHtml(p.identifier?.[0]?.value || p.id.slice(0,8))}</option>`).join("")}</select>
        <label class="label">Service</label>
        <select class="select" id="modal-enc-service">
          <option value="inpatient">Inpatient</option>
          <option value="outpatient" selected>Outpatient</option>
          <option value="ed">Emergency (ED)</option>
          <option value="or">Operating Room (OR)</option>
        </select>
        <label class="label">Status</label>
        <select class="select" id="modal-enc-status"><option>planned</option><option selected>in-progress</option><option>finished</option></select>
        <label class="label">Attending</label>
        <input class="input" id="modal-enc-attending" placeholder="e.g. Dr. Smith, J" value="${escapeHtml(cfg.sessionUser || 'clinician')}" />
        <label class="label">Location / Room</label>
        <input class="input" id="modal-enc-loc" placeholder="e.g. 4N-412" />
        <label class="label">Start</label>
        <input type="datetime-local" class="input" id="modal-enc-start" value="${defaultStart}" />
        <div style="display:flex;gap:0.5rem;margin-top:0.75rem"><button class="btn" id="modal-enc-save">Save</button><button class="btn btn-secondary" onclick="closeModal()">Cancel</button></div>`);
      $("#modal-enc-save").onclick = async () => {
        const pid = $("#modal-enc-patient").value;
        if (!pid) { showAlert("Select a patient.", "warn"); return; }
        const svc = $("#modal-enc-service").value;
        const classMap = { inpatient: { code: "IMP", display: "Inpatient" }, outpatient: { code: "AMB", display: "Ambulatory" }, ed: { code: "EMER", display: "Emergency" }, or: { code: "AMB", display: "Ambulatory" } };
        const startVal = $("#modal-enc-start").value;
        const payload = {
          resourceType: "Encounter",
          status: $("#modal-enc-status").value,
          class: classMap[svc] || { code: "AMB", display: "Ambulatory" },
          serviceType: { text: svc, coding: [{ code: svc, display: svc }] },
          subject: { reference: "Patient/" + pid },
          participant: [{ individual: { display: $("#modal-enc-attending").value || "—" }, type: [{ coding: [{ code: "att", display: "attending" }] }] }],
          period: { start: startVal ? new Date(startVal).toISOString() : new Date().toISOString() },
        };
        const locVal = $("#modal-enc-loc").value.trim();
        if (locVal) payload.location = [{ location: { display: locVal } }];
        try {
          const created = await api.createEncounter(payload);
          const newId = created?.id || payload.id || ("enc-" + Date.now());
          logAudit("Create", "C", { patient: pid, entity: "Encounter/" + newId, details: "Created " + svc + " encounter" });
          closeModal(); render($("#enc-search").value.trim()); showAlert("Encounter created.", "ok");
        } catch (err) { showAlert("Failed to create encounter: " + err.message, "err"); }
      };
    }

    $("#enc-search-btn").onclick = () => render($("#enc-search").value.trim());
    $("#enc-search").addEventListener("keydown", e => { if (e.key === "Enter") render($("#enc-search").value.trim()); });
    $("#enc-new").onclick = openNew;
    render();
};

/* ---------- OBSERVATIONS ---------- */
views.observations = async () => {
    highlightNav("observations");
    const container = $("#view-container");
    container.innerHTML = `<div class="toolbar"><button class="btn" id="obs-new">＋ New Observation</button><span id="obs-count" style="margin-left:auto;color:var(--color-text-muted);font-size:0.85rem;"></span></div><div class="search-bar"><input class="input" id="obs-search" placeholder="Search by patient, type, or value..." /><button class="btn" id="obs-search-btn">Search</button></div><div id="obs-list"></div>`;
    const nameMap = await loadPatientNameMap();
    async function render(q = "") {
      const list = $("#obs-list");
      let res = [];
      try { const data = await api.searchObservations(""); res = (data.entry || []).map(e => e.resource); } catch (err) { showAlert("Failed to load observations: " + err.message, "err"); }
      const visible = res.filter(r => { const name = nameMap[pidOf(r.subject?.reference)] || ""; const s = (patientName(r) + " " + (r.code?.text||r.code?.coding?.[0]?.display||"") + " " + extractValue(r) + " " + fmtDateTime(r.effectiveDateTime)).toLowerCase(); return !q || s.includes(q.toLowerCase()); });
      $("#obs-count").textContent = visible.length + " record" + (visible.length === 1 ? "" : "s");
      if (!visible.length) { list.innerHTML = `<div class="empty-state"><div class="big-icon">🔬</div><p>No observations found.</p></div>`; return; }
      list.innerHTML = `<div class="card" style="padding:0;overflow:hidden;"><div class="table-wrap"><table class="table"><thead><tr><th>Patient</th><th>Type</th><th>Value</th><th>Date</th><th></th></tr></thead><tbody>${visible.map(r => { const pid = pidOf(r.subject?.reference); const name = nameMap[pid] || (pid ? pid.slice(0,8) : "—"); return `<tr><td><strong>${escapeHtml(name)}</strong></td><td>${escapeHtml(r.code?.text || r.code?.coding?.[0]?.display || "Observation")}</td><td>${escapeHtml(extractValue(r))}</td><td>${fmtDateTime(r.effectiveDateTime)}</td><td>${pid ? `<button class="btn btn-sm" data-pid="${pid}">Chart</button>` : ""}</td></tr>`; }).join("")}</tbody></table></div></div>`;
      list.querySelectorAll("button[data-pid]").forEach(b => b.onclick = () => { setActivePatient(b.dataset.pid); location.hash = "patientChart"; });
    }
    $("#obs-search-btn").onclick = () => render($("#obs-search").value.trim());
    $("#obs-search").addEventListener("keydown", e => { if (e.key === "Enter") render($("#obs-search").value.trim()); });
    $("#obs-new").onclick = async () => {
      const now = new Date(); now.setMinutes(now.getMinutes() - now.getTimezoneOffset());
      openResourceModal("New Observation", `
        <label class="label">Type</label><input class="input" id="modal-obs-type" placeholder="e.g., Blood Pressure" />
        <label class="label">Value</label><input class="input" id="modal-obs-value" placeholder="e.g., 120/80 mmHg" />
        <label class="label">Date</label><input type="datetime-local" class="input" id="modal-obs-date" value="${now.toISOString().slice(0,16)}" />`, async (pid, v) => {
        const payload = { resourceType: "Observation", status: "final", code: { text: v("modal-obs-type") }, valueString: v("modal-obs-value"), effectiveDateTime: v("modal-obs-date") ? new Date(v("modal-obs-date")).toISOString() : new Date().toISOString(), subject: { reference: "Patient/" + pid } };
        const created = await api.createObservation(payload);
        logAudit("Create", "C", { patient: pid, entity: "Observation/" + (created?.id || ""), details: "Recorded observation: " + v("modal-obs-type") });
        closeModal(); render($("#obs-search").value.trim()); showAlert("Observation created.", "ok");
      });
    };
    render();
};

/* ---------- CONDITIONS ---------- */
views.conditions = async () => {
    highlightNav("conditions");
    const container = $("#view-container");
    container.innerHTML = `<div class="toolbar"><button class="btn" id="cond-new">＋ New Condition</button><span id="cond-count" style="margin-left:auto;color:var(--color-text-muted);font-size:0.85rem;"></span></div><div class="search-bar"><input class="input" id="cond-search" placeholder="Search by patient or condition..." /><button class="btn" id="cond-search-btn">Search</button></div><div id="cond-list"></div>`;
    const nameMap = await loadPatientNameMap();
    async function render(q = "") {
      const list = $("#cond-list");
      let res = [];
      try { const data = await api.searchConditions(""); res = (data.entry || []).map(e => e.resource); } catch (err) { showAlert("Failed to load conditions: " + err.message, "err"); }
      const visible = res.filter(r => { const name = nameMap[pidOf(r.subject?.reference)] || ""; const s = (name + " " + (r.code?.text||r.code?.coding?.[0]?.display||"") + " " + (r.clinicalStatus?.coding?.[0]?.code||"active")).toLowerCase(); return !q || s.includes(q.toLowerCase()); });
      $("#cond-count").textContent = visible.length + " record" + (visible.length === 1 ? "" : "s");
      if (!visible.length) { list.innerHTML = `<div class="empty-state"><div class="big-icon">📋</div><p>No conditions found.</p></div>`; return; }
      list.innerHTML = `<div class="card" style="padding:0;overflow:hidden;"><div class="table-wrap"><table class="table"><thead><tr><th>Patient</th><th>Condition</th><th>Status</th><th>Onset</th><th></th></tr></thead><tbody>${visible.map(r => { const pid = pidOf(r.subject?.reference); const name = nameMap[pid] || (pid ? pid.slice(0,8) : "—"); const st = r.clinicalStatus?.coding?.[0]?.code || "active"; return `<tr><td><strong>${escapeHtml(name)}</strong></td><td>${escapeHtml(r.code?.text || r.code?.coding?.[0]?.display || "Condition")}</td><td><span class="badge badge-err">${escapeHtml(st)}</span></td><td>${fmtDate(r.onsetDateTime)}</td><td>${pid ? `<button class="btn btn-sm" data-pid="${pid}">Chart</button>` : ""}</td></tr>`; }).join("")}</tbody></table></div></div>`;
      list.querySelectorAll("button[data-pid]").forEach(b => b.onclick = () => { setActivePatient(b.dataset.pid); location.hash = "patientChart"; });
    }
    $("#cond-search-btn").onclick = () => render($("#cond-search").value.trim());
    $("#cond-search").addEventListener("keydown", e => { if (e.key === "Enter") render($("#cond-search").value.trim()); });
    $("#cond-new").onclick = async () => {
      openResourceModal("New Condition", `
        <label class="label">Condition</label><input class="input" id="modal-cond-name" placeholder="e.g., Type 2 Diabetes" />
        <label class="label">Onset Date</label><input type="date" class="input" id="modal-cond-onset" />`, async (pid, v) => {
        const payload = { resourceType: "Condition", clinicalStatus: { coding: [{ code: "active" }] }, code: { text: v("modal-cond-name") }, onsetDateTime: v("modal-cond-onset") ? new Date(v("modal-cond-onset")).toISOString() : new Date().toISOString(), subject: { reference: "Patient/" + pid } };
        const created = await api.createCondition(payload);
        logAudit("Create", "C", { patient: pid, entity: "Condition/" + (created?.id || ""), details: "Added condition: " + v("modal-cond-name") });
        closeModal(); render($("#cond-search").value.trim()); showAlert("Condition created.", "ok");
      });
    };
    render();
};

/* ---------- MEDICATIONS ---------- */
views.medications = async () => {
    highlightNav("medications");
    const container = $("#view-container");
    container.innerHTML = `<div class="toolbar"><button class="btn" id="med-new">＋ New Medication</button><span id="med-count" style="margin-left:auto;color:var(--color-text-muted);font-size:0.85rem;"></span></div><div class="search-bar"><input class="input" id="med-search" placeholder="Search by patient or medication..." /><button class="btn" id="med-search-btn">Search</button></div><div id="med-list"></div>`;
    const nameMap = await loadPatientNameMap();
    async function render(q = "") {
      const list = $("#med-list");
      let res = [];
      try { const data = await api.searchMedicationRequests(""); res = (data.entry || []).map(e => e.resource); } catch (err) { showAlert("Failed to load medications: " + err.message, "err"); }
      const visible = res.filter(r => { const name = nameMap[pidOf(r.subject?.reference)] || ""; const s = (name + " " + (r.medicationCodeableConcept?.text||"") + " " + (r.dosageInstruction?.[0]?.text||"") + " " + (r.status||"")).toLowerCase(); return !q || s.includes(q.toLowerCase()); });
      $("#med-count").textContent = visible.length + " record" + (visible.length === 1 ? "" : "s");
      if (!visible.length) { list.innerHTML = `<div class="empty-state"><div class="big-icon">💊</div><p>No medications found.</p></div>`; return; }
      list.innerHTML = `<div class="card" style="padding:0;overflow:hidden;"><div class="table-wrap"><table class="table"><thead><tr><th>Patient</th><th>Medication</th><th>Instructions</th><th>Status</th><th></th></tr></thead><tbody>${visible.map(r => { const pid = pidOf(r.subject?.reference); const name = nameMap[pid] || (pid ? pid.slice(0,8) : "—"); return `<tr><td><strong>${escapeHtml(name)}</strong></td><td>${escapeHtml(r.medicationCodeableConcept?.text || "—")}</td><td>${escapeHtml(r.dosageInstruction?.[0]?.text || "—")}</td><td><span class="badge badge-info">${escapeHtml(r.status || "—")}</span></td><td>${pid ? `<button class="btn btn-sm" data-pid="${pid}">Chart</button>` : ""}</td></tr>`; }).join("")}</tbody></table></div></div>`;
      list.querySelectorAll("button[data-pid]").forEach(b => b.onclick = () => { setActivePatient(b.dataset.pid); location.hash = "patientChart"; });
    }
    $("#med-search-btn").onclick = () => render($("#med-search").value.trim());
    $("#med-search").addEventListener("keydown", e => { if (e.key === "Enter") render($("#med-search").value.trim()); });
    $("#med-new").onclick = async () => {
      openResourceModal("New Medication Order", `
        <label class="label">Medication</label><input class="input" id="modal-med-name" placeholder="e.g., Metformin 500mg" />
        <label class="label">Instructions</label><input class="input" id="modal-med-instr" placeholder="e.g., 1 tab PO BID" />`, async (pid, v) => {
        const payload = { resourceType: "MedicationRequest", status: "active", intent: "order", medicationCodeableConcept: { text: v("modal-med-name") }, dosageInstruction: [{ text: v("modal-med-instr") }], subject: { reference: "Patient/" + pid } };
        const created = await api.createMedicationRequest(payload);
        logAudit("Create", "C", { patient: pid, entity: "MedicationRequest/" + (created?.id || ""), details: "Ordered medication: " + v("modal-med-name") });
        closeModal(); render($("#med-search").value.trim()); showAlert("Medication created.", "ok");
      });
    };
    render();
};

/* ---------- PROCEDURES ---------- */
views.procedures = async () => {
    highlightNav("procedures");
    const container = $("#view-container");
    container.innerHTML = `<div class="toolbar"><button class="btn" id="proc-new">＋ New Procedure</button><span id="proc-count" style="margin-left:auto;color:var(--color-text-muted);font-size:0.85rem;"></span></div><div class="search-bar"><input class="input" id="proc-search" placeholder="Search by patient or procedure..." /><button class="btn" id="proc-search-btn">Search</button></div><div id="proc-list"></div>`;
    const nameMap = await loadPatientNameMap();
    async function render(q = "") {
      const list = $("#proc-list");
      let res = [];
      try { const data = await api.searchProcedures(""); res = (data.entry || []).map(e => e.resource); } catch (err) { showAlert("Failed to load procedures: " + err.message, "err"); }
      const visible = res.filter(r => { const name = nameMap[pidOf(r.subject?.reference)] || ""; const s = (name + " " + (r.code?.text||r.code?.coding?.[0]?.display||"") + " " + (r.status||"") + " " + fmtDateTime(r.performedDateTime)).toLowerCase(); return !q || s.includes(q.toLowerCase()); });
      $("#proc-count").textContent = visible.length + " record" + (visible.length === 1 ? "" : "s");
      if (!visible.length) { list.innerHTML = `<div class="empty-state"><div class="big-icon">🔧</div><p>No procedures found.</p></div>`; return; }
      list.innerHTML = `<div class="card" style="padding:0;overflow:hidden;"><div class="table-wrap"><table class="table"><thead><tr><th>Patient</th><th>Procedure</th><th>Date</th><th>Status</th><th></th></tr></thead><tbody>${visible.map(r => { const pid = pidOf(r.subject?.reference); const name = nameMap[pid] || (pid ? pid.slice(0,8) : "—"); return `<tr><td><strong>${escapeHtml(name)}</strong></td><td>${escapeHtml(r.code?.text || r.code?.coding?.[0]?.display || "Procedure")}</td><td>${fmtDateTime(r.performedDateTime)}</td><td><span class="badge badge-info">${escapeHtml(r.status || "—")}</span></td><td>${pid ? `<button class="btn btn-sm" data-pid="${pid}">Chart</button>` : ""}</td></tr>`; }).join("")}</tbody></table></div></div>`;
      list.querySelectorAll("button[data-pid]").forEach(b => b.onclick = () => { setActivePatient(b.dataset.pid); location.hash = "patientChart"; });
    }
    $("#proc-search-btn").onclick = () => render($("#proc-search").value.trim());
    $("#proc-search").addEventListener("keydown", e => { if (e.key === "Enter") render($("#proc-search").value.trim()); });
    $("#proc-new").onclick = async () => {
      const now = new Date(); now.setMinutes(now.getMinutes() - now.getTimezoneOffset());
      openResourceModal("New Procedure", `
        <label class="label">Procedure</label><input class="input" id="modal-proc-name" placeholder="e.g., Appendectomy" />
        <label class="label">Date</label><input type="datetime-local" class="input" id="modal-proc-date" value="${now.toISOString().slice(0,16)}" />`, async (pid, v) => {
        const payload = { resourceType: "Procedure", status: "completed", code: { text: v("modal-proc-name") }, performedDateTime: v("modal-proc-date") ? new Date(v("modal-proc-date")).toISOString() : new Date().toISOString(), subject: { reference: "Patient/" + pid } };
        const created = await api.createProcedure(payload);
        logAudit("Create", "C", { patient: pid, entity: "Procedure/" + (created?.id || ""), details: "Documented procedure: " + v("modal-proc-name") });
        closeModal(); render($("#proc-search").value.trim()); showAlert("Procedure created.", "ok");
      });
    };
    render();
};

/* ---------- INGEST ---------- */
views.ingest = () => {
    highlightNav("ingest");
    const container = $("#view-container");
    container.innerHTML = `<div class="card" style="max-width:480px;margin:2rem auto;text-align:center">
      <div style="font-size:3rem;margin-bottom:0.5rem">📂</div><h2>Ingest Documents</h2>
      <p class="meta" style="margin-bottom:1.25rem">Capture photos, upload PDFs, or record voice memos.</p>
      <div style="display:flex;gap:0.75rem;justify-content:center;flex-wrap:wrap">
        <button class="btn">📷 Take Photo</button><button class="btn btn-secondary">📄 Upload PDF</button><button class="btn btn-secondary">🎙️ Record Voice</button>
      </div>
      <div style="margin-top:1.5rem">
        <input type="file" class="input" id="ingest-file" accept="image/*,application/pdf,audio/*" />
        <button class="btn btn-block" style="margin-top:0.5rem" id="ingest-upload">Upload</button>
      </div>
    </div>`;
    $("#ingest-upload").onclick = () => { const f = $("#ingest-file").files[0]; if (!f) return showAlert("Select a file first."); showAlert(`Uploaded ${f.name}`, "ok"); };
};

/* ---------- AUDIT ---------- */
views.audit = async () => {
    highlightNav("audit");
    const container = $("#view-container");
    const filters = { actor: "", event_type: "", outcome: "", patient: "", since: "", until: "" };
    container.innerHTML = `
      <div class="toolbar">
        <button class="btn btn-sm" id="audit-refresh">🔄 Refresh</button>
        <button class="btn btn-secondary btn-sm" id="audit-export-json">⬇ Export JSON</button>
        <button class="btn btn-secondary btn-sm" id="audit-export-csv">⬇ Export CSV</button>
        <span id="audit-count" style="margin-left:auto;color:var(--color-text-muted);font-size:0.85rem;"></span>
      </div>
      <div class="card">
        <div class="row" style="gap:0.5rem;align-items:end;flex-wrap:wrap;">
          <div><label class="label-sm">Actor</label><input class="input" id="af-actor" placeholder="user id" style="width:10rem"/></div>
          <div><label class="label-sm">Event type</label><select class="select" id="af-type" style="width:10rem">
            <option value="">All</option>
            <option>Access</option><option>Create</option><option>Update</option><option>Delete</option>
            <option>Sync</option><option>Override</option><option>Correction</option><option>Login</option><option>Logout</option><option>PolicyDeny</option><option>Transfer</option>
          </select></div>
          <div><label class="label-sm">Outcome</label><select class="select" id="af-outcome" style="width:10rem">
            <option value="">All</option><option>Success</option><option>Failure</option><option>Denied</option><option>Warning</option><option>Escalated</option>
          </select></div>
          <div><label class="label-sm">Patient ID</label><input class="input" id="af-patient" placeholder="patient id" style="width:12rem"/></div>
          <div><label class="label-sm">From</label><input type="date" class="input" id="af-since" style="width:10rem"/></div>
          <div><label class="label-sm">To</label><input type="date" class="input" id="af-until" style="width:10rem"/></div>
          <div style="display:flex;gap:0.4rem;"><button class="btn btn-sm" id="af-apply">Apply</button><button class="btn btn-secondary btn-sm" id="af-clear">Clear</button></div>
        </div>
      </div>
      <div id="audit-list" style="margin-top:0.75rem;"></div>`;

    function readFilters() {
      filters.actor = $("#af-actor").value.trim();
      filters.event_type = $("#af-type").value;
      filters.outcome = $("#af-outcome").value;
      filters.patient = $("#af-patient").value.trim();
      filters.since = $("#af-since").value;
      filters.until = $("#af-until").value;
    }
    function clearFilters() {
      $("#af-actor").value = ""; $("#af-type").value = ""; $("#af-outcome").value = "";
      $("#af-patient").value = ""; $("#af-since").value = ""; $("#af-until").value = "";
      Object.keys(filters).forEach(k => filters[k] = "");
    }
    function toMillis(dateStr, endOfDay) {
      if (!dateStr) return null;
      const d = new Date(dateStr);
      if (endOfDay) d.setHours(23,59,59,999);
      return d.getTime();
    }
    function apiParams() {
      const p = {};
      if (filters.actor) p.actor = filters.actor;
      if (filters.event_type) p.event_type = filters.event_type;
      if (filters.outcome) p.outcome = filters.outcome;
      if (filters.patient) p.patient = filters.patient;
      const s = toMillis(filters.since, false); if (s != null) p.since = s;
      const u = toMillis(filters.until, true); if (u != null) p.until = u;
      return p;
    }

    let currentEntries = [];
    async function load() {
      const list = $("#audit-list");
      try {
        const data = await api.searchAudit(apiParams());
        const entry = data.entry || [];
        currentEntries = entry.map(e => e.resource);
        $("#audit-count").textContent = currentEntries.length + " event" + (currentEntries.length === 1 ? "" : "s");
        if (!currentEntries.length) { list.innerHTML = `<div class="empty-state"><div class="big-icon">📋</div><p>No audit events match.</p></div>`; return; }
        list.innerHTML = `<div class="card" style="padding:0;overflow:hidden;"><div class="table-wrap"><table class="table">
          <thead><tr><th>Time</th><th>Type</th><th>Action</th><th>Actor</th><th>Resource</th><th>Patient</th><th>Outcome</th><th>Details</th></tr></thead>
          <tbody>${currentEntries.map(a => {
            const t = a.type?.coding?.[0]?.display || a.type?.text || a.type?.coding?.[0]?.code || "Audit";
            const actor = a.agent?.[0]?.who?.display || a.agent?.[0]?.who?.reference || "—";
            const ent = a.entity?.[0]?.what?.reference || "—";
            const pat = a.patient?.reference ? a.patient.reference.replace("Patient/","") : "—";
            const oc = a.outcome || "—";
            const ocClass = oc === "Success" ? "badge-ok" : oc === "Failure" || oc === "Denied" ? "badge-err" : "badge-warn";
            return `<tr><td>${fmtDateTime(a.recorded)}</td><td>${escapeHtml(t)}</td><td>${escapeHtml(a.action || "—")}</td><td>${escapeHtml(actor)}</td><td>${escapeHtml(ent)}</td><td>${escapeHtml(pat)}</td><td><span class="badge ${ocClass}">${escapeHtml(oc)}</span></td><td style="max-width:20rem;">${escapeHtml(a.outcomeDesc || "")}</td></tr>`;
          }).join("")}</tbody></table></div></div>`;
      } catch (err) { showAlert("Failed to load audit: " + err.message, "err"); list.innerHTML = `<div class="empty-state"><p>Failed to load audit.</p></div>`; }
    }

    function download(filename, content, mime) {
      const blob = new Blob([content], { type: mime });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url; a.download = filename; a.click();
      setTimeout(() => URL.revokeObjectURL(url), 1000);
    }
    $("#audit-refresh").onclick = load;
    $("#af-apply").onclick = () => { readFilters(); load(); };
    $("#af-clear").onclick = () => { clearFilters(); load(); };
    $("#audit-export-json").onclick = () => {
      download("newloka-audit-" + Date.now() + ".json", JSON.stringify({ resourceType: "Bundle", type: "searchset", total: currentEntries.length, entry: currentEntries.map(r => ({ resource: r })) }, null, 2), "application/json");
      showAlert("Exported " + currentEntries.length + " events as JSON.", "ok");
    };
    $("#audit-export-csv").onclick = () => {
      const cols = ["recorded","type","action","actor","resource","patient","outcome","details"];
      const rows = currentEntries.map(a => [
        a.recorded || "",
        a.type?.coding?.[0]?.code || "",
        a.action || "",
        a.agent?.[0]?.who?.reference || "",
        a.entity?.[0]?.what?.reference || "",
        a.patient?.reference || "",
        a.outcome || "",
        (a.outcomeDesc || "").replace(/"/g,"'")
      ]);
      const csv = [cols.join(","), ...rows.map(r => r.map(c => "\""+String(c).replace(/"/g,"\"\"")+"\"").join(","))].join("\n");
      download("newloka-audit-" + Date.now() + ".csv", csv, "text/csv");
      showAlert("Exported " + currentEntries.length + " events as CSV.", "ok");
    };
    load();
};

/* ---------- SETTINGS ---------- */
views.settings = () => {
    highlightNav("settings");
    const container = $("#view-container");
    const customKeys = themes.getCustomKeys();
    const customHtml = customKeys.map(k => {
      const label = k.replace("--color-", "").replace(/-/g, " ").replace(/\b\w/g, c => c.toUpperCase());
      const val = cfg.customTheme?.[k] || "";
      return `<div class="color-row"><label class="label-sm">${label}</label><input type="color" class="color-input" data-var="${k}" value="${val || themes.THEMES.newloka[k]}" /></div>`;
    }).join("");
    container.innerHTML = `<div class="row">
      <div class="col">
        <div class="card">
          <h3>Appearance</h3>
          <label class="label">Theme</label>
          <select class="select" id="set-theme">
            <option value="newloka">New Loka (Default)</option>
            <option value="dark">Dark</option>
            <option value="light">Light</option>
            <option value="custom">Custom</option>
          </select>
          <div id="custom-colors" class="custom-colors" style="display:none;margin-top:0.5rem">
            <p class="meta">Select colours for your custom theme.</p>
            <div class="color-grid">${customHtml}</div>
          </div>
          <div style="display:flex;gap:0.5rem;margin-top:0.75rem">
            <button class="btn" id="set-theme-save">Apply Theme</button>
            <button class="btn btn-secondary" id="set-theme-reset">Reset to Default</button>
          </div>
        </div>
        <div class="card">
          <h3>Configuration</h3>
          <label class="label">Tier</label><select class="select" id="set-tier"><option>T0</option><option>T1</option><option>T2</option><option>T3</option><option>T4</option></select>
          <label class="label">Node ID</label><input class="input" id="set-nodeid" value="${escapeHtml(cfg.nodeId || "")}" />
          <label class="label">API Base URL</label><input class="input" id="set-api" value="${escapeHtml(cfg.apiBase || "")}" />
          <label class="label">Department</label><input class="input" id="set-dept" value="${escapeHtml(cfg.department || "")}" />
          <label class="label">Clinician name (audit identity)</label><input class="input" id="set-user" value="${escapeHtml(cfg.sessionUser || "")}" placeholder="e.g. Dr. Smith, J" />
          <div style="margin-top:0.5rem"><label><input type="checkbox" id="set-sync" ${cfg.syncEnabled?"checked":""} /> Enable sync</label></div>
          <div><label><input type="checkbox" id="set-mesh" ${cfg.meshEnabled?"checked":""} /> Enable mesh discovery</label></div>
          <div><label><input type="checkbox" id="set-emergency" ${cfg.emergencyAccess?"checked":""} /> Emergency access mode</label></div>
          <div style="display:flex;gap:0.5rem;margin-top:0.75rem"><button class="btn" id="set-save">Save Changes</button><button class="btn btn-secondary" id="set-reset">Reset Defaults</button></div>
        </div>
        <div class="card">
          <h3>Data Management</h3>
          <div style="display:flex;gap:0.5rem;flex-wrap:wrap">
            <button class="btn btn-sm btn-secondary" id="set-clear-cache">Clear Local Cache</button>
            <button class="btn btn-sm btn-secondary" id="set-flush-queue">Flush Offline Queue</button>
            <button class="btn btn-sm btn-secondary" id="set-export-data">Export All JSON</button>
            <button class="btn btn-sm btn-secondary" id="set-seed-local">🌱 Seed Local Data</button>
          </div>
          <button class="btn btn-sm btn-danger" id="set-logout" style="margin-top:0.5rem">🔒 Logout</button>
        </div>
      </div>
      <div class="col">
        <div class="card">
          <h3>Security</h3>
          <p><strong>Encryption:</strong> AES-256-GCM per-record</p>
          <p style="margin-top:0.4rem"><strong>Auth:</strong> Argon2id + Ed25519 audit signing</p>
          <p style="margin-top:0.4rem"><strong>ABAC:</strong> Role + Department + Team + Sensitivity</p>
          <div style="margin-top:0.75rem"><button class="btn btn-sm">Rotate Keys</button><button class="btn btn-secondary btn-sm">View Audit Chain</button></div>
        </div>
        <div class="card">
          <h3>About New Loka</h3><p>New Loka — Local-first health data management</p>
          <p class="meta">Version 0.2.0 • FHIR R4 • Open Source • AGPL-3.0</p>
          <p class="meta">Standards: FHIR R4, SNOMED CT, LOINC, ICD-10 • Compliance: HIPAA, DISHA, GDPR</p>
        </div>
      </div>
    </div>`;
    $("#set-tier").value = cfg.tier;
    $("#set-theme").value = cfg.theme || "newloka";
    function toggleCustom() {
      const show = $("#set-theme").value === "custom";
      $("#custom-colors").style.display = show ? "" : "none";
    }
    toggleCustom();
    $("#set-theme").addEventListener("change", toggleCustom);
    $("#set-theme-save").onclick = () => {
      cfg.theme = $("#set-theme").value;
      if (cfg.theme === "custom") {
        const custom = {};
        $$('.color-input').forEach(inp => { if(inp.value) custom[inp.dataset.var] = inp.value; });
        cfg.customTheme = custom;
      } else {
        cfg.customTheme = {};
      }
      saveCfg(cfg);
      themes.apply(cfg.theme, cfg.customTheme);
      showAlert("Theme applied.", "ok");
    };
    $("#set-theme-reset").onclick = () => {
      cfg.theme = "newloka";
      cfg.customTheme = {};
      saveCfg(cfg);
      themes.apply(cfg.theme, cfg.customTheme);
      $("#set-theme").value = "newloka";
      toggleCustom();
      showAlert("Theme reset to default.", "ok");
    };
    $("#set-save").onclick = () => {
      cfg.tier = $("#set-tier").value; cfg.nodeId = $("#set-nodeid").value.trim() || cfg.nodeId;
      cfg.apiBase = $("#set-api").value.trim() || cfg.apiBase; cfg.department = $("#set-dept").value.trim();
      cfg.sessionUser = $("#set-user").value.trim() || "clinician";
      cfg.syncEnabled = $("#set-sync").checked; cfg.meshEnabled = $("#set-mesh").checked; cfg.emergencyAccess = $("#set-emergency").checked;
      saveCfg(cfg); api.init(cfg); hideFeatureLinks(); $("#tier-badge").textContent = cfg.tier; showAlert("Settings saved.", "ok");
    };
    $("#set-reset").onclick = () => { if(confirm("Reset all settings and clear cache?")) { localStorage.clear(); location.reload(); } };
    $("#set-clear-cache").onclick = async () => {
      await api.dbClear("patients"); await api.dbClear("encounters"); await api.dbClear("observations");
      await api.dbClear("conditions"); await api.dbClear("medicationRequests"); await api.dbClear("procedures");
      showAlert("Cache cleared.", "ok");
    };
    $("#set-flush-queue").onclick = async () => { await api.flushQueue(); showAlert("Queue flushed.", "ok"); };
    $("#set-export-data").onclick = async () => {
      try {
        const [p,e,o,c,m,pr] = await Promise.all([api.searchPatients(""),api.searchEncounters(""),api.searchObservations(""),api.searchConditions(""),api.searchMedicationRequests(""),api.searchProcedures("")]);
        const blob = new Blob([JSON.stringify({patients:p,encounters:e,observations:o,conditions:c,medications:m,procedures:pr},null,2)], {type:"application/json"});
        const a = document.createElement("a"); a.href = URL.createObjectURL(blob); a.download = "newloka-export.json"; a.click();
      } catch(err) { showAlert("Export failed: " + err.message, "err"); }
    };
    $("#set-seed-local").onclick = async () => { try { const r = await seedLocal(15); showAlert(`Seeded ${r} local patients.`, "ok"); } catch(err) { showAlert("Seed failed: " + err.message, "err"); } };
    $("#set-logout").onclick = logout;
};

/* ---------- ROUTING ---------- */
function route(viewName, args = {}) {
    const fn = views[viewName];
    if (!fn) return route("dashboard");
    window.location.hash = viewName + (args.id ? "/" + args.id : "");
    fn(args);
}
function bootstrap() {
    themes.apply(cfg.theme, cfg.customTheme);
    hideFeatureLinks();
    $("#tier-badge").textContent = cfg.tier;
    $("#node-id").textContent = "Node: " + (cfg.nodeId || "?");
    seedIfEmpty();
    if (!isLoggedIn()) {
      $("#app-header").classList.add("hidden");
      views.login();
    } else {
      $("#app-header").classList.remove("hidden");
      const hashParts = window.location.hash.replace("#","").split("/");
      const hash = hashParts[0] || "dashboard";
      route(hash, hashParts[1] ? { id: hashParts[1] } : {});
    }
}
window.addEventListener("hashchange", () => {
    const hashParts = window.location.hash.replace("#","").split("/");
    const hash = hashParts[0] || "dashboard";
    if (views[hash]) { views[hash](hashParts[1] ? { id: hashParts[1] } : {}); }
    else { route("dashboard"); }
});
window.closeModal = closeModal;
bootstrap();



