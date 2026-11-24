"""Build an E6-B-only review diff and prove byte reversibility without worktree writes."""
import difflib
import hashlib
import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
BASE = ROOT / "target/ops-r1/e6b-20261124"


def sha(data):
    return hashlib.sha256(data).hexdigest()


def main():
    old = {}
    baseline = json.loads((BASE / "predecessor.json").read_text(encoding="utf-8"))
    for row in baseline:
        data = (BASE / "before" / row["path"]).read_bytes()
        assert sha(data) == row["sha256"]
        old[row["path"]] = data
    helper = "src/tests/ir/typescript_windows_host_issue_162.rs"
    raw = subprocess.check_output(["git", "show", "HEAD:" + helper], cwd=ROOT)
    prior_build = json.loads((ROOT / "target/ops-r1/e6b-build-r2-20261124/build-sources.json").read_text(encoding="utf-8"))
    expected = next(r["sha256"] for r in prior_build if r["path"] == helper)
    choices = [raw, raw.replace(b"\r\n", b"\n").replace(b"\n", b"\r\n")]
    old[helper] = next(data for data in choices if sha(data) == expected)
    names = subprocess.check_output(["git", "diff", "--name-only"], cwd=ROOT).decode().splitlines()
    names += subprocess.check_output(["git", "ls-files", "--others", "--exclude-standard"], cwd=ROOT).decode().splitlines()
    changes, proof, patch = [], [], []
    for name in sorted(set(names) | set(old)):
        path = ROOT / name
        before = old.get(name)
        after = path.read_bytes() if path.is_file() else None
        if before == after:
            continue
        # All other preexisting modifications were frozen at mission entry.
        assert before is not None or name.startswith(("src/angular_meta/resolver_evidence/",
            "verification/ops-r1/e6b/")) or name in (
                "src/ir/typescript_resolution/observation_bounded.rs",
                "src/ir/typescript_resolution/direct_evidence.rs",
                "src/tests/ir/resolver_evidence_e6b.rs",
                "src/tests/ir/resolver_evidence_e6b_hooks.rs",
                "src/tests/ir/resolver_evidence_e6b_measurements.rs",
                "src/tests/ir/resolver_evidence_e6b_admission.rs"), name
        a = (before or b"").decode("utf-8").splitlines(keepends=True)
        b = (after or b"").decode("utf-8").splitlines(keepends=True)
        delta = list(difflib.ndiff(a, b))
        assert "".join(difflib.restore(delta, 1)).encode("utf-8") == (before or b"")
        assert "".join(difflib.restore(delta, 2)).encode("utf-8") == (after or b"")
        row = {"path": name, "before_sha256": sha(before) if before is not None else None,
               "after_sha256": sha(after) if after is not None else None,
               "before_bytes": len(before or b""), "after_bytes": len(after or b"")}
        changes.append(row)
        chunk=list(difflib.unified_diff(
            (before or b"").decode("utf-8").splitlines(keepends=True),
            (after or b"").decode("utf-8").splitlines(keepends=True),
            fromfile="a/" + name if before is not None else "/dev/null",
            tofile="b/" + name if after is not None else "/dev/null"))
        for line in chunk:
            patch.append(line)
            if not line.endswith("\n"):
                patch.append("\n\\ No newline at end of file\n")
    for row in baseline:
        proof.append({"path": row["path"], "before_sha256": row["sha256"],
                      "current_sha256": sha((ROOT / row["path"]).read_bytes())})
    frozen = "src/tests/ir/written_source_observations_e2.rs"
    assert sha((ROOT / frozen).read_bytes()) == "8a4bd2bf47057c554e94e141e3865b387150e80b7f940f2b341a7b83dcbde3a8"
    candidate = json.loads((ROOT / "target/ops-r1/e6b-preparation-20261124/candidate-sources.json").read_text(encoding="utf-8"))
    for row in candidate:
        assert sha((ROOT / row["path"]).read_bytes()) == row["candidate_sha256"], row["path"]
    destination = BASE / "review-r2"
    destination.mkdir(exist_ok=False)
    (destination / "e6b-only.diff").write_bytes("".join(patch).encode("utf-8"))
    (destination / "identities.json").write_text(json.dumps(changes, indent=2), encoding="utf-8")
    (destination / "reversibility.json").write_text(json.dumps({
        "exact_byte_roundtrip": True, "worktree_modified_by_verification": False,
        "baseline_files_rechecked": len(proof), "baseline": proof,
        "frozen_e2_unchanged": True, "normative_candidate_unchanged": True,
        "helper_predecessor_verified_against_r2_build": expected,
        "files_in_increment": len(changes)}, indent=2), encoding="utf-8")
    print(json.dumps({"files": len(changes), "receipt": str(destination)}))


if __name__ == "__main__":
    main()
