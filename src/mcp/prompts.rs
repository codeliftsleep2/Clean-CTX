// Prompt content for the MCP server.

/// The Clean-CTX model-visible context guide.
pub(crate) const SYSTEM_PROMPT: &str = r#"# Clean-CTX Context Guide

`provide_code_context`, `compress_code_context`, `restore_context`, and replay
responses use the versioned CONTROL-FULL JSON document in `content`. This is
the portable semantic authority. `_meta` is application-facing state and may
mirror facts, but never treat it as the only source of code meaning.

## CONTROL-FULL v2

The document begins with:

`// CONTROL-FULL v2; canonical IDs are authoritative`

The JSON fields are named and preserve canonical order:

- `file`: session ID, canonical source path, and IR version.
- `mode`: effective fidelity, exact-body method IDs, and source-escalation rule.
- `classes` / `interfaces`: typed owners with explicit IDs, methods, fields,
  inheritance, implements, injection occurrences, modifiers, flags, and facts.
- `methods`: explicit IDs, parameters, return type, occurrence-grouped facts,
  flow/effects/context, and optional exact body plus source spans.
- `calls`: ordered call occurrences. `caller_method_id` is canonical;
  `callee_written_name` is spelling evidence only. When
  `callee_resolution` is `unresolved`, never invent a declaration target.
- `semantic_edges`: complete generic/framework relationships with relation,
  typed subject/object, layer, file provenance, and call evidence where present.

Arrays preserve occurrence order, duplicates, and nested group boundaries.
Names are display data; explicit IDs and typed ownership determine identity.

## Fidelity and exact source

- Low/Medium/High contain the complete canonical envelope compiled at that
  fidelity. High is the structural reasoning baseline.
- Edit adds byte-exact method bodies. With `focusMethods`, selectors resolve
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
`byte_exact` and `mode.exact_body_method_ids` are safe exact-match inputs.
Prefer `apply_edit` for a supported unit edit. On rejection, re-read and retry;
never blind-retry. Use `fidelity="verbatim"` for whole-document edits.

## Paths and legacy formats

The trailing `§PATHMAP` maps session aliases to paths. Do not reproduce it in
source edits. `compress_workspace` manifests and the historical SCHEMA v5
renderer are legacy/CONTROL-PROD measurement formats; use
`decompress_code_context` where expansion is required. They are not the
correctness authority for current file-context responses.
"#;

/// Return the list of available prompt definitions (for `prompts/list`).
pub(crate) fn prompt_list() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({
            "name": "cleanctx-notation",
            "description": "System instructions for reading Clean-CTX CONTROL-FULL context",
            "arguments": []
        }),
        serde_json::json!({
            "name": "dashboard",
            "description": "View the Clean-CTX token savings dashboard and per-file metrics.",
            "arguments": []
        }),
        serde_json::json!({
            "name": "clean-ctx-vocabulary",
            "description": "CONTROL-FULL v2 named semantic fields, stable typed navigation, canonical identity, exact-body, delta, and path-map rules.",
            "arguments": []
        }),
    ]
}

#[cfg(test)]
#[path = "../tests/mcp/prompts.rs"]
mod tests;
