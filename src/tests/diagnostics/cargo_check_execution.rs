use super::*;
use crate::diagnostics::cargo_check::{
    ApprovedCargoExecutable, ApprovedWorkspaceRoot, AuthoritySource, CargoCheckRequest,
};
use std::io::Write;

const DESCENDANT_ARGUMENT: &str = "--clean-ctx-owned-execution-test-descendant";

/// The test binary itself is a portable synthetic producer, intercepted before
/// libtest argument handling. Only private fixture commands take this route;
/// production CargoCheck's invocation remains exactly the two frozen args.
#[ctor::ctor]
fn execution_fixture_entrypoint() {
    let arguments: Vec<_> = std::env::args_os().skip(1).collect();
    let descendant = arguments.len() == 1 && arguments[0] == DESCENDANT_ARGUMENT;
    let producer =
        arguments.len() == 2 && arguments[0] == "check" && arguments[1] == "--message-format=json";
    if !descendant && !producer {
        return;
    }
    let Ok(mode) = std::fs::read_to_string(".owned-execution-fixture") else {
        return;
    };
    if descendant {
        std::fs::write("descendant-ready", b"ready").expect("descendant ready");
        if mode == "root-exits-first" {
            std::thread::sleep(Duration::from_millis(150));
            std::fs::write("descendant-finished", b"done").expect("descendant finished");
            std::process::exit(0);
        }
        loop {
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    match mode.as_str() {
        "failure-with-success-evidence" => {
            println!("{{\"reason\":\"build-finished\",\"success\":true}}");
            eprintln!("synthetic producer failure");
            std::process::exit(7);
        }
        "dual-pipe-flood" => {
            let err = std::thread::spawn(|| {
                let mut stderr = std::io::stderr().lock();
                for _ in 0..1024 {
                    stderr.write_all(&[b'e'; 4095]).unwrap();
                    stderr.write_all(b"\n").unwrap();
                }
                stderr.write_all(b"terminal stderr cause\n").unwrap();
            });
            let mut stdout = std::io::stdout().lock();
            for _ in 0..2048 {
                stdout.write_all(&[b'o'; 4095]).unwrap();
                stdout.write_all(b"\n").unwrap();
            }
            stdout
                .write_all(b"{\"reason\":\"build-finished\",\"success\":true}\n")
                .unwrap();
            drop(stdout);
            err.join().unwrap();
            std::process::exit(0);
        }
        "root-exits-first" | "cancel-tree" => {
            let mut child = std::process::Command::new(std::env::current_exe().unwrap())
                .arg(DESCENDANT_ARGUMENT)
                .spawn()
                .expect("fixture descendant");
            std::fs::write("descendant-pid", child.id().to_string()).unwrap();
            if mode == "root-exits-first" {
                std::process::exit(0);
            }
            child.wait().expect("fixture descendant wait");
            std::process::exit(0);
        }
        _ => std::process::exit(0),
    }
}

fn invocation(mode: &str) -> (tempfile::TempDir, CargoCheckInvocation) {
    let workspace = tempfile::tempdir().unwrap();
    std::fs::write(
        workspace.path().join("Cargo.toml"),
        "[package]\nname='fixture'\n",
    )
    .unwrap();
    std::fs::write(workspace.path().join(".owned-execution-fixture"), mode).unwrap();
    let root =
        ApprovedWorkspaceRoot::admit(workspace.path(), AuthoritySource::StartupOption).unwrap();
    let cargo = ApprovedCargoExecutable::admit(
        &std::env::current_exe().unwrap(),
        AuthoritySource::StartupOption,
    )
    .unwrap();
    let invocation =
        CargoCheckInvocation::prepare(&CargoCheckRequest::new(root), &cargo, std::env::vars_os())
            .unwrap();
    (workspace, invocation)
}

#[test]
#[cfg(any(target_os = "linux", windows))]
fn cancellation_is_first_request_wins_and_prevents_launch() {
    let (_workspace, invocation) = invocation("success");
    let cancellation = CargoCheckCancellation::default();
    cancellation.request(CancellationSource::User);
    cancellation.request(CancellationSource::Host);
    assert!(matches!(
        execute_cargo_check(&invocation, &cancellation),
        Err(ExecutionError::CancelledBeforeStart {
            cancellation_source: CancellationSource::User
        })
    ));
}

#[test]
#[cfg(any(target_os = "linux", windows))]
fn authority_is_revalidated_after_preparation_at_launch_boundary() {
    let (workspace, invocation) = invocation("success");
    std::fs::write(workspace.path().join("Cargo.toml"), "changed identity").unwrap();
    assert!(matches!(
        execute_cargo_check(&invocation, &CargoCheckCancellation::default()),
        Err(ExecutionError::Authority(_))
    ));
}

#[cfg(any(target_os = "linux", windows))]
#[test]
fn root_exit_is_independent_of_cargo_build_finished_evidence() {
    let (_workspace, invocation) = invocation("failure-with-success-evidence");
    let result = execute_cargo_check(&invocation, &CargoCheckCancellation::default()).unwrap();
    assert_eq!(result.root_outcome.unwrap().exit_code, Some(7));
    assert_eq!(
        result.semantic.cargo_evidence.build_finished_success,
        Some(true)
    );
    assert!(result.cleanup.descendant_quiescence_observed);
    assert!(!result.cleanup.cleanup_uncertain);
    assert!(result.capture.stdout.eof_observed && result.capture.stderr.eof_observed);
    assert!(!result.intrinsic_timeout && !result.capture.spool_used);
}

#[cfg(any(target_os = "linux", windows))]
#[test]
fn both_pipes_drain_beyond_capture_limits_without_deadlock_or_false_completion() {
    let (_workspace, invocation) = invocation("dual-pipe-flood");
    let result = execute_cargo_check(&invocation, &CargoCheckCancellation::default()).unwrap();
    assert_eq!(result.root_outcome.unwrap().exit_code, Some(0));
    assert_eq!(result.capture.aggregate_admitted_bytes, 12 * 1024 * 1024);
    assert!(result.capture.stdout.capture_limit_activated);
    assert!(result.capture.stderr.capture_limit_activated);
    assert!(result.capture.stdout.discarded_bytes > 0);
    assert!(result.capture.stderr.discarded_bytes > 0);
    assert_eq!(result.semantic.cargo_evidence.build_finished_success, None);
    assert!(
        result
            .semantic
            .evidence
            .iter()
            .any(|item| item.text.contains("terminal stderr cause"))
    );
}

#[cfg(any(target_os = "linux", windows))]
#[test]
fn root_exit_does_not_finish_before_owned_descendants_finish() {
    let (workspace, invocation) = invocation("root-exits-first");
    let result = execute_cargo_check(&invocation, &CargoCheckCancellation::default()).unwrap();
    assert!(workspace.path().join("descendant-finished").exists());
    assert!(result.cleanup.descendant_quiescence_observed);
    assert_eq!(
        result.ownership.controller_death_cleanup_guaranteed,
        cfg!(windows)
    );
    assert_eq!(
        result.ownership.deliberate_detachment_excluded,
        cfg!(target_os = "linux")
    );
}

#[cfg(any(target_os = "linux", windows))]
#[test]
fn cancellation_forces_owned_descendant_cleanup_and_returns_observed_root_outcome() {
    let (workspace, invocation) = invocation("cancel-tree");
    let cancellation = CargoCheckCancellation::default();
    let cancel = cancellation.clone();
    let ready = workspace.path().join("descendant-ready");
    let requester = std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        while !ready.exists() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(10));
        }
        let observed_ready = ready.exists();
        cancel.request(CancellationSource::Host);
        observed_ready
    });
    let result = execute_cargo_check(&invocation, &cancellation).unwrap();
    assert!(
        requester.join().unwrap(),
        "descendant must run before cancellation"
    );
    assert_eq!(
        result.cleanup.cancellation_source,
        Some(CancellationSource::Host)
    );
    assert!(result.cleanup.forced_termination);
    assert!(result.cleanup.termination_request_succeeded);
    assert!(result.cleanup.cancellation_requested_after_millis.is_some());
    assert!(result.root_outcome.is_some());
    assert!(result.cleanup.descendant_quiescence_observed);
    assert!(!result.cleanup.cleanup_uncertain);
    #[cfg(target_os = "linux")]
    assert_eq!(result.root_outcome.unwrap().signal, Some(9));
}
