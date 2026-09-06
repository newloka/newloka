# New Loka Deployment Guide

## Deployment Tiers Overview

New Loka is a polymorphic, modular clinical health data platform. It is engineered to scale from an isolated single-practitioner clinic running on a desktop or laptop to a nationwide research federation.

---

## Tier T0 — Single-Person Clinic / Solo Clinician

Tier T0 is designed for solo doctors, private clinics, outpatient practices, and field practitioners operating independently without IT staff or network servers.

### Architectural Defaults for T0
- **Department**: Automatically defaults to `"Solo Practice"`.
- **Sync**: Mesh peer sync is disabled (`sync_enabled = false`), ensuring zero background network chatter, zero port exposure, and 100% offline data confinement.
- **Storage**: Encrypted local SQLite database at rest with AES-256-GCM.
- **Authentication**: Local-first offline authentication with device-derived master keys.

### Running a Single-Person Clinic Server

Execute with the desired tier and custom database path:

```powershell
# Using newloka-server
newloka-server.exe --tier T0 --db ./clinic.db --bind 127.0.0.1:8080

# Using newloka (root launcher)
newloka.exe --tier T0 --db ./clinic.db --bind 127.0.0.1:8080

# Using newloka-cli
newloka-cli.exe --tier T0 --db ./clinic.db serve --bind 127.0.0.1:8080
```

#### Automatic Directory & Path Creation
When specifying `--db <path>`, the engine checks if the target directory exists. If not, it creates all parent directories automatically on startup with proper operating system permissions. Both raw filesystem paths (e.g. `./clinic.db`, `C:\ClinicData\clinic.db`, `/var/lib/newloka/clinic.db`) and SQLite URIs (`sqlite:clinic.db?mode=rwc`) are supported.

#### Accessing the Web Interface
Once started, open any web browser to:
```
http://127.0.0.1:8080
```
The browser loads the embedded single-page application directly from the binary.

### Modular Practice Profiles in Web UI
Inside the web app under **Settings** (`/static/#settings`), clinicians can configure the **Practice Profile & Modular Features**:

1. **🩺 Single-Person Clinic**: 1-click preset tailored for solo general practice. It automatically enables:
   - Patient Registration & Search
   - Consultations & Clinical Encounters
   - Vitals & SOAP Progress Notes
   - Direct e-Prescriptions & Medication Dispense
   - Solo Diagnostic / Lab Orders
   - Patient Scheduling & Appointments
   - Direct Invoicing & Billing
   - Encrypted FHIR Backup & Export
   - Audit Trail & Emergency Override Access
   
   It automatically hides hospital overhead:
   - Inpatient Wards & Bed Tracking
   - Emergency Triage Queues
   - Shift Handoff Reports
   - Multi-Department Patient Transfers
   - Mesh Peer Sync & Federation Exporters

2. **🔬 Solo Specialist**: Tailored for outpatient specialists (Cardiology, Endocrinology, Pathology) with expanded diagnostic panels, custom observation codes, and referral management.

3. **🏥 Inpatient Hospital**: Activates full inpatient hospital capabilities across all 25 modules.

4. **Granular Feature Toggles**: Any of the 25 individual modules can be independently checked or unchecked to customize the clinical workflow to the exact needs of your clinic.

---

## In-App 1-Click Updates & Maintenance

Small setups and solo clinics do not need to compile code or manage terminal commands to keep their system up to date.

### Browser-Driven 1-Click Updates
1. Navigate to **Settings** (`/static/#settings`).
2. In the **Software Updates** panel:
   - Click **🔄 Check for Updates** to query GitHub for new releases.
   - If a newer version exists, release notes and download details appear.
   - Click **⚡ 1-Click Update Now** — the server streams the release asset, extracts the updated binaries, and safely replaces the existing executable using the atomic rename-and-replace strategy.
   - Click **🔄 Restart Server to Activate** — the server reboots in the background, health is polled automatically, and the browser interface refreshes with the new version.

