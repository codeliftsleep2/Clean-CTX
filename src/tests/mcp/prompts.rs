// src/tests/mcp/prompts.rs
//
// Contract tests for SYSTEM_PROMPT notation documentation.
//
// Guards the portable model-visible CONTROL-FULL contract.

use super::SYSTEM_PROMPT;

#[test]
fn teaches_control_full_as_primary_notation() {
    for frag in [
        "// CONTROL-FULL v2",
        "canonical IDs are authoritative",
        "classes` / `interfaces`",
        "caller_method_id",
        "callee_written_name",
        "semantic_edges",
        "typed ownership",
    ] {
        assert!(
            SYSTEM_PROMPT.contains(frag),
            "SYSTEM_PROMPT must teach CONTROL-FULL fragment `{frag}`"
        );
    }
}

#[test]
fn documents_high_and_edit_behaviors() {
    for frag in [
        "Low/Medium/High",
        "Edit adds byte-exact method bodies",
        "exact_body_method_ids",
        "fidelity=\"verbatim\"",
        "CONTROL-FULL-DELTA v2",
    ] {
        assert!(
            SYSTEM_PROMPT.contains(frag),
            "SYSTEM_PROMPT must document High/Edit fragment `{frag}`"
        );
    }
}

#[test]
fn retired_vocabulary_is_not_taught_as_current_semantics() {
    for tok in [
        "Primitive opcodes",
        "$ctor",
        "$nw",
        "$fr",
        "⊕",
        "§I=",
        "§SYM",
    ] {
        assert!(
            !SYSTEM_PROMPT.contains(tok),
            "SYSTEM_PROMPT still teaches retired notation fragment `{tok}` \
             — the legacy fallback paths are gone; remove the stale table"
        );
    }
}
