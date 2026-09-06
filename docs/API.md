# New Loka API Documentation

## Base URL

- Local CLI: direct core library calls
- HTTP Server (T0–T4): `http://127.0.0.1:8080` (or configured `--bind` address)

## Authentication

- **T0 (Solo Clinician)**: Local PIN / biometric-derived DMK or local session token.
- **T1+ (Mesh / Clinic)**: Session tokens with Argon2id password hashing + optional TOTP.
- **T3+ (Hospital)**: LDAP / Active Directory integration points.


Every request validated through:
1. Identity check (session token validity)
2. ABAC policy evaluation
3. Data-layer decryption (PRK unwrap)
4. Audit logging

## FHIR R4 Endpoints

### Capability Statement
`GET /metadata`

Returns FHIR CapabilityStatement with supported resources and interactions.

### Patient
- `GET    /Patient`              – Search patients (returns FHIR Bundle)
- `GET    /Patient/{id}`         – Read patient by ID
- `POST   /Patient`              – Create patient
- `PUT    /Patient/{id}`         – Update patient
- `DELETE /Patient/{id}`         – Soft delete patient

Search Parameters:
- `patient`  – Patient ID filter
- `_count`   – Result limit (default 500)

### Encounter
- `GET    /Encounter`            – Search encounters
- `GET    /Encounter/{id}`       – Read encounter by ID
- `POST   /Encounter`            – Create encounter
- `PUT    /Encounter/{id}`      – Update encounter
- `DELETE /Encounter/{id}`       – Soft delete encounter

### Observation
- `GET    /Observation`          – Search observations
- `GET    /Observation/{id}`     – Read observation by ID
- `POST   /Observation`          – Create observation
- `PUT    /Observation/{id}`    – Update observation
- `DELETE /Observation/{id}`     – Soft delete observation

### AuditEvent
- `GET    /AuditEvent`           – Query persistent audit log
- `POST   /AuditEvent`           – Record audit event

### Sync Endpoints

#### Node Manifest
`GET /sync/manifest`

Returns node identity, tier, last sync timestamp, and supported resource types.

#### Delta Sync
`POST /sync/delta`

Request body:
```json
{
  "from_node": "node-uuid",
  "to_node": "peer-uuid",
  "since_timestamp": 1718000000000,
  "known_vector_clocks": []
}
```

Response:
```json
{
  "from_node": "node-uuid",
  "records": [],
  "conflicts": [],
  "timestamp": 1718000000000
}
```

## Static Files / Frontend

The server hosts the built-in SPA frontend under `/static/`.

- `GET /`                        – Redirects to `/static/index.html`
- `GET /static/index.html`       – Main SPA entry point
- `GET /static/css/styles.css`   – Application stylesheet
- `GET /static/js/app.js`        – Application logic
- `GET /static/js/config.js`     – Tier-based feature flags
- `GET /static/js/api.js`        – FHIR API client + offline cache
- `GET /static/assets/manifest.json` – PWA manifest

## Health Check

`GET /health`

Response:
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "node_id": "server-node",
  "tier": "T0"
}
```

## System & In-App Updater Endpoints

### System Information
`GET /api/system/version`

Response:
```json
{
  "version": "0.1.0",
  "tier": "T0",
  "os": "windows",
  "arch": "x86_64",
  "exe_path": "C:\\Users\\synch\\.cargo\\bin\\newloka-server.exe",
  "uptime_seconds": 3600
}
```

### Check for Updates
`GET /api/system/update/check`

Queries the latest release from GitHub Releases API:
```json
{
  "status": "update_available",
  "current_version": "0.1.0",
  "latest_version": "0.2.0",
  "update_available": true,
  "release_name": "New Loka v0.2.0",
  "release_notes": "...",
  "published_at": "2026-09-06T12:00:00Z",
  "asset_name": "newloka-windows-x86_64.zip",
  "asset_url": "https://github.com/newloka/newloka/releases/download/v0.2.0/newloka-windows-x86_64.zip",
  "asset_size": 12908000,
  "os": "windows",
  "arch": "x86_64",
  "message": "A new version (v0.2.0) is available for download."
}
```

### Apply Update (1-Click)
`POST /api/system/update/apply`

Request body:
```json
{
  "asset_url": "https://github.com/newloka/newloka/releases/download/v0.2.0/newloka-windows-x86_64.zip"
}
```

Downloads and unpacks the release package, safely replaces running executables on disk, and sets `requires_restart: true`.

### Offline Package Upload
`POST /api/system/update/upload`

Accepts raw bytes of a `.zip` or `.exe` release file for air-gapped clinic deployments.

### Server Restart
`POST /api/system/restart`

Spawns the updated binary with the same arguments and gracefully shuts down the current process.


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
- AuditEvent
- Provenance
- Bundle

## Storage and Encryption

- SQLite backend with CRDT vector clocks
- AES-256-GCM per-record encryption (PRK)
- DMK-derived demo key for development (`DEMOMASTERKEY...`)
- Real deployments: derive DMK from hardware-backed keystore or Argon2id password

## Error Responses

Standard HTTP status codes with FHIR OperationOutcome:
```json
{
  "resourceType": "OperationOutcome",
  "issue": [{
    "severity": "error",
    "code": "security",
    "diagnostics": "ABAC policy denied access"
  }]
}
```
