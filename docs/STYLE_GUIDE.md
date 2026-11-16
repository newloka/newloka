# New Loka Senior Engineering & Code Style Guide
**The Definitive Standard for High-Assurance, Local-First, UX-Forward Healthcare Software**
*Authoritative Architecture Specification | Maintained by Amethyst (`newloka-core`)*

---

## 1. Epistemic Foundation: Zero-Debt Senior Architecture

High-consequence healthcare software exists at the intersection of systems engineering, cryptography, and clinical ethics. Historically, healthcare platforms rot and fail under two equal and opposing poisons:

```
                              THE DUAL POISONS
   ┌──────────────────────────────────┐    ┌──────────────────────────────────┐
   │          TECHNICAL DEBT          │    │         IDEOLOGICAL DEBT         │
   │        (The Slop Trap)           │    │         (The Dogma Trap)         │
   ├──────────────────────────────────┤    ├──────────────────────────────────┤
   │ • Ad-hoc copy-pasted boilerplate │    │ • Premature multi-layer traits   │
   │ • Uncontrolled side-effects      │    │ • Frameworks inside frameworks   │
   │ • Leaky global state & mutations │    │ • Rigid premature "invariants"   │
   │ • Silent failures & unwrapped nulls│   │ • Over-abstracted type lasagna   │
   │ • Cryptic UI error codes         │    │ • Inability to adapt to reality  │
   │ ──► Fatal clinical omissions     │    │ ──► Fragile, convoluted bypasses │
   └──────────────────────────────────┘    └──────────────────────────────────┘
```

New Loka avoids both traps through **Foundational Axioms + Adaptive Senior Engineering**:
1. **Local-First Primacy:** All clinical operations function offline on the local node. Sync is purely additive.
2. **Data-Layer Cryptographic Security:** Confidentiality (AES-256-GCM AEAD) and Attribute-Based Access Control (ABAC) live where data lives, never delegated to client-side UI visibility.
3. **Clinical Prudence & Non-Dangerous Merge:** Critical clinical data (medications, allergies, active orders) must **never** be auto-resolved by CRDT heuristics; machine extractions (OCR, STT, AI) mandate human-in-the-loop clinician sign-off.
4. **Polymorphic Single Core:** The same Rust engine (`newloka_core`) powers T0 (solo doctor) to T4 (federated research network) via runtime presets and configuration without code forks.
5. **Ergonomic Empathy & The 3-Tap Invariant:** Critical patient data must be reachable within 3 taps or clicks from any screen via a persistent Storyboard rail.

Beyond these axioms, code must be clean, pragmatic, testable, and maintainable.

---

## 2. Pattern Engineering: The WATER Standard (WET then DRY)

