# Attribute-Based Access Control and Identity Management

## Epistemic Metadata
- **Entity**: ABAC Policy Engine & Offline Identity
- **Layer**: Layer 1: Structural & Architectural
- **Status**: SHIPPED
- **Domain**: Authorization, Institutional Silos & Emergency Overrides

## Facts
- Core-Enforced ABAC: Attribute-Based Access Control is evaluated directly inside `newloka_core::abac`, ensuring that authorization rules cannot be bypassed by direct API calls, CLI invocation, or manipulated web browser state. [valid_from: 2026-06-11, recorded: 2026-11-13, source: newloka_core/src/abac/mod.rs]
- Evaluation Dimensions: Evaluates access decisions across Subject Attributes (roles, Department ID, Team IDs, Lab Affiliations), Resource Attributes (type, sensitivity, department, lab, patient ID), Action (Read, Create, Update, Delete, Export, Override), and Environment Attributes (Deployment Tier T0–T4, Node ID, time). [valid_from: 2026-06-11, recorded: 2026-11-13, source: newloka_core/src/abac/mod.rs]
- Department & Lab Silos: In T3+ hospital deployments, clinical data is strictly partitioned into departmental silos where Biochemistry staff only access biochemistry orders, Radiologists have permissions exclusively within Radiology imaging and reports, and Pathologists/Microbiologists are restricted to their tissue and culture scopes. [valid_from: 2026-06-20, recorded: 2026-11-13, source: Project_Scope_and_Requirements.txt §4.3 & newloka_core/src/abac/mod.rs]
- Emergency Override ('Break Glass' Protocol): When life-threatening circumstances require immediate cross-silo or elevated access, clinicians can execute an emergency override requiring a mandatory justification string, instantly generating a cryptographically signed high-severity AuditEvent, and granting temporary scoped access. [valid_from: 2026-06-11, recorded: 2026-11-13, source: newloka_core/src/abac/mod.rs & Project_Scope_and_Requirements.txt §4.5]
- Offline-Capable Authentication: Local Argon2id password hashing and PIN authentication allow full authentication and session creation without requiring an external OAuth, OpenID Connect, or cloud IAM provider, with LDAP/Active Directory bridges supported for T3+ hospitals. [valid_from: 2026-06-11, recorded: 2026-11-13, source: newloka_core/src/identity/mod.rs]
- Reception & Billing Role ABAC Data Isolation: Added `Role::Billing` in `newloka_core::identity` with dedicated permissions (`can_create_patient: true`, `can_prescribe: false`, `can_view_patient_list: true`, `can_access_billing: true`). Core ABAC rules strictly forbid billing staff from prescribing or altering clinical orders and automatically redact confidential patient diagnoses (`Condition`, `ClinicalImpression`, and restricted notes) to `🔒 [CLINICAL DATA REDACTED - ABAC CONFIDENTIALITY]`. [valid_from: 2026-11-14, recorded: 2026-11-14, source: newloka_core/src/identity/mod.rs & newloka_core/src/abac/mod.rs]
- Legal Prescriber Identity Verification: Electronic prescribing enforces prescriber credential verification against registered state medical council numbers (e.g. NL-DOC-2026-001) and ABAC clearance flags (`is_legal_prescriber`), rejecting prescription signing by non-authorized roles. [valid_from: 2026-11-14, recorded: 2026-11-14, source: newloka_server/src/lib.rs & newloka_web/js/app.js]

## Superseded
(none)
