# New Loka Security Documentation

## Threat Model

New Loka is designed to protect patient health information across five deployment tiers, from a single clinician's phone to multi-institution research networks.

## Security Principles

1. Defense in depth - Multiple independent barriers
2. Security at the data layer - Not just UI gating
3. Encryption by default - All data at rest encrypted
4. Tamper-evident audit - Every action signed and chained
5. Least privilege - ABAC enforced for every request
6. Local-first - No external dependency for basic security

## Key Hierarchy

Device Master Key (DMK)
  Derived from: PIN/biometric/Argon2id password
  Storage: Hardware-backed keystore (T0) or local KMS (T1+)
  Purpose: Wraps all patient record keys

Patient Record Key (PRK)
  Generated: Per-patient or per-record
  Storage: DMK-wrapped in database
  Purpose: Encrypts individual FHIR resources

Sync Session Key
  Generated: Per-sync session
  Storage: Ephemeral in memory only
  Purpose: Encrypts data in transit between peers

Audit Signing Key
  Generated: Per-node Ed25519 keypair
  Storage: Private key in secure storage
  Purpose: Signs every audit entry

## Three-Barrier Request Model

Barrier 1: Identity Check
  - Argon2id password hash verification
  - Session token validity check
  - TOTP validation (when configured)
  - Fail: Request rejected, data never touched

Barrier 2: ABAC Policy
  - Cedar-style policy evaluation
  - Subject x Resource x Action x Context
  - Department silo enforcement
  - Emergency override with reason capture
  - Fail: PRK never unwrapped

Barrier 3: PRK Unwrap
  - DMK decrypts PRK
  - Only inside verified session
  - Data decrypted in memory only
  - Re-encrypted on write

## Encryption Details

Algorithm: AES-256-GCM
  - Key size: 256 bits
  - Nonce: 96 bits, random per encryption
  - Tag: 128 bits, authenticated

Key Derivation: Argon2id
  - Memory: 64MB
  - Iterations: 3
  - Parallelism: 4 lanes

Signing: Ed25519
  - PureEdDSA for audit entries
  - Deterministic signatures

## Audit Trail

Properties:
  - Append-only
  - Cryptographically chained
  - Per-entry Ed25519 signature
  - Tamper-evident verification
  - Permanent retention, no purge

Entry Types:
  - Access, Create, Update, Delete
  - Sync (send/receive)
  - Override (with reason)
  - Correction (with original reference)
  - Login, Logout
  - Policy Deny
  - AI Review
  - Transfer

## Compliance Targets

DISHA / IT Act (India)
  - Minimum 7 year retention
  - Right to access (FHIR Bundle export)
  - Soft delete semantics

HIPAA (US)
  - Access controls at data layer
  - Audit logging for all PHI access
  - Encryption at rest and in transit

GDPR (EU)
  - Consent-driven research sharing
  - Right to erasure (soft delete + flag)
  - Data portability (FHIR Bundle export)

## Security Checklist

Before deployment:
  [ ] DMK derived from strong credential
  [ ] Hardware keystore used where available
  [ ] Audit chain verified on startup
  [ ] ABAC policies tested for all roles
  [ ] Emergency override workflow tested
  [ ] Sync encryption validated end-to-end
  [ ] Backup keys stored securely offline
  [ ] Retention policy configured per jurisdiction
