"""Guard regressions: a transitive GUI edge and a missing core edge must fail."""
import copy
import importlib.util
from pathlib import Path
import unittest

SPEC = importlib.util.spec_from_file_location(
    "workspace_guard", Path(__file__).parents[1] / "check_workspace.py"
)
guard = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(guard)


class WorkspaceTests(unittest.TestCase):
    def graph(self):
        return {
            "packages": [{"id": name, "name": name} for name in
                         ("lintra", "lintra-ui", "helper", "i-slint-core")],
            "workspace_members": ["lintra", "lintra-ui"],
            "resolve": {"nodes": [
                {"id": "lintra", "dependencies": ["helper"]},
                {"id": "lintra-ui", "dependencies": ["lintra", "i-slint-core"]},
                {"id": "helper", "dependencies": []},
                {"id": "i-slint-core", "dependencies": []},
            ]},
        }

    def test_gui_only_in_adapter_is_allowed(self):
        guard.check(self.graph())

    def test_transitive_gui_in_core_fails(self):
        graph = self.graph()
        graph["resolve"]["nodes"][2]["dependencies"] = ["i-slint-core"]
        with self.assertRaisesRegex(ValueError, "GUI dependency"):
            guard.check(graph)

    def test_absent_or_unresolved_dependency_is_not_success(self):
        graph = self.graph()
        graph["resolve"]["nodes"][1]["dependencies"] = []
        with self.assertRaises(ValueError):
            guard.check(graph)
        broken = copy.deepcopy(self.graph())
        broken["resolve"] = None
        with self.assertRaises(TypeError):
            guard.check(broken)
