# New Loka Knowledge Graph

Bi-temporal, file-based, multi-layered knowledge graph for the New Loka healthcare data engine.
Models the system ontology, crate architecture, clinical models, cryptographic guarantees, operational tooling, public web presence, project roadmap, and sovereign agentics.

Every fact carries `[valid_from, recorded, source]`. Contradicted facts move to `## Superseded` in their node file — never deleted.

---

## Nodes

| File | Entity | Layer | Status | Summary | Last updated |
|---|---|---|---|---|---|
| philosophy-and-principles.md | System Philosophy | Layer 0: Ontological | SHIPPED | Platform-agnostic core, local-first, CRDT sync, single core, adaptive engineering, style guide & WATER pattern management | 2026-11-16 |
| deployment-tiers.md | Deployment Tiers | Layer 0: Ontological | SHIPPED | T0 (solo clinic) to T4 (federation), 22-role declarative presets, differential privacy mesh | 2026-11-14 |
| fhir-clinical-models.md | FHIR R4 Clinical Models | Layer 1: Structural | SHIPPED | 23+ FHIR R4 resources including Consent, polymorphic FhirResource wrapper, soft delete, provenance | 2026-11-14 |
| storage-crdt-crypto.md | Storage, CRDT & Crypto | Layer 1: Structural | SHIPPED | SQLite engine via SQLx, AES-256-GCM AEAD at rest, DMK/PRK key hierarchy, vector clocks, delta sync | 2026-11-13 |
| abac-and-identity.md | ABAC & Identity | Layer 1: Structural | SHIPPED | Core-enforced ABAC, Role::Billing clinical redaction, legal prescriber check, emergency break-glass | 2026-11-14 |
| audit-and-safety.md | Audit Engine & Safety | Layer 1: Structural | SHIPPED | Tamper-evident append-only ledger, Ed25519 cryptographic signing, human review mandatory, HIPAA/DISHA/GDPR | 2026-11-13 |
| pharmacopoeia-and-order-sets.md | Indian Pharmacopoeia & Order Sets | Layer 1: Structural | SHIPPED | Indian Pharmacopoeia (IP 2022) / NFI / NLEM embedded index, brand auto-resolution, 10 evidence order sets, allergy/NSAID shield | 2026-11-15 |
| cpoe-and-diagnostics.md | CPOE & Diagnostics | Layer 2: Operational | SHIPPED | 5-stage order lifecycle, Routine/Urgent/Stat priorities, Radiology/Biochem/Path/Micro silos, panic labs | 2026-11-13 |
| erx-pad-pro.md | eRx Pad Pro & Formulary | Layer 2: Operational | SHIPPED | Native A4 prescription pad, 61k+ drug formulary, digital signature setup, offline sync | 2026-11-16 |
| server-and-api.md | HTTP Server & API | Layer 2: Operational | SHIPPED | Tokio/Axum 0.7 engine, rust-embed SPA, FHIR REST + Consent, departments/transfers, federation/research APIs | 2026-11-14 |
| web-clinical-spa.md | Web Clinical Interface | Layer 2: Operational | SHIPPED | Zero-dependency ES6 vanilla SPA, 35 clinical views (T3/T4 modules), Epic Storyboard rail, PRESET_REGISTRY, full FHIR bedside bridges | 2026-11-15 |
| cli-tooling.md | Command-Line Tooling | Layer 2: Operational | SHIPPED | newloka-cli with Clap v4, 12 command groups (departments, consent, research, federation), direct SQLite | 2026-11-14 |
| self-updater-and-releases.md | Self-Updater & Releases | Layer 2: Operational | SHIPPED | In-app 1-click update, GitHub Releases polling, atomic rename-and-replace, air-gapped offline package upload | 2026-11-13 |
| newloka-site.md | Public Web Presence | Layer 4: External | ACTIVE | newloka.org on Firebase Hosting (new-loka), Sage/Taupe brand system, live EMR v0.2.0 UI showcase, docs hub | 2026-11-13 |
| roadmap-and-status.md | Roadmap & Status | Layer 3: Roadmap | ACTIVE | Current v0.2.0, 51 passing tests, 11-phase roadmap: Phases 1-3, 6, 9 Shipped, living requirements gap doc | 2026-11-14 |
| amethyst-agent.md | Amethyst Sovereign Agent | Layer 5: Sovereign Agentics | ACTIVE | Senior architect & systems engineer, Clinical Invariant Doctrine, WATER pattern management, memory/skill self-evolution loop | 2026-11-16 |

