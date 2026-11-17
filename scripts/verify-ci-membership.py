"""CI-VERIFY-R1: reconcile Cargo's listed tests against actual libtest outcomes.

Consumes evidence only. Never compiles, runs tests, or grants performance acceptance.
"""
import argparse
import collections
import hashlib
import json
import pathlib
import re

ANSI = re.compile(r"\x1b\[[0-?]*[ -/]*[@-~]")
HEADER = re.compile(r"^Running (.+?) \(([^()]+)\)$")


def read_json(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def records(path, listing):
    target = None
    found = collections.defaultdict(list)
    for raw in path.read_text(encoding="utf-8-sig").splitlines():
        line = ANSI.sub("", raw).strip()
        header = HEADER.search(line)
        if header:
            executable = header[2].replace("\\", "/").split("/")[-1]
            package = "clean-ctx-proxy" if executable.startswith("clean_ctx_proxy-") else "clean-ctx"
            target = (package, header[1], executable)
            continue
        match = re.fullmatch(r"(.+): test", line) if listing else re.fullmatch(
            r"test (.+) \.\.\. (ok|FAILED|ignored(?:,.*)?)", line
        )
        if match:
            if target is None:
                raise ValueError("Test record has no Cargo target attribution")
            found[(*target, match[1])].append("listed" if listing else match[2].split(",")[0])
    if not found:
        raise ValueError(f"Empty test evidence: {path}")
    return found


def prerequisites(root, manifest_dir):
    config = read_json(manifest_dir / "membership.json")
    if config["features"] != "all-features" or config["test_threads"] != 1:
        raise ValueError("Unsupported verification configuration")
    groups = read_json(manifest_dir / config["fitness_groups_path"])
    measurements = read_json(manifest_dir / config["measurements_path"])
    if not groups or len(groups) != config["expected_fitness_groups"]:
        raise ValueError("Empty/incomplete required fitness group inventory")
    if not measurements or len(measurements) != config["expected_measurements"]:
        raise ValueError("Empty/incomplete measurement inventory")
    legacy = read_json(manifest_dir / "legacy-ignored.json")
    for fixture in config["oracle_files"]:
        path = root / fixture["path"]
        data = path.read_bytes()
        if fixture["hash_normalization"] != "git-text-lf":
            raise ValueError("Unsupported frozen text identity normalization")
        canonical = data.replace(b"\r\n", b"\n")
        if hashlib.sha256(canonical).hexdigest() != fixture["sha256"]:
            raise ValueError(f"Frozen oracle identity changed: {path}")
        rows = json.loads(data)
        if not isinstance(rows, list) or len(rows) != fixture["rows"]:
            raise ValueError(f"Invalid frozen oracle population: {path}")
    return config, groups, measurements, legacy


def verify(root, listing, execution, suite_exit, manifest_dir):
    config, groups, measurements, legacy = prerequisites(root, manifest_dir)
    allowed_ignored = {(x["package"], x["name"]) for x in legacy + measurements}
    empirical = {(x["package"], x["name"]) for x in measurements}
    listed = records(listing, True)
    ran = records(execution, False)
    errors = []
    for key, values in listed.items():
        if len(values) != 1:
            errors.append(f"Duplicate registration: {key}")
        outcomes = ran.get(key, [])
        if len(outcomes) != 1:
            errors.append(f"Missing/duplicate execution: {key}: {outcomes}")
        elif outcomes[0] == "ignored":
            if (key[0], key[3]) not in allowed_ignored:
                errors.append(f"Unapproved ignored test: {key}")
        elif outcomes[0] != "ok":
            errors.append(f"Failed correctness test: {key}")
    for key in ran.keys() - listed.keys():
        errors.append(f"Executed unregistered test: {key}")
    fitness = []
    for group in groups:
        expected = group["tests"]
        if not expected or len(expected) < group["minimum"]:
            errors.append(f"Empty/incomplete fitness declaration: {group['filter']}")
        selected = [
            key for key in listed
            if key[0] == group["package"]
            and "lib.rs" in key[1]
            and group["filter"] in key[3]
            and (key[0], key[3]) not in empirical
        ]
        if {key[3] for key in selected} != set(expected) or len(selected) != len(expected):
            errors.append(f"Fitness membership drift: {group['filter']}")
        passed = all(ran.get(key) == ["ok"] for key in selected)
        if not passed:
            errors.append(f"Failed deterministic fitness group: {group['filter']}")
        fitness.append({"filter": group["filter"], "tests": expected, "passed": passed})
    for item in measurements:
        selected = [key for key in listed if key[0] == item["package"] and key[3] == item["name"]]
        if len(selected) != 1 or ran.get(selected[0]) != ["ignored"]:
            errors.append(f"Measurement ran ordinarily or registration missing: {item['name']}")
        companions = [key for key in listed if key[0] == item["package"] and key[3] == item["mandatory_companion"]]
        if len(companions) != 1 or ran.get(companions[0]) != ["ok"]:
            errors.append(f"Missing/failed mandatory mixed companion: {item['mandatory_companion']}")
    if suite_exit:
        errors.append(f"Cargo suite exit code: {suite_exit}")
    return {
        "agent": "Agent MaxHeadRoom",
        "category": "correctness execution and deterministic fitness; not empirical performance",
        "registered": len(listed),
        "outcomes": len(ran),
        "errors": errors,
        "fitness_groups": fitness,
        "listing_sha256": hashlib.sha256(listing.read_bytes()).hexdigest(),
        "execution_sha256": hashlib.sha256(execution.read_bytes()).hexdigest(),
        "scope": config["scope"],
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=pathlib.Path, required=True)
    parser.add_argument("--listing", type=pathlib.Path)
    parser.add_argument("--execution", type=pathlib.Path)
    parser.add_argument("--suite-exit", type=int)
    parser.add_argument("--prerequisites-only", action="store_true")
    parser.add_argument("--output", type=pathlib.Path, required=True)
    args = parser.parse_args()
    try:
        manifest = args.root / "verification/ci-verify-r1"
        if args.prerequisites_only:
            prerequisites(args.root, manifest)
            receipt = {"errors": [], "category": "prerequisites only; no execution"}
        else:
            if args.listing is None or args.execution is None or args.suite_exit is None:
                raise ValueError("Execution validation requires listing, execution and actual Cargo exit")
            receipt = verify(args.root, args.listing, args.execution, args.suite_exit, manifest)
    except (ValueError, OSError, KeyError, TypeError) as error:
        receipt = {"errors": [str(error)], "category": "verification refused"}
    args.output.write_text(json.dumps(receipt, indent=2) + "\n", encoding="utf-8")
    for error in receipt["errors"]:
        print(error)
    raise SystemExit(bool(receipt["errors"]))


if __name__ == "__main__":
    main()
