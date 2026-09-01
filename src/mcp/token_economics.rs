// src/mcp/token_economics.rs
//
// Token-economics gate: a two-stage mechanism to ensure Clean-CTX never
// returns a compressed/hybrid representation that costs more tokens than
// the raw source.
//
// # Stage 1 — Preflight heuristic (Optimization hint)
//
// Before entering the expensive compression pipeline (IR compilation +
// render), cheaply predict whether compression at verbatim-body-preserving
// fidelities (Edit) is likely to produce a net token savings. If not, skip
// compression early and return raw passthrough.
//
// This is purely an optimization hint to avoid unnecessary work. The
// actual economic decision is made in Stage 2.
//
// Structural/skeleton fidelity levels (Low, Medium, High) strip method
// bodies entirely and produce substantial savings even on very small files.
// The preflight heuristic does not apply to them — they proceed to
// compression, and Stage 2 (post-compression) makes the final decision.
//
// # Stage 2 — Post-compression verification (Correctness gate)
//
// After the compressed/hybrid representation has been rendered, count the
// actual tokens of both the raw source and the candidate output. If the
// candidate costs more tokens than raw, fall back to raw passthrough.
//
// This check applies uniformly to all fidelity levels (Low, Medium, High,
// Edit). Verbatim fidelity bypasses compression entirely, so it is not
// affected.
//
// # Conservative bias
//
// The preflight estimator is an optimization hint, never a correctness
// mechanism. Both false-positive (compress when uneconomical) and
// false-negative (skip when economical) are safe. Calibration errs toward
// attempting compression near the boundary.

use crate::compression::Fidelity;

/// Calibration parameters for the token-economics estimator.
///
/// - fixed_overhead: Approximate fixed token cost of the compressed response
///   (header, import listing, structural markers, footers).
/// - expected_savings_ratio: Expected fractional savings (0.0-1.0) from
///   compression.
#[derive(Debug, Clone, Copy)]
pub(crate) struct EconomicsParams {
    pub fixed_overhead: f64,
    pub expected_savings_ratio: f64,
}

/// Determine whether compression is economically justified.
///
/// Returns true to attempt compression (or skip the gate for structural
/// fidelities), false to pass through raw content.
///
/// # Conservative bias
///
/// Near the threshold, this function errs toward true (attempt compression)
/// so borderline cases do not leave substantial savings unused.
pub(crate) fn should_attempt_compression(
    raw_tokens: usize,
    fidelity: Fidelity,
    extension: &str,
) -> bool {
    // Structural fidelities (Low, Medium, High) strip method bodies entirely
    // and produce substantial savings even on very small files.
    if matches!(fidelity, Fidelity::Low | Fidelity::Medium | Fidelity::High) {
        return true;
    }
    // Verbatim is already handled before this point with a direct raw return.
    if matches!(fidelity, Fidelity::Verbatim) {
        return true;
    }
    // Edit fidelity: verbatim body-preserving -- check economics.
    let params = params_for(fidelity, extension);
    // Basic threshold: raw_tokens > fixed_overhead / expected_savings_ratio
    let unbiased_threshold = params.fixed_overhead / params.expected_savings_ratio;
    // Conservative bias: multiply threshold by 0.85 to lower it, so
    // borderline cases (near the boundary) err toward attempting compression.
    let biased_threshold = unbiased_threshold * 0.85;
    (raw_tokens as f64) > biased_threshold
}

/// Look up calibration parameters for the given fidelity and ext.
fn params_for(fidelity: Fidelity, extension: &str) -> EconomicsParams {
    match fidelity {
        Fidelity::Edit => params_for_edit(extension),
        _ => EconomicsParams {
            fixed_overhead: 0.0,
            expected_savings_ratio: 1.0,
        },
    }
}

/// Edit-fidelity calibration parameters, keyed by file extension.
///
/// Values are conservative defaults, biased toward compression.
fn params_for_edit(extension: &str) -> EconomicsParams {
    let ext = extension.trim_start_matches(".").to_lowercase();
    match ext.as_str() {
        "ts" | "tsx" | "js" | "jsx" => EconomicsParams {
            fixed_overhead: 160.0,
            expected_savings_ratio: 0.25,
        },
        "cs" => EconomicsParams {
            fixed_overhead: 180.0,
            expected_savings_ratio: 0.25,
        },
        "rs" => EconomicsParams {
            fixed_overhead: 150.0,
            expected_savings_ratio: 0.28,
        },
        "java" => EconomicsParams {
            fixed_overhead: 170.0,
            expected_savings_ratio: 0.30,
        },
        _ => EconomicsParams {
            fixed_overhead: 180.0,
            expected_savings_ratio: 0.20,
        },
    }
}

/// Compute the raw token threshold at or below which compression is
/// predicted unfavorable. Public for testing and observability.
///
/// This function was originally dead-code (test-only). It is now also
/// called from the production path to record the threshold in the tracing
/// span for calibration observability.
pub(crate) fn compression_threshold(fidelity: Fidelity, extension: &str) -> usize {
    if matches!(fidelity, Fidelity::Low | Fidelity::Medium | Fidelity::High) {
        return 0;
    }
    let params = params_for(fidelity, extension);
    let threshold = params.fixed_overhead / params.expected_savings_ratio;
    (threshold * 0.85) as usize
}

#[cfg(test)]
#[path = "../tests/mcp/token_economics.rs"]
mod tests;
