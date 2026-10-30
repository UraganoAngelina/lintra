"""CI-only release orchestration; stdlib tooling, no validation runtime changes."""
import argparse
import gzip
import hashlib
import io
import json
import os
from pathlib import Path
import platform
import re
import stat
import subprocess
import sys
import tarfile
import time
import tomllib
import urllib.error
import urllib.parse
import urllib.request
import zipfile

SOURCE = "UraganoAngelina/lintra"
DESTINATION = "UraganoAngelina/lintra-binaries"
TARGETS = {"x86_64-unknown-linux-gnu": "lintra", "x86_64-pc-windows-msvc": "lintra.exe"}
ROOT = Path(__file__).resolve().parents[1]


def digest(data):
    return hashlib.sha256(data).hexdigest()


def canonical(value):
    return (json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n").encode()


def identity(version, sha, root=ROOT):
    """Explicit version must identify the actual Cargo package, not just a tag."""
    if not re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z.-]+)?", version):
        raise ValueError("invalid release version")
    if not re.fullmatch(r"[0-9a-f]{40}", sha):
        raise ValueError("full lowercase source SHA required")
    if tomllib.loads((root / "Cargo.toml").read_text())["package"]["version"] != version:
        raise ValueError("release version differs from Cargo package")


def require_ci(sha, env=None):
    env = os.environ if env is None else env
    if env.get("GITHUB_ACTIONS") != "true" or env.get("GITHUB_REPOSITORY") != SOURCE or env.get("GITHUB_SHA") != sha:
        raise ValueError("release operations require the matching source GitHub Actions run")


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, request, fp, code, msg, headers, newurl):
        return None  # Never forward Authorization to a redirected host.


class GitHub:
    """Credentials stay in memory; fixed HTTPS hosts, no shell/URL credentials."""
    def __init__(self, token):
        if not token or any(c.isspace() for c in token):
            raise ValueError("required GitHub credential is not configured")
        self.token = token
        self.opener = urllib.request.build_opener(NoRedirect())

    def request(self, method, path, payload=None, *, upload=None, missing=False):
        host = "uploads.github.com" if upload is not None else "api.github.com"
        if not any(path == "/repos/" + repo or path.startswith("/repos/" + repo + "/") for repo in (SOURCE, DESTINATION)):
            raise ValueError("API path outside approved repositories")
        body = upload if upload is not None else (canonical(payload) if payload is not None else None)
        headers = {"Authorization": "Bearer " + self.token, "Accept": "application/vnd.github+json", "X-GitHub-Api-Version": "2022-11-28", "User-Agent": "lintra-release"}
        if body is not None:
            headers["Content-Type"] = "application/octet-stream" if upload is not None else "application/json"
        request = urllib.request.Request("https://" + host + path, data=body, headers=headers, method=method)
        try:
            with self.opener.open(request, timeout=60) as response:
                data = response.read()
                return json.loads(data) if data else None
        except urllib.error.HTTPError as error:
            if missing and error.code == 404:
                return None
            # Response bodies can contain server echoes; log only status and host.
            raise RuntimeError(f"GitHub {method} failed: HTTP {error.code} at {host}") from None


def unused_version(api, version):
    """Include drafts and tags: a failed earlier attempt reserves its version."""
    prefix = "/repos/" + DESTINATION
    if api.request("GET", prefix + "/git/ref/tags/v" + version, missing=True) is not None:
        raise ValueError("release tag already exists; choose a new version")
    page = 1
    while True:
        releases = api.request("GET", prefix + f"/releases?per_page=100&page={page}")
        if any(r["tag_name"] == "v" + version for r in releases):
            raise ValueError("release version already exists, including drafts")
        if len(releases) < 100:
            return
        page += 1


def preflight(api, source_api, version, sha):
    require_ci(sha)
    identity(version, sha)
    repo = api.request("GET", "/repos/" + DESTINATION)
    if repo.get("full_name") != DESTINATION or repo.get("private") is not True or repo.get("archived"):
        raise ValueError("destination differs from approved private repository")
    unused_version(api, version)
    runs = source_api.request("GET", f"/repos/{SOURCE}/actions/workflows/ci.yml/runs?head_sha={sha}&per_page=100")["workflow_runs"]
    matching = [run for run in runs if run.get("head_sha") == sha]
    latest = max(matching, key=lambda run: (run["run_number"], run["run_attempt"]), default=None)
    if latest is None or latest.get("conclusion") != "success" or latest.get("status") != "completed":
        raise ValueError("native development CI has not passed for this exact source SHA")


def product_files(product, target):
    if product not in ("cli", "ui") or target not in TARGETS:
        raise ValueError("unsupported product or native target")
    return TARGETS[target] if product == "cli" else ("lintra-ui.exe" if target.endswith("msvc") else "lintra-ui")


def ui_licenses():
    return {"LICENSE-IBM-PLEX-OFL.txt": (ROOT / "ui/ui/fonts/OFL.md").read_bytes(),
            "LICENSE-SLINT-ROYALTY-FREE-2.0.md": (ROOT / "ui/licenses/SLINT-ROYALTY-FREE-2.0.md").read_bytes()}


