# Storage Engine, CRDT Synchronization, and Cryptography

## Epistemic Metadata
- **Entity**: Encrypted Storage Engine & Cryptographic Key Hierarchy
- **Layer**: Layer 1: Structural & Architectural
- **Status**: SHIPPED
- **Domain**: Data Confidentiality, Key Management & Distributed Consistency

## Facts
- SQLite Storage Engine: High-performance embedded database with connection pooling via SQLx, supporting in-memory instances (`sqlite::memory:?cache=shared`) and durable on-disk files with automated database schema migrations on initialization. [valid_from: 2026-06-11, recorded: 2026-11-13, source: newloka_core/src/storage/mod.rs]
- Cryptographic Confidentiality at Rest: Every FHIR resource payload is symmetrically encrypted using AES-256-GCM (Galois/Counter Mode) with 96-bit nonces, ensuring authenticated encryption with associated data (AEAD) to prevent plaintext inspection or ciphertext tampering on disk. [valid_from: 2026-06-11, recorded: 2026-11-13, source: newloka_core/src/crypto/mod.rs]
- Hierarchical Key Model: Implements a 3-tier key model comprising the Device Master Key (DMK, derived from hardware keystore or Argon2id password hash), Patient Record Key (PRK, wrapped by DMK), and encrypted FHIR record ciphertexts. [valid_from: 2026-06-11, recorded: 2026-11-13, source: newloka_core/src/crypto/mod.rs & docs/SECURITY.md]
- CRDT Sync & Vector Clocks: Employs vector clock tracking across nodes where dominating clocks cleanly fast-forward to the newer version, identical clocks result in zero mutation, and concurrent vector clocks trigger conflict detection. [valid_from: 2026-06-11, recorded: 2026-11-13, source: newloka_core/src/sync/mod.rs]
- Delta Sync Protocol: Asynchronous sync between authorized nodes via `/sync/manifest` and `/sync/delta` transmitting only mutated records since the last shared timestamp to minimize bandwidth. [valid_from: 2026-06-11, recorded: 2026-11-13, source: newloka_core/src/sync/mod.rs & newloka_server/src/lib.rs]
- Conflict Resolution Safety Rule: Non-critical demographic metadata may merge deterministically, but clinically dangerous conflicts (medications, allergies, active orders) are flagged for mandatory clinician review rather than auto-resolved. [valid_from: 2026-06-11, recorded: 2026-11-13, source: Project_Scope_and_Requirements.txt §4.6]
- EncryptedRecord Serialization & Pre-Encrypted Fixture Import: `EncryptedRecord` implements `Serialize` and `Deserialize` to enable storing, exporting, and importing pre-encrypted data packets. `StorageEngine::encrypt_record` performs off-database AEAD encryption with a specified DMK, and `StorageEngine::decrypt_record` cryptographically verifies that a record can be unwrapped and decrypted by the active storage DMK prior to storing it via `store_encrypted_record`. [valid_from: 2026-11-15, recorded: 2026-11-16, source: newloka_core/src/storage/mod.rs]
- Decoupled Storage Crate Extraction (`newloka_storage`): Decouples the SQLite SQLx database engine, EncryptedRecord persistence, and delta sync manifest builder into an independent workspace crate (`newloka_storage`), liberating `newloka_core` from all SQLx and native C-binding dependencies to serve as a pure domain and cryptographic security core. [valid_from: 2026-11-17, recorded: 2026-11-17, source: newloka_storage/src/lib.rs & Cargo.toml]

## Superseded
(none)
