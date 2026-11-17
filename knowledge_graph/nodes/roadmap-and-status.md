# Project Roadmap, Milestones, and Current Status

## Epistemic Metadata
- **Entity**: Project Roadmap & Engineering Telemetry
- **Layer**: Layer 3: Roadmap & Evolution
- **Status**: ACTIVE / MULTI-PHASE
- **Domain**: Delivery Lifecycle, Phase Tracking & Milestones

## Facts
- Current Version Milestone: New Loka is at v0.2.0, compiling cleanly under Rust 2021 Edition with 72 passing unit, integration, and server tests across 7 decoupled workspace crates (core, clinical, pharmacopoeia, storage, demo, server, cli). [valid_from: 2026-11-17, recorded: 2026-11-17, source: Cargo.toml & workspace cargo test run]
- Roadmap Phase 1 (Shipped): Core Offline Clinician Workflow providing encrypted local storage, FHIR R4 resource model, patient CRUD, encounter timelines, offline auth, and signed audit logs. [valid_from: 2026-06-11, recorded: 2026-11-13, source: D:\GitRepos\site\team.html]
- Roadmap Phase 2 (Shipped): Local Sync and Mesh providing LAN peer discovery, delta sync with vector clocks, and automatic conflict detection for clinics operating as a connected mesh. [valid_from: 2026-06-15, recorded: 2026-11-13, source: D:\GitRepos\site\team.html]
- Roadmap Phase 3 (Shipped): Institutional Access Control delivering ABAC policy engine, department silos for Radiology, Biochemistry, Pathology, and Microbiology, emergency override workflows, and LDAP hooks. [valid_from: 2026-06-20, recorded: 2026-11-13, source: D:\GitRepos\site\team.html]
- Roadmap Phase 4 (Shipped): Local Intelligence & Document Ingestion delivering deterministic clinical NLP/NER extraction, ICD-10 condition resolution, LOINC panic value alerts, allergy contraindication cross-checks, /api/ingest REST routes, and human-in-the-loop review workspace in newloka_web. [valid_from: 2026-11-17, recorded: 2026-11-17, source: newloka_clinical/src/ingest.rs & newloka_server/src/routes/ingest.rs]
- Roadmap Phase 5 (Upcoming): Clinical Decision Support Calculators, medical reference knowledge banks, and periodically updated pharmaceutical interaction/contraindication databases. [valid_from: 2026-06-20, recorded: 2026-11-13, source: D:\GitRepos\site\team.html]
- Roadmap Phase 6 (Shipped): Releases for Distributed Deployments & Multi-Department Hospitals delivering 9-department hierarchy, SBAR patient transfer handovers, executive hospital occupancy telemetry, and enterprise LDAP/AD directory authentication. [valid_from: 2026-11-14, recorded: 2026-11-14, source: newloka_core/src/department & newloka_server/src/lib.rs]
- Roadmap Phase 7 (Upcoming): Internal Secure Communications Stack featuring end-to-end encrypted messaging, VoIP, and video conferencing between clinicians and departments with zero third-party cloud relays. [valid_from: 2026-06-20, recorded: 2026-11-13, source: D:\GitRepos\site\team.html]
- Roadmap Phase 8 (Upcoming): Imaging and Lab Integration via DICOM and HL7/FHIR bridges for radiology, pathology, and lab devices enabling automated result ingestion. [valid_from: 2026-06-20, recorded: 2026-11-13, source: D:\GitRepos\site\team.html]
- Roadmap Phase 9 (Shipped): Research Federation and Health Data Mesh supporting privacy-preserving multi-center query exchange, Laplace differential privacy, k-anonymity suppression (k >= 5), de-identified export, and dynamic FHIR R4 Consent directives. [valid_from: 2026-11-14, recorded: 2026-11-14, source: newloka_core/src/federation & newloka_core/src/consent]
- Roadmap Phase 10 (Upcoming): Complete Hospital Information System (HIS) with billing, scheduling, ward bed census, and dispensary inventory. [valid_from: 2026-06-20, recorded: 2026-11-13, source: D:\GitRepos\site\team.html]
- Roadmap Phase 11 (Upcoming): Experimental Feature Sets including ambient consultation room listening assistants, predictive triage signals, and synthetic EHR data generators. [valid_from: 2026-06-20, recorded: 2026-11-13, source: D:\GitRepos\site\team.html]
- Active Engineering Telemetry: Ongoing enhancements in working tree include enhanced CLI commands with SQLite storage integration, refined asset matching in self-updater, and official emblem branding parity. [valid_from: 2026-10-15, recorded: 2026-11-13, source: git status & git diff]
- Vision & Requirements Gap Analysis: Maintained living tracking document (`docs/MISSING_COMPONENTS_AND_REQUIREMENTS.md`) benchmarking New Loka across 11 roadmap phases against initial scope and reference EHRs, organizing remaining missing components into three pragmatic horizons (Immediate Clinical Velocity & AI, Equipment & National ABDM Stack, Enterprise Hospital Operations). [valid_from: 2026-11-14, recorded: 2026-11-14, source: docs/MISSING_COMPONENTS_AND_REQUIREMENTS.md]

## Superseded
- Current Version Milestone: New Loka is at v0.2.0 with 71 passing tests. [superseded: 2026-11-17, reason: expanded test suite to 72 passing tests covering Phase 4 clinical ingestion and server endpoints]
- Roadmap Phase 4 (Upcoming): Planned local intelligence superseded by completed deterministic clinical NLP extraction, LOINC panic detection, and human-in-the-loop review queue. [superseded: 2026-11-17, reason: shipped full Phase 4 deterministic extraction engine in newloka_clinical, /api/ingest REST routes, and interactive #ingest web workspace]
- Current Version Milestone: New Loka is at v0.2.0 with 51 passing tests. [superseded: 2026-11-16, reason: expanded test suite to 71 passing tests covering pure clinical scoring engine, modular routes, and demo fixtures]
- Current Version Milestone: New Loka v0.1.0/v0.2.0 with 38 tests. [superseded: 2026-11-14, reason: expanded test suite to 51 passing tests covering full T3/T4 core, server, and CLI suites]
- Roadmap Phase 6 (Upcoming): Planned distributed deployment packaging superseded by completed T3 departmental multi-tier implementation. [superseded: 2026-11-14, reason: shipped full T3 multi-department hospital engine and CLI/Web parity]
- Roadmap Phase 9 (Upcoming): Planned research federation superseded by completed T4 research engine. [superseded: 2026-11-14, reason: shipped full T4 differential privacy, mesh node ping/exchange, and FHIR Consent directives]
