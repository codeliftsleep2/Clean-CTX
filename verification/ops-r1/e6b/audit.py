"""Audit frozen integrated receipts against the exact reviewed response oracle."""
import argparse
import hashlib
import json
from pathlib import Path

import jsonschema

ROOT = Path(__file__).resolve().parents[3]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def audit(run):
    output = ROOT / "target/ops-r1" / run
    assert output.resolve().is_relative_to((ROOT / "target/ops-r1").resolve())
    schemas = ROOT / "verification/ops-r1/e6b/schemas"
    prepared = ROOT / "target/ops-r1/e6b-preparation-20261124"
    pins = []
    for path in sorted(schemas.glob("*.json")):
        original = prepared / path.name
        assert path.read_bytes() == original.read_bytes(), path
        pins.append({"path": str(path.relative_to(ROOT)), "sha256": sha(path)})
    schema = json.loads((schemas / "response.schema.json").read_text(encoding="utf-8"))
    validator = jsonschema.Draft202012Validator(schema)
    receipts = json.loads((output / "measure-0/measurements.json").read_text(encoding="utf-8"))
    summary = []
    fixture_pins = []
    common = []
    for receipt in receipts:
        case = receipt["case"]
        root = output / "measure-0/fixtures" / case
        for path in sorted(root.rglob("*")):
            if path.is_file():
                fixture_pins.append({"case": case, "path": str(path.relative_to(root)),
                                     "sha256": sha(path), "bytes": path.stat().st_size})
        dto = receipt.get("dto")
        if dto is not None:
            errors = sorted(validator.iter_errors(dto), key=lambda e: str(list(e.absolute_path)))
            assert not errors, [(list(e.absolute_path), e.message) for e in errors[:8]]
            assert dto["canonical_registrations"] == []
            io = receipt["io"]["io"]
            assert io["acquisition_results"] == io["automatic_type_calls"] == io["library_calls"] == 0
            if case in ("present", "irrelevant-malformed", "relevant-malformed", "conflicting"):
                common.append((receipt["source_sha256"], receipt["config_sha256"]))
            summary.append({"case": case, "elapsed_us": receipt["elapsed_us"],
                            "returned": dto["matching_records"]["returned"],
                            "classification": [v["classification"] for v in dto["interpretations"]],
                            "conclusion": dto["search_conclusion"]["state"],
                            "execution": dto["execution"]["state"],
                            "costs": receipt["measurements"], "io": receipt["io"],
                            "final_ledger": receipt["final_ledger"],
                            "cancellation_to_completion_us": receipt.get("cancellation_to_completion_us"),
                            "optional_deadline_ms":receipt.get("optional_deadline_ms")})
        else:
            if case.startswith("workers-"):
                assert len(receipt["results"]) == 2 and all(r["ok"] for r in receipt["results"])
                for result in receipt["results"]:
                    io=result["io"]["io"]
                    assert io["acquisition_results"] == io["automatic_type_calls"] == io["library_calls"] == 0
            summary.append(receipt)
    assert len(common) == 4 and len(set(common)) == 1, "application/configuration drift"
    result = {"schemas": pins, "fixture_identities": fixture_pins,
              "identical_application_and_configuration": True,
              "schema_valid_responses": sum(r.get("dto") is not None for r in receipts),
              "cases": summary}
    destination = output / "audit.json"
    assert not destination.exists(), "Use a fresh audit receipt"
    destination.write_text(json.dumps(result, indent=2), encoding="utf-8")
    print(json.dumps({"receipt": str(destination), "responses": result["schema_valid_responses"],
                      "identical_inputs": True}))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("run", help="existing evidence directory name below target/ops-r1")
    audit(parser.parse_args().run)
