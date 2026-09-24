// src/tests/mcp/prompts.rs
//
// Contract tests for SYSTEM_PROMPT notation documentation.
//
// Guards the portable model-visible SCHEMA-v5 contract.

use super::SYSTEM_PROMPT;

#[test]
fn teaches_schema_v5_as_primary_notation() {
    for frag in [
        "// SCHEMA v5",
        "X=extends",
        "I=implements",
        "F=field",
        "M=method",
        "typed ownership",
        "workspace_query",
        "§PATHMAP",
    ] {
        assert!(
            SYSTEM_PROMPT.contains(frag),
            "SYSTEM_PROMPT must teach SCHEMA-v5 fragment `{frag}`"
        );
    }
}

#[test]
fn documents_fidelity_and_edit_behaviors() {
    for frag in [
        "Low/Medium/High",
        "focusMethods",
        "byte-exact",
        "focused_method_bodies",
        "fidelity=\"verbatim\"",
        "Δ delta for",
    ] {
        assert!(
            SYSTEM_PROMPT.contains(frag),
            "SYSTEM_PROMPT must document fidelity fragment `{frag}`"
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
        "COMPACT-A A2",
        "FILE-CONTEXT-DELTA v1",
    ] {
        assert!(
            !SYSTEM_PROMPT.contains(tok),
            "SYSTEM_PROMPT still teaches retired notation fragment `{tok}` \
             — the legacy fallback paths are gone; remove the stale table"
        );
    }
}
