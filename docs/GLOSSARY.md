# New Loka Glossary

## ABAC
Attribute-Based Access Control. Policy system evaluating Subject x Resource x Action x Context.

## AES-256-GCM
Advanced Encryption Standard with 256-bit key in Galois/Counter Mode. Authenticated encryption used for all patient records.

## Argon2id
Memory-hard password hashing algorithm. Derives the Device Master Key from user credentials.

## Audit Trail
Append-only, cryptographically signed log of every system access and modification.

## CRDT
Conflict-free Replicated Data Type. Guarantees that edits made offline merge deterministically.

## DMK
Device Master Key. Top of the encryption key hierarchy. Derived from user PIN/password.

## Ed25519
Elliptic curve digital signature algorithm. Used to sign every audit entry.

## FHIR
Fast Healthcare Interoperability Resources. HL7 standard for health data exchange. R4 is version 4.

## LAN Mesh
Local network peer-to-peer topology. T1 clinics use this for record sharing without a central server.

## PRK
Patient Record Key. Encrypts individual FHIR resources. Wrapped by the DMK.

## Provenance
Record of who created, modified, or transferred a resource, when, and on which node.

## Soft Delete
Marking a record as deleted while retaining it for regulatory compliance. New Loka never hard-deletes clinical data.

## Vector Clock
Logical timestamp used to track causality across distributed nodes for conflict detection.
