# New Loka Test Suite

Tests are organized within each crate's tests/ directory:

- newloka_core/tests/unit_tests.rs        - Module-level unit tests
- newloka_core/tests/integration_tests.rs - End-to-end integration tests

## Running Tests

From the workspace root:

cargo test                           # All tests across workspace
cargo test -p newloka_core         # Core library tests only
cargo test --test unit_tests         # Unit tests only
cargo test --test integration_tests  # Integration tests only
cargo test -- --nocapture            # With output

## Test Coverage

- FHIR serialization and resource models
- AES-256-GCM encryption and key hierarchy
- SQLite storage with encrypted roundtrip
- ABAC policy evaluation across all tiers
- Vector clock sync and conflict detection
- Audit chain integrity and signature verification
- Full patient registration and chart workflows
