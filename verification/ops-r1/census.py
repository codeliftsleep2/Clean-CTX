"""Offline OPS-R1 evidence census. Never invokes Cargo or a test/runtime process.

Source metadata is lexical, not a Rust AST or proof of assertion effectiveness.
Exact executed identities come from retained CI evidence via its existing parser.
"""
import collections
import csv
import hashlib
import importlib.util
import json
import pathlib
import re
import subprocess
import sys
import zipfile

sys.dont_write_bytecode = True
ROOT = pathlib.Path(__file__).resolve().parents[2]
OUT = ROOT / "target/ops-r1/census"
TOKEN = re.compile(r"//|/\*|(?:br|r)\#*\"|\"|'")


def sha(data):
    return hashlib.sha256(data).hexdigest()


def masked(text):
    """Preserve offsets/newlines while masking comments and Rust string tokens."""
    chars = list(text)
    i = 0
    while i < len(text):
        token = TOKEN.search(text, i)
        if token is None:
            break
        i = token.start()
        end = None
        if text.startswith("//", i):
            end = text.find("\n", i)
            if end < 0:
                end = len(text)
        elif text.startswith("/*", i):
            depth, j = 1, i + 2
            while j < len(text) and depth:
                if text.startswith("/*", j):
                    depth += 1
                    j += 2
                elif text.startswith("*/", j):
                    depth -= 1
                    j += 2
                else:
                    j += 1
            end = j
        else:
            raw = re.compile(r'(?:br|r)(#*)"').match(text, i)
            if raw:
                closing = '"' + raw[1]
                pos = text.find(closing, i + len(raw[0]))
                end = len(text) if pos < 0 else pos + len(closing)
            elif text[i] == '"':
                j = i + 1
                while j < len(text):
                    if text[j] == "\\":
                        j += 2
                    elif text[j] == '"':
                        j += 1
                        break
                    else:
                        j += 1
                end = j
            elif text[i] == "'":
                char = re.match(r"'(?:\\(?:u\{[0-9a-fA-F_]+\}|x[0-9a-fA-F]{2}|.)|[^'\\\n])'", text[i:])
                if char:
                    end = i + len(char[0])
        if end is not None:
            for j in range(i, min(end, len(text))):
                if chars[j] not in "\r\n":
                    chars[j] = " "
            i = end
        else:
            i += 1
    return "".join(chars)


def csv_file(path, rows, fields):
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="") as file:
        writer = csv.DictWriter(file, fieldnames=fields, lineterminator="\n")
        writer.writeheader()
        writer.writerows(rows)


def source_records(paths):
    records, files = [], []
    for name in paths:
        data = (ROOT / name).read_bytes()
        text = data.decode("utf-8-sig")
        code = masked(text)
        file_tests = 0
        for fn in re.finditer(r"\bfn\s+([A-Za-z_][A-Za-z_0-9]*)\s*\(", code):
            boundary = code.rfind("}", 0, fn.start()) + 1
            prefix = code[boundary:fn.start()]
            if not re.search(r"#\[\s*(?:\w+::)*test(?:\s*\([^]]*\))?\s*\]", prefix):
                continue
            start = code.find("{", fn.end())
            if start < 0:
                continue
            depth, end = 1, start + 1
            for brace in re.finditer(r"[{}]", code[start + 1:]):
                depth += 1 if brace[0] == "{" else -1
                end = start + 1 + brace.end()
                if not depth:
                    break
            body = code[start:end]
            raw_prefix = text[boundary:fn.start()]
            line = text.count("\n", 0, fn.start()) + 1
            record = {
                "source": name, "line": line, "function": fn[1],
                "source_sha256": sha(data),
                "test_body_sha256": sha(text[start:end].encode("utf-8")),
                "direct_assert_macros": len(re.findall(r"\bassert(?:_eq|_ne)?\s*!", body)),
                "direct_panic_macros": len(re.findall(r"\bpanic\s*!", body)),
                "direct_unwrap_expect_calls": len(re.findall(r"\.(?:unwrap|expect)\s*\(", body)),
                "direct_loop_tokens": len(re.findall(r"\b(?:for|while|loop)\b", body)),
                "ignore_attribute": bool(re.search(r"#\[\s*ignore\b", prefix)),
                "should_panic_attribute": bool(re.search(r"#\[\s*should_panic\b", prefix)),
                "adjacent_cfg": " | ".join(re.findall(r"#\[cfg[^\n]*", raw_prefix)),
                "qualification": "lexical body facts; helpers/macros/cfg not evaluated",
            }
            records.append(record)
            file_tests += 1
        files.append({"source": name, "sha256": sha(data), "lexical_test_functions": file_tests})
    return records, files


