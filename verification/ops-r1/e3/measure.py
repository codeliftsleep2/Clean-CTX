"""Explicit bounded E3 executions; no public runtime or production default profile."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
ROOT=Path(__file__).resolve().parents[3]
OUT=ROOT/"target/ops-r1/e3-complete-20261123"
spec=importlib.util.spec_from_file_location("capture",ROOT/"verification/ops-r1/e1/measure.py")
capture=importlib.util.module_from_spec(spec)
spec.loader.exec_module(capture)

def main(kind):
    build=ROOT/"target/ops-r1"/os.environ.get("OPS_E3_BUILD","e3-build-r4-20261123")
    identity=json.loads((build/"build-identity.json").read_text(encoding="utf-8"))
    exe=Path(identity["executable"])
    assert hashlib.sha256(exe.read_bytes()).hexdigest()==identity["sha256"]
    assert capture.sources()==json.loads((build/"build-sources.json").read_text(encoding="utf-8"))
    if kind=="lint":
        command=["cargo","clippy","--lib","--all-features","--","-D","warnings"]
        result=capture.bounded(command,OUT/"lint",600,8192)
    elif kind=="perf":
        for index in range(5):
            directory=OUT/f"perf-{index}"
            env=dict(os.environ,OPS_E3_OUTPUT=str(directory))
            result=capture.bounded([str(exe),"e3_request_worker_and_cancellation_measurements","--test-threads=1","--nocapture"],directory,15,512,env)
            result.update(executable_sha256=identity["sha256"],source_manifest_sha256=identity["source_manifest_sha256"])
            capture.save(directory/"receipt.json",result)
            if result.get("exit_code")!=0:raise RuntimeError("E3 measurement failed")
        return
    else:
        patterns=["request_budget::tests::"] if kind=="tests" else ["source_observations::tests::","e2_admission_survives_optional_semantic_failure",
            "issue_162_resolver_registration_required_publication_failure_is_not_empty_success",
            "issue_162_resolver_registration_complete_admission_rejects_missing_extra_crossed"]
        for index,pattern in enumerate(patterns):
            result=capture.bounded([str(exe),pattern,"--test-threads=1","--nocapture"],OUT/f"{kind}-{index}",30,512)
            result.update(executable_sha256=identity["sha256"],source_manifest_sha256=identity["source_manifest_sha256"])
            capture.save(OUT/f"{kind}-{index}/receipt.json",result)
            if result.get("exit_code")!=0:raise RuntimeError("Focused child did not pass")
        return
    if result.get("exit_code")!=0:raise RuntimeError("Scoped lint failed")

if __name__=="__main__":
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("kind",choices=["tests","controls","perf","lint"])
    main(p.parse_args().kind)
