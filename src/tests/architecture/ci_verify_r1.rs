//! Adversarial evidence fixtures exercise the actual CI membership verifier.
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::process::Command;

fn load(path: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

fn evidence(manifest: &Path) -> (String, String, String) {
    let groups = load(&manifest.join("fitness-membership.json"));
    let measurements = load(&manifest.join("measurement-membership.json"));
    let empirical: BTreeSet<_> = measurements
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            (
                row["package"].as_str().unwrap().to_owned(),
                row["name"].as_str().unwrap().to_owned(),
            )
        })
        .collect();
    let mut names = BTreeMap::<String, BTreeSet<String>>::new();
    for group in groups.as_array().unwrap() {
        for name in group["tests"].as_array().unwrap() {
            names
                .entry(group["package"].as_str().unwrap().into())
                .or_default()
                .insert(name.as_str().unwrap().into());
        }
    }
    let first = groups[0]["tests"][0].as_str().unwrap().to_owned();
    for row in measurements.as_array().unwrap() {
        let package = row["package"].as_str().unwrap();
        for field in ["name", "mandatory_companion"] {
            names
                .entry(package.into())
                .or_default()
                .insert(row[field].as_str().unwrap().into());
        }
    }
    names
        .entry("clean-ctx".into())
        .or_default()
        .insert("ci_verify_r1::correctness_sentinel".into());
    let mut listing = String::new();
    let mut execution = String::new();
    for (package, tests) in names {
        let binary = if package == "clean-ctx" {
            "clean_ctx-fixture"
        } else {
            "clean_ctx_proxy-fixture"
        };
        let header = format!("     Running unittests src/lib.rs (target/debug/deps/{binary})\n");
        listing.push_str(&header);
        execution.push_str(&header);
        let count = tests.len();
        execution.push_str(&format!("running {count} tests\n"));
        for test in tests {
            listing.push_str(&format!("{test}: test\n"));
            let outcome = if empirical.contains(&(package.clone(), test.clone())) {
                "ignored, explicit measurement"
            } else {
                "ok"
            };
            execution.push_str(&format!("test {test} ... {outcome}\n"));
        }
        listing.push_str(&format!("{count} tests, 0 benchmarks\n"));
        execution.push_str("test result: ok.\n");
    }
    (listing, execution, first)
}

