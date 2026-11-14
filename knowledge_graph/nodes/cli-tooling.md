# Command-Line Interface and Administrative Tooling

## Epistemic Metadata
- **Entity**: Command-Line Interface (`newloka_cli`)
- **Layer**: Layer 2: Operational & Functional
- **Status**: SHIPPED
- **Domain**: Headless Administration, Batch Workflows & Terminal Operations

## Facts
- Headless Clinical Tooling (`newloka-cli`): Engineered using Clap v4 in `newloka_cli/src/main.rs`. Provides terminal management for servers, batch automation, and air-gapped system maintenance. [valid_from: 2026-06-11, recorded: 2026-11-13, source: newloka_cli/src/main.rs]
- Direct Database & Crypto Integration: Uses `newloka_core::storage::StorageEngine` with real SQLite connections, supporting password-derived Argon2id master key derivation or demo keys. [valid_from: 2026-10-15, recorded: 2026-11-13, source: newloka_cli/src/main.rs & git diff]
- Command Catalog:
  1. `init`: Initialize a new local node, create database directory, derive master key, generate Ed25519 node identity, and stamp genesis audit block.
  2. `serve`: Launch the embedded HTTP server directly from the CLI binary, binding to `--bind` (e.g. `127.0.0.1:8080`) with tier configuration.
  3. `patient`: Subcommands `create`, `list`, `get`, `delete` for command-line clinical demographic management.
  4. `observation`: Subcommands `record`, `list` for vital signs and lab metric entry.
  5. `encounter`: Subcommands `create`, `list` for visit tracking.
  6. `audit`: Query signed audit records with `--limit` and integrity validation.
  7. `export`: Export patient charts as encrypted or unencrypted FHIR R4 JSON bundles.
  8. `sync`: Trigger manual peer discovery and delta sync.
  9. `department`: Subcommands `list`, `transfer` for hospital hierarchy and SBAR clinical handovers.
  10. `consent`: Subcommands `list`, `grant`, `revoke`, `verify` for patient privacy directives.
  11. `research`: Subcommands `query`, `export` for differential privacy analytics and de-identified data.
  12. `federation`: Subcommands `nodes`, `ping` for health data mesh network management.
  [valid_from: 2026-11-14, recorded: 2026-11-14, source: newloka_cli/src/main.rs]
- Root Launcher Integration: The root workspace binary `newloka.exe` (`src/main.rs`) can invoke the CLI or server depending on passed arguments, providing a single convenient multi-tool binary for clinicians. [valid_from: 2026-06-11, recorded: 2026-11-13, source: src/main.rs & README.md]

## Superseded
(none)
