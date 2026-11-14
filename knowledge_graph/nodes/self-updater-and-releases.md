# In-App Self-Updater and Release Packaging

## Epistemic Metadata
- **Entity**: Automated Updater & Cross-Platform Releases
- **Layer**: Layer 2: Operational & Functional
- **Status**: SHIPPED
- **Domain**: Release Engineering, Zero-Toolchain Updates & Packaging

## Facts
- Zero Build-Tool Deployment: Clinicians and healthcare administrators do not require Rust, Cargo, or development environments; ready-to-run releases are published to GitHub Releases (`https://github.com/newloka/newloka/releases/latest`). [valid_from: 2026-06-11, recorded: 2026-11-13, source: README.md]
- Cross-Platform Release Targets: Automated GitHub Actions CI/CD workflows cross-compile and package Windows x86_64 (`newloka-windows-x86_64.zip`), Linux x86_64 (`newloka-linux-x86_64.tar.gz`), and macOS Apple Silicon (`newloka-macos-arm64.tar.gz`). [valid_from: 2026-06-11, recorded: 2026-11-13, source: README.md & .github/workflows/]
- In-App 1-Click Browser Updates: Under Settings (`/static/#settings`), users can click Check for Updates to query GitHub API, preview release notes, click 1-Click Update Now to stream and unpack new binaries, and click Restart Server to reboot the process automatically. [valid_from: 2026-09-06, recorded: 2026-11-13, source: newloka_server/src/updater.rs & README.md]
- Atomic Rename-and-Replace: The updater renames the active running executable to a `.old` backup, drops the new executable in place, and triggers background process spawn and restart without file locking conflicts. [valid_from: 2026-09-06, recorded: 2026-11-13, source: newloka_server/src/updater.rs]
- Air-Gapped Offline Update Invariant: For rural or air-gapped clinics lacking internet access, the web interface supports uploading a pre-downloaded archive via Offline Update Package, applying the binary update without external network access. [valid_from: 2026-09-06, recorded: 2026-11-13, source: newloka_server/src/updater.rs & docs/DEPLOYMENT.md]
- Local Packaging Automation: `scripts/package-release.ps1` compiles optimized release binaries and generates `dist/newloka-windows-x86_64.zip` alongside cryptographic checksums in `dist/SHA256SUMS.txt`. [valid_from: 2026-09-06, recorded: 2026-11-13, source: scripts/package-release.ps1 & README.md]

## Superseded
(none)
