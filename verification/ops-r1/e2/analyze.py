"""Read E2 recorded measurements; verify pins and emit reproducible statistics."""
import hashlib
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[3]
OUT=ROOT/"target/ops-r1/e2-implementation-20261123"
rows=[]
for case in sorted((OUT/"perf").iterdir()):
    samples=json.loads((case/"observations.json").read_text(encoding="utf-8"))
    receipt=json.loads((case/"receipt.json").read_text(encoding="utf-8"))
    assert receipt["exit_code"]==0 and len(samples)==30
    for s in samples:
        io=s["capture"]["io"]
        assert all(io[k]==0 for k in ["acquisition_results","dependency_parses","automatic_type_calls","library_calls"])
        assert len(s["observations"])==1
    rows.append({"fixture":case.name,"samples":len(samples),
        "capture_ms":[min(s["capture_ms"] for s in samples),max(s["capture_ms"] for s in samples)],
        "validation_ms":[min(s["validation_ms"] for s in samples),max(s["validation_ms"] for s in samples)],
        "capture_read_bytes":sorted({s["capture"]["io"]["bytes_read"] for s in samples}),
        "validation_read_bytes":sorted({s["validation"]["io"]["bytes_read"] for s in samples}),
        "validation_witness_checks":sorted({s["validation"]["witness_checks"] for s in samples}),
        "record_text_bytes":sorted({s["retained_record_text_bytes"] for s in samples}),
        "serialized_witness_bytes":sorted({s["serialized_witness_bytes"] for s in samples}),
        "peak_working_set_bytes":receipt["sampled_peak_working_set_bytes"],
        "peak_job_committed_bytes":receipt["peak_job_committed_bytes"]})
pin=json.loads((ROOT/"target/ops-r1/e2-green-20261123/build-identity.json").read_text(encoding="utf-8"))
manifest=json.loads((ROOT/"target/ops-r1/e2-green-20261123/build-sources.json").read_text(encoding="utf-8"))
assert all(hashlib.sha256((ROOT/r["path"]).read_bytes()).hexdigest()==r["sha256"] for r in manifest)
assert hashlib.sha256(Path(pin["executable"]).read_bytes()).hexdigest()==pin["sha256"]
summary={"rows":rows,"total_samples":sum(r["samples"] for r in rows),"pins_verified":True,
         "frozen_regression_sha256":hashlib.sha256((ROOT/"src/tests/ir/written_source_observations_e2.rs").read_bytes()).hexdigest()}
(OUT/"analysis.json").write_text(json.dumps(summary,indent=2),encoding="utf-8")
print(json.dumps(summary,indent=2))
