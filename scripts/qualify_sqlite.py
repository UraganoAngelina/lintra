"""Build/run the real bundled SQLite harness and apply existing artifact gates.

The helper is not distributed. Its optimized executable actually calls SQLite,
so dead-code removal in the CLI cannot hide a system SQLite runtime dependency.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target", required=True)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--offline", action="store_true")
    parser.add_argument("--smoke", action="store_true")
    args = parser.parse_args()
    repo = Path(__file__).resolve().parent.parent
    output = args.output.resolve()
    output.parent.mkdir(parents=True, exist_ok=True)
    build_env = os.environ.copy()
    encoded = build_env.get("CARGO_ENCODED_RUSTFLAGS")
    if encoded is not None:
        flags = encoded.split("\x1f")
    else:
        target_flags = "CARGO_TARGET_" + args.target.upper().replace("-", "_") + "_RUSTFLAGS"
        flags = build_env.get("RUSTFLAGS", build_env.get(target_flags, "")).split()
    # Same required remappings and static MSVC CRT as the shipping release recipe.
    home = Path.home()
    for path, replacement in [(repo, "/lintra"),
                              (Path(os.environ.get("CARGO_HOME", home / ".cargo")), "/cargo"),
                              (Path(os.environ.get("RUSTUP_HOME", home / ".rustup")), "/rustup")]:
        for spelling in {str(path), str(path.resolve())}:
            flags.append(f"--remap-path-prefix={spelling}={replacement}")
    if args.target.endswith("msvc"):
        flags.extend(["-C", "target-feature=+crt-static", "-C", "link-arg=/Brepro"])
    build_env["CARGO_ENCODED_RUSTFLAGS"] = "\x1f".join(flags)
    command = ["cargo", "bench", "--bench", "persistence", "--no-run", "--locked",
               "--target", args.target, "--message-format=json"]
    if args.offline:
        command.append("--offline")
    result = subprocess.run(command, cwd=repo, env=build_env, check=True,
                            stdout=subprocess.PIPE, encoding="utf-8")
    executable = None
    for line in result.stdout.splitlines():
        message = json.loads(line)
        if (message.get("reason") == "compiler-artifact"
                and message.get("target", {}).get("name") == "persistence"
                and message.get("executable")):
            executable = message["executable"]
    if executable is None:
        raise RuntimeError("Cargo did not identify the SQLite qualification executable")
    verify = ["cargo", "xtask", "verify-artifact", "--target", args.target,
              "--artifact", executable]
    if args.offline:
        verify.append("--offline")
    gate = subprocess.run(verify, cwd=repo, check=True, stdout=subprocess.PIPE,
                          encoding="utf-8")
    print(gate.stdout, end="")
    command = [executable, "--output", str(output)]
    if args.smoke:
        command.append("--smoke")
    subprocess.run(command, cwd=repo, check=True)
    report = json.loads(output.read_text(encoding="utf-8"))
    report["target"] = args.target
    report["artifact_gate"] = gate.stdout.strip()
    report["executable_sha256"] = hashlib.sha256(Path(executable).read_bytes()).hexdigest()
    report["commit"] = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=repo,
                                               encoding="utf-8").strip()
    output.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
