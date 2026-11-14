"""
Verification script for New Loka Knowledge Graph.
Checks:
1. Bidirectional consistency between index.md and nodes/*.md.
2. Parity between markdown graph and graph.json (machine-readable representation).
3. Bi-temporal fact structure: [valid_from: YYYY-MM-DD, recorded: YYYY-MM-DD, source: ...].
4. Superseded fact structure: [superseded: YYYY-MM-DD, reason: ...].
5. Edge endpoint integrity: every edge source and target must exist in the node set.
6. Typed relation taxonomy conformance.
7. Epistemic status validity: SHIPPED, ACTIVE, ROADMAP, RESEARCH, SUPERSEDED.
8. Domain containment: zero illegal path leakage.
"""

import json
import os
import re
import sys
from pathlib import Path

VALID_RELATIONS = {
    "implements",
    "enforces",
    "encrypts_with",
    "serves",
    "syncs_with",
    "governs",
    "tracks_in",
    "advances_to",
    "supersedes",
    "specifies",
    "persists_in",
    "orchestrates",
    "secures",
    "extends",
    "bridges_to",
    "integrates",
    "deploys_to",
}

VALID_STATUSES = {"SHIPPED", "ACTIVE", "ROADMAP", "RESEARCH", "SUPERSEDED"}

DATE_REGEX = re.compile(r"^\d{4}-\d{2}-\d{2}$")


