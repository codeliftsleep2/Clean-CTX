// Prompt content for the MCP server.

/// The Clean-CTX model-visible context guide.
pub(crate) const SYSTEM_PROMPT: &str = r#"# Clean-CTX Context Guide

`provide_code_context`, `compress_code_context`, `restore_context`, and replay
responses use either the versioned COMPACT-A A1 presentation or byte-exact raw
source in `content`. A1 decodes to normalized CONTROL-FULL v2 and is the compact
portable semantic authority. `_meta` is application-facing state and may mirror
facts, but never treat it as the only source of code meaning.

## COMPACT-A A1

The document begins with:

`// COMPACT-A A1; decodes to normalized CONTROL-FULL v2`

The following schema legend declares positional rows. Identity rule: canonical IDs are authoritative;
position is presentation, never identity:

`S A1 h[schema,version,file,mode] c[id,name,synthetic,methods,fields,mods,class_flags,extends,implements,injects,patterns] i[id,name,methods,fields,mods,extends] M[id,name,params,return,mods,control_summary,pattern_facts,legacy_flags,patterns,control_flow,data_flow,side_effects,execution_contexts] K[occurrence,caller,callee_written,explicit_arg_count,spread,resolution] E[occurrence,relation,S(domain,type,name,file),O(domain,type,name,file),layer,call_evidence]`

The envelope preserves:

- `h`: CONTROL-FULL schema/version plus file and mode records.
- `d.c` / `d.i`: typed class/interface rows and scoped members.
- `g.K` / `g.E`: ordered call and semantic-edge rows.
- `n.D` / `n.V` / `n.E`: DI, behavior, and endpoint-local navigation.
- ordered call occurrences. `caller_method_id` is canonical;
  `callee_written_name` is spelling evidence only. When
  `callee_resolution` is `unresolved`, never invent a declaration target.
- complete generic/framework relationships with relation,
  typed subject/object, layer, file provenance, and call evidence where present.

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
- Delta content uses `CONTROL-FULL-DELTA v2`; apply it only to the acknowledged
  prior canonical state. Its post-apply semantic-edge snapshot is authoritative.

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

If A1 is not safely cheaper under the selected local tokenizer estimate,
`content` is the byte-exact raw source with no A1 wrapper or footer.
"#;

/// Return the list of available prompt definitions (for `prompts/list`).
pub(crate) fn prompt_list() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({
            "name": "cleanctx-notation",
            "description": "System instructions for reading Clean-CTX COMPACT-A A1 context",
            "arguments": []
        }),
        serde_json::json!({
            "name": "dashboard",
            "description": "View the Clean-CTX token savings dashboard and per-file metrics.",
            "arguments": []
        }),
        serde_json::json!({
            "name": "clean-ctx-vocabulary",
            "description": "COMPACT-A A1 positional schema, canonical identity, exact-body, delta, raw-fallback, and path-map rules.",
            "arguments": []
        }),
    ]
}

#[cfg(test)]
#[path = "../tests/mcp/prompts.rs"]
mod tests;
