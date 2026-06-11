# New Loka Deployment Guide

## Tier-Specific Deployment

### T0 - Single Clinician (Phone / Laptop)

Requirements:
  - Android 10+ or iOS 14+ or desktop OS
  - 2GB RAM minimum
  - 500MB storage minimum

Setup:
  1. Install New Loka app
  2. Set PIN or biometric lock
  3. Device Master Key derived from PIN via Argon2id
  4. All data stored in encrypted SQLite on device

No server required. Works entirely offline.

### T1 - Small Clinic (LAN Mesh)

Requirements:
  - Local network (WiFi or Ethernet)
  - One optional hub device (RPi4+, old PC)
  - New Loka on each provider device

Setup:
  1. Install New Loka on all devices
  2. Designate one device as hub (optional)
  3. Bootstrap trust: admin approves each node
  4. mDNS/Bonjour peer discovery on LAN
  5. Delta sync runs automatically when peers visible

Mesh behavior: any peer can sync with any peer. No single point of failure.

### T2 - Rural Hospital (Local Server + Intermittent Cloud)

Requirements:
  - Local server (min 4GB RAM, 100GB storage)
  - Intermittent internet connection
  - UPS recommended

Setup:
  1. Deploy newloka-server on local server
  2. Configure department list
  3. Set up local backup schedule
  4. Configure async cloud sync (optional)
  5. ABAC policies per department

Offline operation continues during network outages. Sync queues and retries.

### T3 - Multi-Department Hospital

Requirements:
  - Kubernetes cluster or dedicated servers
  - HAPI FHIR server deployment
  - LDAP/Active Directory integration
  - mTLS certificates for all nodes

Setup:
  1. Deploy Kong API Gateway
  2. Deploy HAPI FHIR server (clustered)
  3. Deploy newloka-server instances per department
  4. Configure Cedar ABAC policy files
  5. Integrate LDAP for user provisioning
  6. Enable inter-department transfer workflows
  7. Configure HL7 v2 listener for legacy HIS

Silo enforcement: ABAC policies evaluated at data layer before any query executes.

### T4 - Research Federation

Requirements:
  - Institutional trust chains established
  - Differential privacy infrastructure
  - Federated learning nodes (optional)
  - ABDM/ABHA integration (India)

Setup:
  1. Deploy institutional nodes
  2. Establish cross-institution trust ceremonies
  3. Configure consent management per patient
  4. Enable privacy-preserving query endpoints
  5. Register with NHA sandbox (India)
  6. Implement IHE profiles (PDQm, MHD, PIXm)

Research access never exposes raw patient data by default.

## Environment Variables

NEWLOKA_DB_PATH          - SQLite database path
NEWLOKA_NODE_ID          - Node identity display name
NEWLOKA_TIER             - Deployment tier (t0,t1,t2,t3,t4)
NEWLOKA_BIND_ADDR        - Server bind address (T1+)
NEWLOKA_LOG_LEVEL        - tracing log level
NEWLOKA_AUDIT_MODE       - strict or permissive

## Docker Deployment (T2-T3)

# Build image
docker build -t newloka:latest .

# Run server
docker run -d -p 8080:8080 -v newloka-data:/data newloka:latest

## Backup and Recovery

T0:
  - Encrypted export to USB/SD card
  - QR code transfer to new device

T1+:
  - Automated SQLite backup
  - Encrypted snapshot to NAS/cloud
  - Full FHIR Bundle export per patient

## Monitoring

Health endpoint: GET /health
Metrics: Prometheus-compatible (T3+)
Alerts: Audit escalation patterns, sync failures
