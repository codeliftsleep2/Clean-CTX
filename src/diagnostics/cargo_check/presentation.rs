//! Final adapter boundary: measure the whole structured envelope and render a
//! separate text projection. Neither adapter may serialize the internal core.
use super::budget::{enforce, prefix, serialized_size};
use super::model::CargoCheckSemanticResult;
use super::{BoundedResultError, CargoCheckExecution, CargoCheckPolicy};
use serde::Serialize;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct TextBudgetFacts {
    pub limit_bytes: usize,
    pub limit_lines: usize,
    pub bytes: usize,
    pub lines: usize,
    pub truncated: bool,
    pub diagnostics_represented: usize,
    pub diagnostics_available: usize,
    pub causal_preview_shown: bool,
}

pub struct CargoCheckProjection {
    structured_json: Vec<u8>,
    content: String,
    text_facts: TextBudgetFacts,
}

impl CargoCheckProjection {
    pub fn structured_json(&self) -> &[u8] {
        &self.structured_json
    }
    pub fn content(&self) -> &str {
        &self.content
    }
    pub fn text_facts(&self) -> &TextBudgetFacts {
        &self.text_facts
    }
}

#[derive(Serialize)]
struct Structured<'a> {
    authority: &'a super::InvocationFacts,
    root_outcome: Option<super::ProcessOutcome>,
    ownership: &'a super::OwnershipFacts,
    cleanup: &'a super::CleanupFacts,
    capture: &'a super::CaptureFacts,
    intrinsic_timeout: bool,
    semantic: &'a CargoCheckSemanticResult,
    text_budget: &'a TextBudgetFacts,
}

fn view<'a>(
    execution: &'a CargoCheckExecution,
    semantic: &'a CargoCheckSemanticResult,
    text_budget: &'a TextBudgetFacts,
) -> Structured<'a> {
    Structured {
        authority: &execution.authority,
        root_outcome: execution.root_outcome,
        ownership: &execution.ownership,
        cleanup: &execution.cleanup,
        capture: &execution.capture,
        intrinsic_timeout: execution.intrinsic_timeout,
        semantic,
        text_budget,
    }
}

pub fn project_cargo_check(
    execution: &CargoCheckExecution,
    policy: CargoCheckPolicy,
) -> Result<CargoCheckProjection, BoundedResultError> {
    let mut semantic = execution.semantic.clone();
    let mut text_facts = TextBudgetFacts::default();
    loop {
        enforce(&mut semantic, policy.structured_content_bytes, |semantic| {
            serialized_size(&view(execution, semantic, &text_facts))
        })?;
        let (content, facts) = render_text(execution, &semantic, policy)?;
        text_facts = facts;
        let size = serialized_size(&view(execution, &semantic, &text_facts))?;
        if size != semantic.result_budget.serialized_bytes {
            continue;
        }
        let structured_json = serde_json::to_vec(&view(execution, &semantic, &text_facts))
            .map_err(|_| BoundedResultError::Serialization)?;
        if structured_json.len() > policy.structured_content_bytes {
            return Err(BoundedResultError::MandatoryFacts);
        }
        return Ok(CargoCheckProjection {
            structured_json,
            content,
            text_facts,
        });
    }
}

