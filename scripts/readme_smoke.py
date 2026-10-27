"""Execute README quickstart blocks with a qualified native release, from scratch.

Linux uses bash; Windows uses Windows PowerShell 5.1 (not pwsh 7). Only synthetic
bytes are created in a fresh isolated install. Report contents/encoding and
negative cases are checked independently; the tested commands come from README.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile
import zipfile
import xml.etree.ElementTree as ET


def fragment(readme, shell):
    pattern = rf"<!-- lintra-quickstart:{re.escape(shell)} -->\s*```[^\n]+\n(.*?)\n```\s*<!-- /lintra-quickstart -->"
    matches = re.findall(pattern, readme, re.DOTALL)
    if len(matches) != 1:
        raise ValueError(f"expected exactly one README quickstart for {shell}")
    return matches[0]


def release_artifact(log):
    paths = [line.removeprefix("artifact=") for line in
             log.read_text(encoding="utf-8").splitlines() if line.startswith("artifact=")]
    if len(paths) != 1:
        raise ValueError("qualified release log must identify one artifact")
    return Path(paths[0]).resolve(strict=True)


def smoke(binary, readme, output):
    binary = binary.resolve(strict=True)
    shell = "powershell" if os.name == "nt" else "bash"
    text = fragment(readme.read_text(encoding="utf-8"), shell)
    evidence = {"shell": shell, "native_platform": os.name,
                "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
                "readme_sha256": hashlib.sha256(readme.read_bytes()).hexdigest(),
                "quickstart_sha256": hashlib.sha256(text.encode()).hexdigest(), "cases": []}
    with tempfile.TemporaryDirectory(prefix="lintra README with spaces ") as temporary:
        install = Path(temporary)
        executable = install / ("lintra.exe" if os.name == "nt" else "lintra")
        shutil.copy2(binary, executable)
        env = {k: v for k, v in os.environ.items() if k.upper() not in
               {"LINTRA_SCHEMA_DIR", "LINTRA_REFERENCE_DATA_DIR"}}

        def call(arguments, code):
            result = subprocess.run(arguments, cwd=install, env=env,
                                    capture_output=True, timeout=180)
            if result.returncode != code:
                raise ValueError(f"README case returned {result.returncode}, expected {code}: {result.stderr!r}")
            return result

        version = call([str(executable), "--version"], 0).stdout.decode("utf-8").strip()
        if not re.fullmatch(r"lintra \d+\.\d+\.\d+(?:-[\w.]+)? \(commit [\w.-]+\)", version):
            raise ValueError("--version must report actual version and commit")
        evidence["version"] = version
        script = install / ("quickstart.ps1" if os.name == "nt" else "quickstart.sh")
        script.write_text(("$ErrorActionPreference = 'Stop'\n" if os.name == "nt" else "set -eu\n") + text + "\n", encoding="utf-8")
        if os.name == "nt":
            command = ["powershell.exe", "-NoProfile", "-NonInteractive", "-File", str(script)]
            ps = call(["powershell.exe", "-NoProfile", "-NonInteractive", "-Command",
                       "$PSVersionTable.PSVersion.ToString()"], 0).stdout.decode("utf-8").strip()
            if not ps.startswith("5.1."):
                raise ValueError("Windows README acceptance requires actual PowerShell5.1")
            evidence["powershell_version"] = ps
        else:
            command = ["bash", "--noprofile", "--norc", str(script)]
        call(command, 0)
        # Independent expected byte sequence from the supplied synthetic schema.
        if (install / "input.txt").read_bytes() != b"001A  \r\n":
            raise ValueError("README shell did not preserve fixed-width bytes/CRLF")
        summary = json.loads((install / "summary.json").read_bytes())
        if (summary["result"], summary["records_processed"], summary["total_errors"],
            summary["validation_mode"], summary["bundle_source"]) != ("VALID", 1, 0, "local_only", "embedded"):
            raise ValueError("README valid example has unexpected real validation result")
        if (install / "errors.jsonl").read_bytes():
            raise ValueError("README valid example must produce no diagnostics")
        with zipfile.ZipFile(install / "report.xlsx") as archive:
            names = [node.attrib["name"] for node in ET.fromstring(archive.read("xl/workbook.xml"))
                     .iter() if node.tag.endswith("}sheet")]
            if names != ["Riepilogo", "Errori"]:
                raise ValueError("README XLSX must contain the documented sheets")
        evidence["cases"].append({"case": "literal-readme-shell-block", "exit": 0, "records": 1, "errors": 0})

        def validate_case(name, data, expected, version="v1", extra=()):
            input_path = install / f"{name}.txt"
            input_path.write_bytes(data)
            args = [str(executable), "validate", "--flow", "SYNTHETIC", "--schema-version", version,
                    "--trace", "T1", "--encoding", "utf8", "--input", input_path.name,
                    "--errors", f"{name}.jsonl", "--summary", f"{name}.json", *extra]
            call(args, expected)
            if input_path.read_bytes() != data:
                raise ValueError("CLI changed input bytes")
            errors = install / f"{name}.jsonl"
            if expected == 1:
                if errors.exists() or (install / f"{name}.json").exists():
                    raise ValueError("preflight failure published completed reports")
            else:
                raw = errors.read_bytes()
                raw.decode("utf-8")
                if raw.startswith(b"\xef\xbb\xbf") or b"\r" in raw or not raw.endswith(b"\n"):
                    raise ValueError("diagnostic JSONL is not UTF8/noBOM/LF")
                report = json.loads((install / f"{name}.json").read_bytes())
                if report["result"] != "INVALID" or report["total_errors"] != 1:
                    raise ValueError("exit2 is not the expected completed invalid result")
            evidence["cases"].append({"case": name, "exit": expected})

        validate_case("invalid", b"001Z  \n", 2)
        validate_case("bom", b"\xef\xbb\xbf001A  \r\n", 1)
        validate_case("version-explicit-missing", b"001A  \n", 1, "does-not-exist")
        validate_case("incomplete-external", b"001A  \n", 1, extra=("--schema-dir", str(install)))
        call([str(executable), "--unknown-option"], 1)
        evidence["cases"].append({"case": "parse-error-is-technical", "exit": 1})
        if hashlib.sha256(executable.read_bytes()).hexdigest() != evidence["binary_sha256"]:
            raise ValueError("documentation rehearsal altered the executable")
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_bytes((json.dumps(evidence, indent=2) + "\n").encode())
    print(f"README native shell and real CLI cases passed ({shell})")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    source = parser.add_mutually_exclusive_group(required=True)
    source.add_argument("--binary", type=Path)
    source.add_argument("--build-log", type=Path)
    parser.add_argument("--readme", type=Path, default=Path(__file__).resolve().parents[1] / "README.md")
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    smoke(args.binary or release_artifact(args.build_log), args.readme, args.output)


if __name__ == "__main__":
    main()
