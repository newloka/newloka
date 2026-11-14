# New Loka

[![CI](https://github.com/newloka/newloka/actions/workflows/ci.yml/badge.svg)](https://github.com/newloka/newloka/actions/workflows/ci.yml)
[![Release](https://github.com/newloka/newloka/actions/workflows/release.yml/badge.svg)](https://github.com/newloka/newloka/actions/workflows/release.yml)
[![License: GPL-3.0](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

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
newloka-server.exe --tier T0 --db ./clinic.db --bind 127.0.0.1:8080

# Or using the root launcher
newloka.exe --tier T0 --db ./clinic.db --bind 127.0.0.1:8080

# Or using newloka-cli
newloka-cli.exe --tier T0 --db ./clinic.db serve --bind 127.0.0.1:8080
```

> **Automatic Path Handling**: The `--db` argument accepts any valid relative path, absolute path, or SQLite URI (`sqlite:...`). If parent directories do not already exist, New Loka creates them automatically on startup.

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
| `--db <PATH>` | `-d` | `:memory:` | SQLite database file path (e.g. `./clinic.db`, `/path/to/clinic.db`) |
| `--bind <ADDR>` | `-b` | `127.0.0.1:8080` | HTTP server listening address |
| `--node-id <NAME>` | `-n` | `server-node` | Unique node identifier |

### Environment Variable Equivalents

All CLI arguments can alternatively be configured via environment variables:
- `NEWLOKA_TIER`: `T0`, `T1`, `T2`, `T3`, `T4`
- `NEWLOKA_DB_PATH`: Absolute or relative path to SQLite database (or `:memory:`)
- `NEWLOKA_BIND_ADDR`: Host and port to bind (e.g. `127.0.0.1:8080`)
- `NEWLOKA_NODE_ID`: Node identifier
- `NEWLOKA_MASTER_KEY`: 64-char hex key (optional in demo mode)

---

## CLI Management Commands (`newloka-cli`)

For headless operations, batch scripts, and direct terminal management:

```bash
# Initialize node with derived master key
newloka-cli --db ./clinic.db init --password yourpassword

# Register a patient
newloka-cli --db ./clinic.db patient create --family Sharma --given Anita --gender female --birth-date 1988-04-12

# List patients
newloka-cli --db ./clinic.db patient list

# Query audit log
newloka-cli --db ./clinic.db audit --limit 20
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

- [Deployment Guide](docs/DEPLOYMENT.md) — Step-by-step operational setup for T0 (Solo Clinic) through T4 (Research Federation).
- [API Documentation](docs/API.md) — FHIR R4 endpoints, enterprise authentication, department transfers, and research mesh APIs.
- [Architecture Guide](docs/ARCHITECTURE.md) — Core CRDT data structures, cryptographic envelope, and ABAC engine.
- [Security Guide](docs/SECURITY.md) — Cryptographic threat model, zero-knowledge storage, and audit logs.
- [Developer Guide](docs/DEVELOPER.md) — Local builds, workspace testing, and contribution standards.
- [Vision & Requirements Gap Analysis](docs/MISSING_COMPONENTS_AND_REQUIREMENTS.md) — Living tracking specification comparing implementation against full 11-phase roadmap.
- [Master Role & UI Matrix](docs/role_ui_matrix.md) — Exhaustive 22-role clinical persona taxonomy and ABAC clearance boundaries.

---

## 🛠️ Build Pipeline & Developer Setup

New Loka is designed with a **zero-dependency runtime architecture**: the entire single-page web interface (`newloka_web/`) is embedded directly into the Rust server executable at compile time via `rust-embed`. The compiled binary runs standalone with zero external web servers, runtime interpreters, or Node.js dependencies.

```
┌────────────────────────────────────────────────────────────────────────┐
│ Workspace Build Pipeline Overview                                      │
├────────────────────────────────────────────────────────────────────────┤
│ 1. rust-embed embeds newloka_web (HTML, CSS, JS) into newloka_server  │
│ 2. newloka_core compiles FHIR R4, crypto, SQLite storage, ABAC, consent│
│ 3. newloka_server, newloka_cli, and newloka binaries are generated    │
│ 4. cargo test --workspace runs 51 unit, integration, and server tests │
│ 5. Automated CI/CD builds release matrices (Win/Linux/macOS) on push  │
└────────────────────────────────────────────────────────────────────────┘
```

### 1. Prerequisites
- **Rust & Cargo**: Version 1.75+ (install via [rustup.rs](https://rustup.rs/)).
- **SQLite Dev Libraries** (Linux only):
  ```bash
  sudo apt-get update && sudo apt-get install -y libsqlite3-dev pkg-config libssl-dev
  ```

### 2. Fast Compilation Check
```bash
cargo check
```

### 3. Build Binaries
```bash
# Debug build (faster compilation)
cargo build

# Optimized production release build
cargo build --release
```
Compiled binaries are emitted to `target/release/`:
- `newloka-server` (or `newloka-server.exe`) — Axum HTTP + FHIR API server with embedded web UI.
- `newloka-cli` (or `newloka-cli.exe`) — Headless CLI administration tool.
- `newloka` (or `newloka.exe`) — Unified launcher binary.

### 4. Run the Full Test Suite
Validate the complete workspace across all 51 unit, integration, and server tests:
```bash
cargo test --workspace
```

### 5. Launch Local Development Server
```bash
# Run server directly with embedded web interface
cargo run -p newloka_server

# Or specify a custom database and port
cargo run -p newloka_server -- --db ./clinic.db --bind 127.0.0.1:8080
```
Open `http://127.0.0.1:8080/` in your browser.

---

## 📦 Packaging Releases Locally

To build and package signed release archives locally:

```powershell
# Windows PowerShell
powershell -ExecutionPolicy Bypass -File scripts\package-release.ps1
```

This compiles release binaries and creates `dist/newloka-windows-x86_64.zip` accompanied by a cryptographic checksum file `dist/SHA256SUMS.txt`.

---

## 🚀 Automated CI/CD Pipeline (GitHub Actions)

New Loka includes automated multi-platform CI/CD workflows under `.github/workflows/`:

1. **Continuous Integration (`ci.yml`)**:
   - Triggers on every push and pull request to `main`.
   - Runs `cargo fmt -- --check`, `cargo clippy`, and `cargo test --verbose`.
   - Cross-compiles release binaries across **Ubuntu (Linux)**, **Windows**, and **macOS**.
2. **Automated Release Publisher (`release.yml`)**:
   - Triggers automatically when a version tag (`v*`) is pushed to GitHub, or via manual workflow dispatch.
   - Compiles optimized production archives:
     - `newloka-windows-x86_64.zip`
     - `newloka-linux-x86_64.tar.gz`
     - `newloka-macos-arm64.tar.gz`
   - Generates SHA-256 checksums and automatically publishes the draft/release on GitHub Releases.

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


