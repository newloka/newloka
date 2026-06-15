# New Loka Architecture

## Overview

New Loka is a local-first, polymorphic health data management system. It follows a tiered deployment model (T0–T4) that scales from a single clinician on a phone to a research federation without forking the core codebase.

## Design Principles

1. **Local-first operation** – Works fully offline; sync is additive, not required.
2. **FHIR R4 internal model** – All clinical records are FHIR R4 resources.
3. **CRDT-based data model** – Vector clocks enable deterministic merge and conflict detection.
4. **Security at the data layer** – Encryption and ABAC enforced in the core, not only the UI.
5. **Extensible by modules** – New capabilities are added via modules, not forks.

## Crate Layout

| Crate         | Role                                             |
|---------------|--------------------------------------------------|
| `newloka_core`| Core library: FHIR models, crypto, storage, ABAC, sync, audit, identity |
| `newloka_server`| Axum HTTP server for T1+ deployments           |
| `newloka_cli` | Command-line interface and admin tooling         |
| `newloka_web` | Built-in SPA frontend (HTML/CSS/JS)              |

## Data Flow

```
Client (SPA)  ←→  HTTP/REST (FHIR R4)  ←→  newloka_server
                                    ↓
                              newloka_core
                                    ↓
                     SQLite (encrypted at rest)
```

### Frontend

The built-in web frontend (`newloka_web/`) is a zero-dependency vanilla JS SPA:
- Dark, mobile-first theme with offline banner
- IndexedDB local cache with offline write queue
- Tier-aware navigation (features gated by deployment tier)
- Modular views: Dashboard, Patients, Encounters, Observations, Conditions, Medications, Procedures, Ingest, Handoff, Audit, Settings

### Backend

#### HTTP Layer (`newloka_server`)
- Axum router with CORS
- Serves static files from `newloka_web/`
- FHIR R4 CRUD endpoints for Patient, Encounter, Observation, AuditEvent
- Sync endpoints: `/sync/manifest`, `/sync/delta`

#### Core (`newloka_core`)
- **fhir**: Typed FHIR R4 resource models (Patient, Encounter, Observation, etc.)
- **storage**: SQLite engine with AES-256-GCM per-record encryption, CRDT metadata, soft deletes
- **crypto**: Device Master Key (DMK), Patient Record Key (PRK), Argon2id, Ed25519 audit signing
- **audit**: Append-only signed audit trail with tamper-evident chain
- **abac**: Attribute-Based Access Control with department silos, emergency override, role checks
- **sync**: Vector clock handling, delta sync, deterministic conflict detection
- **identity**: User roles, sessions, offline auth, node identity

## Deployment Tiers

| Tier | Target                          | Key Features                          |
|------|---------------------------------|---------------------------------------|
| T0   | Single clinician (phone)      | Offline, PIN auth, local storage      |
| T1   | Small clinic mesh               | LAN sync, peer discovery              |
| T2   | Rural hospital                  | Local server, intermittent cloud      |
| T3   | Multi-department hospital     | Department silos, LDAP, handoff       |
| T4   | Research federation            | Privacy-preserving exchange, consent  |

## Security Model

- **Encryption at rest**: Every FHIR resource encrypted with a unique PRK, wrapped by DMK.
- **Key hierarchy**: DMK → PRK → record ciphertext. DMK derived from hardware keystore or Argon2id password.
- **Audit signing**: Ed25519 signatures on every audit entry; chain verification ensures tamper evidence.
- **ABAC**: Evaluated at the core layer based on role, department, team, resource sensitivity, and tier context.

## Demo Mode

The development server ships with a deterministic demo DMK and auto-seeds 20 patients with realistic Indian demographics, encounters (ambulatory), vital signs (BP, temperature), laboratory results (HbA1c with LOINC interpretation codes), and 10 audit events on first startup. The default database is an in-memory shared-cache SQLite instance (`sqlite::memory:?cache=shared`).

## Build and Run

```bash
# Build all crates
cargo build

# Start server (T1+)
$env:NEWLOKA_STATIC_DIR = "D:\New Loka\newloka\newloka_web"
cargo run -p newloka_server
```

Open `http://127.0.0.1:8080/` in a browser. The SPA will redirect and prompt for tier + PIN. Use any PIN for demo mode.
