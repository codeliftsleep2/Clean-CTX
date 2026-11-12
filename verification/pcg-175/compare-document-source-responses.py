"""Independently reconstruct paired production responses; never launches a runtime.

Run after invoke-workspace-documents.ps1 with seven alternating inline/ref queries.
The CSV records exact Rust-tokenizer costs from that driver's serialized MCP results.
"""
import argparse
import csv
import hashlib
import json
from pathlib import Path


def load(path):
    return json.loads(path.read_text(encoding="utf-8"))


def reconstruct(payload):
    assert payload["version"] == 2
    assert payload.pop("response_format") == "source_refs_v1"
    payload["version"] = 1
    pool = payload.pop("sources")
    by_ref = {s["source_ref"]: s for s in pool}
    assert len(by_ref) == len(pool)
    assert len({(s["path"], s["sha256"]) for s in pool}) == len(pool)
    used = set()
    for source in pool:
        assert hashlib.sha256(source["text"].encode("utf-8")).hexdigest() == source["sha256"]

    def resolve(owner, hash_field):
        reference = owner.pop("source_ref")
        source = by_ref[reference]
        assert owner["path"] == source["path"]
        assert owner[hash_field] == source["sha256"]
        used.add(reference)
        return source["text"]

    for node in payload["documents"]:
        node["source_text"] = resolve(node, "source_sha256")
        for evidence in node["evidence"]:
            if "source_ref" in evidence:
                evidence["text"] = resolve(evidence, "sha256")
    assert used == set(by_ref)
    return payload


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("capture", type=Path)
    parser.add_argument("legacy_capture", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    receipt = load(args.capture / "receipt.json")
    assert receipt["process_exit_code"] == 0 and not receipt["error"]
    queries = receipt["queries"]
    assert len(queries) == 14
    rows = []
    generations = set()
    for index in range(7):
        inline, packed = queries[2 * index:2 * index + 2]
        a, b = inline["request"], packed["request"]
        legacy = load(args.capture / f"{a}.payload.json")
        source_refs = load(args.capture / f"{b}.payload.json")
        assert inline["node"] == packed["node"]
        assert legacy["generation"] == source_refs["generation"]
        assert legacy["catalog_current"] and source_refs["catalog_current"]
        assert legacy["result_state"] == source_refs["result_state"] == "ready"
        assert legacy["context"]["complete"] and source_refs["context"]["complete"]
        assert (args.capture / f"{a}.serialized-result.json").read_bytes() == (
            args.legacy_capture / f"{index + 2}.serialized-result.json"
        ).read_bytes(), "Default transport changed from pre-edit production binary"
        pool_size = len(source_refs["sources"])
        assert reconstruct(source_refs) == legacy, "Full payload reconstruction differs"
        generations.add(legacy["generation"])
        before, after = inline["serialized_tokens"], packed["serialized_tokens"]
        context = legacy["context"]
        rows.append(dict(
            node=inline["node"], generation=legacy["generation"],
            inline_tokens=before, source_refs_tokens=after, saved_tokens=before-after,
            saved_percent=round(100 * (before-after) / before, 3),
            nodes=len(legacy["documents"]), edges=len(context["relationships"]),
            paths=len(context["paths"]), unique_source_payloads=pool_size,
            reconstruction="exact", revisions="exact", metadata="exact",
            pre_edit_default_bytes="exact",
        ))
    assert len(generations) == 1, "Paired campaign changed source generation"
    with args.output.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(rows[0]))
        writer.writeheader()
        writer.writerows(rows)
    print(json.dumps(rows, indent=2))


if __name__ == "__main__":
    main()
