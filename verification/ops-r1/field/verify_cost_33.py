"""Run the remaining focused preservation tests, one bounded child per test.

An optional prior stdout file excludes only explicitly completed passing tests.
This is a tracked-test launcher, not new regression coverage or a CI substitute.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import time

ROOT = Path(__file__).resolve().parents[3]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--exe", type=Path, required=True)
    parser.add_argument("--prior-stdout", type=Path)
    parser.add_argument("--wall-seconds", type=int, choices=range(1, 181), default=60, metavar="1..180")
    parser.add_argument("--filter", action="append", help="explicit focused test selection; repeat for multiple filters")
    args = parser.parse_args()
    exe = args.exe.resolve(strict=True)
    digest = hashlib.sha256(exe.read_bytes()).hexdigest()
    prior = set()
    if args.prior_stdout:
        prior = set(re.findall(r"^test (\S+) \.\.\. ok$", args.prior_stdout.read_text(encoding="utf-8"), re.M))
    listing = subprocess.run([str(exe), "--list"], capture_output=True, check=True, timeout=10).stdout.decode("utf-8")
    filters = ["issue_162_resolver_registration", "ops_r1_shared_acquisition_public_fitness",
               "ops_r1_warm_resolver_consumption_checks_shared_constraints_once",
               "ops_r1_repeated_absent_queries_preserve_retained_owner_evidence"]
    filters.append("ops_r1_empty_automatic_types_preserve_imported_registrations")
    if args.filter:
        filters = args.filter
    names = [line.removesuffix(": test") for line in listing.splitlines() if line.endswith(": test")]
    names = [name for name in names if any(f in name for f in filters) and name not in prior]
    assert names, "no remaining tests selected"
    out = ROOT / "target/ops-r1/cost-33" / ("preservation-" + time.strftime("%Y%m%d-%H%M%S"))
    out.mkdir(parents=True, exist_ok=False)
    rows = []
    for index, name in enumerate(names):
        command = [str(exe), "--exact", name, "--test-threads=1"]
        started = time.monotonic()
        with (out / f"{index}.stdout.txt").open("wb") as stdout, (out / f"{index}.stderr.txt").open("wb") as stderr:
            try:
                result = subprocess.run(command, cwd=ROOT, stdout=stdout, stderr=stderr, timeout=args.wall_seconds)
                output = (out / f"{index}.stdout.txt").read_text(encoding="utf-8")
                passed = result.returncode == 0 and "1 passed; 0 failed; 0 ignored;" in output
                row = {"test":name, "exit_code":result.returncode, "passed":passed}
            except subprocess.TimeoutExpired:
                row = {"test":name, "passed":False, "stop_reason":f"{args.wall_seconds}-second watchdog"}
        row["elapsed_seconds"] = time.monotonic() - started
        rows.append(row)
        receipt = {"executable_sha256":digest, "prior_passes":sorted(prior), "results":rows,
                   "binary_unchanged":hashlib.sha256(exe.read_bytes()).hexdigest() == digest}
        (out / "results.json").write_text(json.dumps(receipt, indent=2), encoding="utf-8")
        print(json.dumps(row), flush=True)
        if not row["passed"] or not receipt["binary_unchanged"]:
            return 1
    print(out)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
