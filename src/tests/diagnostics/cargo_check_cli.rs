use super::*;
use serde_json::Value;

fn fixture(mode: &str, json: bool) -> (tempfile::TempDir, CargoCheckCliOptions) {
    let workspace = tempfile::tempdir().unwrap();
    std::fs::write(
        workspace.path().join("Cargo.toml"),
        "[package]\nname='fixture'\n",
    )
    .unwrap();
    std::fs::write(workspace.path().join(".owned-execution-fixture"), mode).unwrap();
    let options = CargoCheckCliOptions {
        workspace_root: workspace.path().to_owned(),
        cargo_path: Some(std::env::current_exe().unwrap()),
        json,
    };
    (workspace, options)
}

#[test]
fn cli_requires_explicit_workspace_and_never_uses_the_environment_root() {
    let (_workspace, mut options) = fixture("success", true);
    options.workspace_root = PathBuf::from("relative_PRIVATE_PATH_CANARY");
    let snapshot = vec![(
        OsString::from("CLEAN_CTX_PROJECT_ROOT"),
        OsString::from("/other/root"),
    )];
    let report = run_cargo_check_cli(&options, &snapshot, &CargoCheckCancellation::default());
    assert_eq!(report.disposition, CliDisposition::AdmissionFailure);
    let result: Value = serde_json::from_slice(report.output()).unwrap();
    assert_eq!(result["exit_code"], 2);
    assert_eq!(result["failure"], "workspace_admission_failed");
    assert!(!String::from_utf8_lossy(report.output()).contains("PRIVATE_PATH_CANARY"));
}

#[test]
fn invalid_explicit_cargo_authority_never_falls_back_to_valid_environment_authority() {
    let (_workspace, mut options) = fixture("success", true);
    let valid = options.cargo_path.clone().unwrap();
    options.cargo_path = Some(PathBuf::from("cargo"));
    let snapshot = vec![(
        OsString::from("CLEAN_CTX_CARGO_PATH"),
        valid.into_os_string(),
    )];
    assert!(matches!(
        prepare_cli(&options, &snapshot),
        Err("cargo_admission_failed")
    ));
}

#[test]
fn explicit_option_wins_and_shadowed_environment_authority_is_disclosed() {
    let (_workspace, options) = fixture("success", true);
    let snapshot = vec![
        (
            OsString::from("CLEAN_CTX_CARGO_PATH"),
            OsString::from("relative"),
        ),
        (
            OsString::from("CLEAN_CTX_PROJECT_ROOT"),
            OsString::from("ignored"),
        ),
    ];
    let invocation = prepare_cli(&options, &snapshot).unwrap();
    let facts = invocation.result_facts();
    assert_eq!(facts.cargo_source, AuthoritySource::CliArgument);
    assert!(facts.cargo_environment_present && facts.cargo_environment_shadowed);
    assert_eq!(facts.workspace_source, AuthoritySource::CliArgument);
    assert!(facts.workspace_environment_present && facts.workspace_environment_shadowed);
    assert!(!serde_json::to_string(&facts).unwrap().contains("relative"));
}

#[test]
fn cargo_environment_selection_uses_only_the_supplied_startup_snapshot() {
    let (_workspace, mut options) = fixture("success", true);
    let executable = options.cargo_path.take().unwrap();
    let mut snapshot = vec![(
        OsString::from("CLEAN_CTX_CARGO_PATH"),
        executable.into_os_string(),
    )];
    let invocation = prepare_cli(&options, &snapshot).unwrap();
    snapshot[0].1 = OsString::from("changed_PRIVATE_CANARY");
    assert!(invocation.executable().is_absolute());
    assert_eq!(
        invocation.result_facts().cargo_source,
        AuthoritySource::Environment
    );
    assert!(!invocation.result_facts().cargo_environment_shadowed);
    assert!(
        !serde_json::to_string(&invocation.result_facts())
            .unwrap()
            .contains("changed_PRIVATE_CANARY")
    );
    assert!(matches!(
        prepare_cli(&options, &[]),
        Err("cargo_authority_missing")
    ));
}

