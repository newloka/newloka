/**
 * Pure Clinical Knowledge, Scoring & Telemetry Visualization Engines
 *
 * Implements Functional Core (FCIS) guidelines from Section 3 of New Loka Style Guide.
 * Pure deterministic algorithms with explicit inputs and zero DOM side-effects.
 */

import { escapeHtml } from "./utils.js";

/**
 * National Early Warning Score 2 (NEWS2) Calculator
 * Conforms to Royal College of Physicians (UK) guidelines.
 */
export function calculateNEWS2({ rr, spo2, onOxygen = false, o2 = false, sbp, hr, temp, consciousness = "A", avpu = "A" }) {
  let score = 0;
  let singleParamAlert = false;

  // 1. Respiration Rate (/min)
  if (rr != null) {
    const r = Number(rr);
    if (r <= 8 || r >= 25) { score += 3; singleParamAlert = true; }
    else if (r >= 21) score += 2;
    else if (r <= 11) score += 1;
  }

  // 2. SpO2 Scale 1 (%)
  if (spo2 != null) {
    const s = Number(String(spo2).split(" ")[0]);
    if (s <= 91) { score += 3; singleParamAlert = true; }
    else if (s <= 93) score += 2;
    else if (s <= 95) score += 1;
  }

  // 3. Supplemental Oxygen
  if (onOxygen || o2) score += 2;

  // 4. Systolic Blood Pressure (mmHg)
  if (sbp != null) {
    const bp = Number(sbp);
    if (bp <= 90 || bp >= 220) { score += 3; singleParamAlert = true; }
    else if (bp <= 100) score += 2;
    else if (bp <= 110) score += 1;
  }

  // 5. Heart Rate (Pulse bpm)
  if (hr != null) {
    const p = Number(hr);
    if (p <= 40 || p >= 131) { score += 3; singleParamAlert = true; }
    else if (p >= 111) score += 2;
    else if (p <= 50 || p >= 91) score += 1;
  }

  // 6. Consciousness (Alert, Voice, Pain, Unresponsive)
  const c = (avpu && avpu !== "A") ? avpu : consciousness;
  if (c && c !== "A") { score += 3; singleParamAlert = true; }

  // 7. Temperature (°C)
  if (temp != null) {
    const t = Number(temp);
    if (t <= 35.0) { score += 3; singleParamAlert = true; }
    else if (t >= 39.1) score += 2;
    else if (t <= 36.0 || t >= 38.1) score += 1;
  }

  let risk = "Low";
  let label = "Low Clinical Risk";
  let action = "Routine ward-based observations (q4–12h).";
  let badgeClass = "news2-low";
  let color = "#10b981";

  if (score >= 7) {
    risk = "High";
    label = "High Clinical Risk — Emergency Escalation";
    action = "Immediate assessment by Medical Emergency / Critical Care Team. Continuous monitoring.";
    badgeClass = "news2-high";
    color = "#ef4444";
  } else if (score >= 5 || singleParamAlert) {
    risk = "Medium";
    label = "Medium Clinical Risk — Urgent Response Required";
    action = "Urgent review by registered nurse and attending physician. Increase observations to q1h.";
    badgeClass = "news2-medium";
    color = "#f59e0b";
  }

  return { score, risk, label, action, response: action, badgeClass, color };
}

export const calculateNews2 = calculateNEWS2;

/**
 * Real-Time Drug-Drug & Drug-Allergy Interaction Checker Rules
 */
