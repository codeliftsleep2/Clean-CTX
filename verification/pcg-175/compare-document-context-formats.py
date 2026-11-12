"""Compare any alternating inline_v1/source_refs_v1 production context pairs.

Uses the independent B2 reconstruction consumer. Does not launch a runtime,
change a profile or read/write a database. Counts come from the stdio driver's
exact Rust-tokenizer measurement of serialized MCP result bytes.
"""
import argparse
import csv
import importlib.util
import json
import sys
from pathlib import Path


def load(path):
    return json.loads(path.read_text(encoding="utf-8"))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("capture", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    consumer_path = Path(__file__).with_name("compare-document-source-responses.py")
    sys.dont_write_bytecode = True
    spec = importlib.util.spec_from_file_location("b2_source_consumer", consumer_path)
    consumer = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(consumer)
    receipt = load(args.capture / "receipt.json")
    assert receipt["process_exit_code"] == 0 and not receipt["error"]
    queries = receipt["queries"]
    assert queries and len(queries) % 2 == 0
    generations = set()
    rows = []
    for offset in range(0, len(queries), 2):
        inline, packed = queries[offset:offset + 2]
        a, b = inline["request"], packed["request"]
        requests = [load(args.capture / f"{n}.request.json")["params"]["arguments"]
                    for n in [a, b]]
        assert requests[0].get("responseFormat", "inline_v1") == "inline_v1"
        assert requests[1].pop("responseFormat") == "source_refs_v1"
        requests[0].pop("responseFormat", None)
        assert requests[0] == requests[1]
        payload = load(args.capture / f"{a}.payload.json")
        projected = load(args.capture / f"{b}.payload.json")
        for value in [payload, projected]:
            assert value["catalog_current"] and value["result_state"] == "ready"
            assert value["context"]["complete"] and not value["diagnostics"]
            generations.add(value["generation"])
        sources = len(projected["sources"])
        assert consumer.reconstruct(projected) == payload
        before, after = inline["serialized_tokens"], packed["serialized_tokens"]
        assert before is not None and after is not None
        rows.append(dict(
            node=inline["node"], generation=payload["generation"],
            inline_tokens=before, source_refs_tokens=after,
            saved_tokens=before-after, saved_percent=round(100*(before-after)/before, 3),
            nodes=len(payload["documents"]), unique_sources=sources,
            edges=len(payload["context"]["relationships"]),
            paths=len(payload["context"]["paths"]),
            reconstruction="exact", revisions="exact", metadata="exact",
            completeness="current and complete", tokenizer=requests[0]["tokenizer"],
        ))
    assert len(generations) == 1
    with args.output.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(rows[0]))
        writer.writeheader()
        writer.writerows(rows)
    print(json.dumps(rows, indent=2))


if __name__ == "__main__":
    main()
