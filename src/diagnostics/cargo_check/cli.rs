//! Local operator CLI authority; no repository/CWD/PATH execution fallback.
use super::environment::authority_name_matches;
use super::*;
use serde::Serialize;
use std::ffi::OsString;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Clone, clap::Args)]
pub struct CargoCheckCliOptions {
    /// Approved absolute workspace containing Cargo.toml (required)
    #[arg(long)]
    pub workspace_root: PathBuf,
    /// Approved absolute Cargo executable; otherwise use CLEAN_CTX_CARGO_PATH
    #[arg(long)]
    pub cargo_path: Option<PathBuf>,
    /// Emit the bounded structured result instead of text
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CliDisposition {
    Success,
    CargoNonzero,
    AdmissionFailure,
    InternalFailure,
    Cancelled,
    CleanupUncertain,
}

impl CliDisposition {
    pub fn exit_code(self) -> u8 {
        match self {
            Self::Success => 0,
            Self::CargoNonzero => 1,
            Self::AdmissionFailure => 2,
            Self::InternalFailure => 3,
            Self::Cancelled => 4,
            Self::CleanupUncertain => 5,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::CargoNonzero => "cargo_nonzero",
            Self::AdmissionFailure => "admission_failure",
            Self::InternalFailure => "internal_failure",
            Self::Cancelled => "cancelled",
            Self::CleanupUncertain => "cleanup_uncertain",
        }
    }
    fn output_failed(self) -> Self {
        match self {
            Self::CleanupUncertain | Self::Cancelled => self,
            _ => Self::InternalFailure,
        }
    }
}

pub fn cli_disposition(execution: &CargoCheckExecution, internal_failure: bool) -> CliDisposition {
    if execution.cleanup.cleanup_uncertain || !execution.cleanup.descendant_quiescence_observed {
        CliDisposition::CleanupUncertain
    } else if execution.cleanup.cancellation_source.is_some() {
        CliDisposition::Cancelled
    } else if internal_failure
        || execution.cleanup.control_failed
        || execution.capture.stdout.read_failed
        || execution.capture.stderr.read_failed
        || execution.capture.stdout.stopped_before_eof
        || execution.capture.stderr.stopped_before_eof
        || execution
            .semantic
            .result_budget
            .mandatory_facts_exceed_budget
    {
        CliDisposition::InternalFailure
    } else {
        match execution.root_outcome {
            Some(ProcessOutcome {
                exit_code: Some(0),
                signal: None,
            }) => CliDisposition::Success,
            Some(_) => CliDisposition::CargoNonzero,
            None => CliDisposition::InternalFailure,
        }
    }
}

pub(crate) fn prepare_cli(
    options: &CargoCheckCliOptions,
    snapshot: &[(OsString, OsString)],
) -> Result<CargoCheckInvocation, &'static str> {
    let workspace =
        ApprovedWorkspaceRoot::admit(&options.workspace_root, AuthoritySource::CliArgument)
            .map_err(|_| "workspace_admission_failed")?;
    let (path, source) = if let Some(path) = &options.cargo_path {
        (path.clone(), AuthoritySource::CliArgument)
    } else {
        let path = snapshot
            .iter()
            .rev()
            .find(|(name, _)| authority_name_matches(name, "CLEAN_CTX_CARGO_PATH"))
            .map(|(_, value)| PathBuf::from(value))
            .ok_or("cargo_authority_missing")?;
        (path, AuthoritySource::Environment)
    };
    let cargo =
        ApprovedCargoExecutable::admit(&path, source).map_err(|_| "cargo_admission_failed")?;
    CargoCheckInvocation::prepare(
        &CargoCheckRequest::new(workspace),
        &cargo,
        snapshot.iter().cloned(),
    )
    .map_err(|_| "authority_revalidation_failed")
}

pub struct CargoCheckCliReport {
    pub disposition: CliDisposition,
    output: Vec<u8>,
}

impl CargoCheckCliReport {
    pub fn output(&self) -> &[u8] {
        &self.output
    }
    /// Write exactly the checked projection, without a newline beyond its cap.
    pub fn write_to(&self, writer: &mut impl Write) -> u8 {
        if writer
            .write_all(&self.output)
            .and_then(|_| writer.flush())
            .is_err()
        {
            self.disposition.output_failed().exit_code()
        } else {
            self.disposition.exit_code()
        }
    }

