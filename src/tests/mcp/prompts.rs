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
fn retired_vocabulary_is_scoped_to_the_legacy_section() {
    let primary = SYSTEM_PROMPT
        .find("## Response Notation")
        .expect("primary SCHEMA v2 section missing");
    let legacy = SYSTEM_PROMPT
        .find("## Legacy Notation")
        .expect("scoped legacy section missing");
    assert!(
        primary < legacy,
        "SCHEMA v2 must be taught BEFORE the legacy vocabulary"
    );

    // Retired tokens must still be documented for decoding legacy
    // output, but ONLY inside the scoped legacy section.
    for tok in ["$c ", "$ctor", "$nw", "$fr", "⊕guard", "⊕loop", "⊕⇒"] {
        let pos = SYSTEM_PROMPT.find(tok).unwrap_or_else(|| {
            panic!("legacy section must document retired token `{tok}`")
        });
        assert!(
            pos > legacy,
            "retired token `{tok}` appears OUTSIDE the legacy section \
             (at byte {pos}, legacy starts at {legacy}) — the prompt is \
             teaching retired vocabulary as current"
        );
    }
}