export const DRUG_INTERACTION_RULES = [
  {
    drugA: "Warfarin",
    drugB: "Amiodarone",
    severity: "major",
    effect: "Amiodarone inhibits CYP2C9 and CYP3A4, dramatically elevating plasma warfarin concentrations and INR.",
    evidence: "Lexicomp Level 1 / FDA Boxed Warning. 3–5x increased hemorrhage risk.",
    management: "Reduce warfarin dosage by 35–50%. Check INR every 3–5 days for 2 weeks."
  },
  {
    drugA: "Warfarin",
    drugB: "Aspirin",
    severity: "major",
    effect: "Synergistic antithrombotic and antiplatelet inhibition producing severe gastrointestinal and systemic hemorrhage.",
    evidence: "AHA/ACC Antithrombotic Guidelines.",
    management: "Avoid combination unless clinically indicated for mechanical valves / ACS. Add proton pump inhibitor gastroprotection."
  },
  {
    drugA: "Metformin",
    drugB: "Contrast",
    severity: "major",
    effect: "Intravenous iodinated radiographic contrast in the setting of metformin therapy can precipitate acute kidney injury and fatal lactic acidosis.",
    evidence: "American College of Radiology (ACR) Guidelines.",
    management: "Hold metformin 48 hours prior to and 48 hours following contrast administration. Re-check serum creatinine prior to resumption."
  },
  {
    drugA: "Lisinopril",
    drugB: "Spironolactone",
    severity: "major",
    effect: "Combined renin-angiotensin-aldosterone blockade promotes severe, life-threatening hyperkalemia.",
    evidence: "RALES Trial safety profile.",
    management: "Monitor serum potassium and creatinine at baseline, 1 week, and monthly. Limit potassium supplements."
  },
  {
    drugA: "Ciprofloxacin",
    drugB: "Ondansetron",
    severity: "major",
    effect: "Additive cardiac repolarization delay causing significant QTc prolongation and risk of Torsades de Pointes.",
    evidence: "CredibleMeds Known Risk Category.",
    management: "Obtain baseline 12-lead ECG. Monitor QTc interval; select alternative antiemetic (e.g. metoclopramide) if QTc > 480ms."
  }
];

export const ALLERGY_CROSS_REACTIVITY = [
  {
    allergyPattern: /penicillin|amoxicillin|ampicillin/i,
    drugPattern: /penicillin|amoxicillin|ampicillin|augmentin/i,
    severity: "major",
    effect: "Direct beta-lactam hypersensitivity: Risk of severe IgE-mediated anaphylaxis, bronchospasm, and angioedema.",
    management: "Contraindicated. Select non-beta-lactam alternative (e.g. Vancomycin, Macrolide, Fluoroquinolone)."
  },
  {
    allergyPattern: /penicillin|amoxicillin/i,
    drugPattern: /ceftriaxone|cefazolin|cephalexin|cefuroxime/i,
    severity: "moderate",
    effect: "Cross-reactivity between penicillins and cephalosporins (approximately 2–5% risk).",
    management: "Use with caution. Pre-medicate if mild reaction, or choose alternative class if anaphylaxis history."
  },
  {
    allergyPattern: /sulfa|sulfonamide/i,
    drugPattern: /sulfamethoxazole|bactrim|septra/i,
    severity: "major",
    effect: "Sulfonamide allergy: High risk of Steven-Johnson Syndrome (SJS), toxic epidermal necrolysis, or severe urticaria.",
    management: "Absolute contraindication. Select alternative antimicrobial."
  },
  {
    allergyPattern: /aspirin|nsaid/i,
    drugPattern: /ibuprofen|naproxen|ketorolac|meloxicam/i,
    severity: "major",
    effect: "COX-1 inhibition cross-sensitivity: Risk of severe bronchospasm in triad asthma or severe anaphylactoid reaction.",
    management: "Avoid all non-steroidal anti-inflammatory agents. Use Acetaminophen for analgesia."
  }
];

