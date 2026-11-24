"""Owner-authorized bounded public-boundary gates; no historical campaigns."""
import argparse
import importlib.util
import json
import os
import shutil
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
spec = importlib.util.spec_from_file_location("bounded", ROOT / "verification/ops-r1/e1/measure.py")
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)
OUT = ROOT / "target/ops-r1" / os.environ.get("OPS_E6C_RUN", "e6c-verification-r7-20261124")
BUILD = ROOT / "target/ops-r1" / os.environ.get("OPS_E6C_BUILD", "e6c-build-r7-20261124")


def main(kind):
    if kind == "binary":
        before = m.sources()
        result = m.bounded(["cargo", "build", "--bin", "clean-ctx", "--all-features", "--message-format=json"], OUT / "binary-build", 600, 8192)
        assert result["exit_code"] == 0 and before == m.sources()
        executable = None
        for line in (OUT / "binary-build/stdout.txt").read_text(encoding="utf-8").splitlines():
            value = json.loads(line)
            if value.get("reason") == "compiler-artifact" and value.get("executable") and value["target"]["name"] == "clean-ctx":
                executable = Path(value["executable"])
        assert executable
        archived = OUT / "clean-ctx.exe"
        shutil.copyfile(executable, archived)
        m.save(OUT / "binary-sources.json", before)
        m.save(OUT / "binary-identity.json", {"executable": str(archived), "sha256": m.sha(archived), "source_manifest_sha256": m.sha(OUT / "binary-sources.json")})
        return
    pin = json.loads((BUILD / "build-identity.json").read_text(encoding="utf-8"))
    assert m.sha(Path(pin["executable"])) == pin["sha256"]
    assert m.sources() == json.loads((BUILD / "build-sources.json").read_text(encoding="utf-8"))
    if kind == "lint":
        commands = [["cargo", "clippy", "--lib", "--all-features", "--", "-D", "warnings"]]
    elif kind == "red":
        commands=[[pin["executable"],"e6c_public_finalization_memory_is_reserved_before_optional_expansion","--test-threads=1","--nocapture"]]
    elif kind == "tests":
        commands = [[pin["executable"], "e6c_", "--test-threads=1", "--nocapture"]]
    elif kind == "stdio":
        binary_root=ROOT/"target/ops-r1"/os.environ.get("OPS_E6C_BINARY_RUN",OUT.name)
        binary = json.loads((binary_root / "binary-identity.json").read_text(encoding="utf-8"))
        assert m.sha(Path(binary["executable"])) == binary["sha256"]
        assert m.sources() == json.loads((binary_root / "binary-sources.json").read_text(encoding="utf-8"))
        commands = [["python", "-X", "utf8", str(ROOT / "verification/ops-r1/e6c/stdio.py"), "--binary", binary["executable"], "--output", str(OUT / "stdio-evidence"),"--profile",os.environ.get("OPS_E6C_PROFILE","full")]]
    else:
        patterns = ["e6b_", "source_observations::tests::", "request_budget::tests::", "direct_identity::tests::", "claim_composition::tests::", "tests_batch", "issue_162_resolver_registration_required_publication_failure_is_not_empty_success", "issue_162_resolver_registration_complete_admission_rejects_missing_extra_crossed"]
        commands = [[pin["executable"], p, "--test-threads=1", "--nocapture"] for p in patterns]
    for i, command in enumerate(commands):
        directory = OUT / f"{kind}-{i}"
        if directory.exists():
            prior=json.loads((directory/"receipt.json").read_text(encoding="utf-8"))
            text=(directory/"stdout.txt").read_text(encoding="utf-8")
            if prior.get("command")==command and prior.get("exit_code")==0 and prior.get("source_manifest_sha256")==pin["source_manifest_sha256"] and __import__("re").search(r"test result: ok\. [1-9]\d* passed; 0 failed",text):
                continue
            directory=directory.with_name(directory.name+"-retry1")
        result = m.bounded(command, directory, 600 if kind == "lint" else 60, 8192 if kind == "lint" else 512)
        result.update(category="E6-C " + kind, test_executable_sha256=pin["sha256"], source_manifest_sha256=pin["source_manifest_sha256"])
        m.save(directory / "receipt.json", result)
        assert result["exit_code"] == (101 if kind=="red" else 0), result
        if kind=="red":
            assert "1 failed" in (directory/"stdout.txt").read_text(encoding="utf-8")
        if kind in ("tests", "controls"):
            import re
            text = (directory / "stdout.txt").read_text(encoding="utf-8")
            match = re.search(r"test result: ok\. (\d+) passed; 0 failed", text)
            assert match and int(match[1]) > 0, "No executed registered tests"


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("kind", choices=["tests", "controls", "lint", "binary", "stdio", "red"])
    main(parser.parse_args().kind)
