"""Check the resolved Cargo graph, including transitive core dependencies.

An unresolved graph is an error, never evidence that GUI packages are absent.
The CLI shares the lintra package, so its dependency closure is checked too.
"""
import json
import subprocess


def check(metadata):
    packages = {p["id"]: p for p in metadata["packages"]}
    members = {packages[key]["name"]: key for key in metadata["workspace_members"]}
    if set(members) != {"lintra", "lintra-ui"}:
        raise ValueError("expected exactly the core/CLI and desktop workspace members")
    nodes = {node["id"]: node for node in metadata["resolve"]["nodes"]}
    core, ui = members["lintra"], members["lintra-ui"]
    if core not in nodes[ui]["dependencies"]:
        raise ValueError("desktop must depend on the public core package")
    pending, visited = [core], set()
    while pending:
        key = pending.pop()
        if key in visited:
            continue
        visited.add(key)
        name = packages[key]["name"]
        if key == ui or name == "slint" or name.startswith(("slint-", "i-slint-")):
            raise ValueError(f"GUI dependency reachable from core/CLI: {name}")
        pending.extend(nodes[key]["dependencies"])


if __name__ == "__main__":
    # Qualify the actual native build graph. Unfiltered metadata also downloads
    # unrelated Apple/Android packages; Linux and MSVC each run this guard in CI.
    compiler = subprocess.run(
        ["rustc", "-vV"], check=True, capture_output=True, text=True,
    ).stdout
    host = next(line.removeprefix("host: ") for line in compiler.splitlines()
                if line.startswith("host: "))
    result = subprocess.run(
        ["cargo", "metadata", "--locked", "--all-features", "--format-version", "1",
         "--filter-platform", host],
        check=True, stdout=subprocess.PIPE, text=True,
    )
    check(json.loads(result.stdout))
    print("workspace members and core/CLI dependency boundary verified")
