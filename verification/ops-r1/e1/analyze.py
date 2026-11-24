"""Read-only analysis of E1 receipts. Does not build or launch experiments."""
import collections
import json
import os
import hashlib
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
OUT = ROOT/"target/ops-r1"/os.environ.get("OPS_E1_RUN","e1-20261123")
rows = json.loads((OUT/"matrix-results.json").read_text())
groups = collections.defaultdict(list)
observations = []
checks = {"all_exit_zero": True, "source_stage_isolated": True,
          "no_canonical_publication": True, "same_application": len({r["app_sha256"] for r in rows}) == 1,
          "same_configuration": len({r["config_sha256"] for r in rows}) == 1}
for r in rows:
    checks["all_exit_zero"] &= r["receipt"].get("exit_code") == 0
    result = r.get("result")
    if not result:
        continue
    for worker in result["workers"]:
        one = worker["stages"][0]
        io = one["metrics"]["io"]
        checks["source_stage_isolated"] &= io["dependency_parses"] == 0 and io["acquisition_results"] == 0 and io["automatic_type_calls"] == 0 and io["library_calls"] == 0
        observations.append(one["evidence"]["observations"])
        checks["no_canonical_publication"] &= worker["publication"]["canonical_registrations"] == []
        for stage in worker["stages"]:
            groups[(r["size"],r["environment"],stage["stage"])].append({
                "ms":stage["stage_ms"], "bytes":stage["metrics"]["io"]["bytes_read"],
                "parses":stage["metrics"]["io"]["parses"], "dependency_parses":stage["metrics"]["io"]["dependency_parses"],
                "acquisitions":stage["metrics"]["io"]["acquisition_results"],
                "witnesses":stage["metrics"]["witness_checks"],"unique_witnesses":stage["metrics"]["unique_witnesses"]})
checks["observations_identical"] = bool(observations) and all(x == observations[0] for x in observations)
summaries = []
for (size,environment,stage), values in groups.items():
    summaries.append({"size":size,"environment":environment,"stage":stage,"samples":len(values),
        "range":{key:[min(x[key] for x in values),max(x[key] for x in values)] for key in values[0]}})
summary = {"runs":len(rows),"completed":sum("result" in r for r in rows),"checks":checks,
    "observed_mapping":observations[0] if observations else [], "stages":summaries,
    "max_sampled_working_set_bytes":max(r["receipt"].get("sampled_peak_working_set_bytes",0) for r in rows),
    "max_job_committed_bytes":max(r["receipt"].get("peak_job_committed_bytes",0) for r in rows),
    "max_completed_deadline_overshoot_ms":max((r["result"]["deadline_overshoot_ms"] for r in rows if "result" in r),default=None),
    "max_active_task_overshoot":max((r["result"].get("measured_active_stage_peak",0)-r["tasks"] for r in rows if "result" in r),default=None),
    "source_claims_observed":len(observations),
    "source_claims_published":sum(len(w["publication"]["source_observations"]) for r in rows if "result" in r for w in r["result"]["workers"]),
    "max_retained_host_source_bytes":max((w["stages"][3]["evidence"].get("retained_host_source_bytes",0) for r in rows if "result" in r for w in r["result"]["workers"]),default=0),
    "max_serialized_witness_bytes":max((w["stages"][3]["evidence"].get("serialized_witness_bytes",0) for r in rows if "result" in r for w in r["result"]["workers"]),default=0),
    "budget_runs":[{"case":f'{r["environment"]}-{r["size"]}-{r["seconds"]}s-{r["tasks"]}tasks',
        "elapsed_ms":r.get("result",{}).get("elapsed_ms"),"deadline_overshoot_ms":r.get("result",{}).get("deadline_overshoot_ms"),
        "exit_code":r["receipt"].get("exit_code"),"watchdog_fired":r["receipt"].get("watchdog_fired",False)} for r in rows]}
(OUT/"analysis.json").write_text(json.dumps(summary,indent=2),encoding="utf-8")
print(json.dumps({k:v for k,v in summary.items() if k not in ["stages","budget_runs"]},indent=2))

# Verify frozen input identities and only the files changed by this measurement task.
manifest = json.loads((OUT/"build-sources.json").read_text())
source_matches = all(hashlib.sha256((ROOT/r["path"]).read_bytes()).hexdigest() == r["sha256"] for r in manifest)
fixture_checks = []
for path in sorted((OUT/"fixtures").glob("*-manifest.json")):
    fixture = path.parent/path.name.removesuffix("-manifest.json")
    inputs = json.loads(path.read_text())
    fixture_checks.append({"fixture":fixture.name,"unchanged":all(hashlib.sha256((fixture/r["path"]).read_bytes()).hexdigest() == r["sha256"] for r in inputs)})
changed = subprocess.check_output(["git","diff","--name-only","-z"],cwd=ROOT).decode().split("\0")
changed += subprocess.check_output(["git","ls-files","--others","--exclude-standard","-z","--","src","verification/ops-r1/e1"],cwd=ROOT).decode().split("\0")
text_checks = []
for name in sorted(set(n for n in changed if n)):
    raw = (ROOT/name).read_bytes()
    text = raw.decode("utf-8",errors="strict")
    text_checks.append({"path":name,"sha256":hashlib.sha256(raw).hexdigest(),"utf8_without_bom":not raw.startswith(b"\xef\xbb\xbf"),
                        "lines":len(text.splitlines()),"within_active_ceiling":name.endswith(".md") or len(text.splitlines()) <= 615})
integrity = {"source_manifest_matches":source_matches,"fixtures":fixture_checks,"changed_text":text_checks,
             "git_diff_check_exit":subprocess.run(["git","diff","--check"],cwd=ROOT,capture_output=True).returncode}
(OUT/"final-integrity.json").write_text(json.dumps(integrity,indent=2),encoding="utf-8")
if not source_matches or not all(r["unchanged"] for r in fixture_checks) or not all(r["utf8_without_bom"] and r["within_active_ceiling"] for r in text_checks) or integrity["git_diff_check_exit"]:
    raise ValueError("Final evidence identity/text check failed")