export function checkClinicalInteractions(candidateDrug, activeMedications = [], patientAllergies = []) {
  const alerts = [];
  const candidateLower = candidateDrug.toLowerCase();

  // 1. Drug-Drug Interaction Checking
  for (const existingMed of activeMedications) {
    const existingLower = (existingMed.name || existingMed).toLowerCase();
    for (const rule of DRUG_INTERACTION_RULES) {
      const aLower = rule.drugA.toLowerCase();
      const bLower = rule.drugB.toLowerCase();
      if (
        (candidateLower.includes(aLower) && existingLower.includes(bLower)) ||
        (candidateLower.includes(bLower) && existingLower.includes(aLower))
      ) {
        alerts.push({
          type: "drug-drug",
          severity: rule.severity,
          drugA: existingMed.name || existingMed,
          drugB: candidateDrug,
          effect: rule.effect,
          evidence: rule.evidence,
          management: rule.management
        });
      }
    }
  }

  // 2. Drug-Allergy Checking
  for (const allergy of patientAllergies) {
    const allergyName = allergy.code?.text || allergy.code?.coding?.[0]?.display || allergy.substance || "";
    for (const rule of ALLERGY_CROSS_REACTIVITY) {
      if (rule.allergyPattern.test(allergyName) && rule.drugPattern.test(candidateDrug)) {
        alerts.push({
          type: "drug-allergy",
          severity: rule.severity,
          drugA: "Allergy: " + allergyName,
          drugB: candidateDrug,
          effect: rule.effect,
          evidence: "Documented Patient Allergy Profile",
          management: rule.management
        });
      }
    }
  }

  return alerts;
}

export function isPanicLabValue(testName = "", valueStr = "") {
  const t = testName.toLowerCase();
  const val = parseFloat(valueStr);
  if (isNaN(val)) return { isPanic: false };

  if (t.includes("potassium") || t.includes("k+")) {
    if (val < 3.0) return { isPanic: true, message: `Critical Hypokalemia (${val} mmol/L < 3.0): Immediate cardiac arrest risk` };
    if (val > 6.0) return { isPanic: true, message: `Critical Hyperkalemia (${val} mmol/L > 6.0): Life-threatening arrhythmia risk` };
  }
  if (t.includes("glucose") || t.includes("blood sugar")) {
    if (val < 50) return { isPanic: true, message: `Severe Hypoglycemia (${val} mg/dL < 50): Risk of coma and seizure` };
    if (val > 400) return { isPanic: true, message: `Severe Hyperglycemia (${val} mg/dL > 400): Diabetic ketoacidosis / HHS alert` };
  }
  if (t.includes("hemoglobin") || t.includes("hgb")) {
    if (val < 7.0) return { isPanic: true, message: `Critical Anemia (${val} g/dL < 7.0): Acute transfusion threshold exceeded` };
  }
  if (t.includes("troponin")) {
    if (val > 0.04) return { isPanic: true, message: `Elevated Troponin (${val} ng/mL > 0.04): Acute Myocardial Infarction indicator` };
  }
  if (t.includes("platelet")) {
    if (val < 50000) return { isPanic: true, message: `Critical Thrombocytopenia (${val} /uL < 50k): Severe spontaneous bleeding risk` };
  }
  if (t.includes("creatinine")) {
    if (val > 3.5) return { isPanic: true, message: `Critical Renal Failure (Cr ${val} mg/dL > 3.5): Acute kidney injury / dialysis indicator` };
  }

  return { isPanic: false };
}