def archive_name(version, target, product="cli"):
    if target not in TARGETS:
        raise ValueError("unsupported native release target")
    product_files(product, target)
    prefix = "lintra" if product == "cli" else "lintra-ui"
    return f"{prefix}-{version}-{target}" + (".zip" if target.endswith("msvc") else ".tar.gz")


def pack(binary, output_dir, version, sha, target, audit, product="cli"):
    """Package already verified bytes with fixed archive metadata, no overwrite."""
    filename = product_files(product, target)
    data = binary.read_bytes()
    metadata = canonical(dict(audit, version=version, source_sha=sha, target=target, binary_sha256=digest(data)))
    if product == "ui":
        metadata = canonical(dict(audit, product="ui", version=version, source_sha=sha, target=target, binary_sha256=digest(data)))
    files = {filename: data, "BUILD.json": metadata}
    if product == "ui":
        files.update(ui_licenses())
    files["SHA256SUMS"] = "".join(f"{digest(value)}  {name}\n" for name, value in sorted(files.items())).encode()
    output_dir.mkdir(parents=True, exist_ok=False)
    destination = output_dir / archive_name(version, target, product)
    # This directory belongs to this invocation. Incomplete output is never
    # uploaded by the workflow because the step must succeed before artifact upload.
    with destination.open("xb") as raw:
        if target.endswith("msvc"):
            with zipfile.ZipFile(raw, "w", compression=zipfile.ZIP_STORED) as archive:
                for name, value in sorted(files.items()):
                    entry = zipfile.ZipInfo(name, date_time=(1980, 1, 1, 0, 0, 0))
                    entry.create_system = 3
                    entry.external_attr = (0o755 if name == filename else 0o644) << 16
                    archive.writestr(entry, value)
        else:
            with gzip.GzipFile(filename="", fileobj=raw, mode="wb", mtime=0) as compressed:
                with tarfile.open(fileobj=compressed, mode="w", format=tarfile.USTAR_FORMAT) as archive:
                    for name, value in sorted(files.items()):
                        entry = tarfile.TarInfo(name)
                        entry.size, entry.mode, entry.mtime = len(value), 0o755 if name == filename else 0o644, 0
                        archive.addfile(entry, io.BytesIO(value))
    return destination


def inspect_archive(path, version, sha, target, product="cli"):
    """Check exact product inventory, checksums and source/target identity."""
    if path.is_symlink() or not path.is_file():
        raise ValueError("release archive must be a regular file")
    filename = product_files(product, target)
    expected = {filename, "BUILD.json", "SHA256SUMS"}
    if product == "ui":
        expected.update(ui_licenses())
    files = {}
    if target.endswith("msvc"):
        with zipfile.ZipFile(path) as archive:
            entries = archive.infolist()
            if len(entries) != len(expected) or {e.filename for e in entries} != expected or any(e.is_dir() or stat.S_IFMT(e.external_attr >> 16) not in (0, stat.S_IFREG) or e.file_size > 256 * 1024 * 1024 for e in entries):
                raise ValueError("unexpected ZIP inventory or size")
            files = {e.filename: archive.read(e) for e in entries}
    else:
        with tarfile.open(path, "r:gz") as archive:
            entries = archive.getmembers()
            if len(entries) != len(expected) or {e.name for e in entries} != expected or any(not e.isfile() or e.size > 256 * 1024 * 1024 for e in entries):
                raise ValueError("unexpected tar inventory or size")
            files = {e.name: archive.extractfile(e).read() for e in entries}
    audit = json.loads(files["BUILD.json"])
    if any(audit.get(k) != v for k, v in {"version": version, "source_sha": sha, "target": target, "binary_sha256": digest(files[filename])}.items()):
        raise ValueError("archive build identity or binary digest differs")
    if product == "ui" and (audit.get("product") != "ui" or any(files[name] != data for name, data in ui_licenses().items())):
        raise ValueError("UI product identity or bundled license differs")
    sums = "".join(f"{digest(files[name])}  {name}\n" for name in sorted(expected - {"SHA256SUMS"})).encode()
    if files["SHA256SUMS"] != sums:
        raise ValueError("archive checksums differ")
    return audit


