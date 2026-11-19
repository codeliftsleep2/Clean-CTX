"""Analyze retained serial CI evidence only; never runs Cargo, tests or benchmarks."""
import argparse
import collections
import csv
import datetime
import hashlib
import importlib.util
import json
import pathlib
import re
import sys


def family(name):
    parts = name.split("::")
    if name.startswith("mcp::semantic_compilation::"):
        return "mcp::semantic_compilation"
    if name.startswith("mcp::tool_handlers::core::"):
        return "::".join(parts[:4])
    if name.startswith("mcp::tool_handlers::"):
        return "::".join(parts[:3])
    return "::".join(parts[:2]) if len(parts) > 2 else parts[0]


def main():
    sys.dont_write_bytecode = True
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--capture", type=pathlib.Path, required=True)
    parser.add_argument("--output", type=pathlib.Path, required=True)
    args = parser.parse_args()
    repo = pathlib.Path(__file__).resolve().parents[2]
    spec = importlib.util.spec_from_file_location("membership", repo / "scripts/verify-ci-membership.py")
    membership = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(membership)
    execution = next(args.capture.rglob("execution.log"))
    registration = next(args.capture.rglob("registration.log"))
    results = membership.records(execution, False)
    expected = [(key[-1], values[0]) for key, values in results.items()
                if key[0] == "clean-ctx" and key[1] == "unittests src/lib.rs"]
    if not expected:
        raise ValueError("No attributed root library results")
    rows, stack = [], []
    root_summaries = []
    log = args.capture / "github.log"
    for raw in log.read_text(encoding="utf-8-sig").splitlines():
        raw = membership.ANSI.sub("", raw)
        event = re.search(r"(\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d\.\d+Z) (.*)", raw)
        if not event:
            continue
        stamp = datetime.datetime.fromisoformat(event[1][:-1] + "+00:00")
        text = event[2].strip()
        start = re.fullmatch(r"running (\d+) tests?", text)
        if start:
            stack.append({"root": not stack and int(start[1]) == len(expected),
                          "previous": stamp, "pending": None})
            continue
        complete = re.fullmatch(r"test (.+) \.\.\. (ok|FAILED|ignored(?:,.*)?)", text)
        pending = re.fullmatch(r"test (.+) \.\.\.", text)
        if (complete or pending) and stack:
            if complete:
                name, outcome = complete[1], complete[2].split(",")[0]
            else:
                stack[-1]["pending"] = pending[1]
                continue
        elif text in ("ok", "FAILED", "ignored") and stack and stack[-1]["pending"]:
            name, outcome = stack[-1]["pending"], text
            stack[-1]["pending"] = None
        elif text.startswith("test result:") and stack:
            frame = stack.pop()
            if frame["root"]:
                root_summaries.append(text)
            continue
        else:
            continue
        if stack[-1]["root"]:
            elapsed = (stamp - stack[-1]["previous"]).total_seconds()
            if elapsed < 0:
                raise ValueError("Nonmonotonic result timestamps")
            rows.append({"name": name, "outcome": outcome, "family": family(name),
                         "module": name.rsplit("::", 1)[0],
                         "result_timestamp_utc": event[1],
                         "completion_interval_seconds": elapsed})
            stack[-1]["previous"] = stamp
    if [(row["name"], row["outcome"]) for row in rows] != expected or len(root_summaries) != 1:
        raise ValueError("Timestamped library outcomes do not exactly match attributed execution evidence")
    groups = collections.defaultdict(lambda: {"tests": 0, "interval_seconds": 0.0})
    for row in rows:
        groups[row["family"]]["tests"] += 1
        groups[row["family"]]["interval_seconds"] += row["completion_interval_seconds"]
    args.output.mkdir(parents=True, exist_ok=True)
    with (args.output / "library-result-intervals.csv").open("w", encoding="utf-8", newline="") as stream:
        writer = csv.DictWriter(stream, fieldnames=list(rows[0]))
        writer.writeheader()
        writer.writerows(rows)
    compile_lines = [line.strip() for line in registration.read_text(encoding="utf-8-sig").splitlines()
                     if "Finished `test`" in line]
    summary = {
        "category": "Retained evidence analysis; no executable verification or performance acceptance",
        "capture": str(args.capture), "library_summary": root_summaries[0],
        "outcomes_exactly_reconciled": len(rows), "registration_compile_summary": compile_lines,
        "result_interval_sum_seconds": sum(row["completion_interval_seconds"] for row in rows),
        "timing_limit": "Completion-to-completion console arrival intervals under serial execution, including runner/IO overhead and buffering. Not exact per-test native elapsed/CPU times or causal attribution.",
        "families": sorted(({"family": key, **value} for key, value in groups.items()),
                           key=lambda row: row["interval_seconds"], reverse=True),
        "largest_result_intervals": sorted(rows, key=lambda row: row["completion_interval_seconds"], reverse=True)[:25],
        "inputs": [{"path": str(path), "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}
                   for path in [log, execution, registration]],
    }
    (args.output / "summary.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"outcomes": len(rows), "interval_seconds": summary["result_interval_sum_seconds"],
                      "top_families": summary["families"][:8]}, indent=2))


if __name__ == "__main__":
    main()
