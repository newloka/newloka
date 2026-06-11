# New Loka FAQ

## What is New Loka?

New Loka is an open source, local-first health data management system. It stores patient records as encrypted FHIR R4 resources and works offline from a single phone up to multi-hospital deployments.

## Why local-first?

Because healthcare happens where connectivity does not. A rural clinic, a doctor's phone in a remote village, or a hospital during a network outage must keep working.

## What is FHIR R4?

HL7 Fast Healthcare Interoperability Resources version 4. It is the international standard for exchanging healthcare information electronically. New Loka uses it as the internal data model.

## Can it work without internet?

Yes. T0 (single clinician) works entirely offline. T1 (small clinic) syncs over local LAN without internet. Higher tiers sync to cloud when available but continue operating offline.

## How is data encrypted?

Every patient record is encrypted with AES-256-GCM using a per-record key. That key is wrapped by a Device Master Key derived from your PIN/password via Argon2id.

## What if I forget my password?

There is no backdoor. In T0, losing your password means losing access to data. Institutional deployments (T2+) can configure key escrow with split-key ceremonies.

## How does sync work?

Devices discover each other via mDNS/Bonjour on the same network. They exchange only changed records since last sync using vector clocks. Conflicts are detected and surfaced explicitly.

## What about department silos?

T3 deployments enforce department boundaries at the data layer using ABAC (Attribute-Based Access Control). Even if you bypass the UI, the core will deny unauthorized access.

## Is patient data sent to AI services?

No, not by default. AI processing (OCR, speech-to-text) runs locally or on trusted institutional infrastructure. AI output requires human review before commit.

## What compliance standards?

- DISHA / IT Act (India)
- HIPAA (US)
- GDPR (EU, for research)
- HL7 FHIR R4
- SNOMED CT, LOINC, ICD-10/11

## How do I contribute?

See CONTRIBUTING.md. Start with Phase 1 if new. Work within the tier abstractions. Security changes require audit module updates.