def build(version, sha, target, output):
    require_ci(sha)
    identity(version, sha)
    if (target.endswith("msvc") and sys.platform != "win32") or (target.endswith("gnu") and sys.platform != "linux"):
        raise ValueError("release target requires its native OS")
    command = ["cargo", "xtask", "build-release", "--target", target, "--verify-reproducible", "--verify-artifact"]
    result = subprocess.run(command, cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    print(result.stdout, end="", flush=True)
    if result.returncode:
        raise RuntimeError("verified native release build failed")
    artifacts = [line[len("artifact="):] for line in result.stdout.splitlines() if line.startswith("artifact=")]
    if len(artifacts) != 1:
        raise ValueError("release recipe did not identify exactly one artifact")
    binary = Path(artifacts[0])
    actual = subprocess.check_output([str(binary), "--version"], text=True).strip()
    if actual != f"lintra {version} (commit {sha})":
        raise ValueError("binary version/provenance differs from the requested release")
    libc = platform.libc_ver() if sys.platform == "linux" else None
    if libc is not None and (libc[0] != "glibc" or not libc[1]):
        raise ValueError("GNU release needs an identified glibc baseline")
    audit = {"rustc": subprocess.check_output(["rustc", "--version"], text=True).strip(), "runner_os": os.environ.get("RUNNER_OS"), "runner_image": os.environ.get("ImageOS"), "runner_image_version": os.environ.get("ImageVersion"), "glibc": libc, "python": platform.python_version(), "native_os": sys.platform, "reproducible": True, "artifact_verified": True}
    archive = pack(binary, output, version, sha, target, audit)
    inspect_archive(archive, version, sha, target)
    print("release_archive=" + archive.name)


def verify_assets(actual, expected):
    if len(actual) != len(expected) or {asset["name"] for asset in actual} != set(expected):
        raise ValueError("server asset inventory differs")
    for asset in actual:
        data = expected[asset["name"]]
        if asset.get("digest") != "sha256:" + digest(data) or asset.get("size") != len(data) or asset.get("state") != "uploaded":
            raise ValueError("server asset integrity/size/state differs")


def publish(api, source_api, version, sha, directory):
    preflight(api, source_api, version, sha)
    if directory.is_symlink():
        raise ValueError("artifact directory cannot be a symlink")
    names = {archive_name(version, target) for target in TARGETS}
    if {p.name for p in directory.iterdir()} != names:
        raise ValueError("expected exactly the two native release archives")
    assets = {}
    for target in TARGETS:
        path = directory / archive_name(version, target)
        audit = inspect_archive(path, version, sha, target)
        if audit.get("reproducible") is not True or audit.get("artifact_verified") is not True:
            raise ValueError("native artifact verification evidence missing")
        assets[path.name] = path.read_bytes()
    assets["SHA256SUMS"] = "".join(f"{digest(data)}  {name}\n" for name, data in sorted(assets.items())).encode()
    prefix = "/repos/" + DESTINATION
    repo = api.request("GET", prefix)
    commit = api.request("GET", prefix + "/commits/" + urllib.parse.quote(repo["default_branch"], safe=""))["sha"]
    # Atomic reservation: concurrent attempts cannot reuse the tag. Failures keep
    # the reservation/draft for diagnosis; never delete or silently resume it.
    api.request("POST", prefix + "/git/refs", {"ref": "refs/tags/v" + version, "sha": commit})
    release = api.request("POST", prefix + "/releases", {"tag_name": "v" + version, "target_commitish": commit, "name": "Lintra " + version, "draft": True, "prerelease": "-" in version, "make_latest": "false", "body": f"Built only by source CI: https://github.com/{SOURCE}/commit/{sha}\nVersion {version}. Embedded data is the declared synthetic demonstration bundle."})
    rid = release["id"]
    for name, data in sorted(assets.items()):
        uploaded = api.request("POST", prefix + f"/releases/{rid}/assets?name=" + urllib.parse.quote(name, safe=""), upload=data)
        if uploaded.get("size") != len(data) or uploaded.get("digest") != "sha256:" + digest(data) or uploaded.get("state") != "uploaded":
            raise ValueError("server asset integrity/size/state differs; release remains draft")
    verify_assets(api.request("GET", prefix + f"/releases/{rid}/assets"), assets)
    api.request("PATCH", prefix + f"/releases/{rid}", {"draft": False})
    for attempt in range(5):
        finished = api.request("GET", prefix + f"/releases/{rid}")
        if finished.get("immutable") is True and finished.get("draft") is False:
            if finished.get("tag_name") != "v" + version:
                raise ValueError("published release tag differs")
            verify_assets(finished.get("assets", []), assets)
            print("published_immutable_release=" + finished["html_url"])
            return
        if attempt < 4:
            time.sleep(2)
    raise ValueError("published release immutability was not verified; never overwrite/retry this version")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("operation", choices=["preflight", "build", "publish"])
    parser.add_argument("--version", default=os.environ.get("RELEASE_VERSION"))
    parser.add_argument("--source-sha", default=os.environ.get("GITHUB_SHA"))
    parser.add_argument("--target", choices=TARGETS, default=os.environ.get("RELEASE_TARGET"))
    parser.add_argument("--directory", type=Path)
    args = parser.parse_args()
    if args.version is None or args.source_sha is None:
        parser.error("explicit version and source SHA required through flags or CI input environment")
    if args.operation == "build":
        if args.target is None or args.directory is None:
            parser.error("build requires --target and --directory")
        build(args.version, args.source_sha, args.target, args.directory)
    else:
        api = GitHub(os.environ.get("LINTRA_RELEASE_TOKEN"))
        source_api = GitHub(os.environ.get("GITHUB_TOKEN"))
        if args.operation == "preflight":
            preflight(api, source_api, args.version, args.source_sha)
        else:
            if args.directory is None:
                parser.error("publish requires --directory")
            publish(api, source_api, args.version, args.source_sha, args.directory)


if __name__ == "__main__":
    try:
        main()
    except Exception as error:
        print("release failed: " + str(error), file=sys.stderr)
        sys.exit(1)