New Loka strictly enforces **WATER: Write Everything Twice, then DRY (Don't Repeat Yourself)**.

```
       Occurrence 1              Occurrence 2              Occurrence 3+
   ┌──────────────────┐      ┌──────────────────┐      ┌──────────────────┐
   │  Write Concrete  │ ───► │  Write Concrete  │ ───► │ Distill Pattern  │
   │   (Direct Code)  │      │   (Independent)  │      │  (DRY Helper/Fn) │
   └──────────────────┘      └──────────────────┘      └──────────────────┘
     Resist abstraction!       Verify true commonality    Extract pure abstraction
```

### 2.1 The Mathematical & Cognitive Rationale
- **Two points define a line, not a curve.** When you have only two implementations ($N=1, 2$), you do not have enough variance to identify the true invariant. Creating a shared abstraction at $N=2$ almost always builds the wrong abstraction, forcing future callers to accept awkward parameters, boolean flags, or leaky interfaces.
- **Duplication is far cheaper than the wrong abstraction.** Duplicate code is easily localized and rewritten. A flawed, premature abstraction spreads indirection across the entire repository.
- **Three points define a pattern.** At $N=3$, the common structural invariant is unmistakable. DRY is applied strictly at $N=3+$ to prevent boilerplate rot.

### 2.2 The 3-Strike Extraction Protocol
1. **Strike 1 ($N=1$ - Concrete):** Write the code inline, directly and specifically for the immediate clinical workflow or data model. Do not declare traits, generic macros, or base classes.
2. **Strike 2 ($N=2$ - Concrete & Independent):** Write the second occurrence directly. Resist the urge to abstract! Compare the two call sites side-by-side to observe where they actually diverge.
3. **Strike 3 ($N=3+$ - Distill & Extract):** Now that three independent call sites exist with identical structure, extract the common pattern into a pure helper function, reusable component, or Rust utility. Refactor all three call sites to consume the distilled abstraction.

### 2.3 Concrete Examples of the WATER Lifecycle

#### Rust Example: Anti-Pattern vs. WATER Lifecycle
```rust
// ❌ ANTI-PATTERN: Premature Abstraction at N=1
// An engineer creates an over-engineered trait with 4 associated types for one simple query.
pub trait ClinicalResourceQueryHandler<R: FhirResource, P: QueryPayload> {
    type Output;
    type Error;
    fn execute(&self, resource: R, payload: P) -> Result<Self::Output, Self::Error>;
}

// ✅ WATER STRIKE 1 & 2 (Concrete):
// In patient handler:
let patient: Patient = storage.get_patient(&patient_id).await?;
// In observation handler:
let obs: Observation = storage.get_observation(&obs_id).await?;

// ✅ WATER STRIKE 3 (Distill at N=3):
// When Patient, Observation, and MedicationRequest all need identical encrypted-fetch-and-deserialize logic:
pub async fn fetch_and_decrypt_resource<T: DeserializeOwned>(
    storage: &StorageEngine,
    resource_type: &str,
    id: &str,
) -> Result<T, StorageError> {
    let encrypted = storage.get_raw_record(resource_type, id).await?;
    let decrypted = storage.decrypt_record(&encrypted).await?;
    let resource: T = serde_json::from_slice(&decrypted.payload)
        .map_err(|e| StorageError::SerializationFailure(e.to_string()))?;
    Ok(resource)
}
```

#### JavaScript Example: Anti-Pattern vs. WATER Lifecycle
```javascript
// ❌ ANTI-PATTERN: Premature Class Factory at N=1
class AbstractClinicalCardRendererFactory {
    constructor(config) { this.config = config; }
    create(type) { /* 100 lines of dynamic reflection */ }
}

// ✅ WATER STRIKE 1 & 2 (Concrete):
// In Labs View: renderLabResultCard(lab)
// In Radiology View: renderRadiologyStudyCard(study)

// ✅ WATER STRIKE 3 (Distill at N=3):
// When Labs, Radiology, and Pathology all need identical diagnostic status cards:
export function renderDiagnosticCard({ title, subtitle, status, priority, date, contentHtml, onAction }) {
    const priorityClass = priority === 'stat' ? 'badge-stat' : priority === 'urgent' ? 'badge-urgent' : 'badge-routine';
    return `
        <div class="diagnostic-card ${priorityClass}">
            <div class="card-header">
                <h4>${escapeHtml(title)}</h4>
                <span class="badge ${priorityClass}">${escapeHtml(priority.toUpperCase())}</span>
            </div>
            <p class="card-meta">${escapeHtml(subtitle)} • ${formatClinicalDate(date)}</p>
            <div class="card-body">${contentHtml}</div>
        </div>
    `;
}
```

---

## 3. Functional Core, Imperative Shell (FCIS) & Deterministic Computing

In medical computing, non-determinism kills. A dosage calculation, NEWS2 early warning score, or drug-drug interaction check must **never** depend on ambient environment variables, hidden mutable state, or unpassed clocks.

```
┌─────────────────────────────────────────────────────────────────┐
│                        IMPERATIVE SHELL                         │
│   Network / Axum Handlers / SQLite Storage / DOM Event I/O       │
│                                                                 │
│         ┌─────────────────────────────────────────────┐         │
│         │                  PURE CORE                  │         │
│         │   • Clinical Scoring (NEWS2, eGFR, DKA)     │         │
│         │   • Drug Safety & Allergy Screening Shield  │         │
│         │   • Differential Privacy Laplace Mechanism  │         │
│         │   • FHIR Validations & Transforms           │         │
│         │   • View-Model HTML String Generators       │         │
│         └─────────────────────────────────────────────┘         │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 3.1 The Inviolable Purity Rule
> **All clinical calculations, scoring algorithms, safety screening, formatters, and data transformations MUST be pure functions with zero side-effects.**

#### Properties of Pure Functions:
1. **Explicit Inputs $\to$ Explicit Outputs:** The returned value is entirely determined by its arguments.
2. **Zero Hidden State:** No reading or mutating global variables, outer closures, or singleton states.
3. **No Ambient Time or Entropy:** If logic depends on the current time or a random seed, pass `reference_time: DateTime<Utc>` or `seed: &[u8; 32]` explicitly. Calling `Utc::now()` or random generators deep inside calculation logic is strictly prohibited.
4. **Zero I/O:** No database queries, no file reads, no network calls, and no direct DOM manipulations inside the pure core.

### 3.2 Pure Clinical Calculation Examples

#### NEWS2 Score Calculator (Rust)
```rust
/// Pure calculation of National Early Warning Score 2 (NEWS2).
/// Zero allocations, zero side-effects, 100% deterministic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct News2Result {
    pub total_score: u32,
    pub risk_tier: ClinicalRiskTier,
    pub triggers_stat_alert: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClinicalRiskTier {
    Low,       // Score 0-4
    Medium,    // Score 5-6 or any single parameter = 3
    High,      // Score 7+
}

pub fn calculate_news2(
    resp_rate: u32,
    spo2_percent: u32,
    is_on_supplemental_o2: bool,
    systolic_bp: u32,
    heart_rate: u32,
    conscious_level_avpu: AvpuScore,
    temperature_celsius: f32,
) -> News2Result {
    let mut score = 0;
    let mut has_extreme_trigger = false;

    // Respiration rate
    let rr_score = match resp_rate {
        0..=8 => 3,
        9..=11 => 1,
        12..=20 => 0,
        21..=24 => 2,
        _ => 3,
    };
    if rr_score == 3 { has_extreme_trigger = true; }
    score += rr_score;

    // Pulse oximetry Scale 1
    let spo2_score = match spo2_percent {
        0..=91 => 3,
        92..=93 => 2,
        94..=95 => 1,
        _ => 0,
    };
    if spo2_score == 3 { has_extreme_trigger = true; }
    score += spo2_score;

    // Oxygen delivery
    if is_on_supplemental_o2 { score += 2; }

    // Systolic Blood Pressure
    let sbp_score = match systolic_bp {
        0..=90 => 3,
        91..=100 => 2,
        101..=110 => 1,
        111..=219 => 0,
        _ => 3,
    };
    if sbp_score == 3 { has_extreme_trigger = true; }
    score += sbp_score;

    // Pulse
    let hr_score = match heart_rate {
        0..=40 => 3,
        41..=50 => 1,
        51..=90 => 0,
        91..=110 => 1,
        111..=130 => 2,
        _ => 3,
    };
    if hr_score == 3 { has_extreme_trigger = true; }
    score += hr_score;

    // Consciousness
    let avpu_score = match conscious_level_avpu {
        AvpuScore::Alert => 0,
        AvpuScore::Voice | AvpuScore::Pain | AvpuScore::Unresponsive => 3,
    };
    if avpu_score == 3 { has_extreme_trigger = true; }
    score += avpu_score;

    // Temperature
    let temp_score = if temperature_celsius <= 35.0 {
        3
    } else if temperature_celsius <= 36.0 {
        1
    } else if temperature_celsius <= 38.0 {
        0
    } else if temperature_celsius <= 39.0 {
        1
    } else {
        2
    };
    if temp_score == 3 { has_extreme_trigger = true; }
    score += temp_score;

    let risk_tier = if score >= 7 {
        ClinicalRiskTier::High
    } else if score >= 5 || has_extreme_trigger {
        ClinicalRiskTier::Medium
    } else {
        ClinicalRiskTier::Low
    };

    News2Result {
        total_score: score,
        risk_tier,
        triggers_stat_alert: score >= 7 || has_extreme_trigger,
    }
}
```

### 3.3 High Assertion Density (NASA JPL Rule 5 Adaptation)
For internal invariants that should be logically impossible, use assertions in debug builds and explicit error returns in production:
- Average 1–2 assertions per complex domain transformation checking pre- and post-conditions.
- Assertions must be side-effect free: `debug_assert!(end_time >= start_time)`.
- Never use assertions for external user input or network data validation—use `Result<T, E>`.

---

## 4. Rust Systems Architecture & Coding Standards

New Loka's core systems (`newloka_core`, `newloka_server`, `newloka_cli`, `newloka_demo`) demand the highest standards of safety, memory efficiency, and concurrency discipline.

### 4.1 Type-Driven Domain Modeling (Make Illegal States Unrepresentable)
- **Typestate Pattern:** Use phantom data or dedicated types to enforce clinical workflows at compile time:
  ```rust
  pub struct Prescription<State> {
      id: PrescriptionId,
      patient_id: PatientId,
      medications: Vec<MedicationItem>,
      state: State,
  }
  pub struct Draft;
  pub struct ClinicianSigned { pub signature: Ed25519Signature, pub signer_reg_no: String }
  pub struct Dispensed { pub pharmacist_id: String, pub timestamp: DateTime<Utc> }

  // A prescription cannot be dispensed unless it has been signed:
  impl Prescription<ClinicianSigned> {
      pub fn dispense(self, pharmacist_id: String, now: DateTime<Utc>) -> Prescription<Dispensed> {
          Prescription {
              id: self.id,
              patient_id: self.patient_id,
              medications: self.medications,
              state: Dispensed { pharmacist_id, timestamp: now },
          }
      }
  }
  ```
- **Newtypes over Primitives:** Never use raw `String` for clinical identifiers:
  ```rust
  #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
  pub struct PatientId(pub String);

  #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
  pub struct MedicalRecordNumber(pub String);
  ```

### 4.2 Naming & Idiomatic Rust API Guidelines
- **Conversion Methods:**
  - `as_`: Cheap borrowed view (`fn as_bytes(&self) -> &[u8]`).
  - `to_`: Expensive conversion with allocation (`fn to_fhir_json(&self) -> String`).
  - `into_`: Consumes ownership (`fn into_inner(self) -> T`).
- **Getters:** Omit the `get_` prefix (`patient.name()`, not `patient.get_name()`). Reserve `get_` for fallible or computed lookups (`storage.get_patient(&id)`).
- **Common Traits:** Eagerly implement `Debug`, `Clone`, `PartialEq`, `Eq`, `Default` on all data structs where applicable.

### 4.3 Memory Allocation & Zero-Cost Abstractions
- **Zero-Allocation Lookups:** Hot clinical paths (drug formulary search, brand-to-generic mapping, SNOMED code resolution) must use in-memory Trie or inverted hash indexes over borrowed slices (`&str`), never allocating heap memory per keystroke.
- **Compile-Time Asset Embedding:** Standardized clinical monographs (Indian Pharmacopoeia IP 2022, NLEM) and static SPA web assets must be embedded at compile-time via `include_str!` or `rust-embed`.

### 4.4 Asynchronous Concurrency Hygiene (Tokio / Axum)
- **Zero Blocking in Async Tasks:** Never perform synchronous file I/O, heavy cryptography (Argon2id hashing), or CPU loops on Tokio worker threads.
- **Offload Blocking Work:** Use `tokio::task::spawn_blocking` for:
  - Argon2id password hashing
  - Database schema migrations
  - Heavy report generation / PDF rendering
- **Cancellation Safety:** Ensure asynchronous futures in channels and HTTP request handlers can be dropped safely at any `await` point without corrupting in-memory state or leaving orphaned locks.

### 4.5 Cryptographic & Audit Hygiene
- **AEAD Encryption at Rest:** Every FHIR resource payload written to disk must pass through `AES-256-GCM` with a unique 96-bit nonce.
- **Key Hierarchy:** Device Master Key (DMK) wraps Patient Record Keys (PRK).
- **Constant-Time Comparison:** All authentication tokens, signatures, and cryptographic tags must use constant-time comparison (`subtle::ConstantTimeEq`) to prevent timing side-channel attacks.
- **Zeroize Memory:** Sensitive key buffers must implement `zeroize::Zeroize` to scrub key bytes from RAM on drop.

### 4.6 Error Handling Taxonomy
- **Library Crates (`newloka_core`):** Use `thiserror` for precise, strongly-typed domain errors:
  ```rust
  #[derive(thiserror::Error, Debug)]
  pub enum PharmacopoeiaError {
      #[error("Monograph not found for concept: {0}")]
      MonographNotFound(String),
      #[error("Clinical safety conflict: {0}")]
      SafetyConflict(String),
      #[error("Regulatory schedule restriction: {0}")]
      RegulatoryRestriction(String),
  }
  ```
- **Application Binaries (`newloka_server`, `newloka_cli`):** Use `anyhow::Result` with `.context("...")` to provide clear forensic logs.
- **Banned:** `unwrap()` and `expect()` are strictly prohibited in production server and library paths.

---

## 5. Frontend Architecture, Clinical UX & Human Factors Standards

New Loka's web interface (`newloka_web`) is an air-gapped, zero-dependency, vanilla ES6 Single Page Application engineered for clinical reliability and zero latency.

### 5.1 100% Air-Gapped Autonomy & Zero Third-Party CDNs
- **Absolute CDN Ban:** No `<script>` or `<link>` tags pointing to `unpkg.com`, `cdnjs`, `google fonts`, or external analytics. Every icon, font, and script must be bundled locally inside the binary.
- **Vanilla ES6 Architecture:** Zero heavyweight JS frameworks (React, Angular, Vue) that decay or require heavy node build tools. Clean ES6 modules, custom events, and native Web APIs provide sub-millisecond execution and decade-long durability.

### 5.2 NIST Safety-Enhanced Design: The Persistent 260px Storyboard Rail
To eliminate wrong-patient errors (a leading cause of medical malpractice), the **Epic-Style Storyboard** context rail is permanently mounted on the left viewport across all clinical views:

```
┌─────────────────┬────────────────────────────────────────────────────────┐
│   STORYBOARD    │                     MAIN CLINICAL VIEW                 │
│     (260px)     │                                                        │
│ ┌─────────────┐ │  [Patient Timeline]  [eRx Pad]  [CPOE Orders]  [Labs]  │
│ │   [Photo]   │ │                                                        │
│ └─────────────┘ │  Active Clinical Workspace                             │
│ Patient Name    │                                                        │
│ Age/Sex • MRN   │                                                        │
│ ─────────────── │                                                        │
│ Code Status:    │                                                        │
│ [FULL CODE]     │                                                        │
│ ─────────────── │                                                        │
│ Allergies:      │                                                        │
│ ⚠ Penicillin    │                                                        │
│ ⚠ Sulfa         │                                                        │
│ ─────────────── │                                                        │
│ Vitals & NEWS2: │                                                        │
│ BP: 120/80      │                                                        │
│ HR: 72 | SpO2:98│                                                        │
│ NEWS2: 0 (Low)  │                                                        │
└─────────────────┴────────────────────────────────────────────────────────┘
```
- A clinician must **never** wonder which patient chart is active when switching between eRx, Labs, Radiology, or Progress Notes.

### 5.3 DCB0129 Clinical Risk Management: Alert Fatigue Mitigation
Excessive popups cause alarm normalization and reflex dismissal. New Loka classifies Clinical Decision Support (CDS) alerts into strict tiers:

| Alert Tier | Presentation | Trigger Criteria | Action Required |
|---|---|---|---|
| **Tier 1: Fatal Hazard (Class C)** | Interruptive Modal | Anaphylactic drug allergy, lethal drug-drug interaction, critical panic lab | Explicit clinician acknowledgment + mandatory clinical justification override |
| **Tier 2: Clinical Warning (Class B)** | Inline Amber Banner / Strip | Duplicate therapeutic class, renal dosage adjustment needed | One-click inline remedy button (e.g., "[Switch to Paracetamol]") |
| **Tier 3: Passive Guidance (Class A)** | Subtle Badge / Icon Flag | Guideline recommendation, preventive screening due | Non-interruptive visual badge |

### 5.4 The 3-Tap Invariant & Clinical Velocity
- **Bedside Speed:** Active medications, allergies, code status, emergency contacts, and vital trends must be reachable within 3 taps or clicks from any screen.
- **Keyboard-First Design:**
  - `Ctrl + K`: Global patient spotlight search.
  - `Ctrl + Enter`: Sign and commit prescription or order.
  - `Esc`: Close active modal or dismiss flyout.
  - Logical tab order: Focus automatically advances through clinical entry fields without mouse dependence.

### 5.5 DOM Security & Template Hygiene
- **Never Inject Unsanitized Input:** Always pass patient-entered data through `escapeHtml()` before interpolating into HTML template strings:
  ```javascript
  export function escapeHtml(str) {
      if (str === null || str === undefined) return '';
      return String(str)
          .replace(/&/g, '&amp;')
          .replace(/</g, '&lt;')
          .replace(/>/g, '&gt;')
          .replace(/"/g, '&quot;')
          .replace(/'/g, '&#39;');
  }
  ```
- **Content Security Policy (CSP):** Strict compatibility with `default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'`. No `eval()`, no `Function()` constructors.

### 5.6 Modular Presets (`PRESET_REGISTRY`)
- The clinical SPA includes 36 verified views. Rather than forcing administrators to configure 50 granular switches, the application defines 22 declarative presets in `config.js` across four archetypes:
  1. **Physicians:** T0 Solo GP, Pediatrician, Cardiologist, Surgeon, Emergency Physician.
  2. **Nursing:** Ward Nurse, Triage Nurse, ICU Nurse.
  3. **Diagnostics & Pharmacy:** Radiologist, Pathologist, Hospital Pharmacist.
  4. **Administration:** Cashier, Medical Records Officer, Hospital Administrator.
- Applying a preset dynamically adjusts active navigation, ABAC bounds, and default landing views with zero page reloads.

---

## 6. Clinical Information Systems & HL7 FHIR R4 Standards

### 6.1 Strict FHIR R4 Resource Conformance
All clinical entities are modeled as valid HL7 FHIR R4 resources:
- Patient Demographics: `Patient`
- Clinical Encounters: `Encounter`
- Vitals & Labs: `Observation`
- Diagnoses & Problem List: `Condition`
- Prescriptions: `MedicationRequest`
- Pharmacy Dispensing: `MedicationDispense`
- Diagnostic & Lab Orders: `ServiceRequest`
- Results & Reports: `DiagnosticReport`
- Allergies: `AllergyIntolerance`
- Privacy Directives: `Consent`

### 6.2 5-Stage CPOE State Machine
Every computerized provider order entry (CPOE) follows a strict lifecycle:
$$\text{Draft} \xrightarrow{\text{Sign}} \text{Placed} \xrightarrow{\text{Acknowledge}} \text{In-Progress} \xrightarrow{\text{Verify / Complete}} \text{Completed}$$
$$\text{Any Active State} \xrightarrow{\text{Cancel (Requires Reason)}} \text{Cancelled}$$
- Order state transitions must be audited with Ed25519 signatures.
- Stat/Urgent priority orders trigger visual pulse indicators in ward and department queues.

### 6.3 Non-Dangerous Merge & Anti-Auto-Commit Rule
- **Vector Clock Merge:** Non-clinical administrative metadata (phone number update, spelling correction) may merge automatically across mesh nodes.
- **Inviolable Clinical Rule:** If two nodes generate concurrent, conflicting clinical records (e.g., Node A discontinues Atenolol while Node B titrates Atenolol to 50mg), the system **MUST NOT** auto-merge. It must flag the conflict for human clinician review.
- **Draft AI/OCR Extractions:** Any document parsed by OCR or audio captured by speech-to-text sits in an uncommitted draft staging table until signed off by a human clinician.

---

## 7. Testing, Verification & Continuous Epistemic Governance

### 7.1 Testing Pyramid
```
                   ▲
                  / \     E2E Server & API Tests
                 /   \    (Axum routes, ABAC isolation, SSE)
                /─────\
               /       \   Integration Tests
              /         \  (SQLx in-memory SQLite, CRDT sync, crypto)
             /───────────\
            /             \ Pure Unit Tests
           /               \(NEWS2, eGFR, safety screening, formatters)
          └─────────────────┘
```

1. **Pure Unit Tests:** Fast, zero-allocation, 100% deterministic tests for all mathematical and clinical formulas.
2. **Integration Tests:** Use in-memory SQLite (`sqlite::memory:?cache=shared`) to verify transactional integrity, encryption, and vector clock mechanics without polluting disk state.
3. **Server & ABAC Tests:** Verify that unauthorized roles (e.g., `Role::Billing`) receive redacted clinical data and that emergency break-glass overrides log high-severity audit entries.
4. **Assertion:** `cargo test --workspace` must pass with **0 failures**.

### 7.2 Bi-Temporal Knowledge Graph Governance
Every architectural fact, database schema revision, and clinical invariant must be synchronized to the bi-temporal knowledge graph at `knowledge_graph/`:
- Every fact carries `[valid_from: YYYY-MM-DD, recorded: YYYY-MM-DD, source: ...]`.
- Contradicted facts move to `## Superseded`—history is never deleted.
- Continuous verification gate:
  ```powershell
  python knowledge_graph/tools/verify_graph.py
  ```
  Must strictly output: `Errors: 0, Warnings: 0`.

---

## 8. Senior Architect Review Checklist

Before opening any PR, submitting code, or proposing an architectural iteration, verify:

- [ ] **WATER Compliance:** Did you write it concrete for $N=1, 2$ and resist premature abstraction? If this is $N=3+$, did you distill the pattern cleanly?
- [ ] **Pure Core Separation:** Is the clinical calculation, validation, or transformation a pure function with zero side-effects? Are clocks and seeds passed as arguments?
- [ ] **Type Safety:** Are illegal clinical states unrepresentable? Are newtypes used instead of raw strings?
- [ ] **Error Boundaries:** Are domain errors typed via `thiserror` without a single production `unwrap()`?
- [ ] **Air-Gapped Autonomy:** Is the frontend 100% free of external CDN links or unpinned remote resources?
- [ ] **Human Factors & Alert Fatigue:** Does the UI mount the 260px Storyboard? Are interruptive alerts restricted strictly to Class C fatal hazards?
- [ ] **Clinical Safety & CPOE:** Does the CPOE order follow the 5-stage state machine? Are conflicting clinical merges flagged for human review?
- [ ] **Workspace Health:** Does `cargo test --workspace` pass with 0 failures, and `verify_graph.py` pass with 0 errors and 0 warnings?
