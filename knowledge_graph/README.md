# New Loka Knowledge Graph

A bi-temporal, typed, multi-layered knowledge graph for the New Loka project.

## Key Advancements over Prior File-Based Graphs

1. **Multi-Layered Ontology**: Partitioned into 6 semantic layers (Ontological, Structural, Operational, Roadmap, External Web, Sovereign Agentics).
2. **Strictly Typed Relations**: Replaces informal string connections with standardized predicates (`implements`, `enforces`, `encrypts_with`, `serves`, `syncs_with`, `governs`, `tracks_in`, `advances_to`, `supersedes`, `specifies`, `persists_in`, `orchestrates`, `secures`, `extends`, `bridges_to`).
3. **Bi-Temporal Fact Life-Cycle**: Every factual predicate records both `valid_from` (world occurrence) and `recorded` (transaction ingestion), with non-destructive invalidation into `## Superseded`.
4. **Epistemic Status Tracking**: Every entity node is tagged with its empirical verification state (`SHIPPED`, `ACTIVE`, `ROADMAP`, `RESEARCH`, `SUPERSEDED`).
5. **Dual Representation**:
   - Human-auditable markdown files in `nodes/*.md` and topological index in `index.md`.
   - High-speed machine-readable index in `graph.json` for deterministic graph queries and neighborhood traversals.
6. **Automated Verification & CLI**:
   - `python tools/verify_graph.py` — runs automated referential and structural integrity tests.
   - `python tools/kg_tool.py` — CLI for search, listing, node inspection, and neighborhood path extraction.

## CLI Usage

```powershell
# Verify entire graph consistency (asserts 0 errors, 0 warnings)
python knowledge_graph/tools/verify_graph.py

# List all nodes by layer and epistemic status
python knowledge_graph/tools/kg_tool.py list

# Inspect a node
python knowledge_graph/tools/kg_tool.py node fhir-clinical-models

# Query facts across the entire knowledge graph
python knowledge_graph/tools/kg_tool.py search "AES-256-GCM"

# Traverse inbound and outbound edges for an entity
python knowledge_graph/tools/kg_tool.py neighbors server-and-api
```