### Air-Gapped / Offline Clinics
For clinic workstations without continuous internet access:
1. Download `newloka-windows-x86_64.zip` on any internet-connected device from [GitHub Releases](https://github.com/newloka/newloka/releases/latest).
2. Transfer it to the clinic workstation via USB flash drive.
3. In **Settings** -> **📦 Offline Update Package**, select the file and click **Apply Offline Package**.
4. Restart the server with one click.

---

## 📥 Prebuilt Releases & Automated GitHub CI/CD

Precompiled, zero-dependency release archives are published automatically on GitHub:
- **Windows**: `newloka-windows-x86_64.zip`
- **Linux**: `newloka-linux-x86_64.tar.gz`
- **macOS**: `newloka-macos-arm64.tar.gz`

### Triggering a Release
Pushing any version tag triggers the GitHub Actions workflow (`.github/workflows/release.yml`) to build all targets and publish a GitHub Release with attached binaries and SHA256 checksums:
```bash
git tag v0.1.0
git push origin v0.1.0
```

---

## Command-Line Arguments Reference


The server binaries (`newloka-server.exe` and `newloka.exe`) accept the following command-line flags:

| Flag | Short | Default | Description |
| :--- | :--- | :--- | :--- |
| `--tier <TIER>` | `-t` | `T1` | Target tier: `T0`, `T1`, `T2`, `T3`, `T4` |
| `--db <PATH>` | `-d` | `:memory:` | Database file path (e.g. `./clinic.db`, `/path/to/clinic.db`) |
| `--bind <ADDR>` | `-b` | `127.0.0.1:8080` | IP and port to listen on |
| `--node-id <NAME>` | `-n` | `server-node` | Unique identifier for the instance |

### Environment Variables

Command-line flags take precedence over environment variables:

| Variable | Default | Description |
| :--- | :--- | :--- |
| `NEWLOKA_TIER` | `T1` | Deployment tier (`T0`, `T1`, `T2`, etc.) |
| `NEWLOKA_DB_PATH` | `:memory:` | SQLite path or connection URI |
| `NEWLOKA_BIND_ADDR` | `127.0.0.1:8080` | Server bind host and port |
| `NEWLOKA_NODE_ID` | `server-node` | Local node identity |
| `NEWLOKA_DEPARTMENT` | `"Solo Practice"` for T0, `"default"` for T1+ | Active department name |
| `NEWLOKA_SYNC_ENABLED` | `false` for T0, `true` for T1+ | Enable/disable peer sync |
| `NEWLOKA_MASTER_KEY` | Hex string | 32-byte master encryption key |

---

## Tier T1 — Small Clinic (LAN Mesh)

Requirements:
- Local network (Wi-Fi or Ethernet)
- Provider devices (laptops, desktop PCs, tablets)
- Optional always-on local device acting as a hub (e.g. mini PC or Raspberry Pi 4)

Setup:
1. Start New Loka on each clinic device with `--tier T1`:
   ```bash
   newloka-server.exe --tier T1 --db C:\Clinic\data.db --bind 0.0.0.0:8080
   ```
2. Nodes discover each other via mDNS / LAN discovery.
3. Delta synchronization propagates patient updates and encounters across consultation rooms and front-desk reception deterministically using vector clocks.

---

## Tier T2 — Rural Hospital (Local Server + Intermittent Cloud)

Requirements:
- Dedicated local server (min 4GB RAM, 100GB SSD)
- Intermittent or satellite internet connection
- Uninterruptible Power Supply (UPS) recommended

Setup:
1. Deploy `newloka-server` as a Windows Service or systemd daemon:
   ```bash
   newloka-server.exe --tier T2 --db /var/lib/newloka/hospital.db --bind 0.0.0.0:8080
   ```
2. Configure departmental access (Outpatient, Inpatient, Pharmacy, Emergency).
3. Set up automated encrypted SQLite snapshot backups.
4. During internet outages, all clinical care and charting continues offline without interruption. Sync queues updates and merges automatically upon reconnection.

---

## Tier T3 — Multi-Department Hospital

Requirements:
- High-availability cluster or server hardware
- ABAC department silo policies
- Enterprise identity (LDAP / Active Directory)
- mTLS certificates

Setup:
1. Run `newloka-server` per department with strict ABAC policies:
   ```bash
   newloka-server.exe --tier T3 --db /data/cardiology.db --node-id cardio-node
   ```
2. Department isolation is strictly enforced at the data layer before queries execute.
3. Inter-department patient transfers require cryptographically recorded consent or clinician sign-off.

---

## Tier T4 — Research Federation

Requirements:
- Institutional federated trust certificates
- Differential privacy mechanisms
- Consent-governed patient exchange (e.g. ABDM/ABHA, IHE profiles)

Setup:
1. Deploy institutional federation gateway with `--tier T4`.
2. Cross-institutional queries execute against aggregated or differentially-private views. Raw clinical charts are never exposed without explicit patient consent.

---

## Backup & Disaster Recovery

### For Single-Person Clinic (T0):
- **Encrypted Export**: In the web UI, navigate to **Settings** -> **Export Encrypted Backup** to save a password-encrypted FHIR bundle or SQLite snapshot to a USB flash drive or external SSD.
- **Direct File Copy**: When the server is stopped, simply copy your database file (e.g. `./clinic.db` or your configured `--db` path) to your secure backup location. The database file is encrypted at rest.

### For T1–T3:
- Automated scheduled SQLite VACUUM INTO snapshots.
- Offsite encrypted snapshots with retention policies.

---

## Health & Diagnostics

Verify that the clinic server is operational:

```powershell
Invoke-RestMethod -Uri http://127.0.0.1:8080/health
```

Expected response:
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "node_id": "server-node",
  "tier": "T0"
}
```

