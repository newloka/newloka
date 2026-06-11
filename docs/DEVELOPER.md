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
  test/                - Unit and integration tests
    unit_tests.rs
    integration_tests.rs
    README.md
  docs/                - Documentation
    API.md
    ARCHITECTURE.md
    SECURITY.md
    DEPLOYMENT.md

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

## Testing

# All tests
cargo test

# Unit tests
cargo test --test unit_tests

# Integration tests (require in-memory SQLite)
cargo test --test integration_tests

# With output
cargo test -- --nocapture

## Code Style

- Follow Rust API Guidelines
- All public functions documented with rustdoc
- Error types use thiserror derive macros
- Async functions use tokio runtime
- Tests use in-memory SQLite to avoid disk state

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