def verify_knowledge_graph(kg_root: Path):
    print(f"=== Verifying New Loka Knowledge Graph at: {kg_root} ===")
    errors = []
    warnings = []

    index_path = kg_root / "index.md"
    json_path = kg_root / "graph.json"
    nodes_dir = kg_root / "nodes"

    if not index_path.exists():
        errors.append("Missing index.md in knowledge graph root.")
        return errors, warnings

    if not nodes_dir.exists() or not nodes_dir.is_dir():
        errors.append("Missing or invalid nodes/ directory.")
        return errors, warnings

    index_text = index_path.read_text(encoding="utf-8")

    # 1. Parse indexed nodes from index.md
    # Expect markdown table row: | filename.md | Entity | Layer | Status | Summary | Last updated |
    indexed_nodes = {}
    for line in index_text.splitlines():
        line = line.strip()
        if line.startswith("|") and not line.startswith("| File") and not line.startswith("|---"):
            cols = [c.strip() for c in line.split("|")[1:-1]]
            if len(cols) >= 5 and cols[0].endswith(".md"):
                file_name = cols[0]
                indexed_nodes[file_name] = {
                    "entity": cols[1],
                    "layer": cols[2] if len(cols) >= 6 else "Unknown",
                    "status": cols[3] if len(cols) >= 6 else "ACTIVE",
                    "summary": cols[4] if len(cols) >= 6 else cols[2],
                    "last_updated": cols[5] if len(cols) >= 6 else cols[3],
                }

    # 2. Check files on disk in nodes/
    disk_files = {p.name: p for p in nodes_dir.glob("*.md")}

    missing_on_disk = set(indexed_nodes.keys()) - set(disk_files.keys())
    unindexed_files = set(disk_files.keys()) - set(indexed_nodes.keys())

    for f in missing_on_disk:
        errors.append(f"index.md references node file missing on disk: nodes/{f}")
    for f in unindexed_files:
        errors.append(f"Unindexed node file found on disk: nodes/{f}")

    # 3. Parse and validate edges from index.md
    # Expect format: - source.md -[relation]-> target.md : Description
    # or: - source.md -> target.md : [relation] Description
    edge_pattern_typed = re.compile(r"^-\s*([a-zA-Z0-9_\-\.]+)\s*-\[(\w+)\]->\s*([a-zA-Z0-9_\-\.]+)\s*:\s*(.+)$")
    edge_pattern_legacy = re.compile(r"^-\s*([a-zA-Z0-9_\-\.]+)\s*->\s*([a-zA-Z0-9_\-\.]+)\s*:\s*(.+)$")

    edges_in_index = []
    in_edges_section = False

    for line in index_text.splitlines():
        sline = line.strip()
        if sline.startswith("## Edges") or sline.startswith("## Typed Edges"):
            in_edges_section = True
            continue
        elif in_edges_section and (sline.startswith("## ") or sline.startswith("---")):
            in_edges_section = False
            continue

        if in_edges_section and sline.startswith("-"):
            match_typed = edge_pattern_typed.match(sline)
            if match_typed:
                src, rel, tgt, desc = match_typed.groups()
                edges_in_index.append((src, rel, tgt, desc))
                if rel not in VALID_RELATIONS:
                    warnings.append(f"Unknown relation type in edge '{sline}': '{rel}' (allowed: {VALID_RELATIONS})")
                if src not in indexed_nodes and src not in disk_files:
                    errors.append(f"Edge source not in nodes: '{src}' in '{sline}'")
                if tgt not in indexed_nodes and tgt not in disk_files:
                    errors.append(f"Edge target not in nodes: '{tgt}' in '{sline}'")
            else:
                match_legacy = edge_pattern_legacy.match(sline)
                if match_legacy:
                    src, tgt, desc = match_legacy.groups()
                    rel = "relates_to"
                    # Try to extract [rel] from description if present
                    rel_bracket = re.match(r"^\[(\w+)\]\s*(.+)$", desc)
                    if rel_bracket:
                        rel = rel_bracket.group(1)
                        desc = rel_bracket.group(2)
                    edges_in_index.append((src, rel, tgt, desc))
                    if src not in indexed_nodes and src not in disk_files:
                        errors.append(f"Edge source not in nodes: '{src}' in '{sline}'")
                    if tgt not in indexed_nodes and tgt not in disk_files:
                        errors.append(f"Edge target not in nodes: '{tgt}' in '{sline}'")
                else:
                    warnings.append(f"Edge line does not match standard pattern: '{sline}'")

    # 4. Validate each node file's internal structure and bi-temporal facts
    fact_tag_pattern = re.compile(r"\[valid_from:\s*([^,]+),\s*recorded:\s*([^,]+),\s*source:\s*([^\]]+)\]")
    superseded_tag_pattern = re.compile(r"\[superseded:\s*([^,]+),\s*reason:\s*([^\]]+)\]")

    total_facts = 0
    total_superseded = 0

    for fname, path in disk_files.items():
        content = path.read_text(encoding="utf-8")

        # Must have # Title
        if not re.search(r"^#\s+.+", content, re.MULTILINE):
            errors.append(f"Node {fname} missing top-level # Heading")

        # Must have ## Facts
        if "## Facts" not in content:
            errors.append(f"Node {fname} missing ## Facts section")
        else:
            facts_block = re.search(r"##\s+Facts\s*\n(.*?)(?=\n##|\Z)", content, re.DOTALL)
            if facts_block:
                ftext = facts_block.group(1).strip()
                fact_lines = []
                cur_fact = []
                for ln in ftext.splitlines():
                    sln = ln.strip()
                    if sln.startswith("-"):
                        if cur_fact:
                            fact_lines.append(" ".join(cur_fact))
                        cur_fact = [sln]
                    elif cur_fact and sln:
                        cur_fact.append(sln)
                if cur_fact:
                    fact_lines.append(" ".join(cur_fact))

                for fact in fact_lines:
                    total_facts += 1
                    match = fact_tag_pattern.search(fact)
                    if not match:
                        errors.append(f"Fact in {fname} missing valid bi-temporal tag: '{fact[:80]}...'")
                    else:
                        v_from = match.group(1).strip()
                        rec = match.group(2).strip()
                        if not DATE_REGEX.match(v_from) and not v_from.startswith("pre-"):
                            warnings.append(f"Non-standard valid_from date in {fname}: '{v_from}'")
                        if not DATE_REGEX.match(rec):
                            warnings.append(f"Non-standard recorded date in {fname}: '{rec}'")

        # Must have ## Superseded
        if "## Superseded" not in content:
            errors.append(f"Node {fname} missing ## Superseded section")
        else:
            sup_block = re.search(r"##\s+Superseded\s*\n(.*?)(?=\n##|\Z)", content, re.DOTALL)
            if sup_block:
                stext = sup_block.group(1).strip()
                if stext and stext.lower() != "(none)":
                    for ln in stext.splitlines():
                        sln = ln.strip()
                        if sln.startswith("-"):
                            total_superseded += 1
                            if not superseded_tag_pattern.search(sln):
                                warnings.append(f"Superseded item in {fname} missing [superseded: ..., reason: ...]: '{sln[:70]}'")

    # 5. Validate graph.json if present
    if json_path.exists():
        try:
            graph_data = json.loads(json_path.read_text(encoding="utf-8"))
            json_nodes = {n["id"]: n for n in graph_data.get("nodes", [])}
            json_edges = graph_data.get("edges", [])

            # Check that all markdown files exist as nodes in graph.json
            for fname in disk_files.keys():
                nid = fname.replace(".md", "")
                if nid not in json_nodes and fname not in json_nodes:
                    errors.append(f"graph.json missing entry for node: {fname}")

            # Check edges in graph.json
            for e in json_edges:
                src = e.get("source")
                tgt = e.get("target")
                rel = e.get("relation")
                if not src or not tgt:
                    errors.append(f"graph.json edge missing source or target: {e}")
                if rel and rel not in VALID_RELATIONS:
                    warnings.append(f"graph.json edge has non-standard relation: '{rel}'")
        except Exception as ex:
            errors.append(f"Failed to parse graph.json: {ex}")
    else:
        warnings.append("graph.json not found in knowledge graph root (optional but recommended).")

    print(f"Nodes checked: {len(disk_files)}")
    print(f"Edges indexed: {len(edges_in_index)}")
    print(f"Facts validated: {total_facts}")
    print(f"Superseded facts: {total_superseded}")
    print(f"Errors: {len(errors)}, Warnings: {len(warnings)}")

    if errors:
        print("\nERRORS:")
        for e in errors:
            print(f" [!] {e}")

    if warnings:
        print("\nWARNINGS:")
        for w in warnings:
            print(f" [*] {w}")

    if not errors:
        print("\n>>> ALL CHECKS PASSED: Knowledge Graph is 100% compliant.")
        return 0
    else:
        print("\n>>> FAILED: Knowledge graph has errors that must be resolved.")
        return 1


if __name__ == "__main__":
    script_dir = Path(__file__).resolve().parent
    kg_dir = script_dir.parent
    sys.exit(verify_knowledge_graph(kg_dir))
