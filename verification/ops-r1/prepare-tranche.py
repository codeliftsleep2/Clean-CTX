"""Prepare unapplied Tranche-1 review diffs. No Cargo, CI, tests or server execution."""
import difflib
import ast
import hashlib
import json
import pathlib
import re
import zipfile

ROOT = pathlib.Path(__file__).resolve().parents[2]
OUT = ROOT / "verification/ops-r1/proposals/tranche-1"


def sha(data):
    return hashlib.sha256(data).hexdigest()


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(ROOT / "verification/ci-verify-r1/runtime-accounting-414-evidence.zip") as z:
        name = next(n for n in z.namelist() if "run-414/" in n and n.endswith("registration.log"))
        names = re.findall(r"^(.+): test\s*$", z.read(name).decode("utf-8-sig"), re.M)
    def exact(suffix):
        found = [n for n in names if n == suffix or n.endswith("::" + suffix)]
        assert len(found) == 1, (suffix, found)
        return found[0]

    selections = {
        "GARC-001": ["ci_runtime_core_query_warm_observation_avoids_recompiling_binary_owned_query",
                     "ci_runtime_complete_queries_preserve_captures_and_ir",
                     "ci_runtime_custom_invalid_and_mismatched_queries_stay_dynamic"],
        "GARC-002": ["ci_runtime_native_typescript_evidence_preserves_source_ownership"],
        "GARC-003": ["issue_162_deleted_owner_legacy_membership_is_retired",
                     "issue_162_retirement_acquisition_error_is_not_deletion",
                     "issue_162_retirement_shared_supports_siblings_and_relationship_families",
                     "issue_162_retirement_history_and_owner_ordering_survive_deletion"],
        "GARC-004": ["issue_162_unchanged_bootstrap_durable_registration_boundary",
                     "issue_162_restore_registration_config_preservation_matrix"],
    }
    golden = {"standing": "Tranche-1 obligations; activation requires approved CI/verifier installation", "groups": [], "isolated": {}, "frozen_witnesses": []}
    for path in ["src/tests/ir/ci_runtime_core_query.rs", "src/tests/mcp/angular_deleted_owner_membership_issue_162.rs", "src/tests/mcp/restore_registration_boundary_issue_162.rs"]:
        golden["frozen_witnesses"].append({"path":path,"sha256":sha((ROOT/path).read_bytes().replace(b"\r\n",b"\n"))})
    for garc, suffixes in selections.items():
        for suffix in suffixes:
            name = exact(suffix)
            golden["groups"].append({"package": "clean-ctx", "target": "lib", "filter": name,
                                     "minimum": 1, "tests": [name], "architectural_ids": [garc]})
    control = exact("ci_runtime_complete_queries_preserve_captures_and_ir")
    native = exact("ci_runtime_native_typescript_evidence_preserves_source_ownership")
    golden["isolated"] = {"csharp": [control], "typescript": [control, native]}
    config_path = "verification/ci-verify-r1/membership.json"
    config = json.loads((ROOT / config_path).read_text(encoding="utf-8"))
    config.update(golden_groups_path="golden-membership.json", expected_golden_groups=len(golden["groups"]), expected_frozen_golden_witnesses=3)
    proposed = {config_path: json.dumps(config, indent=2) + "\n",
                "verification/ci-verify-r1/golden-membership.json": json.dumps(golden, indent=2) + "\n"}

    path = "scripts/verify-ci-membership.py"
    source = (ROOT / path).read_text(encoding="utf-8")
    old = "    return config, groups, measurements, legacy\n"
    new = """    golden = read_json(manifest_dir / config["golden_groups_path"])
    required = golden["groups"]
    if not required or len(required) != config["expected_golden_groups"]:
        raise ValueError("Missing/incomplete mandatory golden membership")
    witnesses = golden["frozen_witnesses"]
    if len(witnesses) != config["expected_frozen_golden_witnesses"] or len({w["path"] for w in witnesses}) != len(witnesses):
        raise ValueError("Missing/duplicate frozen golden witnesses")
    for witness in witnesses:
        actual = (root / witness["path"]).read_bytes().replace(b"\\r\\n", b"\\n")
        if hashlib.sha256(actual).hexdigest() != witness["sha256"]:
            raise ValueError(f"Golden witness identity changed: {witness['path']}")
    return config, groups + required, measurements, legacy
"""
    assert source.count(old) == 1
    source = source.replace(old, new)
    old = 'fitness.append({"filter": group["filter"], "tests": expected, "passed": passed})'
    assert old in source
    source = source.replace(old, 'fitness.append({"filter": group["filter"], "tests": expected, "passed": passed, "architectural_ids": group.get("architectural_ids", [])})')
    marker = "\ndef main():\n"
    assert source.count(marker) == 1
    helper = '''
def verify_isolated(root, mode, execution, suite_exit):
    if execution is None or suite_exit is None:
        raise ValueError("Isolated execution requires evidence and actual Cargo exit")
    manifest = root / "verification/ci-verify-r1"
    config, _, _, _ = prerequisites(root, manifest)
    golden = read_json(manifest / config["golden_groups_path"])
    expected = golden["isolated"][mode]
    if not expected or len(set(expected)) != len(expected):
        raise ValueError("Empty/duplicate isolated golden obligations")
    ran = records(execution, False)
    errors = []
    actual = [key[3] for key in ran]
    if set(actual) != set(expected) or len(actual) != len(expected):
        errors.append(f"GARC-002 isolated {mode} membership mismatch")
    for key, outcomes in ran.items():
        if key[0] != "clean-ctx" or key[1].replace("\\\\", "/") != "unittests src/lib.rs" or outcomes != ["ok"]:
            errors.append(f"GARC-002 isolated {mode} failed/unexpected outcome: {key}")
    if suite_exit:
        errors.append(f"GARC-002 isolated {mode} Cargo exit: {suite_exit}")
    return {"errors": errors, "feature_mode": mode, "architectural_ids": ["GARC-002", "AFF-004", "AFF-010"],
            "expected": expected, "outcomes": len(ran),
            "execution_sha256": hashlib.sha256(execution.read_bytes()).hexdigest(),
            "qualification": "Mode identity must also match retained CI command/configuration; output alone cannot prove Cargo features."}

'''
    source = source.replace(marker, "\n" + helper + marker)
    source = source.replace('    parser.add_argument("--prerequisites-only", action="store_true")',
                            '    parser.add_argument("--prerequisites-only", action="store_true")\n    parser.add_argument("--feature-mode", choices=["csharp", "typescript"])')
    source = source.replace("        if args.prerequisites_only:",
                            "        if args.feature_mode:\n            receipt = verify_isolated(args.root, args.feature_mode, args.execution, args.suite_exit)\n        elif args.prerequisites_only:")
    proposed[path] = source

    path = "src/tests/architecture/ci_verify_r1.rs"
    source = (ROOT / path).read_text(encoding="utf-8")
    source = source.replace('    let groups = load(&manifest.join("fitness-membership.json"));',
        '    let mut groups = load(&manifest.join("fitness-membership.json"));\n    groups.as_array_mut().unwrap().extend(load(&manifest.join("golden-membership.json"))["groups"].as_array().unwrap().iter().cloned());')
    source = source.replace('        "fitness-membership.json",', '        "fitness-membership.json",\n        "golden-membership.json",')
    source = source.replace('        "empty_fitness",', '        "empty_fitness",\n        "missing_golden",')
    source = source.replace('        "missing_golden",', '        "missing_golden",\n        "golden_witness_changed",')
    source = source.replace('    for oracle in config["oracle_files"].as_array().unwrap() {',
        '    let mut source_inputs = config["oracle_files"].as_array().unwrap().clone();\n    source_inputs.extend(load(&manifest.join("golden-membership.json"))["frozen_witnesses"].as_array().unwrap().iter().cloned());\n    for oracle in &source_inputs {', 1)
    source = source.replace('            for oracle in config["oracle_files"].as_array().unwrap() {',
        '            let mut source_inputs = config["oracle_files"].as_array().unwrap().clone();\n            source_inputs.extend(load(&manifest.join("golden-membership.json"))["frozen_witnesses"].as_array().unwrap().iter().cloned());\n            for oracle in &source_inputs {')
    marker = '        let fitness_manifest = manifest.join("fitness-membership.json");'
    assert marker in source
    source = source.replace(marker, '''        let golden_path = manifest.join("golden-membership.json");
        let frozen_source = load(&golden_path)["frozen_witnesses"][0]["path"].as_str().unwrap().to_owned();
        if case == "golden_witness_changed" {
            std::fs::write(root.join(&frozen_source), "changed golden witness").unwrap();
        }
        if case == "missing_golden" {
            std::fs::remove_file(&golden_path).unwrap();
        }
''' + marker)
    # The negative case is followed by more fixtures; restore the manifest after checking it.
    marker = '        let output = Command::new(if cfg!(windows) { "python" } else { "python3" })'
    assert marker in source
    # Use an independent config failure with restoration after actual verifier execution.
    marker = '        let expected_success = matches!('
    source = source.replace(marker, '''        if case == "missing_golden" {
            std::fs::copy(repo.join("verification/ci-verify-r1/golden-membership.json"), &golden_path).unwrap();
        }
        if case == "golden_witness_changed" {
            std::fs::copy(repo.join(&frozen_source), root.join(&frozen_source)).unwrap();
        }
''' + marker)
    proposed[path] = source

    isolated_cases = ["isolated_csharp_valid", "isolated_typescript_valid", "isolated_missing",
                      "isolated_ignored", "isolated_missing_exit", "isolated_nonzero_exit"]
    marker = '        "cargo_failure",'
    assert source.count(marker) == 1
    source = source.replace(marker, marker + ''.join('\n        "' + case + '",' for case in isolated_cases))
    marker = '        let mut listing = listed.clone();'
    assert source.count(marker) == 1
    source = source.replace(marker, '''        let isolated_mode = if case.starts_with("isolated_") {
            Some(if case == "isolated_typescript_valid" { "typescript" } else { "csharp" })
        } else { None };
''' + marker)
    marker = '        let listing_path = root.join("listing.log");'
    if marker not in source:
        marker = '        std::fs::write(&listing_path, listing).unwrap();'
    assert source.count(marker) == 1
    source = source.replace(marker, '''        if let Some(mode) = isolated_mode {
            let golden = load(&manifest.join("golden-membership.json"));
            let mut selected = golden["isolated"][mode].as_array().unwrap().clone();
            if case == "isolated_missing" { selected.clear(); }
            execution = format!("     Running unittests src/lib.rs (target/debug/deps/clean_ctx-fixture)\\nrunning {} tests\\n", selected.len());
            for test in selected {
                execution.push_str(&format!("test {} ... {}\\n", test.as_str().unwrap(), if case == "isolated_ignored" { "ignored" } else { "ok" }));
            }
            execution.push_str("test result: ok.\\n");
        }
''' + marker)
    old_args = '''            .args([
                "--suite-exit",
                if case == "cargo_failure" { "1" } else { "0" },
            ])'''
    assert source.count(old_args) == 1
    source = source.replace(old_args, '''            .args(if case == "isolated_missing_exit" { Vec::<&str>::new() } else {
                vec!["--suite-exit", if matches!(case, "cargo_failure" | "isolated_nonzero_exit") { "1" } else { "0" }]
            })
            .args(isolated_mode.map(|mode| vec!["--feature-mode", mode]).unwrap_or_default())''')
    marker = '"valid" | "lf_checkout" | "crlf_checkout" | "interleaved_headers" | "nested_worker"'
    assert source.count(marker) == 1
    source = source.replace(marker,marker+' | "isolated_csharp_valid" | "isolated_typescript_valid"')
    proposed['src/tests/architecture/ci_verify_r1.rs'] = source

    path = ".github/workflows/ci.yml"
    source = (ROOT / path).read_text(encoding="utf-8")
    marker = "      - name: cargo audit\n"
    steps = []
    for mode, selected in golden["isolated"].items():
        selected_args = " ".join(selected)
        steps.append(f'''      - name: GARC-002 isolated {mode}
        shell: pwsh
        run: |
          $out = "target/ops-r1-isolated-{mode}"
          New-Item -ItemType Directory -Path $out -Force | Out-Null
          $profileArgs = @('--config','profile.test.opt-level=1','--config','profile.test.package."*".opt-level=3','--config','profile.test.debug-assertions=true','--config','profile.test.overflow-checks=true')
          $env:CARGO_TERM_COLOR = 'never'
          & cargo test -p clean-ctx --lib --no-default-features --features {mode} @profileArgs -- --exact --test-threads=1 --format=pretty {selected_args} 2>&1 | Tee-Object -FilePath "$out/execution.log"
          $suiteExit = $LASTEXITCODE
          & python scripts/verify-ci-membership.py --root . --feature-mode {mode} --execution "$out/execution.log" --suite-exit $suiteExit --output "$out/receipt.json"
          if ($LASTEXITCODE -ne 0) {{ throw 'GARC-002 {mode} verification failed' }}
''')
    assert source.count(marker) == 1
    source = source.replace(marker, "".join(steps) + """      - name: retain isolated golden evidence
        if: ${{ always() }}
        uses: actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02
        with:
          name: isolated-golden-${{ github.sha }}-${{ github.run_attempt }}
          path: target/ops-r1-isolated-*
          if-no-files-found: error
""" + marker)
    proposed[path] = source

    path = "docs/agent/verification.md"
    source = (ROOT/path).read_text(encoding='utf-8')
    marker = '<!-- pcg:VERIFICATION-obligations:end -->'
    assert source.count(marker) == 1
    clause = '''### OPS-R1 selected golden enforcement

The selected golden membership is an obligation of this final verification gate.
The existing consolidated run executes its ten allfeature identities once; the
existing verifier requires exact nonignored outcomes. Supported C#-only and
TypeScript-only controls execute separately because allfeatures is not isolation.
The three retained behavioral defect witnesses require reviewed successors rather
than silent source/expected-hash updates. Their original RED/GREEN identity remains.
The source-owned golden membership component below is protected change-control
data, not proof of semantic effectiveness; historical negative evidence and
applicable consumer preservation remain mandatory. Changes to its required tests,
feature modes or witness identities require the existing B11 owner review and
authorization. No separate runner, duplicate allfeature suite, global time budget
or waiver of existing fitness/security/field obligations is introduced.

'''
    source = source.replace(marker,clause+marker)
    marker = '  - verification_obligation:completion self#VERIFICATION-obligations'
    assert source.count(marker) == 1
    source = source.replace(marker,marker+'\n  - evidence:golden_membership verification/ci-verify-r1/golden-membership.json#whole')
    proposed[path] = source
    path = 'docs/architecture/knowledge-governance/protected.csv'
    source = (ROOT/path).read_text(encoding='utf-8')
    marker = 'AUTH.FINAL_VERIFICATION,verification_obligation,completion,VERIFICATION-obligations'
    assert source.count(marker) == 1
    source = source.replace(marker,marker+'\nAUTH.FINAL_VERIFICATION,evidence,golden_membership,whole')
    proposed[path] = source

    pins = []
    categories = {"membership": [], "verifier": [], "fixtures": [], "ci": [], "governance": []}
    for path, candidate in proposed.items():
        baseline = (ROOT / path).read_text(encoding="utf-8") if (ROOT / path).exists() else ""
        pins.append({"path": path,
                     "baseline_bytes_sha256": sha((ROOT / path).read_bytes()) if (ROOT / path).exists() else None,
                     "candidate_utf8_lf_sha256": sha(candidate.encode("utf-8")),
                     "candidate_lines": len(candidate.splitlines())})
        assert len(candidate.splitlines()) <= 615, path
        if path.endswith('.py'):
            ast.parse(candidate)
        diff = "".join(difflib.unified_diff(baseline.splitlines(True), candidate.splitlines(True),
                                         fromfile="a/" + path if baseline else "/dev/null", tofile="b/" + path))
        category = "governance" if path.startswith('docs/') else "ci" if path.endswith(".yml") else "fixtures" if path.endswith(".rs") else "verifier" if path.endswith(".py") else "membership"
        categories[category].append(diff)
    for category, diffs in categories.items():
        text = "".join(diffs)
        assert len(text.splitlines()) <= 615, category
        (OUT / (category + ".patch")).write_text(text, encoding="utf-8", newline="\n")
    (OUT / "proposal-index.json").write_text(json.dumps({"standing": "UNAPPLIED AND UNCOMPILED; owner review required",
        "allfeature_required_identities": golden["groups"], "isolated": golden["isolated"], "files": pins,
        "execution_cost": "Zero additional allfeature runtime executions; one C#-only and two TS-only controls; each mode builds the lib test target once. No timing estimates claimed."}, indent=2) + "\n", encoding="utf-8")
    print("Prepared unapplied membership, verifier, fixture and CI diffs; no current CI or source gate modified.")


if __name__ == "__main__":
    main()
