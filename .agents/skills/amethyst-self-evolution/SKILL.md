---
name: amethyst-self-evolution
version: 1.0.0
description: "Autonomous self-evolution and learning loop for Amethyst: dynamically codifies runbooks, synthesizes skills, updates memory, and retains problem-solving patterns."
author: Amethyst Agent
license: GPL-3.0
platforms: [windows, linux, macos]
metadata:
  tags: [SelfEvolution, Learning, SkillAuthoring, Memory, AutonomousAgent]
---

# Amethyst Autonomous Self-Evolution & Learning Loop

Governs the continuous learning, self-evolution, and capability expansion of **Amethyst** (`Amethyst (newloka-core)`), modeled on the autonomous self-evolution doctrine of sovereign agents (such as Eidolon).

## The Learning Mechanism: From Problem to Permanent Skill

Whenever Amethyst encounters a novel engineering problem, architectural refactor, debugging challenge, or workflow requirement:
1. **Never perform fragile manual workarounds repeatedly.**
2. **Diagnose and resolve the issue empirically** (verified by real tool outputs, tests, or compiler logs).
3. **Crystallize the solution into permanent agentic assets** in the same turn without waiting for human prompts:
   - **Skills:** Modify an existing skill or synthesize a net-new skill (`SKILL.md`) in `.agents/skills/<name>/SKILL.md` and `~/.gemini/config/skills/<name>/SKILL.md`.
   - **Knowledge Graph:** Record new facts or supersede outdated facts in `knowledge_graph/nodes/<entity>.md`, updating `index.md` and `graph.json`.
   - **Lean Memory:** Update `.agents/memories/MEMORY.md` with a concise §-separated pointer row.

---

## The 4-Step Self-Evolution Lifecycle

```
[Novel Problem / Bug / Workflow]
               │
               ▼
   [1. Empirical Resolution]
   - Diagnose root cause (compiler errors, test failures, clinical invariants)
   - Implement minimal, robust fix
   - Verify via `cargo test --workspace` or tool output
               │
               ▼
   [2. Skill Synthesis / Mutation]
   - Is this an isolated fix or a repeatable procedure?
   - If repeatable: write/update `SKILL.md` with YAML frontmatter and step-by-step runbook
   - Dual-write to `.agents/skills/` (repo) and `~/.gemini/config/skills/` (global)
               │
               ▼
   [3. Knowledge Graph Sync]
   - Update `knowledge_graph/nodes/<entity>.md` with `[valid_from, recorded, source]`
   - Update `knowledge_graph/index.md` and `knowledge_graph/graph.json`
   - Run `python knowledge_graph/tools/verify_graph.py` (0 errors, 0 warnings)
               │
               ▼
   [4. Memory Ingestion]
   - Append concise pointer to `.agents/memories/MEMORY.md`
   - Retain pattern for zero-shot application to future similar problems
```

---

## Authoring a Net-New Skill

When synthesizing a new skill for Amethyst:
1. Choose a lowercase, hyphenated name (e.g. `newloka-fhir-export`, `newloka-audit-forensics`).
2. Construct `SKILL.md` with standard YAML frontmatter:
   ```yaml
   ---
   name: <skill-name>
   version: 1.0.0
   description: "<Concise description of capabilities and trigger conditions>"
   author: Amethyst Agent
   license: GPL-3.0
   platforms: [windows, linux, macos]
   ---
   ```
3. Structure the document:
   - `# Title`
   - `## When to Use`
   - `## Operational Procedures (exact commands, snippets, flags)`
   - `## Invariants & Guardrails`
4. Write to both:
   - `D:\GitRepos\newloka\.agents\skills\<skill-name>\SKILL.md`
   - `C:\Users\synch\.gemini\config\skills\<skill-name>\SKILL.md`

## Learning Invariant
The agent must never require human reminders to retain knowledge. Learning is continuous, unprompted, auditable, and backed by verified files on disk.
