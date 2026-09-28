// src/tests/mcp/prompts.rs
//
// Contract tests for SYSTEM_PROMPT notation documentation.
//
// Guards the portable model-visible SCHEMA-vNext contract.

use super::SYSTEM_PROMPT;

#[test]
fn teaches_schema_v5_as_primary_notation() {
    for frag in [
        "// SCHEMA vNext",
        "C=class",
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
            "SYSTEM_PROMPT must teach SCHEMA-vNext fragment `{frag}`"
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
        "name(+N)",
    ] {
        assert!(
            !SYSTEM_PROMPT.contains(tok),
            "SYSTEM_PROMPT still teaches retired notation fragment `{tok}` \
             — the legacy fallback paths are gone; remove the stale table"
        );
    }
}

#[test]
fn vocabulary_prompt_description_names_the_production_presentation_boundary() {
    let _serial = crate::protocol::handler_response_serial();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    crate::protocol::captured_responses().clear();

    crate::mcp::handlers::handle_prompts_get(&serde_json::json!(1), "clean-ctx-vocabulary", &state);
    let response = crate::protocol::captured_responses()
        .pop()
        .expect("vocabulary response");
    let description = response["result"]["description"]
        .as_str()
        .expect("vocabulary description");

    assert!(description.contains("SCHEMA-vNext"), "{description}");
    assert!(description.contains("code-side delta"), "{description}");
    assert!(!description.contains("COMPACT-A"), "{description}");
    assert!(!description.contains("CONTROL-FULL"), "{description}");
}

/// Phase 1 MCP-to-LLM contract: clients should receive the compact workflow
/// rules during initialization without having to select an optional prompt.
#[test]
fn initialize_includes_compact_workflow_instructions() {
    let _serial = crate::protocol::handler_response_serial();
    crate::protocol::captured_responses().clear();

    crate::mcp::handlers::handle_initialize(&serde_json::json!(1));
    let response = crate::protocol::captured_responses()
        .pop()
        .expect("initialize response");
    let instructions = response["result"]["instructions"]
        .as_str()
        .expect("initialize result must include server instructions");

    for required in [
        "provide_code_context",
        "workspaceRoot",
        "graph_search",
        "cbm_proxy",
        "delta_code_context",
        "apply_edit",
        "index_repository",
        "indexing",
        "exact known ranges",
    ] {
        assert!(
            instructions.contains(required),
            "initialize instructions must include `{required}`: {instructions}"
        );
    }

    assert!(
        instructions.len() < 2_000,
        "workflow instructions must remain compact: {} bytes",
        instructions.len()
    );
    assert!(
        !instructions.contains("// SCHEMA vNext"),
        "initialization must not duplicate the detailed notation prompt"
    );
}
