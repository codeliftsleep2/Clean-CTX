"""Bounded registered E4 identity tests and actual resolver/I/O measurements."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
ROOT=Path(__file__).resolve().parents[3]
OUT=ROOT/"target/ops-r1"/os.environ.get("OPS_E4_RUN","e4-20261123")
spec=importlib.util.spec_from_file_location("capture",ROOT/"verification/ops-r1/e1/measure.py")
capture=importlib.util.module_from_spec(spec)
spec.loader.exec_module(capture)

def run(kind):
    build=ROOT/"target/ops-r1"/os.environ.get("OPS_E4_BUILD","e4-build-20261123")
    pin=json.loads((build/"build-identity.json").read_text(encoding="utf-8"))
    assert hashlib.sha256(Path(pin["executable"]).read_bytes()).hexdigest()==pin["sha256"]
    assert capture.sources()==json.loads((build/"build-sources.json").read_text(encoding="utf-8"))
    if kind=="lint":
        result=capture.bounded(["cargo","clippy","--lib","--all-features","--","-D","warnings"],OUT/"lint",600,8192)
        if result.get("exit_code")!=0:raise RuntimeError("lint failed")
        return
    if kind=="perf":
        for repeat in range(5):
            out=OUT/f"perf-{repeat}"
            env=dict(os.environ,OPS_E4_OUTPUT=str(out))
            result=capture.bounded([pin["executable"],"e4_measured_direct_identity_and_partial_results","--test-threads=1","--nocapture"],out,30,512,env)
            result.update(executable_sha256=pin["sha256"],source_manifest_sha256=pin["source_manifest_sha256"])
            capture.save(out/"receipt.json",result)
            if result.get("exit_code")!=0:raise RuntimeError("measurement failed")
        return
    patterns=["direct_identity::tests::"] if kind=="tests" else ["request_budget::tests::","source_observations::tests::",
        "issue_162_resolver_registration_required_publication_failure_is_not_empty_success",
        "issue_162_resolver_registration_complete_admission_rejects_missing_extra_crossed"]
    for index,pattern in enumerate(patterns):
        directory=OUT/f"{kind}-{index}"
        result=capture.bounded([pin["executable"],pattern,"--test-threads=1","--nocapture"],directory,30,512)
        result.update(executable_sha256=pin["sha256"],source_manifest_sha256=pin["source_manifest_sha256"])
        capture.save(directory/"receipt.json",result)
        if result.get("exit_code")!=0:raise RuntimeError("focused child failed; inspect receipt/log")

if __name__=="__main__":
    p=argparse.ArgumentParser(description=__doc__);p.add_argument("kind",choices=["tests","controls","perf","lint"])
    run(p.parse_args().kind)