---

## Typed Edges

- amethyst-agent.md -[governs]-> philosophy-and-principles.md : Amethyst enforces the core clinical and local-first invariants
- amethyst-agent.md -[orchestrates]-> server-and-api.md : Amethyst plans, iterates, and compiles the Axum server architecture
- amethyst-agent.md -[orchestrates]-> pharmacopoeia-and-order-sets.md : Amethyst curates the IP 2022 monographs, order sets, and clinical safety shield
- amethyst-agent.md -[orchestrates]-> fhir-clinical-models.md : Amethyst maintains and validates FHIR R4 clinical data models
- amethyst-agent.md -[orchestrates]-> storage-crdt-crypto.md : Amethyst oversees the AES-256-GCM encryption and CRDT sync engine
- amethyst-agent.md -[orchestrates]-> web-clinical-spa.md : Amethyst inspects and updates the 30 clinical views in newloka_web
- amethyst-agent.md -[orchestrates]-> cli-tooling.md : Amethyst manages the headless newloka-cli administrative commands
- amethyst-agent.md -[tracks_in]-> roadmap-and-status.md : Amethyst aligns ongoing work with the 11-phase project roadmap
- amethyst-agent.md -[syncs_with]-> newloka-site.md : Amethyst synchronizes release notes and documentation with the marketing site
- server-and-api.md -[serves]-> web-clinical-spa.md : Axum embeds and serves the SPA frontend via rust-embed
- server-and-api.md -[implements]-> fhir-clinical-models.md : Server exposes standard FHIR R4 CRUD endpoints
- server-and-api.md -[persists_in]-> storage-crdt-crypto.md : Server executes encrypted SQLite read and write transactions
- server-and-api.md -[enforces]-> abac-and-identity.md : Server validates session tokens and evaluates ABAC policies
- server-and-api.md -[integrates]-> erx-pad-pro.md : Server routes /api/rxpad endpoints and broadcasts SSE live events
- server-and-api.md -[extends]-> self-updater-and-releases.md : Server powers in-app 1-click updates and atomic binary replacement
- cli-tooling.md -[persists_in]-> storage-crdt-crypto.md : newloka-cli performs direct encrypted SQLite operations
- cli-tooling.md -[serves]-> server-and-api.md : newloka-cli serve subcommand boots the HTTP engine
- cli-tooling.md -[implements]-> fhir-clinical-models.md : newloka-cli constructs and validates FHIR resources
- storage-crdt-crypto.md -[enforces]-> audit-and-safety.md : Storage triggers Ed25519-signed append-only audit entries
- abac-and-identity.md -[enforces]-> cpoe-and-diagnostics.md : ABAC isolates departmental lab queues and radiology scopes
- abac-and-identity.md -[secures]-> audit-and-safety.md : Emergency overrides log high-severity signed audit records
- cpoe-and-diagnostics.md -[implements]-> fhir-clinical-models.md : CPOE stores orders as ServiceRequest, Specimen, and DiagnosticReport
- erx-pad-pro.md -[syncs_with]-> server-and-api.md : eRx Pad Pro pushes prescriptions via offline queue and receives SSE
- erx-pad-pro.md -[implements]-> fhir-clinical-models.md : Prescriptions translate into FHIR MedicationRequest and Dispense
- erx-pad-pro.md -[integrates]-> pharmacopoeia-and-order-sets.md : Prescribing pad queries pharmacopoeia database for brands, routes, strengths, and regulatory schedules
- pharmacopoeia-and-order-sets.md -[implements]-> fhir-clinical-models.md : Pharmacopoeia entries and order sets map to FHIR MedicationRequest and PlanDefinition
- server-and-api.md -[serves]-> pharmacopoeia-and-order-sets.md : Axum routes /api/pharmacopoeia/* queries, safety checks, and monograph retrieval
- web-clinical-spa.md -[implements]-> deployment-tiers.md : Settings view activates T0 Solo, Specialist, or Hospital presets
- web-clinical-spa.md -[integrates]-> erx-pad-pro.md : Native and embedded prescription pad views inside clinical navigation
- web-clinical-spa.md -[integrates]-> pharmacopoeia-and-order-sets.md : Clinical CPOE orders and prescription pad leverage pharmacopoeia autocomplete and safety shield
- self-updater-and-releases.md -[deploys_to]-> deployment-tiers.md : Packages cross-platform archives for solo clinics to hospital servers
- newloka-site.md -[tracks_in]-> roadmap-and-status.md : team.html publishes the official 11-phase project roadmap
