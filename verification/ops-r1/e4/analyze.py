"""Read exact E4 direct resolver measurements and verify frozen fixture bytes."""
import hashlib
import json
import os
from pathlib import Path
ROOT=Path(__file__).resolve().parents[3]
OUT=ROOT/"target/ops-r1"/os.environ.get("OPS_E4_RUN","e4-final-20261123")
rows=[]
for repeat in range(5):
    directory=OUT/f"perf-{repeat}"
    receipt=json.loads((directory/"receipt.json").read_text(encoding="utf-8"))
    assert receipt["exit_code"]==0
    data=json.loads((directory/"measurements.json").read_text(encoding="utf-8"))
    for row in data:
        io=row["work"]["io"]
        assert io["acquisition_results"]==0 and io["automatic_type_calls"]==0 and io["library_calls"]==0
        assert row["observation_count"]==1
        if row["form"]=="local":assert row["local_binding"] and row["target"] and row["validated"]
        if row["form"]=="import":assert row["target"] and not row["local_binding"] and row["validated"]
        if row["form"] in ["missing","budget"]:assert row["target"] is None
        for source in row["fixture_files"]:
            copied=directory/"fixtures"/row["form"]/source["relative"]
            assert hashlib.sha256(copied.read_bytes()).hexdigest()==source["sha256"]
        rows.append(row)
summary={"samples":len(rows),"forms":[],"all_no_program_acquisition":True,"all_original_observations_preserved":True}
for form in ["local","import","missing","budget"]:
    chosen=[r for r in rows if r["form"]==form]
    summary["forms"].append({"form":form,
        "capture_ms":[min(r["capture_ms"] for r in chosen),max(r["capture_ms"] for r in chosen)],
        "validation_ms":[min(r["validation_ms"] for r in chosen),max(r["validation_ms"] for r in chosen)],
        "actual_read_bytes":sorted({r["work"]["io"]["bytes_read"] for r in chosen}),
        "dependency_parses":sorted({r["work"]["io"]["dependency_parses"] for r in chosen}),
        "parsed_units":sorted({r["work"]["io"]["parses"] for r in chosen}),
        "validation_bytes":sorted({r["validation"]["io"]["validation_bytes"] for r in chosen}),
        "witness_checks":sorted({r["validation"]["witness_checks"] for r in chosen}),
        "module_edges":sorted({r["edges"] for r in chosen}),
        "ledger_edges":sorted({r["reserved_edges"] for r in chosen}),
        "reserved_retained_peak":max(r["reserved_retained_peak"] for r in chosen),
        "witness_serialized_bytes":sorted({r["witness_serialized_bytes"] for r in chosen}),
        "max_deadline_overshoot_ms":max(r["deadline_overshoot_ms"] for r in chosen),
        "obligations":sorted({r["obligations"] for r in chosen})})
summary["peak_working_set_bytes"]=max(json.loads((OUT/f"perf-{i}/receipt.json").read_text(encoding="utf-8"))["sampled_peak_working_set_bytes"] for i in range(5))
summary["peak_job_committed_bytes"]=max(json.loads((OUT/f"perf-{i}/receipt.json").read_text(encoding="utf-8"))["peak_job_committed_bytes"] for i in range(5))
(OUT/"analysis.json").write_text(json.dumps(summary,indent=2),encoding="utf-8")
print(json.dumps(summary,indent=2))
