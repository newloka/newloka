"""
Knowledge Graph Query Tool for New Loka.
Provides search, entity lookup, edge traversal, and neighborhood extraction.
"""

import json
import os
import re
import sys
from pathlib import Path


class NewLokaKG:
    def __init__(self, root: Path = None):
        if root is None:
            root = Path(__file__).resolve().parent.parent
        self.root = root
        self.index_path = self.root / "index.md"
        self.json_path = self.root / "graph.json"
        self.nodes_dir = self.root / "nodes"
        self._load()

    def _load(self):
        if self.json_path.exists():
            self.data = json.loads(self.json_path.read_text(encoding="utf-8"))
        else:
            self.data = {"nodes": [], "edges": []}
        self.node_map = {n["id"]: n for n in self.data.get("nodes", [])}

    def list_nodes(self, layer: str = None, status: str = None):
        results = []
        for n in self.data.get("nodes", []):
            if layer and n.get("layer") != layer:
                continue
            if status and n.get("epistemic_status") != status:
                continue
            results.append(n)
        return results

    def get_node(self, node_id: str):
        node = self.node_map.get(node_id)
        if not node:
            # Try by file name
            node = self.node_map.get(node_id.replace(".md", ""))
        return node

    def read_node_markdown(self, node_id: str):
        if not node_id.endswith(".md"):
            node_id = f"{node_id}.md"
        node_file = self.nodes_dir / node_id
        if node_file.exists():
            return node_file.read_text(encoding="utf-8")
        return None

    def search_facts(self, query: str):
        query_lower = query.lower()
        results = []
        for p in self.nodes_dir.glob("*.md"):
            content = p.read_text(encoding="utf-8")
            for line in content.splitlines():
                if query_lower in line.lower() and line.strip().startswith("-"):
                    results.append({"node": p.name, "fact": line.strip()})
        return results

    def get_neighbors(self, node_id: str, direction: str = "both"):
        # node_id without .md
        nid = node_id.replace(".md", "")
        inbound = []
        outbound = []
        for e in self.data.get("edges", []):
            src = e["source"].replace(".md", "")
            tgt = e["target"].replace(".md", "")
            if src == nid:
                outbound.append(e)
            if tgt == nid:
                inbound.append(e)
        if direction == "out":
            return outbound
        elif direction == "in":
            return inbound
        else:
            return {"outbound": outbound, "inbound": inbound}


def main():
    if len(sys.argv) < 2:
        print("Usage: kg_tool.py [list|node <id>|search <term>|neighbors <id>|verify]")
        sys.exit(1)

    cmd = sys.argv[1].lower()
    kg = NewLokaKG()

    if cmd == "list":
        nodes = kg.list_nodes()
        print(f"Total Nodes: {len(nodes)}")
        for n in nodes:
            print(f"- [{n.get('layer', 'Core')}] {n['id']} ({n.get('epistemic_status', 'ACTIVE')}): {n.get('label')} - {n.get('summary')}")
    elif cmd == "node" and len(sys.argv) > 2:
        nid = sys.argv[2]
        node = kg.get_node(nid)
        if node:
            print(json.dumps(node, indent=2))
            md = kg.read_node_markdown(nid)
            if md:
                print("\n--- Markdown Content ---\n")
                print(md)
        else:
            print(f"Node '{nid}' not found.")
    elif cmd == "search" and len(sys.argv) > 2:
        q = " ".join(sys.argv[2:])
        facts = kg.search_facts(q)
        print(f"Found {len(facts)} matching facts for '{q}':")
        for f in facts:
            print(f"[{f['node']}] {f['fact']}")
    elif cmd == "neighbors" and len(sys.argv) > 2:
        nid = sys.argv[2]
        nbrs = kg.get_neighbors(nid)
        print(f"Neighbors of {nid}:")
        print("Outbound edges:")
        for e in nbrs["outbound"]:
            print(f"  -> [{e.get('relation')}] {e.get('target')} : {e.get('description')}")
        print("Inbound edges:")
        for e in nbrs["inbound"]:
            print(f"  <- [{e.get('relation')}] {e.get('source')} : {e.get('description')}")
    else:
        print("Unknown command or missing arguments.")


if __name__ == "__main__":
    main()
