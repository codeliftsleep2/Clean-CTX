"""Offline evidence accounting only; no parser, test execution or runtime authority."""
import argparse
import collections
import hashlib
import json
import re
from pathlib import Path


def read(path):
    return json.loads(path.read_text(encoding="utf-8"))


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


# Finite human-inspected operations. This is not a safe-file allowlist or a
# generic rule approving scans. All unmatched records remain blocking review.
REVIEWED = [
    ("src/workspace/index/declarations.rs", "register_declaration", "occurrences.iter().any", "D1", "MEASURED DEFECT", "FIX", "add_edges self-Defines declaration branch", "exact entity.file within EntityKey", "physical declaration occurrences", "n(n-1)/2 measured through 256"),
    ("src/workspace/index/declarations.rs", "register_declaration", "keys.iter().any", "D2", "MEASURED DEFECT", "FIX", "add_edges self-Defines declaration branch", "EntityKey within asserting publisher", "unique per-file declaration keys", "n(n-1)/2 measured through 256"),
    ("src/workspace/index.rs", "resolve_selector", "subject_keys.contains", "S1", "STRUCTURAL RISK", "MONITOR", "angular_templates::selector_candidates -> resolve_selector", "exact EntityKey, first-seen output order", "incoming assertions and unique subjects", "source-derived O(assertions*subjects), unmeasured"),
    ("src/cbm/client.rs", "extract_trace_edges", "edges.iter().any", "T1", "STRUCTURAL RISK", "MONITOR", "CbmClient::trace_path/collection -> decoder::trace", "exact from/to pair, provider order", "decoded direct pairs and distinct pairs", "source-derived O(pairs*distinct), unmeasured"),
    ("src/angular_meta/reactive_forms_normalize.rs", "normalize_artifacts", "normalized.iter().position", "F1", "STRUCTURAL RISK", "MONITOR", "AngularMetaLayer extraction -> reactive_forms -> normalize_artifacts", "form name or field name+kind, stable order", "artifacts, unique compatible artifacts", "source-derived growing scan, unmeasured"),
    ("src/angular_meta/reactive_forms_normalize.rs", "merge_fields", "existing.iter_mut().find", "F2", "STRUCTURAL RISK", "MONITOR", "reactive_forms/normalize_artifacts -> merge_artifact -> merge_fields", "field name+kind, incompatible evidence retained", "field width and recursive depth", "source-derived growing join, unmeasured"),
    ("src/compression/opcodes.rs", "is_primitive_opcode", "PRIMITIVE_OPCODES.iter().any", "O1", "JUSTIFIED WORK", "ACCEPT", "SymbolDictionary::format_footer -> is_primitive_opcode", "fixed primitive opcode vocabulary", "table is source-defined, not accumulated input", "fixed vocabulary scan per lookup; caller repetition separate"),
]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("baseline", type=Path, help="frozen operator packet directory")
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--parser-supplement", type=Path)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[3]
    baseline = args.baseline.resolve()
    output = args.output.resolve()
    target = (root / "target/af-perf-002").resolve()
    assert baseline.is_relative_to(target) and output.is_relative_to(target)
    assert not output.exists(), "Preserve old accounting; use a fresh output directory"
    files = read(baseline / "census/files.json")
    sites = read(baseline / "census/sites.json")
    assert len({f["path"] for f in files}) == len(files)
    hashes = {f["path"]: f["sha256"].lower() for f in files}
    current = {}
    for path, expected in hashes.items():
        actual = root / path
        current[path] = digest(actual) if actual.is_file() else None
    drift = {p for p in hashes if current[p] != hashes[p]}
    parse_errors = {f["path"] for f in files if f["parse_error"]}
    rows = []
    for ordinal, site in enumerate(sites, 1):
        assert site["path"] in hashes
        rows.append({"id": f"C{ordinal:05d}", "original_ordinal": ordinal,
                     "original_record_sha256": hashlib.sha256(json.dumps(site, sort_keys=True).encode()).hexdigest(),
                     "source_sha256": hashes[site["path"]], "site": site,
                     "accountability": "MACRO REVIEW" if site["kind"] == "unexpanded-macro-review" else "REVIEW REQUIRED",
                     "blocking": True, "operation": None,
                     "reason": "unexpanded macro: expansion/semantic/caller review unresolved" if site["kind"] == "unexpanded-macro-review" else "ownership/caller/growth classification unresolved"})
    operations = []
    for path, function, expression, name, classification, disposition, caller, identity, dimensions, complexity in REVIEWED:
        if path in drift or path in parse_errors:
            continue
        matches = [r for r in rows if r["site"]["path"] == path
                   and r["site"]["function"] == function
                   and "".join(r["site"]["expression"].split()) == expression]
        assert len(matches) == 1, f"Reviewed anchor changed: {name}"
        canonical = matches[0]
        canonical.update(accountability="INCIDENTAL/BOUNDED" if name == "O1" else "PRODUCTION OPERATION",
                         blocking=False, operation=name, reason=complexity)
        linked = [canonical["id"]]
        # Only reviewed overlapping iterator syntax is linked. Same-function
        # membership is never enough to exclude an unrelated operation.
        for row in rows:
            s = row["site"]
            candidate = "".join(s["expression"].split())
            if row is not canonical and s["path"] == path and s["function"] == function and s["byte"] == canonical["site"]["byte"] and candidate and expression.startswith(candidate + "()"):
                row.update(accountability="DUPLICATE SYNTAX", blocking=False, operation=name,
                           canonical_candidate=canonical["id"], reason="same source start; reviewed iterator prefix of canonical terminal scan")
                linked.append(row["id"])
        operations.append({"id": name, "path": path, "function": function,
                           "owner": "WorkspaceIndex" if name in {"D1", "D2", "S1"} else "module function",
                           "canonical_candidate": canonical["id"],
                           "anchor_byte": canonical["site"]["byte"],
                           "boundary_note": "manually inspected named body; full function/impl interval absent in frozen census, not inferred from bare name",
                           "line": canonical["site"]["line"], "caller": caller,
                           "identity": identity, "growth": dimensions,
                           "frequency": "one scan per participating candidate in named caller; O1 per opcode lookup",
                           "complexity": complexity, "classification": classification,
                           "disposition": disposition, "syntax_records": linked,
                           "coverage": "declaration characterization for D1/D2; other growth budgets absent; see tracked investigation ledger"})
    for row in rows:
        path = row["site"]["path"]
        row["source_drift"] = path in drift
        row["parser_review_required"] = path in parse_errors
        if path in drift or path in parse_errors:
            row["blocking"] = True
    recovery = None
    if args.parser_supplement:
        diagnostic = read(args.parser_supplement)
        path = diagnostic["path"]
        assert path == "src/mcp/semantic_compilation/artifact.rs"
        assert diagnostic["source_sha256"] == hashes[path]
        source = (root / path).read_bytes()
        # These exact fragments were read by MaxHeadRoom and independently
        # checked by Popper; this is not an automatic parser-error allowlist.
        expected = {(4728, 4738), (4794, 5020), (5078, 5089)}
        assert {(e["start_byte"], e["end_byte"]) for e in diagnostic["errors"]} == expected
        if path not in drift:
            for boundary in diagnostic["errors"] + diagnostic["recovered_functions"]:
                assert source[boundary["start_byte"]:boundary["end_byte"]].decode("utf-8") == boundary["source"]
        else:
            assert digest(args.parser_supplement) == "e6ed8c57a4ab2c595870af3ca7117e1a45902063f1088ebdd6112df359e1b37c", "Frozen manual recovery evidence changed; review required"
        mapped = 0
        for row in rows:
            if row["site"]["path"] != path:
                continue
            matches = [f for f in diagnostic["recovered_functions"]
                       if f["start_byte"] <= row["site"]["byte"] < f["end_byte"]]
            assert len(matches) <= 1
            row["parser_review_required"] = False
            row["manual_recovery_source_scope"] = "accepted frozen source boundary; changed current source requires separate drift review" if path in drift else "accepted exact current source boundary"
            row["parser_manual_recovery"] = "cfg-qualified destructuring spans manually inspected; scoped Popper acceptance; operation classification unchanged"
            if matches:
                function = matches[0]
                row["recovered_function_boundary"] = {k: function[k] for k in ["start_byte", "end_byte", "start_line", "end_line", "impl_owner"]}
                mapped += 1
        recovery = {"path": path, "diagnostic_sha256": digest(args.parser_supplement),
                    "original_parse_error_retained": True,
                    "manual_syntax_recovery": True,
                    "recovered_function_count": len(diagnostic["recovered_functions"]),
                    "mapped_candidate_records": mapped,
                    "whole_operation_coverage_accepted": False}
    assert len(rows) == len(sites) and len({r["id"] for r in rows}) == len(sites)
    known = {r["id"] for r in rows}
    assert all(r.get("canonical_candidate", r["id"]) in known for r in rows)
    counts = dict(collections.Counter(r["accountability"] for r in rows))
    calls = read(baseline / "census/calls.json")
    supplemental = [{"id": f"R{i:05d}", "call": call,
                     "accountability": "REVIEW REQUIRED",
                     "reason": "all callback multiplicity omitted from original candidate-method list"}
                    for i, call in enumerate(calls, 1)
                    if re.search(r"\.all$", call["callee"])]
    output.mkdir(parents=True)
    with (output / "candidates.jsonl").open("w", encoding="utf-8") as stream:
        for row in rows:
            stream.write(json.dumps(row, ensure_ascii=True) + "\n")
    file_rows = [{**f, "current_sha256": current[f["path"]], "source_drift": f["path"] in drift} for f in files]
    summary = {"agent": "Agent MaxHeadRoom", "baseline": str(baseline),
               "original_candidates": len(sites), "accounted_candidates": len(rows),
               "counts": counts, "reviewed_logical_operations": len(operations),
               "blocking_candidates": sum(r["blocking"] for r in rows),
               "source_drift_files": sorted(drift), "parse_error_files": sorted(parse_errors),
               "active_parser_boundary_blockers": sorted(parse_errors - ({recovery["path"]} if recovery else set())),
               "test_support_exclusions": 0, "inventory_complete": False,
               "supplemental_raw_call_obligations": len(supplemental),
               "parser_manual_supplement": recovery,
               "note": "closed row accounting is not completed classification or safety approval",
               "driver_sha256": digest(Path(__file__))}
    for name, value in [("operations.json", operations), ("files.json", file_rows), ("summary.json", summary), ("supplemental-calls.json", supplemental)]:
        (output / name).write_text(json.dumps(value, indent=2), encoding="utf-8")
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
