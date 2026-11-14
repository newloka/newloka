# Deployment Tiers and Polymorphic Scaling

## Epistemic Metadata
- **Entity**: Deployment Tiers (T0 through T4)
- **Layer**: Layer 0: Ontological & Foundational
- **Status**: SHIPPED
- **Domain**: Infrastructure Adaptation & Operational Scaling

## Facts
- Tier T0 (Single-Person Clinic / Solo Clinician): Operates on a phone, tablet, laptop, or desktop with a zero-configuration encrypted SQLite database (`clinic.db`). Mesh peer sync is disabled (`sync_enabled = false`) for zero unnecessary background traffic and complete data isolation; department defaults automatically to 'Solo Practice'. [valid_from: 2026-06-11, recorded: 2026-11-13, source: docs/DEPLOYMENT.md §1 & README.md]
- Tier T1 (Small Clinic Mesh): Operates across a local consultation room network and reception using local area network (LAN) mDNS/peer discovery and delta sync with vector clock conflict detection. No central internet server required. [valid_from: 2026-06-11, recorded: 2026-11-13, source: docs/DEPLOYMENT.md §2]
- Tier T2 (Rural Hospital): Deploys a local hospital server maintaining continuous operations during extended internet blackouts, with asynchronous delta sync to regional cloud relays when connectivity is intermittently restored. [valid_from: 2026-06-11, recorded: 2026-11-13, source: docs/DEPLOYMENT.md §3]
- Tier T3 (Multi-Department Hospital): Enforces institutional Attribute-Based Access Control (ABAC), departmental data silos (Radiology, Biochemistry, Pathology, Microbiology), shift handoff workflows, CPOE order lifecycles, and institutional LDAP/Active Directory identity integration. [valid_from: 2026-06-11, recorded: 2026-11-13, source: docs/DEPLOYMENT.md §4]
- Tier T3 Multi-Department Hospital Architecture (Shipped): Implements 9 clinical departments hierarchy, inter-department patient transfers with SBAR handover notes and bed allocation, executive hospital occupancy telemetry, and enterprise LDAP/AD directory authentication. [valid_from: 2026-11-14, recorded: 2026-11-14, source: newloka_core/src/department/mod.rs & newloka_server/src/lib.rs]
- Tier T4 Research Federation & Health Data Mesh (Shipped): Privacy-preserving multi-center query exchange, Laplace mechanism differential privacy with formal epsilon calibration, k-anonymity suppression (k >= 5) against difference attacks, de-identified cohort export, and dynamic FHIR R4 Consent directives. [valid_from: 2026-11-14, recorded: 2026-11-14, source: newloka_core/src/federation/mod.rs & newloka_core/src/consent/mod.rs]
- Enterprise Modular Presets Engine: Features a 22-role declarative preset registry (`PRESET_REGISTRY`) spanning Physicians & Specialists, Nursing Staff, Pharmacy & Diagnostics, and Administration & Operations across T0–T4. Automatically activates tailored viewports, Epic-style Storyboard context rails, active module subsets, and ABAC clinical data isolations on demand. [valid_from: 2026-11-14, recorded: 2026-11-14, source: newloka_web/js/config.js & docs/role_ui_matrix.md]
- Automatic Database Path Provisioning: All server launchers (`newloka-server`, `newloka`, `newloka-cli`) dynamically verify and automatically create parent directories on disk when provided any custom `--db <path>` or SQLite URI (`sqlite:...`). [valid_from: 2026-09-06, recorded: 2026-11-13, source: newloka_core/src/storage/mod.rs & README.md]

## Superseded
- Tier T4 (Research Federation): Speculative design replaced by complete implemented differential privacy and research mesh engine. [superseded: 2026-11-14, reason: T4 research federation engine, k-anonymity, and FHIR Consent directives fully implemented and shipped]
- Web UI Modular Configurator: Legacy 3-profile preset system replaced by comprehensive 22-role declarative preset engine. [superseded: 2026-11-14, reason: upgraded to comprehensive PRESET_REGISTRY with Epic Storyboard rails and dynamic pipeline binding]
