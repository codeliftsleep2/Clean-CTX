//! OS process authority is independent of the Cargo JSON producer's evidence.
//! This boundary has no shell, caller arguments, spool, or intrinsic deadline.
mod capture;
mod facts;
mod platform;

use super::{CargoCheckCompiler, CargoCheckInvocation, CargoCheckSemanticResult};
pub use facts::{
    CancellationSource, CaptureFacts, CargoCheckCancellation, CleanupFacts, ExecutionError,
    OwnershipFacts, ProcessOutcome, StreamCaptureFacts,
};
use platform::OwnedProcess;
use serde::Serialize;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

#[derive(Serialize)]
pub struct CargoCheckExecution {
    pub authority: super::invocation::InvocationFacts,
    pub root_outcome: Option<ProcessOutcome>,
    pub ownership: OwnershipFacts,
    pub cleanup: CleanupFacts,
    pub capture: CaptureFacts,
    pub intrinsic_timeout: bool,
    pub semantic: CargoCheckSemanticResult,
}

/// Run exactly one prepared CargoCheck operation. A caller may cancel using a
/// cloned handle from another thread. The handle does not supply new authority.
pub fn execute_cargo_check(
    invocation: &CargoCheckInvocation,
    cancellation: &CargoCheckCancellation,
) -> Result<CargoCheckExecution, ExecutionError> {
    if !cfg!(any(target_os = "linux", windows)) {
        return Err(ExecutionError::UnsupportedPlatform);
    }
    if let Some((source, _)) = cancellation.observed() {
        return Err(ExecutionError::CancelledBeforeStart {
            cancellation_source: source,
        });
    }
    let started = Instant::now();
    let (mut process, stdout, stderr) = OwnedProcess::start(invocation)?;
    let compiler = Mutex::new(CargoCheckCompiler::for_workspace(
        invocation.workspace_authority().clone(),
    ));
    let stop = AtomicBool::new(false);
    let mut cleanup = CleanupFacts::default();
    let mut root_outcome = None;
    let (stdout_facts, stderr_facts) = std::thread::scope(|scope| {
        let out = std::thread::Builder::new()
            .name("cargo-check-stdout".into())
            .spawn_scoped(scope, || capture::drain(stdout, true, &compiler, &stop));
        let err = std::thread::Builder::new()
            .name("cargo-check-stderr".into())
            .spawn_scoped(scope, || capture::drain(stderr, false, &compiler, &stop));
        if let Err(error) = &out {
            control_failure(&mut cleanup, error);
        }
        if let Err(error) = &err {
            control_failure(&mut cleanup, error);
        }
        let mut out = out.ok();
        let mut err = err.ok();
        let mut out_facts = None;
        let mut err_facts = None;
        loop {
            for (reader, facts) in [(&mut out, &mut out_facts), (&mut err, &mut err_facts)] {
                if reader.as_ref().is_some_and(|reader| reader.is_finished()) {
                    let observed = join_reader(reader.take());
                    if observed.read_failed {
                        cleanup.control_failed = true;
                        cleanup.control_error_os_code = observed.read_error_os_code;
                    }
                    *facts = Some(observed);
                }
            }
            if cleanup.cancellation_source.is_none() {
                if let Some((source, time)) = cancellation.observed() {
                    cleanup.cancellation_source = Some(source);
                    cleanup.cancellation_requested_after_millis = Some(
                        time.saturating_duration_since(started)
                            .as_millis()
                            .min(u64::MAX as u128) as u64,
                    );
                }
            }
            if (cleanup.cancellation_source.is_some() || cleanup.control_failed)
                && !cleanup.forced_termination
            {
                cleanup.forced_termination = true;
                cleanup.forced_mechanism = Some(OwnedProcess::FORCED_MECHANISM);
                match process.terminate() {
                    Ok(()) => cleanup.termination_request_succeeded = true,
                    Err(error) => control_failure(&mut cleanup, &error),
                }
                // Nonblocking readers must not be held hostage by a deliberately
                // detached pipe holder after host/user cancellation.
                stop.store(true, Ordering::Release);
            }
            match process.poll() {
                Ok(outcome) => {
                    if outcome.is_some() {
                        root_outcome = outcome;
                    }
                }
                Err(error) => control_failure(&mut cleanup, &error),
            }
            match process.quiescent() {
                Ok(quiet) => cleanup.descendant_quiescence_observed = quiet,
                Err(error) => control_failure(&mut cleanup, &error),
            }
            let readers_done = out_facts.is_some() && err_facts.is_some();
            if root_outcome.is_some() && cleanup.descendant_quiescence_observed && readers_done {
                break;
            }
            if cleanup.control_failed && cleanup.forced_termination {
                cleanup.cleanup_uncertain = true;
                stop.store(true, Ordering::Release);
                break;
            }
            // Local scheduling cadence, not a producer timeout or grace period.
            std::thread::sleep(Duration::from_millis(10));
        }
        (
            out_facts.unwrap_or_else(|| join_reader(out)),
            err_facts.unwrap_or_else(|| join_reader(err)),
        )
    });
    if root_outcome.is_some() && cleanup.descendant_quiescence_observed {
        if let Err(error) = process.finish() {
            control_failure(&mut cleanup, &error);
        }
    }
    cleanup.cleanup_uncertain |= cleanup.control_failed || !cleanup.descendant_quiescence_observed;
    let capture = CaptureFacts {
        aggregate_admitted_bytes: stdout_facts.admitted_bytes + stderr_facts.admitted_bytes,
        aggregate_discarded_bytes: stdout_facts
            .discarded_bytes
            .saturating_add(stderr_facts.discarded_bytes),
        stdout: stdout_facts,
        stderr: stderr_facts,
        spool_used: false,
    };
    let semantic = compiler
        .into_inner()
        .unwrap_or_else(|error| error.into_inner())
        .finish();
    Ok(CargoCheckExecution {
        authority: invocation.result_facts(),
        root_outcome,
        ownership: OwnershipFacts {
            mechanism: OwnedProcess::MECHANISM,
            ownership_established_before_execution: true,
            controller_death_cleanup_guaranteed: cfg!(windows),
            deliberate_detachment_excluded: cfg!(target_os = "linux"),
        },
        cleanup,
        capture,
        intrinsic_timeout: false,
        semantic,
    })
}

fn control_failure(facts: &mut CleanupFacts, error: &std::io::Error) {
    facts.control_failed = true;
    if facts.control_error_os_code.is_none() {
        facts.control_error_os_code = error.raw_os_error();
    }
}

fn join_reader(
    reader: Option<std::thread::ScopedJoinHandle<'_, StreamCaptureFacts>>,
) -> StreamCaptureFacts {
    reader
        .and_then(|reader| reader.join().ok())
        .unwrap_or(StreamCaptureFacts {
            read_failed: true,
            stopped_before_eof: true,
            ..StreamCaptureFacts::default()
        })
}

#[cfg(test)]
#[path = "../../../tests/diagnostics/cargo_check_execution.rs"]
mod tests;
