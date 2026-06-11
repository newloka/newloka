# New Loka API Documentation

## Base URL

- Local CLI: direct core library calls
- T1+ Server: http://localhost:8080

## FHIR R4 Endpoints

### Capability Statement
GET /metadata

Returns FHIR CapabilityStatement with supported resources and interactions.

### Patient
GET    /Patient              - Search patients
GET    /Patient/{id}         - Read patient
POST   /Patient              - Create patient
PUT    /Patient/{id}         - Update patient
DELETE /Patient/{id}         - Soft delete patient

Search Parameters:
- patient  - Patient ID filter
- _count   - Result limit

### Encounter
GET    /Encounter            - Search encounters
POST   /Encounter            - Create encounter

### Observation
GET    /Observation          - Search observations
POST   /Observation          - Create observation

### AuditEvent
GET    /AuditEvent           - Query audit log
POST   /AuditEvent           - Record audit event

## Sync Endpoints

### Node Manifest
GET /sync/manifest

Returns node identity, tier, last sync timestamp, and supported resource types.

### Delta Sync
POST /sync/delta

Request body:
{
  from_node: node-uuid,
  to_node: peer-uuid,
  since_timestamp: 1718000000000,
  known_vector_clocks: []
}

Response:
{
  from_node: node-uuid,
  records: [],
  conflicts: [],
  timestamp: 1718000000000
}

## Health Check
GET /health

Response:
{
  status: healthy,
  version: 0.1.0,
  node_id: server-node,
  tier: T1
}

## Data Model

All clinical records use FHIR R4 resources internally.
Supported resources:
- Patient
- Encounter
- Observation
- Condition
- MedicationRequest
- Procedure
- DiagnosticReport
- Composition
- DetectedIssue
- AuditEvent
- Provenance
- Bundle

## Authentication

T0: Local PIN/biometric-derived DMK
T1+: Session tokens with Argon2id + optional TOTP
T3+: LDAP/Active Directory integration points

Every request is validated through:
1. Identity check (session token validity)
2. ABAC policy evaluation
3. Data-layer decryption (PRK unwrap)
4. Audit logging

## Error Responses

Standard HTTP status codes with FHIR OperationOutcome:
{
  resourceType: OperationOutcome,
  issue: [{
    severity: error,
    code: security,
    diagnostics: ABAC policy denied access
  }]
}