export function renderSvgTrendChart(series, { title, width = 700, height = 180, yLabel = "" }) {
  if (!series || !series.length || !series[0].points || !series[0].points.length) {
    return `<div class="chart-placeholder"><p>No trend data available.</p></div>`;
  }

  const padLeft = 55;
  const padRight = 20;
  const padTop = 20;
  const padBottom = 35;
  const graphW = width - padLeft - padRight;
  const graphH = height - padTop - padBottom;

  let allY = [];
  series.forEach(s => s.points.forEach(p => allY.push(p.y)));
  let minY = Math.min(...allY);
  let maxY = Math.max(...allY);
  if (minY === maxY) { minY -= 10; maxY += 10; }
  const ySpan = maxY - minY || 1;

  const pointCount = Math.max(...series.map(s => s.points.length));
  const xStep = pointCount > 1 ? graphW / (pointCount - 1) : graphW / 2;

  let svgContent = `
    <svg viewBox="0 0 ${width} ${height}" preserveAspectRatio="none" xmlns="http://www.w3.org/2000/svg">
      <line x1="${padLeft}" y1="${padTop}" x2="${width - padRight}" y2="${padTop}" stroke="var(--color-border-subtle)" stroke-dasharray="3,3" />
      <line x1="${padLeft}" y1="${padTop + graphH / 2}" x2="${width - padRight}" y2="${padTop + graphH / 2}" stroke="var(--color-border-subtle)" stroke-dasharray="3,3" />
      <line x1="${padLeft}" y1="${padTop + graphH}" x2="${width - padRight}" y2="${padTop + graphH}" stroke="var(--color-border)" />
      <text x="${padLeft - 8}" y="${padTop + 4}" font-size="10" fill="var(--color-text-muted)" text-anchor="end">${Math.round(maxY)}</text>
      <text x="${padLeft - 8}" y="${padTop + graphH / 2 + 3}" font-size="10" fill="var(--color-text-muted)" text-anchor="end">${Math.round((minY + maxY) / 2)}</text>
      <text x="${padLeft - 8}" y="${padTop + graphH + 3}" font-size="10" fill="var(--color-text-muted)" text-anchor="end">${Math.round(minY)}</text>
  `;

  series.forEach(s => {
    const pts = s.points.map((p, idx) => {
      const x = padLeft + (idx * xStep);
      const y = padTop + graphH - ((p.y - minY) / ySpan) * graphH;
      return { x, y, val: p.y, label: p.label };
    });

    const pathD = pts.map((p, i) => (i === 0 ? `M ${p.x} ${p.y}` : `L ${p.x} ${p.y}`)).join(" ");
    svgContent += `<path d="${pathD}" fill="none" stroke="${s.color}" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" />`;

    pts.forEach(p => {
      svgContent += `
        <circle cx="${p.x}" cy="${p.y}" r="4" fill="${s.color}">
          <title>${s.name}: ${p.val} (${p.label || ""})</title>
        </circle>`;
    });
  });

  const firstS = series[0];
  firstS.points.forEach((p, idx) => {
    const x = padLeft + (idx * xStep);
    svgContent += `<text x="${x}" y="${height - 10}" font-size="10" fill="var(--color-text-muted)" text-anchor="middle">${escapeHtml(p.label || "")}</text>`;
  });

  svgContent += `</svg>`;

  return `
    <div class="svg-chart-container">
      ${svgContent}
    </div>
    <div class="chart-legend">
      ${series.map(s => `
        <div class="chart-legend-item">
          <span class="chart-legend-dot" style="background:${s.color};"></span>
          <span>${escapeHtml(s.name)}</span>
        </div>`).join("")}
    </div>`;
}

export function renderSvgSparkline(points = [], width = 120, height = 30, color = "var(--color-primary, #2a9d8f)") {
  if (!points || !points.length) {
    return `<svg width="${width}" height="${height}"><text x="5" y="${height / 2}" font-size="10" fill="var(--color-text-muted)">—</text></svg>`;
  }
  const min = Math.min(...points);
  const max = Math.max(...points);
  const range = max - min || 1;
  const pad = 4;
  const w = width - pad * 2;
  const h = height - pad * 2;
  const step = points.length > 1 ? w / (points.length - 1) : w;

  const pts = points.map((val, idx) => {
    const x = pad + idx * step;
    const y = pad + h - ((val - min) / range) * h;
    return `${x.toFixed(1)},${y.toFixed(1)}`;
  });

  return `
    <svg width="${width}" height="${height}" viewBox="0 0 ${width} ${height}" style="overflow:visible;">
      <polyline fill="none" stroke="${color}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" points="${pts.join(' ')}" />
      <circle cx="${pts[pts.length - 1].split(',')[0]}" cy="${pts[pts.length - 1].split(',')[1]}" r="3" fill="${color}" />
    </svg>`;
}