    pub fn handler_failure(json: bool) -> Self {
        failure_report(
            json,
            CliDisposition::InternalFailure,
            "cancellation_handler_unavailable",
            None,
            None,
        )
    }
}

pub fn run_cargo_check_cli(
    options: &CargoCheckCliOptions,
    snapshot: &[(OsString, OsString)],
    cancellation: &CargoCheckCancellation,
) -> CargoCheckCliReport {
    let invocation = match prepare_cli(options, snapshot) {
        Ok(invocation) => invocation,
        Err(kind) => {
            return failure_report(
                options.json,
                CliDisposition::AdmissionFailure,
                kind,
                None,
                None,
            );
        }
    };
    let execution = match execute_cargo_check(&invocation, cancellation) {
        Ok(execution) => execution,
        Err(error) => {
            let (disposition, kind, source) = match error {
                ExecutionError::Authority(_) => (
                    CliDisposition::AdmissionFailure,
                    "authority_changed_before_start",
                    None,
                ),
                ExecutionError::UnsupportedPlatform => (
                    CliDisposition::AdmissionFailure,
                    "ownership_platform_unsupported",
                    None,
                ),
                ExecutionError::CancelledBeforeStart {
                    cancellation_source,
                } => (
                    CliDisposition::Cancelled,
                    "cancelled_before_start",
                    Some(cancellation_source),
                ),
                ExecutionError::Start { .. } => (
                    CliDisposition::InternalFailure,
                    "owned_execution_start_failed",
                    None,
                ),
            };
            return failure_report(options.json, disposition, kind, None, source);
        }
    };
    let disposition = cli_disposition(&execution, false);
    match super::presentation::project_for_cli(&execution, CargoCheckPolicy::APPROVED, disposition)
    {
        Ok(projection) => CargoCheckCliReport {
            disposition,
            output: if options.json {
                projection.structured_json().to_vec()
            } else {
                projection.content().as_bytes().to_vec()
            },
        },
        Err(_) => failure_report(
            options.json,
            cli_disposition(&execution, true),
            "bounded_projection_failed",
            Some(&execution),
            None,
        ),
    }
}

#[derive(Serialize)]
struct Failure<'a> {
    operation: &'static str,
    cli_outcome: CliDisposition,
    exit_code: u8,
    failure: &'static str,
    authority_details_withheld: bool,
    root_outcome: Option<ProcessOutcome>,
    cancellation_before_start: Option<CancellationSource>,
    cleanup: Option<&'a CleanupFacts>,
    capture: Option<&'a CaptureFacts>,
    parser_coverage: Option<&'a ParserCoverage>,
    cargo_evidence: Option<&'a CargoEvidence>,
}

fn failure_report(
    json: bool,
    disposition: CliDisposition,
    kind: &'static str,
    execution: Option<&CargoCheckExecution>,
    cancellation_before_start: Option<CancellationSource>,
) -> CargoCheckCliReport {
    let failure = Failure {
        operation: "cargo_check",
        cli_outcome: disposition,
        exit_code: disposition.exit_code(),
        failure: kind,
        authority_details_withheld: execution.is_some(),
        root_outcome: execution.and_then(|execution| execution.root_outcome),
        cancellation_before_start,
        cleanup: execution.map(|execution| &execution.cleanup),
        capture: execution.map(|execution| &execution.capture),
        parser_coverage: execution.map(|execution| &execution.semantic.parser_coverage),
        cargo_evidence: execution.map(|execution| &execution.semantic.cargo_evidence),
    };
    // This closed error schema contains only fixed labels, enum values, and
    // bounded numeric facts; no caller path, environment value, or producer text.
    let output = if json {
        serde_json::to_vec(&failure).expect("closed CLI failure schema is serializable")
    } else {
        format!("CargoCheck outcome: {} (exit {}); failure: {kind}; actual Cargo process: {:?}; cleanup uncertain: {}; cancellation: {:?}",
            disposition.label(), disposition.exit_code(), failure.root_outcome,
            execution.is_some_and(|execution| execution.cleanup.cleanup_uncertain),
            cancellation_before_start.or_else(|| execution.and_then(|execution| execution.cleanup.cancellation_source))).into_bytes()
    };
    CargoCheckCliReport {
        disposition,
        output,
    }
}

#[cfg(test)]
#[path = "../../tests/diagnostics/cargo_check_cli.rs"]
mod tests;
