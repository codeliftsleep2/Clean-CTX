"""Bounded E2 registered-test execution; never invokes a public v2 tool."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
OUT = ROOT/"target/ops-r1/e2-implementation-20261123"
spec = importlib.util.spec_from_file_location("capture",ROOT/"verification/ops-r1/e1/measure.py")
capture = importlib.util.module_from_spec(spec)
spec.loader.exec_module(capture)


def execute(kind):
    pin_root = ROOT/"target/ops-r1"/("e2-red-20261123" if kind=="red" else "e2-green-20261123")
    pin=json.loads((pin_root/"build-identity.json").read_text(encoding="utf-8"))
    exe=Path(pin["executable"])
    assert hashlib.sha256(exe.read_bytes()).hexdigest()==pin["sha256"]
    assert capture.sources()==json.loads((pin_root/"build-sources.json").read_text(encoding="utf-8"))
    if kind=="perf":
        for size in ["small","growth"]:
            for case in ["valid","irrelevant-malformed","relevant-malformed","conflict"]:
                workspace=ROOT/"target/ops-r1/e1-r2-20261123/fixtures"/f"{case}-{size}"
                frozen=json.loads((workspace.parent/(workspace.name+"-manifest.json")).read_text(encoding="utf-8"))
                assert all(hashlib.sha256((workspace/r["path"]).read_bytes()).hexdigest()==r["sha256"] for r in frozen)
                directory=OUT/"perf"/workspace.name
                env=dict(os.environ,OPS_E2_WORKSPACE=str(workspace),OPS_E2_OUTPUT=str(directory))
                receipt=capture.bounded([str(exe),"e2_observation_cost","--test-threads=1","--nocapture"],directory,15,512,env)
                receipt.update(category="E2 observation-only performance, not field acceptance",executable_sha256=pin["sha256"],
                    source_manifest_sha256=pin["source_manifest_sha256"],fixture_manifest_sha256=hashlib.sha256((workspace.parent/(workspace.name+"-manifest.json")).read_bytes()).hexdigest())
                capture.save(directory/"receipt.json",receipt)
                if receipt.get("exit_code")!=0:raise RuntimeError("Performance child did not pass")
        return
    if kind=="lint":
        capture.bounded(["cargo","clippy","--lib","--all-features","--","-D","warnings"],OUT/"lint",600,8192)
        return
    if kind=="extra":
        for pattern in ["e2_admission_survives_optional_semantic_failure","e2_escaped_keys_and_utf8_spans_are_conservative",
                        "issue_162_resolver_registration_required_publication_failure_is_not_empty_success",
                        "issue_162_resolver_registration_complete_admission_rejects_missing_extra_crossed"]:
            capture.bounded([str(exe),pattern,"--test-threads=1","--nocapture"],OUT/pattern,30,512)
        return
    pattern="e2_source_observations_do_not_acquire_dependency_programs" if kind=="red" else "source_observations::tests::"
    receipt=capture.bounded([str(exe),pattern,"--test-threads=1","--nocapture"],OUT/kind,30,512)
    receipt.update(category="E2 mechanical baseline RED" if kind=="red" else "E2 focused registered tests",
                   executable_sha256=pin["sha256"],source_manifest_sha256=pin["source_manifest_sha256"])
    capture.save(OUT/kind/"receipt.json",receipt)


if __name__=="__main__":
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("kind",choices=["red","green","perf","extra","lint"])
    execute(p.parse_args().kind)