fn render_text(
    execution: &CargoCheckExecution,
    result: &CargoCheckSemanticResult,
    policy: CargoCheckPolicy,
) -> Result<(String, TextBudgetFacts), BoundedResultError> {
    let coverage = &result.parser_coverage;
    let capture = &execution.capture;
    let incomplete = coverage.unparsed_stdout_frames != 0
        || coverage.non_json_stdout != 0
        || coverage.unparsed_stderr_frames != 0
        || coverage.malformed_json != 0
        || coverage.incompatible_structured != 0
        || coverage.unknown_structured != 0
        || !capture.stdout.eof_observed
        || !capture.stderr.eof_observed
        || execution.cleanup.cleanup_uncertain;
    let cargo_label = serde_json::to_string(&execution.authority.cargo)
        .map_err(|_| BoundedResultError::Serialization)?;
    let mut lines = vec![
        format!(
            "CargoCheck: {} via {} ({:?}); {}",
            execution.authority.workspace,
            cargo_label,
            execution.authority.cargo_source,
            execution.authority.command
        ),
        format!(
            "Process outcome: {:?}; Cargo build-finished assertion: {:?} (separate authority)",
            execution.root_outcome, result.cargo_evidence.build_finished_success
        ),
        format!(
            "Ownership: {}; cancellation: {:?}; forced: {}; descendants quiet: {}; cleanup uncertain: {}",
            execution.ownership.mechanism,
            execution.cleanup.cancellation_source,
            execution.cleanup.forced_termination,
            execution.cleanup.descendant_quiescence_observed,
            execution.cleanup.cleanup_uncertain
        ),
        format!(
            "Coverage incomplete: {incomplete}; stdout frames: {}; stderr frames: {}; non-JSON stdout: {}; unparsed: {}; malformed: {}; incompatible: {}; unknown: {}",
            coverage.stdout_frames,
            coverage.stderr_frames,
            coverage.non_json_stdout,
            coverage.unparsed_stdout_frames + coverage.unparsed_stderr_frames,
            coverage.malformed_json,
            coverage.incompatible_structured,
            coverage.unknown_structured
        ),
        format!(
            "Capture discarded bytes: {}; stdout EOF: {}; stderr EOF: {}; control failed: {}; intrinsic timeout: {}",
            capture.aggregate_discarded_bytes,
            capture.stdout.eof_observed,
            capture.stderr.eof_observed,
            execution.cleanup.control_failed,
            execution.intrinsic_timeout
        ),
        format!(
            "Diagnostics seen: {}; retained: {}; omitted: {}; repeats compacted: {}; structured budget activated: {}; reductions: {:?}",
            result.retention.diagnostics_seen,
            result.diagnostics.len(),
            result.retention.diagnostics_omitted,
            result.retention.exact_repeats_collapsed,
            result.result_budget.activated,
            result.result_budget.fields_reduced
        ),
    ];
    let available = result.diagnostics.len();
    let footer = |shown, truncated| {
        format!(
            "Text truncated: {truncated}; diagnostic messages shown: {shown}/{available}. More bounded details and omission facts are in the structured result."
        )
    };
    let reserve = footer(available, false).len();
    let mandatory_bytes: usize = lines.iter().map(|line| line.len() + 1).sum::<usize>() + reserve;
    if mandatory_bytes > policy.content_bytes || lines.len() + 1 > policy.content_lines {
        return Err(BoundedResultError::MandatoryText);
    }
    let mut represented = 0;
    let mut clipped = false;
    let mut causal_preview_shown = false;
    let candidates: Vec<_> = if available != 0 {
        result
            .diagnostics
            .iter()
            .map(|item| (item.level, item.message.as_str(), false))
            .collect()
    } else {
        result
            .result_budget
            .causal_preview
            .as_ref()
            .map(|preview| vec![(preview.level, preview.message.as_str(), true)])
            .unwrap_or_default()
    };
    for (level, message, preview) in candidates {
        let used = lines.iter().map(|line| line.len() + 1).sum::<usize>();
        let remaining = policy.content_bytes.saturating_sub(used + reserve + 1);
        if lines.len() + 2 > policy.content_lines
            || remaining < format!("{level:?}: [message truncated]").len()
        {
            clipped = true;
            break;
        }
        // JSON quoting keeps producer newlines/control text inside one labeled
        // line. Text fragments are labeled excerpts when an escaped line is cut.
        let escaped =
            serde_json::to_string(message).map_err(|_| BoundedResultError::Serialization)?;
        let label = if preview {
            format!("{level:?} causal preview (structured diagnostic omitted): ")
        } else {
            format!("{level:?}: ")
        };
        let complete = format!("{label}{escaped}");
        let line = if complete.len() <= remaining {
            complete
        } else {
            let suffix = " [message truncated]";
            if remaining < label.len() + suffix.len() {
                clipped = true;
                break;
            }
            clipped = true;
            format!(
                "{label}{}{suffix}",
                prefix(&escaped, remaining - label.len() - suffix.len())
            )
        };
        lines.push(line);
        if preview {
            causal_preview_shown = true;
        } else {
            represented += 1;
        }
    }
    if (available != 0 && represented == 0)
        || (available == 0
            && result.result_budget.causal_preview.is_some()
            && !causal_preview_shown)
    {
        return Err(BoundedResultError::MandatoryText);
    }
    let truncated = clipped
        || represented < available
        || result
            .result_budget
            .causal_preview
            .as_ref()
            .is_some_and(|preview| causal_preview_shown && preview.truncated);
    lines.push(footer(represented, truncated));
    let content = lines.join("\n");
    if content.len() > policy.content_bytes || lines.len() > policy.content_lines {
        return Err(BoundedResultError::MandatoryText);
    }
    let facts = TextBudgetFacts {
        limit_bytes: policy.content_bytes,
        limit_lines: policy.content_lines,
        bytes: content.len(),
        lines: lines.len(),
        truncated,
        diagnostics_represented: represented,
        diagnostics_available: available,
        causal_preview_shown,
    };
    Ok((content, facts))
}

#[cfg(test)]
#[path = "../../tests/diagnostics/cargo_check_projection.rs"]
mod tests;
