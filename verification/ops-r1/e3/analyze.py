"""Analyze actual E3 budget tests and controlled worker/quantum measurements."""
import hashlib
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[3]
OUT=ROOT/"target/ops-r1/e3-complete-20261123"
rows=[]
for index in range(5):
    directory=OUT/f"perf-{index}"
    receipt=json.loads((directory/"receipt.json").read_text(encoding="utf-8"))
    assert receipt["exit_code"]==0
    data=json.loads((directory/"measurements.json").read_text(encoding="utf-8"))
    rows.append({"repeat":index,"measurements":data,"receipt":receipt})
workers=[m for r in rows for m in r["measurements"] if "workers" in m]
overshoot=[m["overshoot_ms"] for r in rows for m in r["measurements"] if m.get("kind")=="non_interruptible_quantum"]
cancel=[m["cancel_to_return_ms"] for r in rows for m in r["measurements"] if m.get("kind")=="inflight_cancel"]
summary={"repeats":5,"worker_comparisons":[],"deadline_overshoot_ms":[min(overshoot),max(overshoot)],
    "cancel_to_return_ms":[min(cancel),max(cancel)],
    "peak_working_set_bytes":max(r["receipt"]["sampled_peak_working_set_bytes"] for r in rows),
    "peak_job_committed_bytes":max(r["receipt"]["peak_job_committed_bytes"] for r in rows)}
for count in [1,2]:
    matches=[m for m in workers if m["workers"]==count]
    assert all(m["support_bytes"]==65536 and m["completed_units"]==8 and m["active_peak"]<=count and m["observations_survived"] for m in matches)
    summary["worker_comparisons"].append({"workers":count,"elapsed_ms":[min(m["elapsed_ms"] for m in matches),max(m["elapsed_ms"] for m in matches)],
        "active_peak":max(m["active_peak"] for m in matches),"reserved_retained_peak":max(m["reserved_retained_peak"] for m in matches),
        "support_bytes":65536,"completed_units":8})
manifest=json.loads((ROOT/"target/ops-r1/e3-build-r4-20261123/build-sources.json").read_text(encoding="utf-8"))
assert all(hashlib.sha256((ROOT/r["path"]).read_bytes()).hexdigest()==r["sha256"] for r in manifest)
summary["source_pins_current"]=True
(OUT/"analysis.json").write_text(json.dumps(summary,indent=2),encoding="utf-8")
print(json.dumps(summary,indent=2))
