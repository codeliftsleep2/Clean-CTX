"""Bounded E5 actual Angular-shaped acquisition/classification checks."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
ROOT=Path(__file__).resolve().parents[3]
OUT=ROOT/"target/ops-r1"/os.environ.get("OPS_E5_RUN","e5-20261123")
spec=importlib.util.spec_from_file_location("capture",ROOT/"verification/ops-r1/e1/measure.py")
capture=importlib.util.module_from_spec(spec);spec.loader.exec_module(capture)

def run(kind):
    build=ROOT/"target/ops-r1"/os.environ.get("OPS_E5_BUILD","e5-build-20261123")
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
            result=capture.bounded([pin["executable"],"e5_real_angular_acquisition_and_pure_classification_measurements","--test-threads=1","--nocapture"],out,30,512,
                dict(os.environ,OPS_E5_OUTPUT=str(out)))
            result.update(executable_sha256=pin["sha256"],source_manifest_sha256=pin["source_manifest_sha256"])
            capture.save(out/"receipt.json",result)
            if result.get("exit_code")!=0:raise RuntimeError("performance child failed")
        return
    patterns=["claim_composition::tests::"] if kind=="tests" else ["direct_identity::tests::","request_budget::tests::","source_observations::tests::",
        "issue_162_resolver_registration_required_publication_failure_is_not_empty_success",
        "issue_162_resolver_registration_complete_admission_rejects_missing_extra_crossed"]
    for index,pattern in enumerate(patterns):
        directory=OUT/f"{kind}-{index}"
        result=capture.bounded([pin["executable"],pattern,"--test-threads=1","--nocapture"],directory,30,512)
        result.update(executable_sha256=pin["sha256"],source_manifest_sha256=pin["source_manifest_sha256"])
        capture.save(directory/"receipt.json",result)
        if result.get("exit_code")!=0:raise RuntimeError("focused child failed")
if __name__=="__main__":
    p=argparse.ArgumentParser(description=__doc__);p.add_argument("kind",choices=["tests","controls","perf","lint"])
    run(p.parse_args().kind)
