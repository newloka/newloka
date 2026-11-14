---
name: amethyst-knowledge-graph
version: 1.0.0
description: "Operate, query, mutate, and verify the New Loka Knowledge Graph at D:\\GitRepos\\newloka\\knowledge_graph."
author: Amethyst Agent
license: GPL-3.0
platforms: [windows, linux, macos]
metadata:
  tags: [KnowledgeGraph, Memory, BiTemporal, GraphQuery, NewLoka]
---

# Amethyst Knowledge Graph Operations

Maintains, queries, mutates, and verifies the New Loka Knowledge Graph located at `D:\GitRepos\newloka\knowledge_graph/`.

The graph uses a bi-temporal, typed, multi-layer architecture with dual representation:
- Human-auditable markdown files in `nodes/*.md` and master topology in `index.md`.
- Machine-readable high-speed graph index in `graph.json`.

## Core Operations

### 1. Querying the Knowledge Graph
Before implementing or refactoring features, query the graph to understand established facts and dependencies:

```powershell
# List all nodes across layers
python knowledge_graph/tools/kg_tool.py list

# Inspect an entity and read its facts
python knowledge_graph/tools/kg_tool.py node fhir-clinical-models

# Search for specific clinical or technical facts across all nodes
python knowledge_graph/tools/kg_tool.py search "AES-256-GCM"

# Inspect inbound and outbound typed edges for an entity
python knowledge_graph/tools/kg_tool.py neighbors server-and-api
```

### 2. Recording New Facts (Bi-Temporal Protocol)
Every new fact added to a node file must include:
`- <Factual assertion>. [valid_from: YYYY-MM-DD, recorded: YYYY-MM-DD, source: <file|pr|issue|commit|test>]`

1. Open the relevant node file in `knowledge_graph/nodes/<entity>.md`.
2. Append the fact to `## Facts`.
3. Update the `Last updated` date in `knowledge_graph/index.md`.
4. Update `knowledge_graph/graph.json` if node metadata or summary changed.
5. If creating a brand new node file:
   - Create `knowledge_graph/nodes/<entity>.md` with `# Title`, `## Epistemic Metadata`, `## Facts`, and `## Superseded`.
   - Add a row in `knowledge_graph/index.md` under `## Nodes`.
   - Add at least one typed edge under `## Typed Edges` in `index.md` using `- src.md -[relation]-> tgt.md : Description`.
   - Add the node and edges to `knowledge_graph/graph.json`.

### 3. Invalidation Protocol (Never Delete History)
When a fact is superseded, contradicted, or upgraded:
1. Locate the stale fact in `## Facts`.
2. Move it to `## Superseded` in the same file.
3. Append `[superseded: YYYY-MM-DD, reason: <why it was superseded and what replaced it>]`.
4. Add the replacement fact under `## Facts` with fresh dates.

### 4. Mandatory Verification Gate
After ANY edit to `nodes/`, `index.md`, or `graph.json`, run the automated verification script:
```powershell
python knowledge_graph/tools/verify_graph.py
```
**Invariant:** Verification must report `0 errors, 0 warnings`. Never complete a turn with broken graph references, orphan nodes, or malformed bi-temporal tags.
