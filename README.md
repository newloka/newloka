# New Loka

New Loka is an open source, local-first, polymorphic health data management system for patient care, clinical operations, and later research.

It is designed to run from the smallest deployment to the largest:

- a single clinician on a phone with offline storage
- a small clinic operating as a peer mesh
- a rural hospital with intermittent connectivity
- a multi-department hospital with siloed access controls
- a research federation with privacy-preserving exchange

The same codebase must serve all tiers. What changes by deployment is which modules are activated, how trust is established, and what infrastructure is available.

## Philosophy

- Platform-agnostic core, platform-specific UI
- Local-first operation with optional sync
- CRDT-native data model for deterministic merge
- FHIR R4 as the internal clinical schema
- Security at the data layer, not only the UI layer
- No patient data egress to external services without explicit consent and institutional approval
- Extensible by design: features should be added as modules, not forks

## Architecture

New Loka is structured as three layers around a Rust core:

1. Core
   - FHIR resource model
   - CRDT storage
   - crypto and key management
   - identity and session handling
   - ABAC policy evaluation
   - sync, conflict resolution, and audit logging
   - local AI orchestration and clinical rules
2. Mesh and services
   - peer discovery and delta sync
   - local OCR, speech-to-text, and extraction
   - institutional FHIR services at higher tiers
   - deployment-specific adapters and integrations
3. UI
   - mobile, desktop, and web front ends
   - platform-specific shell around the shared core

## Deployment Tiers

- T0: single clinician, offline first, device-local encrypted storage
- T1: small clinic, LAN mesh, shared hub optional
- T2: rural hospital, local server plus intermittent cloud sync
- T3: multi-department hospital, ABAC silos, full institutional controls
- T4: research federation, privacy-preserving exchange and federated workflows

## Core Requirements

- Every record must map cleanly to a FHIR R4 resource
- Every write must be attributable, encrypted, and auditable
- Every deployment must work without external network access
- Every policy decision must be enforced by the core, not the UI
- Every AI output must pass through human review before commit
- Every sync path must preserve provenance and handle conflict explicitly

## Major Capabilities

- patient registration and lookup
- chart timelines and encounter views
- vitals, notes, orders, medications, and discharge workflows
- document ingestion from camera, PDF, voice, and dictation
- local transcription and structured extraction
- emergency override and retrospective audit review
- patient transfer and secure record sharing between teams or nodes

## Standards and Compliance Targets

- HL7 FHIR R4
- SNOMED CT
- LOINC
- ICD-10 / ICD-11 where applicable
- HIPAA for US-facing deployments
- DISHA / IT Act for India-facing deployments
- GDPR for research federation contexts

## Build Sequence

The build sequence is intentionally bottom-up. Contributors should treat each phase as a dependency gate for the next one. Do not start on higher-tier features until the lower-tier core is stable, testable, and usable in isolation.

### Phase 1: Core Offline Clinician Workflow

Goal: make the system useful to a single clinician with no network and no external dependencies.

Required deliverables:

- local encrypted patient storage
- FHIR R4 resource model in the core
- patient creation, search, edit, and review
- encounter timeline and chart view
- notes, vitals, medications, and basic orders
- offline authentication and session handling
- audit logging for every write and access
- minimal UI shells for mobile and desktop

Definition of done:

- a clinician can manage a patient record entirely offline
- the system remains usable after app restart
- all writes are attributable and encrypted
- no server is required for normal use

### Phase 2: Local Sync and Mesh Networking

Goal: let multiple authorized nodes exchange records without central dependence.

Required deliverables:

- node identity generation and trust bootstrap
- local peer discovery
- delta sync and vector clock handling
- deterministic conflict detection and merge behavior
- secure record transfer between nodes
- sync status visibility in the UI
- offline-safe retry behavior

Definition of done:

- two or more devices can exchange records on the same local network
- no single peer is required for the mesh to function
- conflicts are surfaced explicitly and never hidden
- provenance survives merge and transfer

### Phase 3: Institutional Access Control and Silo Enforcement

Goal: support clinics and hospitals where different teams must be separated by policy, not by convention.

Required deliverables:

- ABAC policy engine in the core
- roles, departments, teams, and resource scope mapping
- enforced data-layer silos
- emergency override with reason capture and logging
- institutional admin workflows
- secure patient transfer between teams and departments
- integration points for hospital identity systems and internal FHIR services

Definition of done:

- a user can only access data allowed by policy
- the policy decision is enforced even if the UI is bypassed
- department boundaries are real in the data layer
- emergency access is possible but fully auditable

### Phase 4: Local Intelligence and Automated Extraction

Goal: add local AI assistance without making the product dependent on external services.

Required deliverables:

- handwriting OCR from images
- PDF ingestion and parsing
- voice note and dictation transcription
- structured extraction into FHIR resources
- review queue for every AI-generated result
- confidence scoring and warning metadata
- local clinical rules and safety checks

Definition of done:

- AI can assist with data entry on-device or on trusted local infrastructure
- nothing AI-generated is committed without human review
- extraction output is structured, inspectable, and reversible
- safety checks can block or flag dangerous entries

### Phase 5: Federated and Research Workflows

Goal: support cross-institution research and privacy-preserving exchange without weakening the core clinical model.

Required deliverables:

- consent-driven sharing and revocation
- privacy-preserving query and export paths
- federated or aggregate research workflows
- standards-aligned external exchange surfaces
- auditability for research access and transfer
- deployment profiles for institutions participating in shared ecosystems

Definition of done:

- research use does not expose raw patient data by default
- consent is explicit and enforceable
- clinical operation remains independent of research workflows

Contributor guidance:

- If you are new, start with Phase 1 core work.
- If you are working on sync, stay inside Phase 2 until merge behavior is stable.
- If you are working on policy or hospital features, depend on Phase 3 abstractions instead of bypassing the core.
- If you are adding AI, route everything through Phase 4 review queues.
- If you are working on research features, assume Phase 5 is layered on top of a finished clinical system, not the other way around.

Each phase should leave the system more usable, not more fragile. The intended order is: local clinical value first, then trusted mesh exchange, then institutional control, then intelligence, then federation.

The goal is not to build separate products for different sites. The goal is one system whose behavior is shaped by deployment context, policy, and available infrastructure.
