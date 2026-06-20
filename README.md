# New Loka

Open source, local-first, polymorphic health data management system for patient care, clinical operations, and research.

Designed to run from the smallest deployment to the largest:
- T0: Single clinician on a phone with offline storage
- T1: Small clinic operating as a peer mesh
- T2: Rural hospital with intermittent connectivity
- T3: Multi-department hospital with siloed access controls
- T4: Research federation with privacy-preserving exchange

## Quick Start

# Build
cargo build --release

# Initialize node
newloka init --password yourpassword

# Create patient
newloka patient create --family Doe --given Jane --gender female --birth-date 1985-03-15

# Start server (T1+)
newloka serve --bind 127.0.0.1:8080

## Project Structure

newloka/
  newloka_core/     - Rust core (FHIR, CRDT, crypto, ABAC, sync, audit)
  newloka_cli/      - Command-line interface
  newloka_server/   - HTTP server for T1+ deployments
  test/             - Comprehensive test suite
  docs/             - API, architecture, security, deployment guides

## Philosophy

- Platform-agnostic core, platform-specific UI
- Local-first operation with optional sync
- CRDT-native data model for deterministic merge
- FHIR R4 as the internal clinical schema
- Security at the data layer, not only the UI layer
- No patient data egress without explicit consent
- Extensible by modules, not forks

## Documentation

- docs/API.md           - REST API and FHIR endpoints
- docs/ARCHITECTURE.md  - System layers and design decisions
- docs/DEVELOPER.md     - Build, test, and contribution guide
- docs/SECURITY.md      - Threat model, encryption, audit, compliance
- docs/DEPLOYMENT.md    - Tier-specific deployment instructions

## License

GPL-3.0

## Roadmap

- **Mobile app** ? Offline-first React Native / Flutter companion for field clinicians, syncing with the mesh via Bluetooth and Wi-Fi Direct.
- **Biometric auth** ? Fingerprint / face unlock integration for T0 deployments.
- **Telemedicine module** ? Embedded video consults with audit logging.
- **OpenHealth integration** ? UHI (India) and FHIRcast event subscriptions.
