// Prompt content for the MCP server.

/// The Clean-CTX model-visible context guide.
pub(crate) const SYSTEM_PROMPT: &str = r#"# Clean-CTX Context Guide

`provide_code_context`, `compress_code_context`, `restore_context`, and replay
responses use either the versioned COMPACT-A A2 file-local presentation or
byte-exact raw source in `content`. Workspace graph facts are retrieved on
demand with `workspace_query`; they are not repeated in file context. `_meta`
is application-facing state, not model context.

## COMPACT-A A2

The document begins with:

`// COMPACT-A A2; file-local; workspace graph via workspace_query`

The following schema legend declares positional rows. Identity rule: canonical IDs are authoritative;
position is presentation, never identity:

`S A2 h[schema,version,file,mode] c[id,name,synthetic,methods,fields,mods,class_flags,extends,implements,injects,patterns] i[id,name,methods,fields,mods,extends] M[id,name,params,return,mods,control_summary,pattern_facts,legacy_flags,patterns,control_flow,data_flow,side_effects,execution_contexts] K[occurrence,caller,callee_written,explicit_arg_count,spread,resolution]`

The envelope preserves:

- `h`: CONTROL-FULL schema/version plus file and mode records.
- `d.c` / `d.i`: typed class/interface rows and scoped members.
- `g.K`: ordered canonical file-local call rows.
- `n.D` / `n.V`: DI and behavior navigation over existing local facts.
- ordered call occurrences. `caller_method_id` is canonical;
  `callee_written_name` is spelling evidence only. When
  `callee_resolution` is `unresolved`, never invent a declaration target.
- workspace relationships are intentionally absent. Call `workspace_query` for
  forward/reverse dependencies, reachability, framework edges, and provenance.

Arrays preserve occurrence order, duplicates, and nested group boundaries.
Names are display data; explicit IDs and typed ownership determine identity.

## Fidelity and exact source

- Low/Medium/High contain the complete canonical envelope compiled at that
  fidelity. High is the structural reasoning baseline.
- Edit adds byte-exact `B[method_id,start,end,utf8_bytes]` frames. With `focusMethods`, selectors resolve
  through typed owner identity to canonical method IDs before other bodies are
  removed. Ambiguous selectors are errors, never guesses.
- Verbatim is the explicit byte-exact whole-document mode. Request it for
  signatures, imports, class-level structure, or any edit outside exact bodies.
- `navigation` uses stable typed owner/member/edge locators. It never uses array
  offsets; occurrence-group descriptors preserve outer occurrence order and
  inner group boundaries, while endpoint descriptors keep `subject.file` and
  `object.file` independently addressable.
- Delta content uses `FILE-CONTEXT-DELTA v1`; apply it only to the acknowledged
  prior canonical state. Query `workspace_query` after apply when graph facts
  are needed.

## Editing

Use `intent="edit"` before a body edit. Only regions identified by
`byte_exact` and the mode record's exact body IDs are safe exact-match inputs.
Prefer `apply_edit` for a supported unit edit. On rejection, re-read and retry;
never blind-retry. Use `fidelity="verbatim"` for whole-document edits.

## Paths and legacy formats

The trailing `§PATHMAP` maps session aliases to paths. Do not reproduce it in
source edits. `compress_workspace` manifests and the historical SCHEMA v5
renderer are legacy/CONTROL-PROD measurement formats; use
`decompress_code_context` where expansion is required. They are not the
correctness authority for current file-context responses.

If A2 is not safely cheaper under the selected local tokenizer estimate,
`content` is the byte-exact raw source with no A2 wrapper or footer.
"#;

/// Return the list of available prompt definitions (for `prompts/list`).
pub(crate) fn prompt_list() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({
            "name": "cleanctx-notation",
            "description": "System instructions for reading Clean-CTX COMPACT-A A2 file context",
            "arguments": []
        }),
        serde_json::json!({
            "name": "dashboard",
            "description": "View the Clean-CTX token savings dashboard and per-file metrics.",
            "arguments": []
        }),
        serde_json::json!({
            "name": "clean-ctx-vocabulary",
            "description": "COMPACT-A A2 file-local schema, canonical identity, exact-body, delta, raw-fallback, and path-map rules.",
            "arguments": []
        }),
    ]
}

#[cfg(test)]
#[path = "../tests/mcp/prompts.rs"]
mod tests;
