---
name: newloka-release-ops
version: 1.0.0
description: "Release engineering, cross-platform packaging, in-app updater testing, GitHub releases, and marketing site deployment for New Loka."
author: Amethyst Agent
license: GPL-3.0
platforms: [windows, linux, macos]
metadata:
  tags: [Release, Packaging, CI_CD, Updater, Firebase, GitHub]
---

# New Loka Release Operations & Site Deployment

Guides cross-platform packaging, release publication, self-updater validation, and synchronization with the public marketing site (`newloka-site`).

## 1. Local Release Packaging

To build optimized production release binaries and generate the distribution archive on Windows:
```powershell
powershell -ExecutionPolicy Bypass -File scripts\package-release.ps1
```
This compiles `newloka`, `newloka_server`, and `newloka_cli` with release optimizations, bundles them into `dist/newloka-windows-x86_64.zip`, and generates cryptographic checksums in `dist/SHA256SUMS.txt`.

## 2. In-App 1-Click Updater Verification

The updater (`newloka_server::updater`) enables in-browser updates without terminal access:
- **API Endpoints:**
  - `GET /api/system/version` -> returns current runtime version, OS, arch, and commit SHA.
  - `GET /api/system/update/check` -> queries GitHub Releases API for newer semantic version tags.
  - `POST /api/system/update/apply` -> streams release asset, unzips to temporary directory, renames active executable to `.old`, copies new binary into place.
  - `POST /api/system/update/upload` -> handles air-gapped multipart `.zip` upload for offline clinics.
  - `POST /api/system/restart` -> spawns replacement process in detached mode and terminates current process.

To test the updater endpoints locally:
```powershell
# Query version
Invoke-RestMethod -Uri "http://127.0.0.1:8080/api/system/version"
```

## 3. GitHub Actions CI/CD & Releases

To publish an official automated release:
1. Ensure all tests pass: `cargo test --workspace`.
2. Update versions in `Cargo.toml` and workspace crates if bumping version.
3. Commit and tag:
   ```bash
   git tag v0.2.0
   git push origin v0.2.0
   ```
4. GitHub Actions workflow (`.github/workflows/release.yml`) cross-compiles Windows, Linux, and macOS release bundles and publishes the GitHub Release.

## 4. Public Website Deployment (`newloka-site`)

The public website lives at `D:\GitRepos\site` (`https://github.com/newloka/site.git`) and deploys to Firebase Hosting (`new-loka` project / `https://newloka.org`):
```powershell
cd D:\GitRepos\site

# Check deployment status
firebase hosting:channel:list

# Deploy to production hosting
firebase deploy --only hosting
```
Always verify that release download links on `index.html` and `docs.html` match the latest GitHub release tag.
