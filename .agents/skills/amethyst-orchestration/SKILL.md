---
name: amethyst-orchestration
version: 1.0.0
description: "Plan, orchestrate, build, test, and iterate New Loka multi-crate Rust architecture (core, server, cli, web)."
author: Amethyst Agent
license: GPL-3.0
platforms: [windows, linux, macos]
metadata:
  tags: [Rust, MultiCrate, Architecture, Testing, NewLoka]
---

# Amethyst Codebase & Architecture Orchestration

Orchestrates development, testing, refactoring, and architectural iteration across the New Loka workspace:
- `newloka_core`: FHIR R4 data models, storage engine, cryptography, CRDT sync, ABAC, and audit log.
- `newloka_server`: Axum 0.7 REST/FHIR server, static SPA hosting, demo data seeding, eRx pad integration, and self-updater.
- `newloka_cli`: Headless terminal commands for database init, clinical record CRUD, and serving.
- `newloka_web`: Embedded vanilla ES6 clinical single-page application with 27 clinical views.

## Core Operational Procedures

### 1. Preflight Compilation Check
Always verify compilation health across all workspace members before and after code changes:
```powershell
cargo check
```
For individual crates:
```powershell
cargo check -p newloka_core
cargo check -p newloka_server
cargo check -p newloka_cli
```

### 2. Full Workspace Test Suite
Run the comprehensive test suite (unit tests, integration tests, server tests):
```powershell
cargo test --workspace
```
Ensure all 38+ tests pass with zero failures.

### 3. Adding New FHIR R4 Endpoints
When expanding clinical resource coverage:
1. Define the typed resource struct in `newloka_core/src/fhir/mod.rs` implementing `Serialize`, `Deserialize`, and `Clone`.
2. Register the resource variant inside the `FhirResource` enum wrapper.
3. Update `StorageEngine` in `newloka_core/src/storage/mod.rs` to handle search and retrieval queries.
4. Add the Axum routes in `newloka_server/src/lib.rs` (e.g. `GET /<Resource>`, `GET /<Resource>/:id`, `POST /<Resource>`, `PUT /<Resource>/:id`, `DELETE /<Resource>/:id`).
5. Wire the client API call in `newloka_web/js/api.js` and render in `newloka_web/js/app.js`.
6. Add unit and integration tests in `newloka_core/tests/` and `newloka_server/tests/`.
7. Update the knowledge graph (`knowledge_graph/nodes/fhir-clinical-models.md` and `server-and-api.md`).

### 4. Running the Development Server
```powershell
# In PowerShell:
$env:NEWLOKA_STATIC_DIR = ".\newloka_web"
cargo run -p newloka_server -- --tier T0 --db ./clinic.db --bind 127.0.0.1:8080
```
Open `http://127.0.0.1:8080` in the browser to inspect UI behavior.

### 5. Architectural Invariants
- Never break backward compatibility for stored encrypted SQLite records.
- Never bypass the `newloka_core` ABAC evaluation layer.
- Never add external remote network dependencies to the offline core clinical path.
