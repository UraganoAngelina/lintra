"""Qualify and package the desktop executable in native CI; never publish tags."""
import argparse
import json
import os
from pathlib import Path, PurePosixPath
import platform
import posixpath
import re
import subprocess
import sys
import tempfile
import tomllib
import release


def runtime_libraries(trace, allowed):
    """Qualify actual Linux loader observations; an empty trace is no evidence."""
    paths = re.findall(r"calling init:\s+([^\n ]+)", trace)
    if not paths:
        raise ValueError("native loader trace contains no initialized libraries; runtime is unverified")
    roots = [PurePosixPath(value) for value in ("/lib", "/lib64", "/usr/lib", "/usr/lib64")]
    for value in paths:
        path = PurePosixPath(posixpath.normpath(value))
        if not path.is_absolute() or not any(path.is_relative_to(root) for root in roots):
            raise ValueError("loaded UI library is outside OS library roots: " + value)
        if sys.platform == "linux":
            resolved = Path(value).resolve()
            if not any(resolved.is_relative_to(Path(str(root)).resolve()) for root in roots):
                raise ValueError("loaded UI library resolves outside OS roots: " + value)
    loaded = sorted({PurePosixPath(value).name for value in paths})
    unexpected = sorted(set(loaded) - set(allowed))
    if unexpected:
        raise ValueError("unqualified loaded UI libraries: " + repr(unexpected))
    return loaded


def package(version, sha, target, output):
    release.require_ci(sha)
    release.identity(version, sha)
    if tomllib.loads((release.ROOT / "ui/Cargo.toml").read_text())["package"]["version"] != version:
        raise ValueError("UI and requested versions differ")
    if (target.endswith("msvc") and sys.platform != "win32") or (target.endswith("gnu") and sys.platform != "linux"):
        raise ValueError("UI packaging requires its native OS")
    command = ["cargo", "xtask", "build-release", "--product", "ui", "--target", target, "--verify-reproducible", "--verify-artifact"]
    result = subprocess.run(command, cwd=release.ROOT, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    print(result.stdout, end="", flush=True)
    if result.returncode:
        raise RuntimeError("UI release qualification failed")
    artifacts = [line[len("artifact="):] for line in result.stdout.splitlines() if line.startswith("artifact=")]
    if len(artifacts) != 1:
        raise ValueError("expected exactly one verified UI executable")
    binary = Path(artifacts[0])
    libc = platform.libc_ver() if sys.platform == "linux" else None
    if libc is not None and (libc[0] != "glibc" or not libc[1]):
        raise ValueError("GNU desktop release requires an identified glibc baseline")
    audit = {"product": "ui", "native_os": sys.platform, "runner_os": os.environ.get("RUNNER_OS"), "runner_image": os.environ.get("ImageOS"), "glibc": libc, "rustc": subprocess.check_output(["rustc", "--version"], text=True).strip(), "python": platform.python_version(), "runner_image_version": os.environ.get("ImageVersion"), "reproducible": True, "artifact_verified": True, "renderer": "software", "slint": "1.18.1"}
    archive = release.pack(binary, output, version, sha, target, audit, product="ui")
    release.inspect_archive(archive, version, sha, target, product="ui")
    # Consume actual archive bytes, not the executable in the build directory.
    with tempfile.TemporaryDirectory(prefix="lintra-ui-install-") as temporary:
        install = Path(temporary)
        if target.endswith("msvc"):
            with release.zipfile.ZipFile(archive) as packed:
                packed.extractall(install)
        else:
            with release.tarfile.open(archive, "r:gz") as packed:
                packed.extractall(install, filter="data")
        installed = install / release.product_files("ui", target)
        env = {key: value for key, value in os.environ.items() if not key.startswith("LINTRA_")}
        env.update(PATH="", XDG_CACHE_HOME=str(install / "cache"), XDG_DATA_HOME=str(install / "data"), APPDATA=str(install / "appdata"), LOCALAPPDATA=str(install / "localappdata"))
        actual = subprocess.check_output([str(installed), "--version"], cwd=install, env=env, text=True).strip()
        if actual != f"lintra-ui {version} (commit {sha})":
            raise ValueError("installed UI version/provenance differs")
        check = subprocess.check_output([str(installed), "--packaging-check"], cwd=install, env=env, text=True).strip()
        if not check.startswith("ui_self_check=ok schemas="):
            raise ValueError("installed UI resources/software probe failed")
        native_env = dict(env)
        native_env.pop("LD_LIBRARY_PATH", None)
        native_env.pop("LD_PRELOAD", None)
        if sys.platform == "linux":
            native_env["LD_DEBUG"] = "libs"
        native = subprocess.run([str(installed), "--packaging-startup-check"], cwd=install, env=native_env, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=30)
        if native.returncode or "ui_native_startup=ok" not in native.stdout:
            raise RuntimeError("installed native UI startup failed: " + native.stderr[-4000:])
        loaded = []
        if sys.platform == "linux":
            allowed = set((release.ROOT / "ui/linux-runtime-libraries.list").read_text().splitlines())
            loaded = runtime_libraries(native.stderr, allowed)
    print("ui_native_libraries=" + json.dumps(loaded))
    print("ui_archive=" + str(archive))
    print("ui_install_probe=" + json.dumps({"version": actual, "check": check, "path_empty": True}))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--version", required=True)
    parser.add_argument("--source-sha", required=True)
    parser.add_argument("--target", required=True, choices=release.TARGETS)
    parser.add_argument("--directory", required=True, type=Path)
    args = parser.parse_args()
    package(args.version, args.source_sha, args.target, args.directory)


if __name__ == "__main__":
    main()
