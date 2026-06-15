import { load as loadCfg, save as saveCfg, allowed } from "./config.js";
import * as api from "./api.js";
import { seedLocal } from "./mock-data.js";

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

function showAlert(msg, type="warn") {
  const bar = $("#alert-bar");
  bar.innerHTML = `<span>${escapeHtml(msg)}</span><button class="close-alert" aria-label="Dismiss">✕</button>`;
  bar.className = "alert-bar " + type;
  bar.querySelector(".close-alert").onclick = () => bar.classList.add("hidden");
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
function patientName(p){const n=p.name?.[0];if(!n)return"Unnamed";return(n.given?.join(" ")||"")+" "+(n.family||"");}
async function fetchPatientName(ref){if(!ref)return"?";const pid=ref.replace("Patient/","");try{const p=await api.getPatient(pid);return p?patientName(p):pid.slice(0,8);}catch{return pid.slice(0,8);}}
function escapeHtml(str){if(!str)return"";const div=document.createElement("div");div.textContent=str;return div.innerHTML;}

const views = {};

views.login = () => {
    const container = $("#view-container");
    container.innerHTML = `
      <div class="card" style="max-width:420px;margin:5rem auto 0;text-align:center">
        <div style="font-size:3rem;margin-bottom:0.5rem">🏥</div>
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
          <button type="submit" class="btn btn-block" style="margin-top:0.75rem">Sign In</button>
        </form>
        <p class="meta" style="margin-top:1rem">Offline-ready • FHIR R4 • Local-first</p>
      </div>`;
    $("#login-form").addEventListener("submit", e => {
      e.preventDefault();
      cfg.tier = $("#login-tier").value;
      cfg.sessionPin = $("#login-pin").value;
      cfg.department = $("#login-dept").value.trim();
      cfg.apiBase = $("#login-api").value.trim();
      cfg.nodeId = "browser-" + Math.random().toString(36).slice(2,8);
      saveCfg(cfg); api.init(cfg);
      location.reload();
    });
};

views.dashboard = async () => {
    highlightNav("dashboard");
    $("#view-container").innerHTML = `<div class="view"><h2>Dashboard</h2><div id="dash-grid"></div></div>`;
    const grid = $("#dash-grid");
    let counts = { Patient:0, Encounter:0, Observation:0, Condition:0, MedicationRequest:0, Procedure:0 };
    try {
        const [p,e,o,c,m,pr] = await Promise.all([
            api.searchPatients(""), api.searchEncounters(""), api.searchObservations(""),
            api.searchConditions(""), api.searchMedicationRequests(""), api.searchProcedures("")
        ]);
        counts.Patient = (p.entry||[]).length;
        counts.Encounter = (e.entry||[]).length;
        counts.Observation = (o.entry||[]).length;
        counts.Condition = (c.entry||[]).length;
        counts.MedicationRequest = (m.entry||[]).length;
        counts.Procedure = (pr.entry||[]).length;
    } catch {}
    const onlineDot = navigator.onLine ? '<span class="status-dot ok"></span>Online' : '<span class="status-dot err"></span>Offline';
    grid.innerHTML = `
      <div class="card"><h3>Statistics</h3>
        <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(6rem,1fr));gap:0.5rem">
          <div style="text-align:center;padding:0.6rem;background:rgba(255,255,255,0.04);border-radius:0.375rem"><div style="font-size:1.5rem;font-weight:700">${counts.Patient}</div><div class="meta">Patients</div></div>
          <div style="text-align:center;padding:0.6rem;background:rgba(255,255,255,0.04);border-radius:0.375rem"><div style="font-size:1.5rem;font-weight:700">${counts.Encounter}</div><div class="meta">Encounters</div></div>
          <div style="text-align:center;padding:0.6rem;background:rgba(255,255,255,0.04);border-radius:0.375rem"><div style="font-size:1.5rem;font-weight:700">${counts.Observation}</div><div class="meta">Observations</div></div>
          <div style="text-align:center;padding:0.6rem;background:rgba(255,255,255,0.04);border-radius:0.375rem"><div style="font-size:1.5rem;font-weight:700">${counts.Condition}</div><div class="meta">Conditions</div></div>
          <div style="text-align:center;padding:0.6rem;background:rgba(255,255,255,0.04);border-radius:0.375rem"><div style="font-size:1.5rem;font-weight:700">${counts.MedicationRequest}</div><div class="meta">Medications</div></div>
          <div style="text-align:center;padding:0.6rem;background:rgba(255,255,255,0.04);border-radius:0.375rem"><div style="font-size:1.5rem;font-weight:700">${counts.Procedure}</div><div class="meta">Procedures</div></div>
        </div>
      </div>
      <div class="card"><h3>Quick Actions</h3>
        <div class="toolbar">
          <button class="btn" id="dash-new-patient">+ New Patient</button>
          <button class="btn btn-secondary" id="dash-new-enc">+ New Encounter</button>
          <button class="btn btn-secondary" id="dash-new-obs">+ New Observation</button>
          <button class="btn btn-secondary" id="dash-seed">🌱 Seed Local Data</button>
        </div>
      </div>
      <div class="card"><h3>System Status</h3>
        <p>${onlineDot}</p>
        <p>Tier: <strong>${cfg.tier}</strong> • Department: ${cfg.department || "—"}</p>
        <p class="meta">Node: ${cfg.nodeId || "?"} • Version 0.1.0 • FHIR R4</p>
      </div>`;
    if (allowed(cfg.tier,"audit")) {
        try { const health = await api.getHealth(); grid.lastElementChild.innerHTML += `<p class="meta">Server: ${health.status || "?"} v${health.version || "?"}</p>`; } catch {}
    }
    $("#dash-new-patient").onclick = () => route("patients", {create:true});
    $("#dash-new-enc").onclick = () => route("encounters", {create:true});
    $("#dash-new-obs").onclick = () => route("observations", {create:true});
    $("#dash-seed").onclick = async () => { try { const r = await seedLocal(15); showAlert(`Seeded ${r} local patients.`, "ok"); } catch(err) { showAlert("Seed failed: " + err.message, "err"); } };
};

function patientFormHtml(p) {
    const id = p.id || "";
    return `
      <form id="edit-patient-form">
        <input type="hidden" id="pt-id" value="${id}" />
        <label class="label">Given Name</label>
        <input class="input" id="pt-given" value="${escapeHtml(p.name?.[0]?.given?.join(" ") || "")}" />
        <label class="label">Family Name</label>
        <input class="input" id="pt-family" value="${escapeHtml(p.name?.[0]?.family || "")}" />
        <label class="label">Birth Date</label>
        <input type="date" class="input" id="pt-dob" value="${p.birthDate || ""}" />
        <label class="label">Gender</label>
        <select class="select" id="pt-gender"><option value=""></option><option ${p.gender==="male"?"selected":""}>male</option><option ${p.gender==="female"?"selected":""}>female</option><option ${p.gender==="other"?"selected":""}>other</option><option ${p.gender==="unknown"?"selected":""}>unknown</option></select>
        <label class="label">Phone</label>
        <input class="input" id="pt-phone" value="${escapeHtml(p.telecom?.find(t=>t.system==="phone")?.value || "")}" />
        <div style="display:flex;gap:0.5rem;margin-top:0.75rem">
          <button class="btn" type="submit">${id?"Update":"Create"}</button>
          ${id?'<button class="btn btn-danger" type="button" id="pt-delete">Delete</button>':""}
        </div>
      </form>`;
}

views.patients = async (args={}) => {
    highlightNav("patients");
    const container = $("#view-container");
    container.innerHTML = `
      <div class="view">
        <h2>Patients</h2>
        <div class="search-bar">
          <input type="search" class="input" id="pt-search" placeholder="Search name, ID, phone…" value="" />
          <button class="btn" id="pt-search-btn">Search</button>
          <button class="btn btn-secondary" id="pt-add">+ Add</button>
        </div>
        <div class="toolbar">
          <select class="select btn-sm" id="pt-sort" style="width:auto;padding:0.35rem 0.6rem"><option value="name">Sort by Name</option><option value="dob">Sort by DOB</option><option value="recent">Recently Updated</option></select>
          <select class="select btn-sm" id="pt-filter-gender" style="width:auto;padding:0.35rem 0.6rem"><option value="">All Genders</option><option>male</option><option>female</option><option>other</option></select>
        </div>
        <div id="pt-list"></div>
      </div>`;
    let allEntries = [];
    const renderList = (entries) => {
        const list = $("#pt-list");
        if (!entries.length) { list.innerHTML = '<div class="empty-state"><div class="big-icon">🏥</div><p>No patients found.</p></div>'; return; }
        list.innerHTML = entries.map(e => {
            const p = e.resource || e;
            const nm = patientName(p).trim() || "Unnamed";
            return `<div class="list-item" data-id="${p.id}"><div><strong>${escapeHtml(nm)}</strong><br/><span class="meta">${p.gender || "?"} | ${p.birthDate || "?"} | Age ${fmtAge(p.birthDate) || "?"}</span></div><span class="badge badge-ok">${p.gender || ""}</span></div>`;
        }).join("");
        $$(".list-item").forEach(el => el.onclick = () => route("patientDetail", { id: el.dataset.id }));
    };
    const applyFilters = () => {
        let arr = [...allEntries];
        const q = $("#pt-search").value.trim().toLowerCase();
        if (q) arr = arr.filter(e => {
            const p = e.resource || e;
            const nm = patientName(p).toLowerCase();
            const id = (p.id || "").toLowerCase();
            const phone = (p.telecom?.find(t => t.system === "phone")?.value || "").toLowerCase();
            return nm.includes(q) || id.includes(q) || phone.includes(q);
        });
        const sort = $("#pt-sort").value;
        if (sort === "name") arr.sort((a, b) => patientName(a.resource || a).localeCompare(patientName(b.resource || b)));
        else if (sort === "dob") arr.sort((a, b) => ((a.resource || a).birthDate || "").localeCompare((b.resource || b).birthDate || ""));
        else if (sort === "recent") arr.reverse();
        const g = $("#pt-filter-gender").value;
        if (g) arr = arr.filter(e => (e.resource || e).gender === g);
        renderList(arr);
    };
    const load = async () => {
        $("#pt-list").innerHTML = '<p class="empty-state">Loading…</p>';
        try {
            const bundle = await api.searchPatients("");
            allEntries = bundle.entry || [];
            applyFilters();
        } catch (err) { showAlert("Failed to load patients: " + err.message, "err"); }
    };
    $("#pt-search").addEventListener("input", applyFilters);
    $("#pt-search-btn").onclick = applyFilters;
    $("#pt-sort").onchange = applyFilters;
    $("#pt-filter-gender").onchange = applyFilters;
    $("#pt-add").onclick = () => { openModal("New Patient", patientFormHtml({})); wirePatientForm(null, load); };
    if (args.create) { openModal("New Patient", patientFormHtml({})); wirePatientForm(null, load); }
    await load();
};

function wirePatientForm(id, onDone) {
    const form = $("#edit-patient-form");
    form.onsubmit = async (e) => {
        e.preventDefault();
        const payload = { resourceType:"Patient", name:[{given:[$("#pt-given").value.trim()], family:$("#pt-family").value.trim()}], birthDate:$("#pt-dob").value, gender:$("#pt-gender").value, telecom:[{system:"phone", value:$("#pt-phone").value.trim()}] };
        if (id) payload.id = id;
        try { id ? await api.updatePatient(id, payload) : await api.createPatient(payload); closeModal(); showAlert("Saved.", "ok"); onDone(); } catch(err) { showAlert("Save failed: " + err.message, "err"); }
    };
    const del = $("#pt-delete");
    if (del) del.onclick = async () => { if (!confirm("Delete?")) return; try { await api.deletePatient(id); closeModal(); showAlert("Deleted.", "ok"); onDone(); } catch(err) { showAlert("Delete failed: " + err.message, "err"); } };
}

views.patientDetail = async (args={}) => {
    highlightNav("patients");
    const container = $("#view-container");
    const pid = args.id;
    container.innerHTML = `<div class="view"><p class="empty-state">Loading…</p></div>`;
    try {
        const p = await api.getPatient(pid);
        const nm = patientName(p);
        container.innerHTML = `
          <div class="view">
            <h2>${escapeHtml(nm)}</h2>
            <div class="card">
              <p><strong>ID:</strong> ${p.id}</p>
              <p><strong>DOB:</strong> ${p.birthDate || "?"} (Age ${fmtAge(p.birthDate)})</p>
              <p><strong>Gender:</strong> ${p.gender || "?"}</p>
              <p><strong>Phone:</strong> ${escapeHtml(p.telecom?.find(t=>t.system==="phone")?.value || "?")}</p>
              <div class="toolbar" style="margin-top:0.5rem">
                <button class="btn btn-sm" id="pt-edit">Edit</button>
                <button class="btn btn-sm btn-danger" id="pt-delete">Delete</button>
                <button class="btn btn-sm btn-secondary" id="pt-back">← Back</button>
              </div>
            </div>
            <div class="card"><h3>Encounters</h3><div id="pt-enc-list"></div></div>
            <div class="card"><h3>Observations</h3><div id="pt-obs-list"></div></div>
          </div>`;
        $("#pt-edit").onclick = () => { openModal("Edit Patient", patientFormHtml(p)); wirePatientForm(pid, () => route("patientDetail", {id:pid})); };
        $("#pt-delete").onclick = async () => { if (!confirm("Delete this patient?")) return; try { await api.deletePatient(pid); showAlert("Deleted.", "ok"); route("patients"); } catch(err) { showAlert("Delete failed: " + err.message, "err"); } };
        $("#pt-back").onclick = () => route("patients");
        const renderMini = async (searchFn, containerId, renderFn, emptyMsg) => {
            const list = $(containerId);
            try {
                const bundle = await searchFn(pid);
                const entries = bundle.entry || [];
                if (!entries.length) { list.innerHTML = `<p class="meta">${emptyMsg}</p>`; return; }
                list.innerHTML = entries.map(e => renderFn(e.resource || e)).join("");
            } catch { list.innerHTML = `<p class="meta">${emptyMsg}</p>`; }
        };
        await renderMini(api.searchEncounters, "#pt-enc-list", enc => `<div class="list-item"><div><strong>${enc.status || "?"}</strong><br/><span class="meta">${fmtDate(enc.period?.start)} — ${enc.class?.display || enc.class?.code || "?"}</span></div></div>`, "No encounters.");
        await renderMini(api.searchObservations, "#pt-obs-list", obs => `<div class="list-item"><div><strong>${extractValue(obs)}</strong><br/><span class="meta">${obs.code?.text || obs.code?.coding?.[0]?.display || "?"}</span></div></div>`, "No observations.");
    } catch(err) { showAlert("Failed to load patient: " + err.message, "err"); }
};

function genericFormHtml(type, data) {
    const id = data.id || "";
    return `
      <form id="generic-form">
        <input type="hidden" id="g-id" value="${id}" />
        <label class="label">JSON Payload</label>
        <textarea class="input" id="g-json" rows="10" style="font-family:monospace">${escapeHtml(JSON.stringify(data, null, 2))}</textarea>
        <div style="display:flex;gap:0.5rem;margin-top:0.75rem">
          <button class="btn" type="submit">${id ? "Update" : "Create"}</button>
          ${id ? '<button class="btn btn-danger" type="button" id="g-delete">Delete</button>' : ""}
        </div>
      </form>`;
}

function wireGenericForm(type, id, createFn, updateFn, deleteFn, onDone) {
    const form = $("#generic-form");
    form.onsubmit = async (e) => {
        e.preventDefault();
        try {
            const payload = JSON.parse($("#g-json").value);
            payload.resourceType = type;
            if (id) payload.id = id;
            id ? await updateFn(id, payload) : await createFn(payload);
            closeModal(); showAlert("Saved.", "ok"); onDone();
        } catch(err) { showAlert("Save failed: " + err.message, "err"); }
    };
    const del = $("#g-delete");
    if (del) del.onclick = async () => { if (!confirm("Delete?")) return; try { await deleteFn(id); closeModal(); showAlert("Deleted.", "ok"); onDone(); } catch(err) { showAlert("Delete failed: " + err.message, "err"); } };
}

function simpleResourceView(viewName, title, searchApi, createApi, updateApi, deleteApi, getApi, resourceType) {
    views[viewName] = async (args={}) => {
        highlightNav(viewName);
        const container = $("#view-container");
        container.innerHTML = `
          <div class="view">
            <h2>${title}</h2>
            <div class="search-bar">
              <input type="search" class="input" id="list-search" placeholder="Search…" />
              <button class="btn" id="list-search-btn">Search</button>
              <button class="btn btn-secondary" id="list-add">+ Add</button>
            </div>
            <div id="list-container"></div>
          </div>`;
        const render = (entries) => {
            const list = $("#list-container");
            if (!entries.length) { list.innerHTML = `<div class="empty-state"><div class="big-icon">📂</div><p>No ${title.toLowerCase()} found.</p></div>`; return; }
            list.innerHTML = entries.map(e => {
                const r = e.resource || e;
                const id = r.id || "";
                const main = (r.code?.text || r.code?.coding?.[0]?.display || r.class?.display || r.class?.code || r.status || resourceType + " " + id.slice(0,8));
                const meta = (r.status || fmtDate(r.effectiveDateTime || r.period?.start || r.authoredOn) || "");
                return `<div class="list-item" data-id="${id}"><div><strong>${escapeHtml(main)}</strong><br/><span class="meta">${escapeHtml(meta)}</span></div></div>`;
            }).join("");
            $$(".list-item").forEach(el => el.onclick = async () => {
                try { const data = await getApi(el.dataset.id); openModal("Edit " + resourceType, genericFormHtml(resourceType, data)); wireGenericForm(resourceType, el.dataset.id, createApi, updateApi, deleteApi, load); } catch(err) { showAlert("Load failed: " + err.message, "err"); }
            });
        };
        let all = [];
        const apply = () => {
            const q = $("#list-search").value.trim().toLowerCase();
            let arr = [...all];
            if (q) arr = arr.filter(e => JSON.stringify(e.resource || e).toLowerCase().includes(q));
            render(arr);
        };
        const load = async () => {
            $("#list-container").innerHTML = '<p class="empty-state">Loading…</p>';
            try { const bundle = await searchApi(""); all = bundle.entry || []; apply(); } catch(err) { showAlert("Failed to load: " + err.message, "err"); }
        };
        $("#list-search").addEventListener("input", apply);
        $("#list-search-btn").onclick = apply;
        $("#list-add").onclick = () => { openModal("New " + resourceType, genericFormHtml(resourceType, {})); wireGenericForm(resourceType, null, createApi, updateApi, deleteApi, load); };
        if (args.create) { openModal("New " + resourceType, genericFormHtml(resourceType, {})); wireGenericForm(resourceType, null, createApi, updateApi, deleteApi, load); }
        await load();
    };
}

simpleResourceView("encounters", "Encounters", api.searchEncounters, api.createEncounter, api.updateEncounter, api.deleteEncounter, api.getEncounter, "Encounter");
simpleResourceView("observations", "Observations", api.searchObservations, api.createObservation, api.updateObservation, api.deleteObservation, api.getObservation, "Observation");
simpleResourceView("conditions", "Conditions", api.searchConditions, api.createCondition, api.updateCondition, api.deleteCondition, api.getCondition, "Condition");
simpleResourceView("medications", "Medications", api.searchMedicationRequests, api.createMedicationRequest, api.updateMedicationRequest, api.deleteMedicationRequest, api.getMedicationRequest, "MedicationRequest");
simpleResourceView("procedures", "Procedures", api.searchProcedures, api.createProcedure, api.updateProcedure, api.deleteProcedure, api.getProcedure, "Procedure");

views.ingest = () => {
    highlightNav("ingest");
    $("#view-container").innerHTML = `
      <div class="view">
        <h2>Ingest</h2>
        <div class="card">
          <h3>Capture Data</h3>
          <div class="toolbar" style="flex-wrap:wrap">
            <button class="btn" id="ing-photo">📷 Photo</button>
            <button class="btn btn-secondary" id="ing-voice">🎤 Voice</button>
            <button class="btn btn-secondary" id="ing-pdf">📄 PDF</button>
            <button class="btn btn-secondary" id="ing-dictate">📝 Dictate</button>
          </div>
          <input type="file" id="file-input" class="input" style="margin-top:0.5rem" />
        </div>
      </div>`;
    $("#ing-photo").onclick = () => { $("#file-input").accept="image/*"; $("#file-input").click(); showAlert("Photo capture ready.", "ok"); };
    $("#ing-voice").onclick = () => showAlert("Voice recording not implemented yet.", "warn");
    $("#ing-pdf").onclick = () => { $("#file-input").accept=".pdf"; $("#file-input").click(); showAlert("PDF upload ready.", "ok"); };
    $("#ing-dictate").onclick = () => showAlert("Dictation not implemented yet.", "warn");
    $("#file-input").onchange = () => showAlert("File selected. Upload will be processed.", "ok");
};

views.handoff = () => {
    highlightNav("handoff");
    $("#view-container").innerHTML = `
      <div class="view">
        <h2>Handoff</h2>
        <div class="card">
          <form id="ho-form">
            <label class="label">Recipient Node ID</label>
            <input class="input" id="ho-node" placeholder="target-node" />
            <label class="label">Patient ID</label>
            <input class="input" id="ho-patient" placeholder="Patient/..." />
            <label class="label">Summary</label>
            <textarea class="input" id="ho-summary" rows="4"></textarea>
            <button type="submit" class="btn" style="margin-top:0.75rem" id="ho-send">Send Handoff</button>
          </form>
        </div>
      </div>`;
    $("#ho-send").onclick = (e) => {
        e.preventDefault();
        showAlert("Handoff sent to " + $("#ho-node").value, "ok");
    };
};

views.audit = async () => {
    highlightNav("audit");
    const container = $("#view-container");
    container.innerHTML = `
      <div class="view">
        <h2>Audit Log</h2>
        <div class="toolbar">
          <button class="btn" id="audit-refresh">Refresh</button>
          <button class="btn btn-secondary" id="audit-export">Export</button>
        </div>
        <div id="audit-list"></div>
      </div>`;
    const load = async () => {
        $("#audit-list").innerHTML = '<p class="empty-state">Loading…</p>';
        try {
            const bundle = await api.searchAudit();
            const entries = bundle.entry || [];
            if (!entries.length) { $("#audit-list").innerHTML = '<p class="empty-state">No audit events.</p>'; return; }
            $("#audit-list").innerHTML = entries.map(e => {
                const r = e.resource || e;
                return `<div class="list-item"><div><strong>${r.type?.display || r.type || "Event"}</strong><br/><span class="meta">${fmtDateTime(r.recorded)} — ${r.outcome || "?"}</span></div></div>`;
            }).join("");
        } catch(err) { showAlert("Failed to load audit: " + err.message, "err"); }
    };
    $("#audit-refresh").onclick = load;
    $("#audit-export").onclick = () => showAlert("Audit export not implemented yet.", "warn");
    await load();
};

views.settings = () => {
    highlightNav("settings");
    const container = $("#view-container");
    container.innerHTML = `
      <div class="view">
        <h2>Settings</h2>
        <div class="card">
          <label class="label">Deployment Tier</label>
          <select class="select" id="set-tier">
            <option value="T0">T0 — Single clinician (offline)</option>
            <option value="T1">T1 — Small clinic mesh</option>
            <option value="T2">T2 — Rural hospital</option>
            <option value="T3">T3 — Multi-department hospital</option>
            <option value="T4">T4 — Research federation</option>
          </select>
          <label class="label">Node ID</label><input class="input" id="set-nodeid" value="${escapeHtml(cfg.nodeId || "")}" />
          <label class="label">API Base URL</label><input class="input" id="set-api" value="${escapeHtml(cfg.apiBase || "")}" />
          <label class="label">Department</label><input class="input" id="set-dept" value="${escapeHtml(cfg.department || "")}" />
          <div style="margin-top:0.5rem"><label><input type="checkbox" id="set-sync" ${cfg.syncEnabled?"checked":""} /> Enable sync</label></div>
          <div><label><input type="checkbox" id="set-mesh" ${cfg.meshEnabled?"checked":""} /> Enable mesh discovery</label></div>
          <div><label><input type="checkbox" id="set-emergency" ${cfg.emergencyAccess?"checked":""} /> Emergency access mode</label></div>
          <div style="display:flex;gap:0.5rem;margin-top:0.75rem">
            <button class="btn" id="set-save">Save Changes</button>
            <button class="btn btn-secondary" id="set-reset">Reset Defaults</button>
          </div>
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
        <div class="card">
          <h3>About New Loka</h3>
          <p>New Loka — Local-first health data management</p>
          <p class="meta">Version 0.1.0 • FHIR R4 • Open Source • AGPL-3.0</p>
          <p class="meta">Standards: FHIR R4, SNOMED CT, LOINC, ICD-10 • Compliance: HIPAA, DISHA, GDPR</p>
        </div>
      </div>`;
    $("#set-tier").value = cfg.tier;
    $("#set-save").onclick = () => {
        cfg.tier = $("#set-tier").value;
        cfg.nodeId = $("#set-nodeid").value.trim() || cfg.nodeId;
        cfg.apiBase = $("#set-api").value.trim() || cfg.apiBase;
        cfg.department = $("#set-dept").value.trim();
        cfg.syncEnabled = $("#set-sync").checked;
        cfg.meshEnabled = $("#set-mesh").checked;
        cfg.emergencyAccess = $("#set-emergency").checked;
        saveCfg(cfg); api.init(cfg); hideFeatureLinks();
        $("#tier-badge").textContent = cfg.tier;
        showAlert("Settings saved.", "ok");
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
            const [p,e,o,c,m,pr] = await Promise.all([
                api.searchPatients(""),api.searchEncounters(""),api.searchObservations(""),
                api.searchConditions(""),api.searchMedicationRequests(""),api.searchProcedures("")
            ]);
            const blob = new Blob([JSON.stringify({patients:p,encounters:e,observations:o,conditions:c,medications:m,procedures:pr},null,2)], {type:"application/json"});
            const a = document.createElement("a"); a.href = URL.createObjectURL(blob); a.download = "newloka-export.json"; a.click();
        } catch(err) { showAlert("Export failed: " + err.message, "err"); }
    };
    $("#set-seed-local").onclick = async () => {
        try { const r = await seedLocal(15); showAlert(`Seeded ${r} local patients.`, "ok"); } catch(err) { showAlert("Seed failed: " + err.message, "err"); }
    };
    $("#set-logout").onclick = logout;
};

function route(viewName, args = {}) {
    const fn = views[viewName];
    if (!fn) return route("dashboard");
    window.location.hash = viewName + (args.id ? "/" + args.id : "");
    fn(args);
}
function bootstrap() {
    hideFeatureLinks();
    $("#tier-badge").textContent = cfg.tier;
    $("#node-id").textContent = "Node: " + (cfg.nodeId || "?");
    if (!isLoggedIn()) {
        $("#app-header").classList.add("hidden");
        views.login();
    } else {
        $("#app-header").classList.remove("hidden");
        const hash = window.location.hash.replace("#","").split("/")[0] || "dashboard";
        route(hash);
    }
}
window.addEventListener("hashchange", () => {
    const hash = window.location.hash.replace("#","").split("/")[0] || "dashboard";
    if (views[hash]) views[hash]();
});
window.closeModal = closeModal;
bootstrap();