export function renderMultiLineSvg(series = [], width = 640, height = 180) {
  if (!series || !series.length) {
    return `<div class="chart-placeholder"><p>No vital trends recorded.</p></div>`;
  }
  const padLeft = 45;
  const padRight = 20;
  const padTop = 15;
  const padBottom = 25;
  const graphW = width - padLeft - padRight;
  const graphH = height - padTop - padBottom;

  let allVals = [];
  series.forEach(s => (s.points || []).forEach(p => allVals.push(Number(p) || 0)));
  if (!allVals.length) return `<div class="chart-placeholder"><p>No vital trends recorded.</p></div>`;

  let min = Math.min(...allVals);
  let max = Math.max(...allVals);
  if (min === max) { min -= 10; max += 10; }
  const range = max - min || 1;

  const maxPts = Math.max(...series.map(s => (s.points || []).length));
  const xStep = maxPts > 1 ? graphW / (maxPts - 1) : graphW / 2;

  let svg = `<svg viewBox="0 0 ${width} ${height}" preserveAspectRatio="none" style="width:100%;height:${height}px;background:var(--color-surface);border-radius:6px;">
    <line x1="${padLeft}" y1="${padTop}" x2="${width - padRight}" y2="${padTop}" stroke="var(--color-border-subtle, rgba(0,0,0,0.06))" stroke-dasharray="3,3" />
    <line x1="${padLeft}" y1="${padTop + graphH / 2}" x2="${width - padRight}" y2="${padTop + graphH / 2}" stroke="var(--color-border-subtle, rgba(0,0,0,0.06))" stroke-dasharray="3,3" />
    <line x1="${padLeft}" y1="${padTop + graphH}" x2="${width - padRight}" y2="${padTop + graphH}" stroke="var(--color-border, rgba(0,0,0,0.15))" />
    <text x="${padLeft - 6}" y="${padTop + 4}" font-size="10" fill="var(--color-text-muted)" text-anchor="end">${Math.round(max)}</text>
    <text x="${padLeft - 6}" y="${padTop + graphH / 2 + 3}" font-size="10" fill="var(--color-text-muted)" text-anchor="end">${Math.round((min + max) / 2)}</text>
    <text x="${padLeft - 6}" y="${padTop + graphH + 3}" font-size="10" fill="var(--color-text-muted)" text-anchor="end">${Math.round(min)}</text>`;

  series.forEach(s => {
    const pts = (s.points || []).map((val, idx) => {
      const v = Number(val) || 0;
      const x = padLeft + idx * xStep;
      const y = padTop + graphH - ((v - min) / range) * graphH;
      return { x, y, v };
    });
    if (pts.length > 0) {
      const polyPts = pts.map(p => `${p.x.toFixed(1)},${p.y.toFixed(1)}`).join(' ');
      svg += `<polyline fill="none" stroke="${s.color}" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" points="${polyPts}" />`;
      pts.forEach(p => {
        svg += `<circle cx="${p.x.toFixed(1)}" cy="${p.y.toFixed(1)}" r="3.5" fill="${s.color}"><title>${escapeHtml(s.label)}: ${p.v}</title></circle>`;
      });
    }
  });

  svg += `</svg>`;
  return svg;
}

export function renderNews2Gauge(score = 0) {
  const s = Math.max(0, Math.min(Number(score) || 0, 20));
  let color = "#10b981";
  let label = "LOW";
  if (s >= 7) { color = "#ef4444"; label = "HIGH"; }
  else if (s >= 5) { color = "#f59e0b"; label = "MED"; }

  return `
    <div style="text-align:center;width:80px;">
      <div style="font-size:2.2rem;font-weight:800;color:${color};line-height:1;">${s}</div>
      <div style="font-size:0.72rem;font-weight:700;color:${color};margin-top:0.25rem;">${label} RISK</div>
    </div>`;
}
