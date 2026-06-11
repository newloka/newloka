# Changelog

All notable changes to New Loka will be documented in this file.

## [0.1.0] - 2026-06-11

### Added
- Core Rust library with FHIR R4 resource models
- AES-256-GCM encryption with hierarchical key management (DMK -> PRK)
- Ed25519 audit signing for tamper-evident chains
- Argon2id password hashing for offline authentication
- SQLite CRDT storage engine with encrypted at-rest data
- ABAC policy engine with department silo enforcement
- Emergency override with reason capture and audit logging
- Vector clock sync engine with deterministic merge
- Conflict detection for concurrent edits and medication changes
- Node identity generation for mesh networking
- CLI application with patient, encounter, observation, sync, audit commands
- HTTP server with FHIR R4 REST endpoints (T1+)
- CapabilityStatement at /metadata
- Delta sync endpoints for mesh networking
- Comprehensive unit and integration test suite
- Docker and docker-compose deployment configs
- GitHub Actions CI/CD pipeline
- API documentation
- Architecture, security, developer, and deployment guides
- Open source under GPL-3.0

### Deployment Tiers Supported
- T0: Single clinician offline
- T1: Small clinic LAN mesh
- T2: Rural hospital with intermittent sync
- T3: Multi-department hospital with ABAC silos
- T4: Research federation (infrastructure ready)
