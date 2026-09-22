// src/tests/mcp/prompts.rs
//
// Contract tests for SYSTEM_PROMPT notation documentation.
//
// Guards the portable model-visible COMPACT-A contract.

use super::SYSTEM_PROMPT;

#[test]
fn teaches_compact_a_as_primary_notation() {
    for frag in [
        "// COMPACT-A A2",
        "canonical IDs are authoritative",
        "d.c` / `d.i",
        "caller_method_id",
        "callee_written_name",
        "`g.K`",
        "workspace_query",
        "typed ownership",
    ] {
        assert!(
            SYSTEM_PROMPT.contains(frag),
            "SYSTEM_PROMPT must teach COMPACT-A fragment `{frag}`"
        );
    }
}

#[test]
fn documents_high_and_edit_behaviors() {
    for frag in [
        "Low/Medium/High",
        "Edit adds byte-exact `B[method_id,start,end,utf8_bytes]` frames",
        "exact body IDs",
        "fidelity=\"verbatim\"",
        "FILE-CONTEXT-DELTA v1",
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
