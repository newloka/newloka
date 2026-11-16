/**
 * Pure Utility & Formatting Functions
 *
 * Strictly adheres to Section 5.5 of the New Loka Style Guide.
 * Pure, side-effect free, DOM-less string escaping and formatters.
 */

export const $ = s => document.querySelector(s);
export const $$ = s => Array.from(document.querySelectorAll(s));

/**
 * Escapes HTML characters using fast string replacements without DOM allocation.
 * Conforms to Section 5.5 of New Loka Style Guide.
 */
export function escapeHtml(str) {
  if (str === null || str === undefined) return '';
  return String(str)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
}

export function fmtDate(iso) {
  if (!iso) return "?";
  const d = new Date(iso);
  return isNaN(d) ? iso : d.toLocaleDateString();
}

export function fmtDateTime(iso) {
  if (!iso) return "?";
  const d = new Date(iso);
  return isNaN(d) ? iso : d.toLocaleString();
}

export function fmtAge(dob) {
  if (!dob) return "?";
  const birth = new Date(dob);
  if (isNaN(birth)) return "?";
  const today = new Date();
  let years = today.getFullYear() - birth.getFullYear();
  const m = today.getMonth() - birth.getMonth();
  if (m < 0 || (m === 0 && today.getDate() < birth.getDate())) years--;
  return years + "y";
}

export function safeJson(val, len) {
  const s = JSON.stringify(val);
  if (!s || s === "{}") return "?";
  if (s.length > (len || 80)) return s.slice(0, len || 80) + "…";
  return s;
}

export function extractValue(obs) {
  if (!obs) return "?";
  if (obs.valueQuantity) return "" + obs.valueQuantity.value + " " + (obs.valueQuantity.unit || "");
  if (obs.valueString) return obs.valueString;
  if (obs.component && obs.component.length) {
    return obs.component.map(c => c.valueQuantity ? c.valueQuantity.value + " " + (c.valueQuantity.unit || "") : c.valueString || "").join(" / ");
  }
  return "?";
}

export function patientName(p) {
  if (!p) return "Unnamed";
  const n = p.name?.[0];
  if (!n) return "Unnamed";
  return (n.given?.join(" ") || "") + " " + (n.family || "");
}

export function pidOf(ref) {
  return (ref || "").replace("Patient/", "");
}

export function lsGet(key, fallback) {
  try {
    const v = localStorage.getItem(key);
    return v ? JSON.parse(v) : fallback;
  } catch {
    return fallback;
  }
}

export function lsSet(key, val) {
  try {
    localStorage.setItem(key, JSON.stringify(val));
  } catch {}
}
