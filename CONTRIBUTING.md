# Contributing to New Loka

Thank you for considering contributing to New Loka.

## Getting Started

1. Read docs/ARCHITECTURE.md to understand the system layers
2. Read docs/DEVELOPER.md for build and test instructions
3. Choose a phase that matches your expertise

## Contribution Workflow

1. Fork the repository
2. Create a feature branch
3. Make your changes with tests
4. Run cargo fmt and cargo clippy
5. Run the full test suite
6. Submit a pull request

## Phase-Based Contribution

### Phase 1 - Core Offline
Focus areas:
- FHIR resource model completeness
- Storage engine performance
- Mobile/desktop UI shells
- Offline authentication robustness

### Phase 2 - Mesh Sync
Focus areas:
- Peer discovery protocols
- Delta sync efficiency
- Conflict resolution UX
- Network partition handling

### Phase 3 - Institutional Control
Focus areas:
- ABAC policy engine
- Department silo enforcement
- Emergency override workflows
- LDAP/AD integration

### Phase 4 - Local Intelligence
Focus areas:
- OCR pipeline integration
- Speech-to-text accuracy
- Structured extraction quality
- Review queue UX

### Phase 5 - Federation
Focus areas:
- Consent management
- Privacy-preserving queries
- ABDM compliance
- IHE profile conformance

## Code Standards

- All public APIs must have rustdoc comments
- Security-sensitive code requires audit logging
- ABAC policies must be data-layer enforced
- No patient data sent to external APIs by default
- Soft delete only - never hard delete clinical records
- Every write must be attributable

## Security

If you discover a security vulnerability, please email security@newloka.org instead of opening a public issue.

## License

By contributing, you agree that your contributions will be licensed under the GPL-3.0 license.
