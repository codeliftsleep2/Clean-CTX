"""E6-C-only byte-reversible diff against the exact owner-accepted E6-B predecessor."""
import difflib
import hashlib
import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
OUT = ROOT / "target/ops-r1/e6c-20261124"


def sha(data):
    return hashlib.sha256(data).hexdigest()


def main():
    accepted = ROOT / "target/ops-r1/e6b-20261124/review-r2/e6b-only.diff"
    assert sha(accepted.read_bytes()) == "023fc8dc940b36001a68c91f5c4c721c783277e71a4321231d845b94af5147e3"
    baseline = json.loads((OUT / "predecessor.json").read_text(encoding="utf-8"))
    old = {}
    for row in baseline:
        data = (OUT / "before" / row["path"]).read_bytes()
        assert sha(data) == row["sha256"]
        old[row["path"]] = data
    manifest = ROOT / "target/ops-r1/e6b-build-r9-20261124/build-sources.json"
    assert sha(manifest.read_bytes()) == "5c562f36c24591ff39a408b353d184d85c66a81eb95b9946eba0d9a29b518684"
    pins = {r["path"]: r["sha256"] for r in json.loads(manifest.read_text(encoding="utf-8"))}
    names = subprocess.check_output(["git", "diff", "--name-only"], cwd=ROOT).decode().splitlines()
    names += subprocess.check_output(["git", "ls-files", "--others", "--exclude-standard"], cwd=ROOT).decode().splitlines()
    patch, rows = [], []
    for name in sorted(set(names) | set(old)):
        before = old.get(name)
        if name not in old and name in pins:
            raw = subprocess.check_output(["git", "show", "HEAD:" + name], cwd=ROOT)
            before = next(b for b in (raw, raw.replace(b"\r\n", b"\n").replace(b"\n", b"\r\n")) if sha(b) == pins[name])
        path = ROOT / name
        after = path.read_bytes() if path.is_file() else None
        if before == after:
            continue
        assert before is not None or name.startswith(("src/mcp/resolver_evidence/", "verification/ops-r1/e6c/")) or name in (
            "src/mcp/tool_handlers/query_version.rs", "src/tests/mcp/resolver_evidence_e6c.rs",
            "src/tests/ir/e6c_legacy_probe.rs", "src/ir/typescript_resolution/response_capacity.rs",
            "src/ir/typescript_resolution/evidence_telemetry.rs"), name
        a = (before or b"").decode("utf-8").splitlines(keepends=True)
        b = (after or b"").decode("utf-8").splitlines(keepends=True)
        delta = list(difflib.ndiff(a, b))
        assert "".join(difflib.restore(delta, 1)).encode("utf-8") == (before or b"")
        assert "".join(difflib.restore(delta, 2)).encode("utf-8") == (after or b"")
        for line in difflib.unified_diff(a, b, fromfile="a/" + name if before is not None else "/dev/null", tofile="b/" + name if after is not None else "/dev/null"):
            patch.append(line)
            if not line.endswith("\n"):
                patch.append("\n\\ No newline at end of file\n")
        rows.append(dict(path=name, before_sha256=sha(before) if before is not None else None,
                         after_sha256=sha(after) if after is not None else None,
                         before_bytes=len(before or b""), after_bytes=len(after or b"")))
    frozen = ROOT / "src/tests/ir/written_source_observations_e2.rs"
    assert sha(frozen.read_bytes()) == "8a4bd2bf47057c554e94e141e3865b387150e80b7f940f2b341a7b83dcbde3a8"
    for row in json.loads((ROOT / "target/ops-r1/e6b-preparation-20261124/candidate-sources.json").read_text()):
        assert sha((ROOT / row["path"]).read_bytes()) == row["candidate_sha256"]
    destination = OUT / "review"
    destination.mkdir(exist_ok=False)
    (destination / "e6c-only.diff").write_bytes("".join(patch).encode("utf-8"))
    (destination / "identities.json").write_text(json.dumps(rows, indent=2), encoding="utf-8")
    reverse = subprocess.run(["git", "apply", "--reverse", "--check", str(destination / "e6c-only.diff")], cwd=ROOT, capture_output=True, text=True)
    assert reverse.returncode == 0, reverse.stderr
    proof = dict(accepted_e6b_diff_sha256=sha(accepted.read_bytes()), e6c_diff_sha256=sha((destination / "e6c-only.diff").read_bytes()),
        exact_byte_roundtrip=True, git_reverse_check_exit_code=reverse.returncode,
        preexisting_dirty_files_rechecked=len(baseline), untouched_tracked_predecessors_checked_against_e6b_manifest=True,
        frozen_e2_unchanged=True, normative_sources_unchanged=True, worktree_modified_by_verification=False,
        files_in_increment=len(rows))
    (destination / "reversibility.json").write_text(json.dumps(proof, indent=2), encoding="utf-8")
    print(json.dumps(proof))


if __name__ == "__main__":
    main()
