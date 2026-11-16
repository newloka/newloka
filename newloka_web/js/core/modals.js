/**
 * Modal System & Tiered Clinical Decision Support (CDS) Alert Manager
 *
 * Conforms to Section 5.3 of New Loka Style Guide (DCB0129 Clinical Risk Management).
 * Prevents alert fatigue by distinguishing Class C Fatal Hazards, Class B Warnings, and Class A Guidance.
 */

import { $, escapeHtml } from "./utils.js";

export function openModal(title, contentHtml, onReady) {
  const titleEl = $("#modal-title");
  if (titleEl) titleEl.textContent = title;
  const body = $("#modal-body");
  if (body) {
    body.innerHTML = contentHtml;
    const overlay = $("#modal-overlay");
    if (overlay) overlay.classList.remove("hidden");
    if (typeof onReady === "function") {
      onReady(body);
    }
  }
}

export function closeModal() {
  const overlay = $("#modal-overlay");
  if (overlay) overlay.classList.add("hidden");
  const body = $("#modal-body");
  if (body) body.innerHTML = "";
}

export function initModalListeners() {
  const closeBtn = $("#modal-close");
  if (closeBtn) {
    closeBtn.addEventListener("click", closeModal);
  }
  const overlay = $("#modal-overlay");
  if (overlay) {
    overlay.addEventListener("click", e => {
      if (e.target === overlay) closeModal();
    });
  }
}

let _alertTimer = null;

/**
 * Tier 2 (Class B) Amber Banner or General Notification
 */
export function showAlert(msg, type = "warn") {
  const bar = $("#alert-bar");
  if (!bar) return;
  bar.innerHTML = `<span>${escapeHtml(msg)}</span><button class="close-alert" aria-label="Dismiss">✕</button>`;
  bar.className = "alert-bar " + type;
  const closeBtn = bar.querySelector(".close-alert");
  if (closeBtn) {
    closeBtn.onclick = () => bar.classList.add("hidden");
  }
  if (_alertTimer) clearTimeout(_alertTimer);
  _alertTimer = setTimeout(() => bar.classList.add("hidden"), type === "err" ? 6000 : 3500);
}

/**
 * Tier 1 (Class C) Fatal Hazard Interruptive Modal
 * Mandates explicit clinical justification override.
 */
export function showClinicalHazardModal({ title = "FATAL CLINICAL HAZARD", hazardText, onOverride, onCancel }) {
  const content = `
    <div class="hazard-modal-container" style="border-left: 6px solid #ef4444; padding: 1rem; background: rgba(239,68,68,0.08); border-radius: 4px;">
      <h4 style="color:#ef4444; margin-top:0;">⚠️ Class C Alert: Lethal Risk Detected</h4>
      <p style="font-weight:600; font-size:1.05rem;">${escapeHtml(hazardText)}</p>
      <div style="margin-top:1.5rem;">
        <label style="display:block; font-size:0.85rem; font-weight:700; margin-bottom:0.4rem;">
          Mandatory Clinical Justification for Override:
        </label>
        <textarea id="hazard-override-reason" class="input" rows="3" placeholder="Enter clinical rationale, risk-benefit evaluation, and monitoring plan..." style="width:100%;"></textarea>
      </div>
      <div style="display:flex; justify-content:flex-end; gap:0.75rem; margin-top:1.25rem;">
        <button id="hazard-cancel-btn" class="btn btn-outline">Cancel / Modify Order</button>
        <button id="hazard-override-btn" class="btn btn-danger" style="background:#ef4444; color:#fff;">Confirm Break-Glass Override</button>
      </div>
    </div>
  `;

  openModal(title, content, body => {
    const cancelBtn = body.querySelector("#hazard-cancel-btn");
    const overrideBtn = body.querySelector("#hazard-override-btn");
    const reasonEl = body.querySelector("#hazard-override-reason");

    if (cancelBtn) {
      cancelBtn.onclick = () => {
        closeModal();
        if (typeof onCancel === "function") onCancel();
      };
    }

    if (overrideBtn) {
      overrideBtn.onclick = () => {
        const reason = reasonEl ? reasonEl.value.trim() : "";
        if (!reason) {
          showAlert("Clinical justification is mandatory to override a Class C hazard.", "err");
          if (reasonEl) reasonEl.focus();
          return;
        }
        closeModal();
        if (typeof onOverride === "function") onOverride(reason);
      };
    }
  });
}
