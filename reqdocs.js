const {
  Document, Packer, Paragraph, TextRun, Table, TableRow, TableCell,
  HeadingLevel, AlignmentType, BorderStyle, WidthType, ShadingType,
  Footer, LevelFormat, PageBreak, TabStopType, TabStopPosition,
  PageNumber, NumberFormat
} = require('docx');
const fs = require('fs');

const BRAND = "1A2E4A";      // Deep navy
const ACCENT = "00A99D";     // Teal
const ACCENT2 = "E8734A";    // Warm orange
const LIGHT_BG = "EEF4F7";
const MID_BG = "D0E8E5";
const WHITE = "FFFFFF";
const DARK_TEXT = "1A2E4A";
const GRAY = "6B7C8D";

const border = { style: BorderStyle.SINGLE, size: 1, color: "C8D8E0" };
const borders = { top: border, bottom: border, left: border, right: border };
const noBorder = { style: BorderStyle.NONE, size: 0, color: "FFFFFF" };
const noBorders = { top: noBorder, bottom: noBorder, left: noBorder, right: noBorder };

function h1(text) {
  return new Paragraph({
    heading: HeadingLevel.HEADING_1,
    spacing: { before: 400, after: 160 },
    border: { bottom: { style: BorderStyle.SINGLE, size: 8, color: ACCENT, space: 6 } },
    children: [new TextRun({ text, font: "Arial", size: 32, bold: true, color: BRAND })]
  });
}

function h2(text) {
  return new Paragraph({
    heading: HeadingLevel.HEADING_2,
    spacing: { before: 280, after: 120 },
    children: [new TextRun({ text, font: "Arial", size: 26, bold: true, color: ACCENT })]
  });
}

function h3(text) {
  return new Paragraph({
    heading: HeadingLevel.HEADING_3,
    spacing: { before: 200, after: 80 },
    children: [new TextRun({ text, font: "Arial", size: 22, bold: true, color: DARK_TEXT })]
  });
}

function body(text, opts = {}) {
  return new Paragraph({
    spacing: { before: 60, after: 80 },
    children: [new TextRun({ text, font: "Arial", size: 20, color: DARK_TEXT, ...opts })]
  });
}

function mono(text) {
  return new Paragraph({
    spacing: { before: 40, after: 40 },
    indent: { left: 360 },
    children: [new TextRun({ text, font: "Courier New", size: 18, color: "2D6A4F" })]
  });
}

function bullet(text, level = 0) {
  return new Paragraph({
    numbering: { reference: "bullets", level },
    spacing: { before: 40, after: 40 },
    children: [new TextRun({ text, font: "Arial", size: 20, color: DARK_TEXT })]
  });
}

function numbered(text, level = 0) {
  return new Paragraph({
    numbering: { reference: "numbers", level },
    spacing: { before: 40, after: 40 },
    children: [new TextRun({ text, font: "Arial", size: 20, color: DARK_TEXT })]
  });
}

function note(text) {
  return new Paragraph({
    spacing: { before: 80, after: 80 },
    indent: { left: 360 },
    border: { left: { style: BorderStyle.SINGLE, size: 16, color: ACCENT2, space: 8 } },
    children: [new TextRun({ text, font: "Arial", size: 19, color: "5A4030", italics: true })]
  });
}

function gap(size = 120) {
  return new Paragraph({ spacing: { before: size, after: 0 }, children: [new TextRun("")] });
}

function headerRow(cols, widths) {
  return new TableRow({
    tableHeader: true,
    children: cols.map((c, i) => new TableCell({
      width: { size: widths[i], type: WidthType.DXA },
      shading: { fill: BRAND, type: ShadingType.CLEAR },
      borders,
      margins: { top: 80, bottom: 80, left: 120, right: 120 },
      children: [new Paragraph({ children: [new TextRun({ text: c, font: "Arial", size: 18, bold: true, color: WHITE })] })]
    }))
  });
}

function dataRow(cols, widths, shade = false) {
  return new TableRow({
    children: cols.map((c, i) => new TableCell({
      width: { size: widths[i], type: WidthType.DXA },
      shading: { fill: shade ? LIGHT_BG : WHITE, type: ShadingType.CLEAR },
      borders,
      margins: { top: 80, bottom: 80, left: 120, right: 120 },
      children: [new Paragraph({ children: [new TextRun({ text: c, font: "Arial", size: 18, color: DARK_TEXT })] })]
    }))
  });
}

function table(headers, rows, widths) {
  const total = widths.reduce((a, b) => a + b, 0);
  return new Table({
    width: { size: total, type: WidthType.DXA },
    columnWidths: widths,
    rows: [
      headerRow(headers, widths),
      ...rows.map((r, i) => dataRow(r, widths, i % 2 === 1))
    ]
  });
}

