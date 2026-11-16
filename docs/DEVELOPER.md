# New Loka Developer Guide

## Prerequisites

- Rust 1.75+ (install via rustup)
- SQLite development libraries
- Node.js 18+ (for web UI builds)

## Project Structure

newloka/
  Cargo.toml           - Workspace manifest
  newloka_core/        - Rust core library
    src/
      lib.rs           - Library entry point
      fhir/            - FHIR R4 resource models
      crypto/          - Encryption and key management
      storage/         - CRDT SQLite backend
      identity/        - Users, sessions, nodes
      abac/            - Attribute-based access control
      sync/            - Mesh sync and conflict resolution
      audit/           - Tamper-evident audit logging
  newloka_cli/         - Command-line application
    src/main.rs
  newloka_server/      - HTTP server (T1+)
    src/main.rs
    src/lib.rs         - Axum REST API with in-memory stores and mock data seeding
  newloka_web/         - Static web UI (T0?T4)
    index.html
    css/styles.css
    js/
      config.js        - Tier/feature config and localStorage persistence
      api.js           - FHIR R4 API client with IndexedDB offline cache
      mock-data.js     - Local-first mock patient database generator
      app.js           - Single-page application with full CRUD for all tabs
  test/                - Unit and integration tests
    unit_tests.rs
    integration_tests.rs
    README.md
  docs/                - Documentation
    API.md
    ARCHITECTURE.md
    SECURITY.md
    DEPLOYMENT.md
    MISSING_COMPONENTS_AND_REQUIREMENTS.md
    role_ui_matrix.md

## Building

# Build entire workspace
cargo build

# Build release
cargo build --release

# Build specific crate
cargo build -p newloka_cli
cargo build -p newloka_server

## Running

# Initialize a local node
newloka init --password mysecurepassword

# Create a patient
newloka patient create --family Doe --given Jane --gender female --birth-date 1985-03-15

# Start server
newloka serve --bind 127.0.0.1:8080

# Seed mock data via API
curl -X POST http://127.0.0.1:8080/seed?count=20

## Testing

# All tests
cargo test

# Unit tests
cargo test --test unit_tests

# Integration tests (require in-memory SQLite)
cargo test --test integration_tests

# With output
cargo test -- --nocapture

## Code & Architectural Style

See the definitive and comprehensive **[New Loka Code & Architectural Style Guide](file:///d:/GitRepos/newloka/docs/STYLE_GUIDE.md)** for our complete engineering standards, including:
- **WATER Pattern Management Strategy:** WET on 1 and 2, DRY on 3+ to prevent both premature abstraction and boilerplate rot.
- **Pure Functions First:** Clinical calculations, safety checks, scoring algorithms, and data transformations are strictly pure functions with zero side-effects.
- **Rust Systems Engineering:** High-assurance domain modeling, `thiserror`/`anyhow` error boundaries, zero-cost embedded indexes, and async Tokio hygiene.
- **Web & UX Standards:** 100% air-gapped zero-CDN vanilla ES6 SPA, 260px Epic Storyboard rail, and the 3-Tap Invariant for clinical safety.
- **HL7 FHIR R4 & CPOE:** 5-stage order lifecycle, Indian Pharmacopoeia (IP 2022) drug safety shield, and non-dangerous merge rules.

## Contributing

1. Start with Phase 1 core work if new
2. Run tests before submitting changes
3. Document public APIs
4. Follow the tier abstraction - do not bypass core layers
5. Security changes require audit module updates

## Phase Roadmap

Phase 1 - Core Offline Clinician Workflow
  - Local encrypted patient storage
  - FHIR R4 resource model
  - Patient CRUD, encounter timeline
  - Offline auth and audit

Phase 2 - Local Sync and Mesh
  - Peer discovery, delta sync
  - Vector clock merge
  - Conflict detection

Phase 3 - Institutional Access Control
  - ABAC policy engine
  - Department silos
  - Emergency override with audit

Phase 4 - Local Intelligence
  - OCR, speech-to-text
  - Structured extraction
  - Review queues

Phase 5 - Federation and Research
  - Consent-driven sharing
  - Privacy-preserving queries
  - ABDM/ABHA integration

## Mock Data

The server (
ewloka_server) seeds 20 mock patients on startup with realistic
FHIR R4 encounters, observations, conditions, medications, and procedures.
Use POST /seed?count=N to regenerate or adjust the count.

The web UI (
ewloka_web) includes a client-side mock data generator
(mock-data.js) that seeds IndexedDB for offline-first testing. Click
"?? Seed Local Data" on the Dashboard or in Settings.