def retained_execution():
    spec = importlib.util.spec_from_file_location("ci_membership", ROOT / "scripts/verify-ci-membership.py")
    parser = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(parser)
    archive = ROOT / "verification/ci-verify-r1/runtime-accounting-414-evidence.zip"
    scratch = ROOT / "target/ops-r1/retained-414"
    scratch.mkdir(parents=True, exist_ok=True)
    extracted = {}
    with zipfile.ZipFile(archive) as z:
        for name in z.namelist():
            if "run-414/" in name and name.endswith(("registration.log", "execution.log", "membership-result.json", "configuration.json")):
                path = scratch / pathlib.PurePosixPath(name).name
                path.write_bytes(z.read(name))
                extracted[path.name] = path
    listing = parser.records(extracted["registration.log"], True)
    execution = parser.records(extracted["execution.log"], False)
    configuration = json.loads(extracted["configuration.json"].read_text(encoding="utf-8-sig"))
    receipt = json.loads(extracted["membership-result.json"].read_text(encoding="utf-8-sig"))
    assert not receipt["errors"]
    assert sha(extracted["registration.log"].read_bytes()) == receipt["listing_sha256"]
    assert sha(extracted["execution.log"].read_bytes()) == receipt["execution_sha256"]
    return listing, execution, configuration, receipt, archive