function pageBreak() {
  return new Paragraph({ children: [new PageBreak()] });
}

const doc = new Document({
  numbering: {
    config: [
      {
        reference: "bullets",
        levels: [
          { level: 0, format: LevelFormat.BULLET, text: "•", alignment: AlignmentType.LEFT,
            style: { paragraph: { indent: { left: 720, hanging: 360 } } } },
          { level: 1, format: LevelFormat.BULLET, text: "◦", alignment: AlignmentType.LEFT,
            style: { paragraph: { indent: { left: 1080, hanging: 360 } } } },
        ]
      },
      {
        reference: "numbers",
        levels: [
          { level: 0, format: LevelFormat.DECIMAL, text: "%1.", alignment: AlignmentType.LEFT,
            style: { paragraph: { indent: { left: 720, hanging: 360 } } } },
        ]
      },
    ]
  },
  styles: {
    default: { document: { run: { font: "Arial", size: 20 } } },
    paragraphStyles: [
      { id: "Heading1", name: "Heading 1", basedOn: "Normal", next: "Normal", quickFormat: true,
        run: { size: 32, bold: true, font: "Arial", color: BRAND },
        paragraph: { spacing: { before: 400, after: 160 }, outlineLevel: 0 } },
      { id: "Heading2", name: "Heading 2", basedOn: "Normal", next: "Normal", quickFormat: true,
        run: { size: 26, bold: true, font: "Arial", color: ACCENT },
        paragraph: { spacing: { before: 280, after: 120 }, outlineLevel: 1 } },
      { id: "Heading3", name: "Heading 3", basedOn: "Normal", next: "Normal", quickFormat: true,
        run: { size: 22, bold: true, font: "Arial", color: DARK_TEXT },
        paragraph: { spacing: { before: 200, after: 80 }, outlineLevel: 2 } },
    ]
  },
  sections: [{
    properties: {
      page: {
        size: { width: 12240, height: 15840 },
        margin: { top: 1080, right: 1080, bottom: 1080, left: 1080 }
      }
    },
    footers: {
      default: new Footer({
        children: [new Paragraph({
          tabStops: [{ type: TabStopType.RIGHT, position: 10080 }],
          children: [
            new TextRun({ text: "New Loka — Architecture & Requirements Reference  |  CONFIDENTIAL", font: "Arial", size: 16, color: GRAY }),
            new TextRun({ text: "\tPage ", font: "Arial", size: 16, color: GRAY }),
            new TextRun({ children: [PageNumber.CURRENT], font: "Arial", size: 16, color: GRAY }),
          ]
        })]
      })
    },
    children: [

      // ── COVER ──────────────────────────────────────────────────────────────
      gap(800),
      new Paragraph({
        alignment: AlignmentType.CENTER,
        spacing: { before: 0, after: 40 },
        children: [new TextRun({ text: "NEW LOKA", font: "Arial", size: 72, bold: true, color: BRAND })]
      }),
      new Paragraph({
        alignment: AlignmentType.CENTER,
        spacing: { before: 0, after: 120 },
        children: [new TextRun({ text: "Open-Source Polymorphic Health Data Management System", font: "Arial", size: 28, color: ACCENT, italics: true })]
      }),
      new Paragraph({
        alignment: AlignmentType.CENTER,
        border: { bottom: { style: BorderStyle.SINGLE, size: 4, color: ACCENT, space: 4 } },
        spacing: { before: 0, after: 240 },
        children: [new TextRun("")]
      }),
      new Paragraph({
        alignment: AlignmentType.CENTER,
        spacing: { before: 80, after: 40 },
        children: [new TextRun({ text: "Architecture, Requirements & Coding Agent Reference", font: "Arial", size: 24, bold: true, color: DARK_TEXT })]
      }),
      new Paragraph({
        alignment: AlignmentType.CENTER,
        spacing: { before: 0, after: 40 },
        children: [new TextRun({ text: "Version 0.1 — May 2026", font: "Arial", size: 20, color: GRAY })]
      }),
      gap(600),
      new Paragraph({
        alignment: AlignmentType.CENTER,
        spacing: { before: 0, after: 40 },
        children: [new TextRun({ text: "Classification: Confidential / Working Document", font: "Arial", size: 18, color: ACCENT2, bold: true })]
      }),

      pageBreak(),

      // ── SECTION 1: VISION ─────────────────────────────────────────────────
      h1("1. Project Vision & Goals"),
      body("New Loka is an open-source, highly polymorphic health data management system designed to guide and facilitate patient care, clinical management, and research. It is engineered to serve deployments from a single clinician on a legacy smartphone with no connectivity, to a multi-department hospital with hundreds of concurrent users, without forking the codebase or compromising security at any tier."),
      gap(),
      h2("1.1 Core Design Principles"),
      bullet("Platform agnostic core: all logic, data, security, and sync in a single Rust library"),
      bullet("Local-first: every deployment is fully functional offline; network is an enhancement, never a dependency"),
      bullet("CRDT-native: all data modelled as conflict-free replicated data types; no master, no single point of failure"),
      bullet("FHIR R4 as the internal data model: interoperability is structural, not retrofitted"),
      bullet("Security by construction: encryption, identity, and access control are enforced at the data layer, not the UI layer"),
      bullet("Beat the phone call: any piece of clinical information must be accessible in ≤3 taps"),
      bullet("Zero external dependencies for core function: AI models, drug databases, and terminology sets are bundled offline"),

      gap(),
      h2("1.2 Deployment Tier Model"),
      body("All tiers run the same codebase. Tier capabilities are activated by environment detection, not separate builds."),
      gap(80),
      table(
        ["Tier", "Context", "Connectivity", "Storage", "Auth Model"],
        [
          ["T0", "Single clinician, phone-only", "Offline / occasional sync", "Local device only", "Single user, PIN/biometric"],
          ["T1", "Small clinic, 2–10 providers", "LAN only, no internet", "Local server or peer mesh", "RBAC, local admin"],
          ["T2", "Rural hospital", "Intermittent internet", "Local server + async cloud sync", "RBAC, department-aware"],
          ["T3", "Multi-department hospital", "Reliable internet", "Hybrid: local + cloud", "Full ABAC, silo enforcement"],
          ["T4", "Research / federated network", "Full internet", "Federated, privacy-preserving", "Institutional trust chains"],
        ],
        [900, 2000, 2000, 2200, 2260]
      ),

      pageBreak(),

      // ── SECTION 2: ARCHITECTURE ───────────────────────────────────────────
      h1("2. System Architecture"),
      h2("2.1 Layered Architecture Model"),
      body("New Loka is composed of three nested systems. Every deployment contains all three; higher tiers activate more of each layer's capabilities."),
      gap(80),
      numbered("The Core — a portable, encrypted health data kernel that runs anywhere"),
      numbered("The Mesh Layer — sync, conflict resolution, and trust protocols that activate when peers are present"),
      numbered("The Services Layer — local AI, compliance, team management, and research modules that scale with available resources"),

      gap(),
      h2("2.2 Component Architecture"),
      gap(80),
      table(
        ["Layer", "Component", "Technology", "License"],
        [
          ["UI — Mobile/Desktop", "Cross-platform UI", "Flutter", "BSD"],
          ["UI — Web Admin", "Hospital admin console", "React + Rust/WASM", "MIT"],
          ["Bridge", "FFI bindings", "uniffi-rs + flutter_rust_bridge", "MPL / MIT"],
          ["Core — Data", "CRDT database engine", "CR-SQLite", "MIT"],
          ["Core — Sync", "CRDT sync protocol", "Automerge-rs", "MIT"],
          ["Core — Crypto", "Cryptographic primitives", "libsodium / dryoc (Rust)", "MIT"],
          ["Core — ABAC", "Policy engine", "AWS Cedar (Rust SDK)", "Apache 2.0"],
          ["Core — AI Runtime", "Local model inference", "llama.cpp / ONNX Runtime", "MIT"],
          ["Services — OCR", "Handwriting recognition", "TrOCR (ONNX quantized)", "MIT"],
          ["Services — STT", "Speech-to-text", "whisper.cpp", "MIT"],
          ["Services — LLM", "Clinical NLP / extraction", "Phi-3-mini or Llama 3.2 3B", "MIT / Llama"],
          ["Services — FHIR", "Institutional FHIR server (T3+)", "HAPI FHIR", "Apache 2.0"],
          ["Infra — Container", "T1/T2 deployment", "Docker Compose", "Apache 2.0"],
          ["Infra — Orchestration", "T3 hospital deployment", "Kubernetes", "Apache 2.0"],
        ],
        [2000, 2000, 2500, 1860]
      ),

      gap(),
      h2("2.3 The Rust Core Library"),
      note("CRITICAL: The Rust core library IS the system. All other components are shells around it. It must be written once, tested exhaustively, and trusted completely."),
      gap(80),
      body("The core exposes a clean FFI surface via uniffi-rs, generating idiomatic bindings for:"),
      bullet("Kotlin — Android native"),
      bullet("Swift — iOS native"),
      bullet("Dart — Flutter (via flutter_rust_bridge)"),
      bullet("WASM — Browser / web admin console"),
      bullet("Python — Tooling, testing, data migration scripts"),
      gap(80),
      body("Modules within the Rust core:"),
      bullet("fhir_model — FHIR R4 resource definitions, serialisation, validation"),
      bullet("crdt_store — CR-SQLite integration, vector clocks, merge strategies"),
      bullet("crypto — Key hierarchy, encryption/decryption, signing"),
      bullet("identity — Node identity, user auth, session management"),
      bullet("abac — Cedar policy engine integration, policy evaluation"),
      bullet("sync — Peer discovery, session establishment, delta sync"),
      bullet("audit — Append-only audit log, Ed25519 signing, reconciliation engine"),
      bullet("ai_pipeline — Model runner coordination, review queue management"),
      bullet("rules — Clinical heuristics engine, drug interaction checker"),

      pageBreak(),

      // ── SECTION 3: DATA LAYER ─────────────────────────────────────────────
      h1("3. Data Layer Requirements"),
      h2("3.1 CRDT Storage Engine"),
      bullet("Embed CR-SQLite as the local database engine on all platforms"),
      bullet("All patient records modelled as CRDT resources — offline edits always merge deterministically"),
      bullet("All data encrypted at rest: AES-256-GCM with per-patient, per-record keys"),
      bullet("Key management: device hardware keystore at T0; HashiCorp Vault (dev mode) at T1+"),
      bullet("FHIR R4 as the canonical internal data model — every record is a FHIR resource"),
      bullet("Immutable, cryptographically chained audit log for every access, edit, and transmission event"),

      gap(),
      h2("3.2 FHIR Resource Schema"),
      body("Every record in New Loka is a valid FHIR R4 resource. Primary resources used:"),
      gap(80),
      table(
        ["FHIR Resource", "Clinical Use", "Key Fields"],
        [
          ["Patient", "Patient identity and demographics", "id (deterministic UUID), name, DOB, gender, meta"],
          ["Encounter", "Admission, visit, OT booking", "class, type, period, serviceProvider, participant"],
          ["Condition", "Diagnosis, provisional or confirmed", "code (SNOMED), clinicalStatus, verificationStatus, evidence"],
          ["Observation", "Vitals, labs, measurements", "code (LOINC), valueQuantity, interpretation, referenceRange"],
          ["MedicationRequest", "Prescriptions, planned medications", "medication, dosage, requester, status"],
          ["MedicationAdministration", "Drugs actually given", "medication, dosage, performer, effective, override flags"],
          ["ServiceRequest", "Lab orders, consults, procedures", "priority, code, requester, performer, intent"],
          ["DiagnosticReport", "Lab results, imaging reports", "status, result (Observation refs), conclusion"],
          ["Procedure", "Surgeries, interventions", "code (SNOMED), performer, outcome, complication"],
          ["AllergyIntolerance", "Drug and other allergies", "code, criticality, reaction, verificationStatus"],
          ["Composition", "Discharge summaries, referral letters", "type, section, author, date, status"],
          ["DetectedIssue", "Automated flags, safety alerts", "severity, code, detail, implicated resources"],
          ["RiskAssessment", "ASA class, pre-op risk", "prediction, basis, performer"],
          ["AuditEvent", "Security and access audit trail", "type, agent, source, entity, outcome"],
        ],
        [2000, 2500, 3860]
      ),

      gap(),
      h2("3.3 Required Metadata on Every Resource"),
      body("Every FHIR resource stored in New Loka carries these mandatory meta fields:"),
      bullet("created_by — Practitioner reference"),
      bullet("created_at — ISO 8601 timestamp"),
      bullet("modified_by — Practitioner reference"),
      bullet("modified_at — ISO 8601 timestamp"),
      bullet("origin_node_id — Ed25519 public key fingerprint of originating node"),
      bullet("sync_vector_clock — CRDT vector clock for merge ordering"),
      bullet("department_id — owning department for ABAC silo enforcement"),
      bullet("heuristic_warnings — array of any clinical rule flags that fired on creation"),
      bullet("override_flag + override_reason — if clinical rules were overridden"),

      gap(),
      h2("3.4 Patient Identity"),
      bullet("Deterministic UUID: BLAKE3(name_normalised + DOB_ISO + sex)"),
      bullet("Collision detection: on new patient creation, query local store for UUID match before creating"),
      bullet("Cross-node deduplication: fuzzy match on demographics, flag for admin review with merge UI"),
      bullet("Soft deletes only — records are never physically deleted; status set to 'entered-in-error'"),
      bullet("Legal hold flag: required before any status change; data retained per jurisdiction rules"),

      pageBreak(),

      // ── SECTION 4: SECURITY ───────────────────────────────────────────────
      h1("4. Security Architecture"),
      h2("4.1 Three-Layer Security Model"),
      body("Security is enforced at three independent layers. Each is a complete barrier independently — defence in depth means a failure at any one layer does not compromise patient data."),
      gap(80),
      table(
        ["Layer", "What It Answers", "Technology", "Location"],
        [
          ["Layer 1 — Cryptographic", "Is the data physically protected?", "libsodium (dryoc Rust crate)", "Rust core — always active"],
          ["Layer 2 — Identity & Auth", "Are you who you claim to be?", "Argon2id + TOTP + Ed25519 node certs", "Rust core — per session"],
          ["Layer 3 — ABAC Policy", "Are you permitted to do this?", "AWS Cedar policy engine (Rust)", "Rust core — per request"],
        ],
        [2000, 2500, 2500, 2360]
      ),

      gap(),
      h2("4.2 Cryptographic Key Hierarchy"),
      body("All keys are derived and stored in this hierarchy. No key is ever stored in plaintext."),
      gap(80),
      mono("Device Master Key (DMK)"),
      mono("  └─ derived: hardware keystore + user passphrase (Argon2id)"),
      mono("  └─ never leaves device"),
      mono(""),
      mono("  ├─ Patient Record Key (PRK) — one per patient"),
      mono("  │   encrypted with DMK, stored alongside record"),
      mono("  │   decrypted only within an authenticated session"),
      mono(""),
      mono("  ├─ Sync Session Key (SSK) — ephemeral, per sync session"),
      mono("  │   X25519 key exchange between node identities"),
      mono("  │   forward secrecy: past sessions cannot be decrypted"),
      mono(""),
      mono("  └─ Audit Log Key (ALK) — append-only signing key"),
      mono("      Ed25519 signing of each audit entry"),
      mono("      cannot decrypt anything"),
      gap(80),
      body("Platform key storage:"),
      bullet("Android: Android Keystore System (hardware-backed)"),
      bullet("iOS: Secure Enclave (hardware-backed)"),
      bullet("Linux / Raspberry Pi: TPM 2.0 if available; key file + OS permissions + passphrase as fallback"),

      gap(),
      h2("4.3 Node Identity & Trust"),
      bullet("Each New Loka installation generates an Ed25519 keypair at first run — this is the node's permanent identity"),
      bullet("Node identity stored in hardware keystore; used to sign all data originating from this node"),
      bullet("Trust ceremony required when joining a clinic/hospital mesh: admin node signs a trust certificate for the new node"),
      bullet("At T3: integrates with institution's existing certificate authority"),
      bullet("Certificate chain: Root CA (institution) → Admin node cert → Peer node cert"),

      gap(),
      h2("4.4 ABAC Policy Model"),
      body("Policies are evaluated against: Subject × Resource × Action × Context → Permit | Deny"),
      gap(80),
      table(
        ["Dimension", "Attributes"],
        [
          ["Subject", "user_id, roles[], department_id, node_id, is_on_call"],
          ["Resource", "patient_id, record_type, owning_department, sensitivity_level"],
          ["Action", "read | write | delete | export | share | print | transfer"],
          ["Context", "time_of_day, is_emergency_override, patient_assigned_to_subject, sync_state"],
        ],
        [2000, 7360]
      ),
      gap(80),
      body("Key policies (implement these as Cedar policy files):"),
      bullet("Clinician READ patient record WHERE patient assigned to their team AND same department"),
      bullet("ANY role READ patient record WHERE emergency override = true AND override logged with reason"),
      bullet("DENY any READ WHERE department mismatch AND role is not {Admin, Super-Admin} AND no emergency override"),
      bullet("Nurse WRITE vitals WHERE patient assigned to their ward; DENY Nurse DELETE any record"),
      bullet("Department head READ cross-team records within their department; cannot WRITE to other teams' records"),
      bullet("Audit log: readable by Admin for their department only; Super-Admin for all"),

      gap(),
      h2("4.5 Emergency Override ('Break Glass')"),
      bullet("A clinician must always be able to access a record in a life-threatening emergency"),
      bullet("Explicit UI action required: 'Access Emergency Record' + mandatory reason text entry"),
      bullet("Immediately logged to immutable audit trail with actor, reason, and timestamp"),
      bullet("Generates an alert to the owning department's admin"),
      bullet("Time-limited access: 2 hours default (configurable per institution)"),
      bullet("Retrospective review queue generated for department head"),

      gap(),
      h2("4.6 Audit Log Requirements"),
      bullet("Append-only, cryptographically chained log — each entry signed with ALK (Ed25519)"),
      bullet("Tamper-evident: any modification to a historical entry invalidates all subsequent signatures"),
      bullet("Every event logged: PATIENT_CREATED, RECORD_ACCESSED, RECORD_MODIFIED, LAB_ORDER_SENT, MEDICATION_ADMINISTERED, SYNC_SESSION_OPENED, EMERGENCY_OVERRIDE, MEDICATION_CORRECTION, DISCHARGE_SIGNED"),
      bullet("Log is retained permanently; never purged (soft archival after jurisdiction-defined period)"),
      bullet("Exportable as FHIR AuditEvent bundle for regulatory inspection"),
      bullet("Background reconciliation pass every 15 minutes in OT, every 60 minutes on wards"),

      pageBreak(),

      // ── SECTION 5: SYNC ───────────────────────────────────────────────────
      h1("5. Sync & Mesh Layer"),
      h2("5.1 Peer Discovery"),
      bullet("mDNS / Bonjour — LAN peer discovery, zero-config, works on any LAN"),
      bullet("Bluetooth Low Energy (BLE) — sub-LAN scenarios, two devices with no router"),
      bullet("WiFi Direct — peer-to-peer data transfer fallback"),
      bullet("Each node broadcasts: node_id (Ed25519 pubkey fingerprint) + capability flags + sync vector clock"),

      gap(),
      h2("5.2 Sync Protocol"),
      bullet("Build on Automerge-rs — mature, Rust-native CRDT sync library"),
      bullet("Sync is pull-based with push notifications: nodes advertise vector clock; peers pull only the delta"),
      bullet("All sync sessions encrypted with ephemeral SSK (X25519 key exchange per session)"),
      bullet("Bandwidth-aware: detect available bandwidth, prioritise active records over historical"),
      bullet("Sync state machine per resource: local_only → pending_sync → synced → conflict → resolved"),

      gap(),
      h2("5.3 Conflict Resolution Rules"),
      body("Domain-specific conflict resolution — never a blanket 'last write wins':"),
      gap(80),
      table(
        ["Resource Type", "Conflict Strategy", "Rationale"],
        [
          ["Medication records", "Flag for human review — never auto-resolve", "Patient safety — dosing errors are life-threatening"],
          ["Vital signs (timeseries)", "Merge by timestamp; deduplicate by content hash", "Additive — each reading is an independent event"],
          ["Administrative fields", "Last-write-wins acceptable", "Low clinical risk"],
          ["Diagnoses / Conditions", "Merge as array; flag semantic duplicates for review", "Clinician must resolve contradictory diagnoses"],
          ["Patient demographics", "Flag mismatch for admin review with merge UI", "Identity accuracy is foundational"],
          ["Audit log entries", "Append only — no conflict possible by design", "Immutability is the requirement"],
        ],
        [2000, 3000, 4360]
      ),

      gap(),
      h2("5.4 Data Transit Security"),
      body("When a FHIR Bundle is transmitted between nodes:"),
      numbered("Sending node: ABAC check — am I permitted to share this record to this node?"),
      numbered("Sending node: establish SSK via X25519 with receiving node's identity"),
      numbered("Sending node: sign bundle manifest (Ed25519); encrypt with SSK"),
      numbered("Receiving node: verify HMAC — payload unmodified in transit"),
      numbered("Receiving node: verify bundle signature — data originated from claimed sender"),
      numbered("Receiving node: ABAC check — am I permitted to receive and store this?"),
      numbered("Receiving node: decrypt with SSK; re-encrypt with local PRK; store; log receipt"),
      gap(80),
      note("Patient data is never in plaintext in transit. A compromised sending node cannot force unauthorised records onto a receiving node — the receiving node's ABAC independently validates every transfer."),

      pageBreak(),

      // ── SECTION 6: AI PIPELINE ────────────────────────────────────────────
      h1("6. AI & Local Intelligence Pipeline"),
      h2("6.1 Fundamental Constraint"),
      note("HARD RULE: No patient data is ever sent to an external API without explicit patient consent AND institutional approval. All AI models run on-device or on local institutional infrastructure. This is an architectural constraint, not a preference."),

      gap(),
      h2("6.2 Input Processing Models"),
      gap(80),
      table(
        ["Input Type", "Model", "Runtime", "Tier Available"],
        [
          ["Handwritten records / photos", "TrOCR (ONNX quantized)", "ONNX Runtime", "T0+"],
          ["Printed / digital PDFs", "pdfplumber / pypdf (text layer)", "Local Python service", "T0+"],
          ["Scanned PDFs", "Rasterize → TrOCR pipeline", "ONNX Runtime", "T0+"],
          ["Voice dictation (live)", "whisper.cpp (tiny/base model)", "Native, streaming", "T0+"],
          ["Voice dictation (T2+ quality)", "whisper.cpp (medium model)", "Native", "T2+"],
          ["Clinical NLP / extraction", "Phi-3-mini or Llama 3.2 3B (GGUF)", "llama.cpp", "T0+"],
          ["Drug interaction checking", "OpenFDA dataset (local copy)", "Local SQLite query", "T0+"],
          ["Lab report parsing", "Regex + lightweight ML hybrid", "Rust rules engine", "T0+"],
          ["Speaker diarization", "pyannote.audio", "Local Python service", "T2+"],
        ],
        [2500, 2000, 1800, 1360 + 360]
      ),

      gap(),
      h2("6.3 AI Output Pipeline — Human Review Required"),
      body("All AI-extracted clinical data goes through a mandatory review queue before being committed to the patient record:"),
      numbered("AI model produces structured FHIR resource candidates with confidence scores"),
      numbered("Resources with confidence ≥ 0.90: surfaced in review queue with 'Accept' pre-highlighted"),
      numbered("Resources with confidence 0.70–0.89: surfaced with caution indicator"),
      numbered("Resources with confidence < 0.70: surfaced with explicit low-confidence warning"),
      numbered("Clinician reviews, edits, and confirms each item"),
      numbered("Only confirmed items are encrypted and committed to CR-SQLite"),
      numbered("AI model confidence scores are permanently embedded in resource meta — never stripped"),
      gap(80),
      note("AI suggestions are never auto-committed to patient records. The human is always the final authority."),

      gap(),
      h2("6.4 Clinical Heuristics Engine"),
      body("A Rust-native rules engine that runs synchronously on every medication and order entry, before any UI confirmation is accepted:"),
      bullet("Dose range validation: drug × indication × route × patient weight → expected dose range"),
      bullet("Magnitude check: if entered dose differs from typical by >10×, trigger UNIT_ERROR_SUSPECTED flag"),
      bullet("Drug-drug interaction check: against local OpenFDA interaction dataset"),
      bullet("Allergy cross-reference: check entered drug against patient's AllergyIntolerance resources"),
      bullet("Route validity: e.g. flag oral medication ordered for an NPO (nil per os) patient"),
      bullet("Duplicate order detection: same drug already active in MedicationRequest list"),
      gap(80),
      body("Heuristic alert levels:"),
      bullet("INFO — informational, dismissable without reason"),
      bullet("WARN — requires acknowledgement, dismissable with one tap"),
      bullet("HIGH — modal alert, cannot dismiss by tapping outside, requires active choice"),
      bullet("CRITICAL — requires reason text entry AND PIN re-authentication to override"),
      gap(80),
      note("Heuristic warnings are permanently embedded in the resource record and cannot be stripped. If overridden, the override reason and authentication event are also stored. The audit reconciliation engine checks for pattern of overrides."),

      pageBreak(),

      // ── SECTION 7: BUILD PLAN ─────────────────────────────────────────────
      h1("7. Phased Build Plan"),
      h2("Phase 1 — The Core (T0 working end-to-end)"),
      body("Target: a single clinician can manage patients fully offline on their phone."),
      bullet("FHIR R4 data model in Rust (Patient, Encounter, Condition, Observation, MedicationRequest)"),
      bullet("CR-SQLite integration with AES-256-GCM encryption"),
      bullet("Device hardware keystore integration (Android Keystore + iOS Secure Enclave)"),
      bullet("Argon2id user authentication, offline TOTP"),
      bullet("Basic ABAC engine (single-user policies)"),
      bullet("Audit log (append-only, Ed25519 signed)"),
      bullet("Flutter UI: patient list, chart view, quick vitals entry, medication entry"),
      bullet("Heuristics engine: dose range validation, drug interaction checker (OpenFDA local)"),
      bullet("whisper.cpp integration: live dictation → transcript → review queue"),
      bullet("Phi-3-mini integration: clinical NLP extraction → FHIR resource candidates"),
      bullet("TrOCR integration: photo of handwritten record → text → extraction pipeline"),
      bullet("uniffi-rs bindings: Kotlin + Swift + Dart"),

      gap(),
      h2("Phase 2 — The Mesh (T1 working)"),
      body("Target: a small clinic operates as a peer mesh with no internet dependency."),
      bullet("Ed25519 node identity generation and storage"),
      bullet("mDNS peer discovery"),
      bullet("Automerge-rs sync protocol"),
      bullet("X25519 ephemeral session key exchange"),
      bullet("Trust ceremony: admin node signs peer trust certificates"),
      bullet("Conflict resolution rules engine"),
      bullet("Multi-user ABAC policies (roles, department assignment)"),
      bullet("Raspberry Pi Docker Compose deployment package"),
      bullet("Admin console (Flutter desktop or React/WASM): user management, sync status, audit log viewer"),
      bullet("BLE fallback discovery"),

      gap(),
      h2("Phase 3 — The Institution (T3 working)"),
      body("Target: a multi-department hospital with full ABAC silo enforcement."),
      bullet("Department silo enforcement at data layer (not just UI)"),
      bullet("Cedar ABAC policy files for all department configurations"),
      bullet("Emergency override (break glass) with alerting and retrospective review"),
      bullet("HAPI FHIR server deployment (Kubernetes)"),
      bullet("API Gateway (Kong): department-level policy enforcement before data layer"),
      bullet("mTLS between all hospital nodes"),
      bullet("LDAP / Active Directory integration for user provisioning"),
      bullet("Audit log export as FHIR AuditEvent bundle"),
      bullet("HL7 v2 listener for legacy HIS integration"),
      bullet("Patient transfer between departments with ABAC-governed record handoff"),
      bullet("React web admin console (WASM Rust core)"),

      gap(),
      h2("Phase 4 — Intelligence"),
      body("Target: institution-grade AI assistance running entirely on local infrastructure."),
      bullet("Local LLM server: Ollama or vLLM on institutional hardware"),
      bullet("Whisper medium model for ward round recording with speaker diarization"),
      bullet("Lab report parser: structured value extraction → FHIR Observation batch"),
      bullet("Discharge summary auto-generation from Encounter timeline"),
      bullet("Handoff note generation (one-tap)"),
      bullet("Audit reconciliation engine: background detection of dose errors, override patterns"),
      bullet("Model registry and signed model package distribution"),

      gap(),
      h2("Phase 5 — Federation & Research"),
      body("Target: multi-institution federated network with privacy-preserving research capability."),
      bullet("Federated learning infrastructure: local training, gradient sharing only"),
      bullet("Differential privacy on all aggregate queries leaving institution"),
      bullet("SMART on FHIR authorisation for third-party research apps"),
      bullet("Patient consent management: granular per data type per research purpose with revocation"),
      bullet("ABDM / ABHA integration (India NHA FHIR profiles)"),
      bullet("IHE profile conformance: PDQm, MHD, PIXm"),
      bullet("NHA integration testing and certification process"),

      pageBreak(),

      // ── SECTION 8: COMPLIANCE ─────────────────────────────────────────────
      h1("8. Regulatory & Compliance Requirements"),
      h2("8.1 FHIR Certification Path"),
      gap(80),
      table(
        ["Phase", "Action", "Mandatory?", "Target Market"],
        [
          ["Phase 1–2", "Build to FHIR R4 spec internally; self-declare conformance", "No", "All"],
          ["Phase 1–2", "Generate and publish FHIR Capability Statement at [base]/metadata", "Yes — for any FHIR server", "T3+"],
          ["Phase 2–3", "Run implementation through HL7 FHIR Validator (free tool)", "Best practice", "All"],
          ["Phase 3", "Participate in IHE Connectathon (annual, open registration)", "Voluntary but expected", "T3 hospital sales"],
          ["Phase 4–5", "Register with NHA sandbox; implement ABDM FHIR profiles", "Yes — for ABHA integration", "India"],
          ["Phase 5", "Complete NHA integration testing process", "Yes — for ABHA integration", "India"],
        ],
        [1200, 3500, 1600, 2060]
      ),

      gap(),
      h2("8.2 Standards Alignment"),
      bullet("HL7 FHIR R4 — primary data standard"),
      bullet("SNOMED CT — clinical terminology for diagnoses and procedures"),
      bullet("LOINC — laboratory and observation codes"),
      bullet("ICD-10 / ICD-11 — billing and statistical classification codes"),
      bullet("DISHA / IT Act (India) — primary legal framework for health data in India"),
      bullet("HIPAA (US) — baseline for any deployment with global research partners"),
      bullet("GDPR (EU) — for T4 federated research involving European participants"),
      bullet("IHE PDQm, MHD, PIXm — integration profiles for T3+ interoperability"),

      gap(),
      h2("8.3 Data Retention"),
      bullet("India (DISHA): minimum 7 years retention for patient records"),
      bullet("All deletions are soft: status = 'entered-in-error'; physical data retained"),
      bullet("Audit log: permanent retention, no purge"),
      bullet("Right to access: implement FHIR Bundle export (all resources for a patient)"),
      bullet("Legal hold flag: blocks any status change pending legal proceedings"),

    ]
  }]
});

Packer.toBuffer(doc).then(buf => {
  fs.writeFileSync('/mnt/user-data/outputs/NewLoka_Requirements_Reference.docx', buf);
  console.log('Done: Requirements doc written.');
});