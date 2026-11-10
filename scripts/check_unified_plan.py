#!/usr/bin/env python3
"""Verify the ratified task graph and source provenance without executing tasks.

This governance check rejects dropped/duplicated IDs, changed canonical
dependencies, cycles and missing evidence for DONE. It cannot attest a native
runtime gate or authorize publication.
"""

import hashlib
import json
from pathlib import Path
import re


def check(root: Path) -> dict:
    base = root / "docs/unified-post-m9"
    graph = json.loads((base / "DEPENDENCY_GRAPH.json").read_text())
    tasks = graph["tasks"]
    by_id = {task["id"]: task for task in tasks}
    assert len(tasks) == len(by_id) == 128, "Dropped or duplicated task IDs"
    assert sum(key.startswith("U-") for key in by_id) == 39
    assert sum(key.startswith("T-MIG-") for key in by_id) == 89

    # Compare every original edge directly to the ratified master. Integration
    # edges are separate so they cannot silently replace a canonical dependency.
    canonical = {}
    for line in (base / "MASTER_PLAN_UNIFICATO.md").read_text().splitlines():
        cells = [cell.strip() for cell in line.strip("|").split("|")]
        if len(cells) >= 4 and re.fullmatch(
            r"(?:U-[A-Z]+-\d{2}|T-MIG-P\d-\d{2})", cells[0]
        ):
            assert cells[0] not in canonical, cells[0]
            canonical[cells[0]] = re.findall(
                r"(?:U-[A-Z]+-\d{2}|T-MIG-P\d-\d{2})", cells[2]
            )
    assert set(canonical) == set(by_id)
    visited, visiting = set(), set()

    def visit(key):
        assert key in by_id, f"Unknown dependency {key}"
        assert key not in visiting, f"Dependency cycle at {key}"
        if key in visited:
            return
        task = by_id[key]
        assert task["original_dependencies"] == canonical[key], key
        assert task["integration_dependencies"] == (
            ["U-UI-01"] if key == "T-MIG-P0-01" else []
        ), f"Unratified integration edge: {key}"
        assert (root / task["prompt"]).is_file(), key
        visiting.add(key)
        for dependency in task["original_dependencies"] + task["integration_dependencies"]:
            visit(dependency)
        visiting.remove(key)
        visited.add(key)

    for key in by_id:
        visit(key)

    manifest = json.loads((base / "SOURCE_MANIFEST.json").read_text())
    corrections = set(manifest["prompt_corrections"])
    before = "Le decisioni di prodotto già USER_APPROVED del kit migrazione 1.1 restano valide; ratifiche nuove necessarie solo per il Decision Pack post-M9."
    after = "Le decisioni di prodotto MIG e D01–D15 sono già ratificate; il Decision Pack non è pendente. Recepire la ratifica repository del 10 novembre; chiedere soltanto decisioni sostanziali non coperte."
    for item in manifest["files"]:
        data = (root / item["repository_path"]).read_bytes()
        if item["repository_path"] in corrections:
            text = data.decode()
            assert after in text and before not in text
            data = text.replace(after, before).encode()
        assert hashlib.sha256(data).hexdigest() == item["source_sha256"], item

    done = []
    for line in (root / "UNIFIED_IMPLEMENTATION_STATUS.md").read_text().splitlines():
        cells = [cell.strip() for cell in line.strip("|").split("|")]
        if len(cells) >= 3 and cells[0] in by_id and cells[2] == "DONE":
            key = cells[0]
            assert (root / f"docs/verification/unified-post-m9/{key}.md").is_file(), key
            done.append(key)
    for key in done:
        task = by_id[key]
        assert all(
            dependency in done
            for dependency in task["original_dependencies"] + task["integration_dependencies"]
        ), f"DONE with unfinished prerequisite: {key}"
    assert graph["release_gate"]["requires"] == [
        "G-MIG-9", "G-CADUI3", "G-SIOC1", "G-OPS2", "G-DUR"
    ]
    assert "140/140 DONE" in (root / "IMPLEMENTATION_STATUS.md").read_text()
    return {"tasks": len(tasks), "acyclic": True, "sources": len(manifest["files"]),
            "done_with_dossiers": done, "runtime_gates_verified_by_this_check": []}


if __name__ == "__main__":
    print(json.dumps(check(Path(__file__).resolve().parents[1]), indent=2))
