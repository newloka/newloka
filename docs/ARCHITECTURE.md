# New Loka Architecture

## System Layers

UI Layer (Mobile / Desktop / Web)
  Flutter / React / Tauri shells

Bridge Layer (FFI / WASM / HTTP)
  Platform-specific adapters

Core Layer (Rust)
  - FHIR R4 Resource Model
  - CRDT Storage Engine
  - AES-256-GCM Encryption
  - Ed25519 Audit Signing
  - ABAC Policy Engine
  - Vector Clock Sync
  - Identity and Session Management

Services Layer
  - Peer Discovery (mDNS / BLE / WiFi Direct)
  - Delta Sync and Conflict Resolution
  - Local OCR / Speech-to-Text
  - Clinical Rules Engine

Infrastructure Layer
  - SQLite (local encrypted)
  - Hardware Keystore (T0)
  - HAPI FHIR Server (T3)
  - Kubernetes / Vault (T3-T4)

## Deployment Tiers

Tier | Context                | Connectivity              | Auth Model
-----|------------------------|---------------------------|---------------------------
T0   | Single clinician       | Offline / occasional sync | Single user, PIN/biometric
T1   | Small clinic (2-10)    | LAN only                  | Role-based, local admin
T2   | Rural hospital         | Intermittent internet     | RBAC, department-aware
T3   | Multi-dept hospital    | Reliable internet         | Full RBAC, silo enforcement
T4   | Research federation    | Full internet             | Institutional trust chains

## Data Flow

Single Write (T0):
  UI -> Rust Core -> FHIR JSON -> AES-GCM encrypt -> SQLite

Sync (T1+):
  Node A: SQLite -> encrypt -> delta sync -> mDNS discovery -> Node B
  Node B: ABAC validate -> decrypt -> CRDT merge -> SQLite

Security Request Flow:
  Request -> Identity Check -> ABAC Policy -> PRK Unwrap -> Serve + Audit Sign

## Key Design Decisions

1. FHIR R4 as internal schema - Interoperability is structural
2. CRDT-native - Offline edits always merge deterministically
3. Security at data layer - Policy misconfig cannot expose encrypted data
4. Same codebase, all tiers - Behavior shaped by context, not forks
5. Append-only audit - Cryptographically chained, tamper-evident
6. Soft deletes only - Regulatory compliance requires retention

## Module Responsibilities

fhir     - Canonical data model, all clinical records
crypto   - Encryption, key hierarchy, signing, hashing
storage  - CRDT SQLite backend, encrypted at rest
identity - Users, sessions, node identity, offline auth
abac     - Policy evaluation, department silos, override
sync     - Vector clocks, delta sync, conflict detection
audit    - Signed chain, every access logged
