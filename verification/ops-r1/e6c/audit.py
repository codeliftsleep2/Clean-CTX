"""Audit real MCP wire receipts and align process samples with request windows."""
import argparse
import hashlib
import json
from pathlib import Path

import jsonschema

ROOT = Path(__file__).resolve().parents[3]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def audit(name):
    directory = ROOT / "target/ops-r1" / name
    output = directory / "stdio-evidence"
    enabled = output / "enabled"
    receipts = json.loads((enabled / "receipts.json").read_text(encoding="utf-8"))
    samples = json.loads((enabled / "memory.json").read_text(encoding="utf-8"))
    schema_root = ROOT / "verification/ops-r1/e6b/schemas"
    schemas = {name: json.loads((schema_root / f"{name}.schema.json").read_text(encoding="utf-8")) for name in ("response", "batch-response")}
    sources = ROOT / "src/mcp/resolver_evidence/schemas"
    for name in ("request", "response", "batch-request", "batch-response"):
        assert (sources / f"{name}.json").read_bytes() == (schema_root / f"{name}.schema.json").read_bytes()
    counts = {}
    stages = {}
    diagnostics = (enabled / "stderr.txt").read_text(encoding="utf-8")
    assert "Failed to parse" not in diagnostics and "Persistence enabled:" not in diagnostics and "[cbm-stderr]" not in diagnostics
    for line in diagnostics.splitlines():
        if line.startswith("[resolver-evidence-v2] counts="):
            value = json.loads(line.split("counts=", 1)[1])
            counts[value["id"]] = value["counts"]
        if line.startswith("[resolver-evidence-v2] stages="):
            value = json.loads(line.split("stages=", 1)[1])
            stages.setdefault(value["id"], []).append(value)
    summary = []
    valid = 0
    for receipt in receipts:
        record = dict(receipt)
        identity = receipt["id"]
        response_path = enabled / f"{identity}.response.json"
        request_path = enabled / f"{identity}.request.json"
        response = json.loads(response_path.read_text(encoding="utf-8"))
        assert receipt["wire_bytes"] == len(response_path.read_bytes())
        request = json.loads(request_path.read_text(encoding="utf-8"))
        if request["method"] != "tools/call":
            continue
        record["request_sha256"] = sha(request_path)
        record["response_sha256"] = sha(response_path)
        if "error" in response:
            record["error"] = response["error"]
        else:
            body = response["result"]
            value = body["structuredContent"]
            assert json.loads(body["content"][0]["text"]) == value
            batch = value["schema"] == "clean-ctx/workspace-query-batch-answer"
            jsonschema.validate(value, schemas["batch-response" if batch else "response"])
            assert value["execution"]["used"]["response_bytes"] >= receipt["wire_bytes"]
            assert value["execution"]["used"]["response_bytes"] <= value["execution"]["effective_limits"]["response_bytes"]
            records = [r["result"] for r in value["results"] if r["status"] == "ok"] if batch else [value]
            for item in records:
                assert item["canonical_registrations"] == []
                assert item["matching_records"]["established"] == item["matching_records"]["returned"] + item["matching_records"]["withheld"]
                assert item["currentness"]["atomic_filesystem_snapshot"] is False
                assert item["currentness"]["continuous_currentness"] is False
                for observation in item["observations"]:
                    source = observation["source"]
                    assert observation["classification"] == "proven" and observation["claim"] == "written_source_mapping"
                    assert observation["claim_scope"]["source"] == source
                    assert sha(Path(source["path"])) == source["sha256"]
            valid += 1
            record.update(execution=value["execution"], items=[dict(matching=item["matching_records"],
                conclusion=item["search_conclusion"]["state"], classification=[c["classification"] for c in item["interpretations"]]) for item in records])
        if identity in counts:
            record["counts"] = counts[identity]
            assert all(counts[identity][k] == 0 for k in ("program_attempts", "program_results", "program_successes", "automatic_type_calls", "compiler_library_calls"))
        record["stages"] = stages.get(identity, [])
        window = [s["working_set_bytes"] for s in samples if receipt["started_seconds"] <= s["seconds"] <= receipt["ended_seconds"]]
        record["sampled_window"] = dict(samples=len(window), minimum=min(window, default=None), maximum=max(window, default=None))
        summary.append(record)
    trials = [r for r in summary if r["case"].startswith("concurrent-")]
    assert len(trials) == 2
    start = min(r["started_seconds"] for r in trials)
    end = max(r["ended_seconds"] for r in trials)
    concurrent_samples = [s["working_set_bytes"] for s in samples if start <= s["seconds"] <= end]
    manifest = json.loads((output / "fixture-identities.json").read_text(encoding="utf-8"))
    for row in manifest:
        assert sha(output / "fixtures" / row["case"] / row["path"]) == row["sha256"]
    for relative in ("app.ts", "tsconfig.json"):
        hashes = {sha(output / "fixtures" / case / relative) for case in ("present", "irrelevant-malformed", "relevant-malformed", "conflicting")}
        assert len(hashes) == 1, "Dependency comparison changed application/configuration"
    result = dict(schema_valid_calls=valid, configured=True, identical_application_configuration=True,
        executable=json.loads((output / "executable.json").read_text()),
        configuration_identities={str(p.relative_to(output)): sha(p) for p in output.glob("*/launch/*.json")},
        process=json.loads((enabled / "lifecycle.json").read_text()), calls=summary,
        concurrent=dict(completion_span_us=int((end-start)*1e6), request_elapsed_ms=[r["execution"]["used"]["elapsed_ms"] for r in trials],
            peak_envelope_sum=sum(r["execution"]["peak_retained_bytes"] for r in trials),
            sampled_process_peak=max(concurrent_samples, default=None), samples=len(concurrent_samples),
            simultaneous_total_ledger_retention_measured=False),
        schemas={p.name: sha(p) for p in sources.glob("*.json")})
    target = directory / "audit.json"
    assert not target.exists()
    target.write_text(json.dumps(result, indent=2), encoding="utf-8")
    print(json.dumps({"receipt": str(target), "schema_valid_calls": valid, "identical_inputs": True}))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", help="existing run directory name below target/ops-r1")
    audit(parser.parse_args().directory)
