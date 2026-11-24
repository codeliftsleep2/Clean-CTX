"""Read E5 acquisition/validation/classification counters separately."""
import hashlib
import json
import os
from pathlib import Path
ROOT=Path(__file__).resolve().parents[3]
OUT=ROOT/"target/ops-r1"/os.environ.get("OPS_E5_RUN","e5-final-20261123")
rows=[];fixtures=[]
for repeat in range(5):
    directory=OUT/f"perf-{repeat}"
    receipt=json.loads((directory/"receipt.json").read_text(encoding="utf-8"));assert receipt["exit_code"]==0
    for row in json.loads((directory/"measurements.json").read_text(encoding="utf-8")):
        io=row["classification_io"]["io"]
        assert io["bytes_read"]==0 and io["parses"]==0 and io["acquisition_results"]==0 and row["classification_io"]["witness_checks"]==0
        assert row["acquisition"]["io"]["acquisition_results"]==0 and row["acquisition"]["io"]["automatic_type_calls"]==0 and row["acquisition"]["io"]["library_calls"]==0
        assert "StronglySupported" in row["classification"] if row["form"]=="angular" else "Partial" in row["classification"]
        rows.append(row)
    fixtures.extend({"path":str(p.relative_to(OUT)),"sha256":hashlib.sha256(p.read_bytes()).hexdigest(),"bytes":p.stat().st_size} for p in sorted((directory/"fixtures").rglob('*')) if p.is_file())
summary={"samples":len(rows),"classification_calls":len(rows)*1001,"classification_io_zero":True,"forms":[]}
for form in ["angular","lookalike","missing"]:
    chosen=[r for r in rows if r["form"]==form]
    summary["forms"].append({"form":form,
        "acquisition_ms":[min(r["acquisition_ms"] for r in chosen),max(r["acquisition_ms"] for r in chosen)],
        "admission_ms":[min(r["admission_ms"] for r in chosen),max(r["admission_ms"] for r in chosen)],
        "classification_1001_ms":[min(r["classify_1001_ms"] for r in chosen),max(r["classify_1001_ms"] for r in chosen)],
        "read_bytes":sorted({r["acquisition"]["io"]["bytes_read"] for r in chosen}),
        "parses":sorted({r["acquisition"]["io"]["parses"] for r in chosen}),
        "dependency_parses":sorted({r["acquisition"]["io"]["dependency_parses"] for r in chosen}),
        "witness_checks":sorted({r["admission"]["witness_checks"] for r in chosen}),
        "validation_bytes":sorted({r["admission"]["io"]["validation_bytes"] for r in chosen}),
        "reserved_edges":sorted({r["resolution_edges"] for r in chosen}),
        "reserved_retained_peak":max(r["retained_reservation_peak"] for r in chosen)})
summary["peak_working_set_bytes"]=max(json.loads((OUT/f"perf-{i}/receipt.json").read_text(encoding="utf-8"))["sampled_peak_working_set_bytes"] for i in range(5))
(OUT/"analysis.json").write_text(json.dumps(summary,indent=2),encoding="utf-8")
(OUT/"fixture-manifest.json").write_text(json.dumps(fixtures,indent=2),encoding="utf-8")
print(json.dumps(summary,indent=2))