def main():
    names = subprocess.check_output(["git", "ls-files", "-z", "--", "*.rs"], cwd=ROOT).decode().split("\0")
    paths = sorted(n for n in names if n and (ROOT / n).is_file())
    sources, files = source_records(paths)
    by_function = collections.defaultdict(list)
    for row in sources:
        by_function[row["function"]].append(row)
    listed, ran, config, receipt, archive = retained_execution()
    runtime = []
    for key, outcomes in sorted(ran.items()):
        package, target, executable, name = key
        candidates = by_function[name.split("::")[-1]]
        runtime.append({"package": package, "target": target, "test": name,
                        "outcome": " | ".join(outcomes), "execution_count": len(outcomes),
                        "source_matches": " | ".join(f'{r["source"]}:{r["line"]}' for r in candidates),
                        "source_match_status": "unique-name candidate" if len(candidates) == 1 else "ambiguous/unresolved",
                        "configuration": "historical run 414; workspace/all-targets/all-features; one thread"})
    OUT.mkdir(parents=True, exist_ok=True)
    # Cohesive source shards and target-qualified runtime families, never one giant file.
    source_groups = collections.defaultdict(list)
    for row in sources:
        source_groups[row["source"]].append(row)
    for source, rows in source_groups.items():
        csv_file(OUT / "source" / (source + ".csv"), rows, list(rows[0]))
    families = collections.defaultdict(list)
    for row in runtime:
        parts = row["test"].split("::")
        family = "::".join(parts[:min(2, len(parts)-1)]) or "root"
        families[(row["package"], row["target"], family)].append(row)
    pending = list(families.items())
    while pending:
        (package, target, family), rows = pending.pop()
        if len(rows) > 590:
            deeper = collections.defaultdict(list)
            level = 0 if family == "root" else len(family.split("::"))
            for row in rows:
                deeper["::".join(row["test"].split("::")[:level+1])].append(row)
            assert len(deeper) > 1, (family, "needs semantic decomposition")
            pending.extend(((package, target, f), r) for f, r in deeper.items())
        else:
            safe = re.sub(r"[^A-Za-z0-9_.-]", "_", f"{package}--{target}--{family}")
            csv_file(OUT / "execution" / (safe + ".csv"), rows, list(rows[0]))
    aggregate = collections.defaultdict(collections.Counter)
    for row in runtime:
        group = "::".join(row["test"].split("::")[:2])
        aggregate[group]["executed_identities"] += 1
        aggregate[group]["ignored"] += row["outcome"].startswith("ignored")
        aggregate[group]["passed"] += row["outcome"] == "ok"
    family_rows = [{"family": k, **v} for k, v in sorted(aggregate.items())]
    csv_file(OUT / "family-summary.csv", family_rows, ["family", "executed_identities", "ignored", "passed"])
    summary = {
        "agent": "Agent MaxHeadRoom", "head": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT).decode().strip(),
        "generator_sha256": sha(pathlib.Path(__file__).read_bytes()),
        "tracked_rust_files_scanned": len(files), "source_test_functions_lexical": len(sources),
        "source_test_files_lexical": len(source_groups), "retained_registered_identities": len(listed),
        "retained_execution_identities": len(runtime), "outcomes": dict(collections.Counter(r["outcome"] for r in runtime)),
        "unique_function_name_source_candidates": sum(r["source_match_status"] == "unique-name candidate" for r in runtime),
        "unresolved_or_ambiguous_source_candidates": sum(r["source_match_status"] != "unique-name candidate" for r in runtime),
        "historical_ci_source_head": config["head"], "fitness_groups": len(receipt["fitness_groups"]),
        "archive": archive.relative_to(ROOT).as_posix(), "archive_sha256": sha(archive.read_bytes()),
        "limitations": ["Source scanner is lexical, not an AST/cfg evaluator.", "Runtime identity is exact retained Cargo/libtest evidence, not current execution.",
                        "Function-name matches are candidates, not proof of module registration.", "No coverage percentages, assertion value or semantic equivalence inferred from counts.",
                        "Macro-generated/parameterized tests may have no lexical function match.", "Non-Rust script tests are inventoried by CI design, not represented as Rust tests."]}
    (OUT / "summary.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
    csv_file(OUT / "source-files.csv", files, ["source", "sha256", "lexical_test_functions"])
    # Source-file inventory is split by actual source directory when it exceeds active-file limits.
    if len(files) > 590:
        (OUT / "source-files.csv").unlink()
        directories = collections.defaultdict(list)
        for row in files:
            directories[str(pathlib.PurePosixPath(row["source"]).parent)].append(row)
        for directory, rows in directories.items():
            assert len(rows) < 590, directory
            csv_file(OUT / "files" / (directory + "/inventory.csv"), rows, ["source", "sha256", "lexical_test_functions"])
    assert all(len(p.read_bytes().splitlines()) <= 615 for p in OUT.rglob("*") if p.is_file())
    artifact = ROOT / "verification/ops-r1/test-census-evidence.zip"
    members = []
    with zipfile.ZipFile(artifact, "w", zipfile.ZIP_DEFLATED) as z:
        for path in sorted(OUT.rglob("*")):
            if path.is_file():
                data = path.read_bytes()
                entry = path.relative_to(OUT).as_posix()
                z.writestr(entry, data)
                members.append({"entry": entry, "sha256": sha(data)})
        manifest = json.dumps(members, indent=2).encode("utf-8")
        z.writestr("member-manifest.json", manifest)
    summary["generated_archive"] = artifact.relative_to(ROOT).as_posix()
    summary["generated_archive_sha256"] = sha(artifact.read_bytes())
    summary["member_manifest_sha256"] = sha(manifest)
    summary["member_count"] = len(members)
    (ROOT / "verification/ops-r1/test-census-index.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
