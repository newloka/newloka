# New Loka

Open source, local-first, polymorphic health data management system for patient care, clinical operations, and research.

Designed to adapt across deployment scales:
- **T0: Single-Person Clinic / Solo Clinician** — Zero-configuration local database, offline-first EMR, solo practice workflows.
- **T1: Small Clinic** — Peer-to-peer LAN mesh sync across local consultation rooms and reception.
- **T2: Rural Hospital** — Local server with intermittent connectivity and asynchronous delta sync.
- **T3: Multi-Department Hospital** — Role-based and ABAC department isolation with full CPOE/pharmacy workflows.
- **T4: Research Federation** — Privacy-preserving cross-institution query exchange and consent-governed data sharing.

---

## 📥 Precompiled Releases (No Build Tools Required)

Clinicians and administrators do **not** need to install Rust, Cargo, or any build tools to use New Loka. Download ready-to-run release archives directly from GitHub:

👉 **[Download Latest Precompiled Release](https://github.com/newloka/newloka/releases/latest)**

| Platform | Package | Contents |
| :--- | :--- | :--- |
| **Windows (x86_64)** | `newloka-windows-x86_64.zip` | `newloka-server.exe`, `newloka.exe`, `newloka-cli.exe` |
| **Linux (x86_64)** | `newloka-linux-x86_64.tar.gz` | `newloka-server`, `newloka`, `newloka-cli` |
| **macOS (Apple Silicon)** | `newloka-macos-arm64.tar.gz` | `newloka-server`, `newloka`, `newloka-cli` |

Simply unzip the archive and launch the server!

---

## Quick Start

### 1. Single-Person Clinic (Tier T0)

To launch New Loka configured for a solo practitioner or single-person clinic with a local encrypted database:

```bash
# Using the dedicated HTTP server binary
newloka-server.exe --tier T0 --db D:\Medical\Clinic\clinic.db --bind 127.0.0.1:8080

# Or using the root launcher
newloka.exe --tier T0 --db D:\Medical\Clinic\clinic.db --bind 127.0.0.1:8080

# Or using newloka-cli
newloka-cli.exe --tier T0 --db D:\Medical\Clinic\clinic.db serve --bind 127.0.0.1:8080
```

> **Automatic Path Handling**: Parent directories (e.g. `D:\Medical\Clinic`) are created automatically if they do not exist. Both Windows backslash paths and SQLite connection URIs (`sqlite:...`) are supported.

When launched with `--tier T0`:
- **Department**: Automatically set to `"Solo Practice"`.
- **Sync**: Mesh peer synchronization is disabled (`sync_enabled = false`) for zero unnecessary network traffic and complete data isolation.
- **Security**: Local AES-256-GCM encryption at rest with device master key (DMK).
- **Web UI**: Open `http://127.0.0.1:8080` in any modern browser to access the local web interface.

### 2. In-App 1-Click Updates & Maintenance

Small clinics can maintain and update New Loka directly from the browser without reinstalling or using the terminal:

1. Open **Settings** (`/static/#settings`) in the web interface.
2. Under the **Software Updates** card, click **🔄 Check for Updates**.
3. If an update is detected, click **⚡ 1-Click Update Now** — the server downloads the release package, safely swaps the binary on disk, and prompts you to restart.
4. Click **🔄 Restart Server** — the server reboots with the new version and reloads the browser automatically.
5. **Air-Gapped Clinics**: If your clinic computer is offline, use the **📦 Offline Update Package** section to select a pre-downloaded `.zip` or `.exe` file to apply updates immediately.

### 3. Practice Profile & Modular Feature Configurator

New Loka is fully modular. In the web interface, open **Settings** to access the **Practice Profile & Modular Features** panel:

- **🩺 Single-Person Clinic Preset**: Automatically activates solo clinical essentials (Patients, Consultations, SOAP Notes, Prescriptions, Lab Orders, Scheduling, Direct Billing, Encrypted Export) while cleanly hiding hospital-only clutter (inpatient wards, triage queues, shift handoffs, multi-department transfers).
- **🔬 Solo Specialist Preset**: Adds advanced diagnostic imaging and lab panels for specialized outpatient practices.
- **🏥 Inpatient Hospital Preset**: Full multi-department inpatient system with ward bed tracking and pharmacy dispensary queues.
- **Granular Feature Toggles**: 25 individual clinical switches can be toggled on or off to tailor the interface to your exact practice requirements.


---

## Command-Line Arguments

Both `newloka-server.exe` and `newloka.exe` support the following options:

| Flag | Short | Default | Description |
| :--- | :--- | :--- | :--- |
| `--tier <TIER>` | `-t` | `T1` | Deployment tier: `T0`, `T1`, `T2`, `T3`, `T4` |
| `--db <PATH>` | `-d` | `:memory:` | SQLite database file path (e.g. `D:\Medical\Clinic\clinic.db`) |
| `--bind <ADDR>` | `-b` | `127.0.0.1:8080` | HTTP server listening address |
| `--node-id <NAME>` | `-n` | `server-node` | Unique node identifier |

### Environment Variable Equivalents

All CLI arguments can alternatively be configured via environment variables:
- `NEWLOKA_TIER`: `T0`, `T1`, `T2`, `T3`, `T4`
- `NEWLOKA_DB_PATH`: Absolute or relative path to SQLite database
- `NEWLOKA_BIND_ADDR`: Host and port to bind (e.g. `127.0.0.1:8080`)
- `NEWLOKA_NODE_ID`: Node identifier
- `NEWLOKA_MASTER_KEY`: 64-char hex key (optional in demo mode)

---

## CLI Management Commands (`newloka-cli`)

For headless operations, batch scripts, and direct terminal management:

```bash
# Initialize node with derived master key
newloka-cli --db D:\Medical\Clinic\clinic.db init --password yourpassword

# Register a patient
newloka-cli --db D:\Medical\Clinic\clinic.db patient create --family Sharma --given Anita --gender female --birth-date 1988-04-12

# List patients
newloka-cli --db D:\Medical\Clinic\clinic.db patient list

# Query audit log
newloka-cli --db D:\Medical\Clinic\clinic.db audit --limit 20
```

---

## Project Structure

```
newloka/
  newloka_core/     - Rust core: FHIR R4 models, CRDT storage, AES-256-GCM encryption, ABAC policies, audit log
  newloka_server/   - Axum HTTP server: REST/FHIR endpoints, embedded SPA web interface, static assets
  newloka_cli/      - Command-line interface for terminal operations, init, and administration
  newloka_web/      - Built-in clinical web application (embedded into server binary via rust-embed)
  docs/             - Comprehensive architectural, API, and deployment documentation
```

---

## Documentation

- [Deployment Guide](file:///D:/New%20Loka/newloka/docs/DEPLOYMENT.md) — Step-by-step instructions for T0 through T4.
- [API Documentation](file:///D:/New%20Loka/newloka/docs/API.md) — FHIR R4 endpoints, authentication, and sync protocols.
- [Architecture Guide](file:///D:/New%20Loka/newloka/docs/ARCHITECTURE.md) — CRDT data structures, cryptographic envelope, and ABAC.
- [Security Guide](file:///D:/New%20Loka/newloka/docs/SECURITY.md) — Cryptographic threat model, zero-knowledge storage, and audit logs.
- [Developer Guide](file:///D:/New%20Loka/newloka/docs/DEVELOPER.md) — Local builds, testing, and contribution standards.

---

## 📦 Packaging Releases Locally

To build and package release archives locally:

```powershell
# Windows PowerShell
powershell -ExecutionPolicy Bypass -File scripts\package-release.ps1
```

This compiles release binaries and creates `dist/newloka-windows-x86_64.zip` accompanied by a cryptographic checksum file `dist/SHA256SUMS.txt`.

---

## 🔐 GitHub Releases & Git Push Credentials

To publish releases to [https://github.com/newloka/newloka](https://github.com/newloka/newloka):

### 1. Triggering an Automated GitHub Release
Pusing a version tag automatically triggers GitHub Actions to cross-compile Windows, Linux, and macOS binaries and publish them to GitHub Releases:

```bash
git tag v0.1.0
git push origin v0.1.0
```

Alternatively, navigate to **Actions** -> **Release** on GitHub and click **Run workflow**.

### 2. Setting Up Push Credentials in Git Keychain
If your terminal prompts for credentials or credentials aren't saved in your system keychain:

- **Git Credential Manager (Recommended on Windows)**:
  ```powershell
  git config --global credential.helper manager
  ```
  Next time you run `git push`, a browser prompt will appear to log in with GitHub and save credentials securely in the Windows Credential Manager.

- **GitHub Personal Access Token (PAT)**:
  1. Generate a token at [GitHub Settings → Developer Settings → Personal Access Tokens](https://github.com/settings/tokens) with `repo` and `workflow` scopes.
  2. Configure Git to store credentials:
     ```powershell
     git config --global credential.helper store
     ```
  3. When pushing, enter your GitHub username and paste the Personal Access Token (PAT) as the password.

- **SSH Keys**:
  ```powershell
  git remote set-url origin git@github.com:newloka/newloka.git
  ```

---

## License

GPL-3.0