#[test]
#[cfg(any(target_os = "linux", windows))]
fn cli_reports_actual_nonzero_cargo_status_even_when_cargo_asserts_success() {
    let (_workspace, options) = fixture("failure-with-success-evidence", true);
    let report = run_cargo_check_cli(
        &options,
        &std::env::vars_os().collect::<Vec<_>>(),
        &CargoCheckCancellation::default(),
    );
    assert_eq!(report.disposition, CliDisposition::CargoNonzero);
    let result: Value = serde_json::from_slice(report.output()).unwrap();
    assert_eq!(result["cli_outcome"], "cargo_nonzero");
    assert_eq!(result["exit_code"], 1);
    assert_eq!(result["root_outcome"]["exit_code"], 7);
    assert_eq!(
        result["semantic"]["cargo_evidence"]["build_finished_success"],
        true
    );
    assert_eq!(
        result["semantic"]["result_budget"]["serialized_bytes"],
        report.output().len()
    );
    assert!(report.output().len() <= CargoCheckPolicy::APPROVED.structured_content_bytes);
}

#[test]
#[cfg(any(target_os = "linux", windows))]
fn cli_exit_precedence_preserves_cleanup_cancellation_and_internal_failure_distinctions() {
    let (_workspace, options) = fixture("success", true);
    let invocation = prepare_cli(&options, &std::env::vars_os().collect::<Vec<_>>()).unwrap();
    let mut execution =
        execute_cargo_check(&invocation, &CargoCheckCancellation::default()).unwrap();
    assert_eq!(cli_disposition(&execution, false).exit_code(), 0);
    execution.root_outcome = Some(ProcessOutcome {
        exit_code: Some(99),
        signal: None,
    });
    assert_eq!(cli_disposition(&execution, false).exit_code(), 1);
    assert_eq!(cli_disposition(&execution, true).exit_code(), 3);
    execution.cleanup.cancellation_source = Some(CancellationSource::User);
    assert_eq!(cli_disposition(&execution, true).exit_code(), 4);
    execution.cleanup.cleanup_uncertain = true;
    assert_eq!(cli_disposition(&execution, true).exit_code(), 5);
    execution.cleanup.cleanup_uncertain = false;
    execution.cleanup.cancellation_source = None;
    execution.root_outcome = Some(ProcessOutcome {
        exit_code: Some(0),
        signal: None,
    });
    execution.capture.stdout.read_failed = true;
    assert_eq!(cli_disposition(&execution, false).exit_code(), 3);
}

#[test]
#[cfg(any(target_os = "linux", windows))]
fn cli_cancellation_before_start_has_no_cargo_status_and_exits_four() {
    let (workspace, options) = fixture("success", true);
    let cancellation = CargoCheckCancellation::default();
    cancellation.request(CancellationSource::User);
    let report = run_cargo_check_cli(&options, &[], &cancellation);
    let result: Value = serde_json::from_slice(report.output()).unwrap();
    assert_eq!(report.disposition.exit_code(), 4);
    assert!(result["root_outcome"].is_null());
    assert_eq!(result["cancellation_before_start"], "user");
    assert!(!workspace.path().join("descendant-ready").exists());
}

#[test]
#[cfg(any(target_os = "linux", windows))]
fn cli_text_names_success_and_writes_only_the_checked_projection() {
    let (_workspace, options) = fixture("success", false);
    let report = run_cargo_check_cli(
        &options,
        &std::env::vars_os().collect::<Vec<_>>(),
        &CargoCheckCancellation::default(),
    );
    let text = String::from_utf8_lossy(report.output());
    assert!(text.contains("CargoCheck outcome: success (exit 0)"));
    assert!(text.contains("Process outcome:"));
    assert!(text.contains("Coverage incomplete:"));
    assert!(report.output().len() <= 24 * 1024);
    let mut output = Vec::new();
    assert_eq!(report.write_to(&mut output), 0);
    assert_eq!(output, report.output());
}

#[test]
fn output_failure_becomes_internal_failure_without_overriding_cleanup_or_cancellation() {
    struct BrokenWriter;
    impl Write for BrokenWriter {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("private output error"))
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    for (disposition, expected) in [
        (CliDisposition::AdmissionFailure, 3),
        (CliDisposition::Cancelled, 4),
        (CliDisposition::CleanupUncertain, 5),
    ] {
        let report = failure_report(true, disposition, "fixture_failure", None, None);
        assert_eq!(report.write_to(&mut BrokenWriter), expected);
        assert!(!String::from_utf8_lossy(report.output()).contains("private output error"));
    }
}
