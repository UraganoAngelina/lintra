#!/usr/bin/env python3
"""Read-only governance audit, not a runtime/normative validator.

Checks every migrated prompt against the recorded, reversible operational edits,
and guards the plan outside §14 and immutable source documents. The adjacent kit
is optional on a clone; when present its originals must still match provenance.
"""
from pathlib import Path
import hashlib
import json
import re

ROOT = Path(__file__).resolve().parents[2]


def digest(data):
    """Exact bytes: no newline, Unicode or whitespace normalization."""
    return hashlib.sha256(data).hexdigest()


def audit():
    """Fail on a contradictory active clause or an unauthorized contract edit."""
    manifest = json.loads((ROOT / "docs/governance/migration-manifest.json").read_text())
    kit = ROOT.parent / manifest["source_kit"]
    plan = (ROOT / "docs/MASTER_IMPLEMENTATION_PLAN.md").read_bytes()
    start = plan.index(b"## 14. `AI_AGENT_EXECUTION_PROTOCOL`")
    end = plan.index(b"## 15.", start)
    assert digest(plan[:start]) == manifest["plan_prefix_sha256"], "plan before §14 changed"
    assert digest(plan[end:]) == manifest["plan_suffix_sha256"], "plan after §14 changed"
    master = (ROOT / "MASTER_AGENT_PROMPT.md").read_bytes()
    a = master.index(b"## VINCOLI INVARIANTI")
    b = master.index(b"## STATO DI PARTENZA", a)
    assert digest(master[a:b]) == manifest["master_invariants_sha256"], "master invariants changed"
    dependency_map = (ROOT / "TASK_DEPENDENCY_MAP.md").read_bytes().split(b"```mermaid", 1)[1]
    assert digest(dependency_map) == manifest["dependency_map_body_sha256"], "task dependencies changed"
    for relative, expected in manifest["protected_documents"].items():
        assert digest((ROOT / relative).read_bytes()) == expected, relative
    if kit.exists():
        for relative, expected in manifest["source_governance"].items():
            assert digest((kit / relative).read_bytes()) == expected, relative
    for name, record in manifest["active_prompts"].items():
        active = (ROOT / "TASK_PROMPTS" / name).read_bytes()
        assert digest(active) == record["active_sha256"], name
        reconstructed = active.decode("utf-8")
        for change in reversed(record["operational_replacements"]):
            assert reconstructed.count(change["after"]) == change["count"], name
            reconstructed = reconstructed.replace(change["after"], change["before"])
        assert digest(reconstructed.encode()) == record["source_sha256"], name
        if kit.exists():
            assert digest((kit / "TASK_PROMPTS" / name).read_bytes()) == record["source_sha256"], name
    # Only the current part of AGENTS is executable policy, never its history.
    agents = (ROOT / "AGENTS.md").read_text().split("## Registro storico e ratifiche precedenti")[0]
    assert "## GLOBAL AGENT EXECUTION POLICY" in agents
    assert all(term in agents for term in ["Superpowers", "subagenti", "permanentemente", "Prosecuzione automatica", "DONE_UNVERIFIED"])
    active_files = [ROOT / "TASK_DEPENDENCY_MAP.md", *list((ROOT / "TASK_PROMPTS").glob("*.md")), *list((ROOT / "docs/governance/templates").glob("*.md"))]
    forbidden = re.compile(r"FERMATI: nessun task successivo autonomo|Non avviare task successive autonomamente|ferma la sessione|ferma l'esecuzione|Fermati e attendi successivo comando|non avviare in autonomia la successiva", re.I)
    for path in active_files:
        assert not forbidden.search(path.read_text()), path
    # Overrides must be explicitly reviewed, never silently skipped by gitignore.
    overrides = [p for p in ROOT.rglob("AGENTS.override.md") if not any(part in {"target", ".git"} for part in p.relative_to(ROOT).parts)]
    assert not overrides, f"review unexpected overrides: {overrides}"
    print(f"PASS: {len(manifest['active_prompts'])} active prompts; reversible operational edits only")
    print("PASS: plan outside §14, ARCH/DET and original kit unchanged; no contradictory active stop or override")


if __name__ == "__main__":
    audit()