#[test]
fn ci_verify_r1_required_execution_and_failure_contracts() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"));
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let manifest = root.join("verification/ci-verify-r1");
    std::fs::create_dir_all(&manifest).unwrap();
    for file in [
        "membership.json",
        "fitness-membership.json",
        "measurement-membership.json",
        "legacy-ignored.json",
    ] {
        std::fs::copy(
            repo.join("verification/ci-verify-r1").join(file),
            manifest.join(file),
        )
        .unwrap();
    }
    let config = load(&manifest.join("membership.json"));
    for oracle in config["oracle_files"].as_array().unwrap() {
        let relative = oracle["path"].as_str().unwrap();
        let target = root.join(relative);
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::copy(repo.join(relative), target).unwrap();
    }
    let (listed, executed, fitness) = evidence(&manifest);
    let measurement = load(&manifest.join("measurement-membership.json"))[0]["name"]
        .as_str()
        .unwrap()
        .to_owned();
    for case in [
        "valid",
        "interleaved_headers",
        "nested_worker",
        "lf_checkout",
        "crlf_checkout",
        "missing_registration",
        "duplicate_registration",
        "missing_execution",
        "failed_correctness",
        "failed_fitness",
        "ignored_required",
        "empty_execution",
        "duplicate_execution",
        "measurement_ran",
        "missing_measurement_registration",
        "empty_fitness",
        "incomplete_fitness",
        "invalid_oracle",
        "oracle_content_changed",
        "missing_oracle",
        "cargo_failure",
    ] {
        let mut listing = listed.clone();
        let mut execution = executed.clone();
        let fitness_line = format!("test {fitness} ... ok\n");
        match case {
            "interleaved_headers" => {
                let header = "     Running unittests src/lib.rs (target/debug/deps/clean_ctx_proxy-fixture)\n";
                listing = listing.replace(header, "");
                listing = listing.replacen(
                    &format!("{fitness}: test\n"),
                    &format!("{header}{fitness}: test\n"),
                    1,
                );
            }
            "nested_worker" => {
                execution = execution.replace(
                    "test ci_verify_r1::correctness_sentinel ... ok\n",
                    "test ci_verify_r1::correctness_sentinel ... \nrunning 1 test\ntest nested_worker ... ok\ntest result: ok.\nok\n",
                );
            }
            "missing_registration" => {
                listing = listing.replace(&format!("{fitness}: test\n"), "");
                execution = execution.replace(&fitness_line, "");
            }
            "missing_execution" => execution = execution.replace(&fitness_line, ""),
            "duplicate_registration" => {
                let line = format!("{fitness}: test\n");
                listing = listing.replace(&line, &line.repeat(2));
            }
            "failed_correctness" => {
                execution = execution.replace(
                    "test ci_verify_r1::correctness_sentinel ... ok",
                    "test ci_verify_r1::correctness_sentinel ... FAILED",
                );
            }
            "failed_fitness" => {
                execution =
                    execution.replace(&fitness_line, &format!("test {fitness} ... FAILED\n"));
            }
            "ignored_required" => {
                execution =
                    execution.replace(&fitness_line, &format!("test {fitness} ... ignored\n"));
            }
            "empty_execution" => execution.clear(),
            "duplicate_execution" => {
                execution = execution.replace(&fitness_line, &fitness_line.repeat(2));
            }
            "measurement_ran" => {
                execution = execution.replace(
                    &format!("test {measurement} ... ignored, explicit measurement"),
                    &format!("test {measurement} ... ok"),
                );
            }
            "missing_measurement_registration" => {
                listing = listing.replace(&format!("{measurement}: test\n"), "");
                execution = execution.replace(
                    &format!("test {measurement} ... ignored, explicit measurement\n"),
                    "",
                );
            }
            _ => {}
        }
        let fitness_manifest = manifest.join("fitness-membership.json");
        if matches!(case, "empty_fitness" | "incomplete_fitness") {
            let mut value = load(&fitness_manifest);
            if case == "empty_fitness" {
                value.as_array_mut().unwrap().clear();
            } else {
                value.as_array_mut().unwrap().pop().unwrap();
            }
            std::fs::write(&fitness_manifest, serde_json::to_vec(&value).unwrap()).unwrap();
        }
        let oracle_path = config["oracle_files"][0]["path"].as_str().unwrap();
        if matches!(case, "lf_checkout" | "crlf_checkout") {
            for oracle in config["oracle_files"].as_array().unwrap() {
                let path = root.join(oracle["path"].as_str().unwrap());
                let original = std::fs::read_to_string(&path)
                    .unwrap()
                    .replace("\r\n", "\n");
                let changed = if case == "crlf_checkout" {
                    original.replace('\n', "\r\n")
                } else {
                    original
                };
                std::fs::write(path, changed).unwrap();
            }
        }
        if case == "invalid_oracle" {
            std::fs::write(root.join(oracle_path), b"not JSON").unwrap();
        } else if case == "oracle_content_changed" {
            let mut changed = load(&root.join(oracle_path));
            changed[0]["unexpected_content"] = Value::Bool(true);
            std::fs::write(
                root.join(oracle_path),
                serde_json::to_vec(&changed).unwrap(),
            )
            .unwrap();
        } else if case == "missing_oracle" {
            std::fs::remove_file(root.join(oracle_path)).unwrap();
        }
        let listing_path = root.join("listing.log");
        let execution_path = root.join("execution.log");
        let result_path = root.join("result.json");
        std::fs::write(&listing_path, listing).unwrap();
        std::fs::write(&execution_path, execution).unwrap();
        let output = Command::new(if cfg!(windows) { "python" } else { "python3" })
            .arg(repo.join("scripts/verify-ci-membership.py"))
            .arg("--root")
            .arg(root)
            .arg("--listing")
            .arg(&listing_path)
            .arg("--execution")
            .arg(&execution_path)
            .args([
                "--suite-exit",
                if case == "cargo_failure" { "1" } else { "0" },
            ])
            .arg("--output")
            .arg(&result_path)
            .output()
            .expect("Python is an existing repository verification prerequisite");
        let expected_success = matches!(
            case,
            "valid" | "lf_checkout" | "crlf_checkout" | "interleaved_headers" | "nested_worker"
        );
        assert_eq!(
            output.status.success(),
            expected_success,
            "{case}: stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let result = load(&result_path);
        assert_eq!(
            result["errors"].as_array().unwrap().is_empty(),
            expected_success
        );
        if matches!(
            case,
            "invalid_oracle" | "oracle_content_changed" | "missing_oracle"
        ) {
            std::fs::copy(repo.join(oracle_path), root.join(oracle_path)).unwrap();
        }
        if matches!(case, "empty_fitness" | "incomplete_fitness") {
            std::fs::copy(
                repo.join("verification/ci-verify-r1/fitness-membership.json"),
                &fitness_manifest,
            )
            .unwrap();
        }
    }
}
