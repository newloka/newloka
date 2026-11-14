# Audit Engine and Clinical Safety Architecture

## Epistemic Metadata
- **Entity**: Tamper-Evident Audit Trail & Clinical Safety Guards
- **Layer**: Layer 1: Structural & Architectural
- **Status**: SHIPPED
- **Domain**: Cryptographic Accountability, Safety Invariants & Forensics

## Facts
- Append-Only Tamper-Evident Audit Engine (`AuditEngine`): Every security-relevant and clinical operation (login, record view, record mutation, soft deletion, sync transaction, CPOE order placement, emergency override) is recorded in an immutable append-only audit ledger. [valid_from: 2026-06-11, recorded: 2026-11-13, source: newloka_core/src/audit/mod.rs]
- Cryptographic Signatures (Ed25519): Each audit entry is digitally signed using an Ed25519 private key tied to the operating node (`AuditSigner`). Signatures include previous block hashes, forming a verifiable cryptographic hash chain that reveals any post-hoc database tampering or record manipulation. [valid_from: 2026-06-11, recorded: 2026-11-13, source: newloka_core/src/audit/mod.rs & newloka_core/src/crypto/mod.rs]
- Human-in-the-Loop Review Invariant: AI, OCR, or automated extraction systems (such as handwritten prescription scans, PDF lab reports, or speech-to-text dictations) are strictly prohibited from committing directly to the permanent EHR chart; all extracted entities must pass through a clinician review interface before persistence. [valid_from: 2026-06-11, recorded: 2026-11-13, source: Project_Scope_and_Requirements.txt §2, §4.7]
- Regulatory Compliance Mapping: The audit and privacy envelope is engineered to comply with HIPAA (Security & Privacy Rules, 45 CFR Part 160/164), India Digital Information Security in Healthcare Act (DISHA), Information Technology Act 2000, and European General Data Protection Regulation (GDPR) for clinical research. [valid_from: 2026-06-11, recorded: 2026-11-13, source: Project_Scope_and_Requirements.txt §7]
- Retrospective Forensic Querying: The `/AuditEvent` REST endpoint and `newloka-cli audit` command allow rapid forensic verification, inspection of chain continuity, and extraction of timeline proof for regulatory reviews. [valid_from: 2026-06-11, recorded: 2026-11-13, source: newloka_server/src/lib.rs & newloka_cli/src/main.rs]

## Superseded
(none)
