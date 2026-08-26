// src/tests/mcp/prompts.rs
//
// Contract tests for SYSTEM_PROMPT notation documentation.
//
// Guards against silent drift back to the retired `$`-opcode / `⊕`
// marker tables: the PRIMARY response notation (SCHEMA v2) must stay
// taught first, High/Edit behaviors must stay documented, and the
// retired text-compressor vocabulary must remain explicitly scoped to
// the legacy section (compress_workspace / delta_text_context).

use super::SYSTEM_PROMPT;

#[test]
fn teaches_schema_v2_legend_as_primary_notation() {
    let legend_fragments = [
        "// SCHEMA v2",
        "@=meta",
        "X=extends",
        "I=implements",
        "F=field",
        "M=method",
        "$=import",
        "→=scope",
        "fl:=flags",
        "cl:=class-flags",
        "P=pattern",
        "T=type-alias",
    ];
    for frag in legend_fragments {
        assert!(
            SYSTEM_PROMPT.contains(frag),
            "SYSTEM_PROMPT must teach the SCHEMA v2 legend fragment `{frag}`"
        );
    }
    assert!(
        SYSTEM_PROMPT.contains("## Response Notation (SCHEMA v2)"),
        "primary notation section missing"
    );
}

#[test]
fn documents_high_and_edit_behaviors() {
    for frag in ["cf:", "df:", "se:", "ec:", "VERBATIM source body"] {
        assert!(
            SYSTEM_PROMPT.contains(frag),
            "SYSTEM_PROMPT must document High/Edit fragment `{frag}`"
        );
    }
}

#[test]
fn retired_vocabulary_is_absent_from_the_prompt_entirely() {
    // Phase A retirement: with the three IR fallbacks converted to
    // structured `ir_unavailable` errors, no LLM-facing prompt teaches
    // the retired `$`-primitive / `⊕`-marker / `§`-micro-code tables.
    for tok in [
        "## Legacy Notation",
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
