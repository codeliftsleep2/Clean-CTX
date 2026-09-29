// Prompt content for the MCP server.

/// Compact server-wide workflow guidance returned during MCP initialization.
///
/// Keep this focused on tool selection and lifecycle boundaries. Detailed
/// SCHEMA-vNext notation remains in [`SYSTEM_PROMPT`] and the optional prompts,
/// so initialization does not charge every session for the full vocabulary.
pub(crate) const WORKFLOW_INSTRUCTIONS: &str = r#"# Clean-CTX Tool Workflow

- `provide_code_context` accepts either one singular `filePath` request or one top-level `files` batch (1–8 items); never send both. Prefer singular for one file and batch for several. Pass `workspaceRoot` explicitly. In a batch, keep scope top-level, give each item a unique `id` plus its own file/fidelity/focus, inspect every status, and read each success's exact `content`; `content_index` correlates its top-level mirror. Failures do not suppress siblings, and batching grants no cross-file edit authority.
- Use `graph_search` as the normal typed symbol/file discovery entry point. Use the other structured graph wrappers when typed nodes, edges, paths, or modules are required.
- Use one `workspace_query` call with top-level `queries` for multiple independent questions sharing a workspace scope; mixed operation types are supported, results stay ordered by unique item ID, and one item failure does not suppress its siblings. Each item's `name` is one string: for several names, create one item per name; never pass `name` as an array or invent `names`.
- Use `cbm_proxy` only when compact or explicitly fresh raw CBM output is preferable to a typed structured result. Never bypass Clean-CTX to call CBM directly.
- Use `delta_code_context`, `apply_delta`, and persistence tools only when the caller intentionally owns their version, acknowledgement, or durable-state lifecycle.
- Read Edit or Verbatim context before `apply_edit`, and edit only byte-exact regions supplied by the current session.
- Do not call `index_repository` after `apply_edit`; the next graph operation performs the lazy refresh. Use explicit indexing only after external edits when graph freshness is required.
- Treat CBM indexing responses as temporary state, not as authoritative empty results; retry in a bounded way or use the documented fallback.
- Use native file reads for unsupported or non-code files and for exact known ranges when structured context is insufficient.
"#;

/// The Clean-CTX model-visible context guide.
pub(crate) const SYSTEM_PROMPT: &str = r#"# Clean-CTX Context Guide

`provide_code_context`, `compress_code_context`, `restore_context`, and replay
responses use the SCHEMA-vNext presentation in `content`, or byte-exact raw source
when the presentation is not safely cheaper under the local tokenizer estimate.
Workspace graph facts are retrieved on demand with `workspace_query`; they are
not repeated in file context. Use its `calls_in_file` operation when detailed
owner-qualified local calls, overload separation, occurrence order, written
argument count, or spread evidence is needed. `_meta` is application-facing
state, not model context.

## SCHEMA vNext

Every presentation opens with this header, which also declares the
single-character markers used below:

`// SCHEMA vNext  @=meta C=class X=extends I=implements F=field M=method $=import p:=params →=return mod:=method-modifiers cmod:=class-modifiers ctl:=control-summary pf:=pattern-facts fl:=legacy-flags cl:=class-metadata P=pattern T=type-alias`

Structure:

- `C ClassName` opens each class; interfaces open with `// Q=interface`
  followed by `Q Name`.
- `X Parent` extends, `I Iface` implements, `F name:type` declares a field.
- `M name` declares a method; visible parameter signatures distinguish
  overloads without changing the method name.
- `$ alias module [named]` imports; `T alias = original` aliases a type;
  `P name args` records a pattern.

A method line continues with optional `p:name:type` parameters, the declared
return type after `→`, then `mod:`, `ctl:`, `pf:`,
`fl:`, `cf:`, `df:`, `se:`, and `ec:` annotation groups. Names are display
data; the typed owner (class line) plus the method name determine identity.
Member order, duplicates, and overload groups preserve source order.

## Fidelity and exact source

- Low/Medium/High render structural members. Low keeps fields on one line and
  methods minimal; Medium/High render one member per line and show parameters.
  High is the structural reasoning baseline with control-flow, data-flow,
  side-effect, and execution-context annotations.
- Edit appends byte-exact method bodies. With `focusMethods`, selectors resolve
  through typed ownership to the focused method before other bodies are
  dropped; only those bodies are byte-exact and `_meta.byte_exact` reports
  `focused_method_bodies`. Ambiguous selectors are errors, never guesses.
- Verbatim is the explicit byte-exact whole-document mode. Request it for
  signatures, imports, class-level structure, or any edit outside exact bodies.
- Delta content is the minimal summary `Δ delta for <file> (v{from} → v{to}):
  +N ~N -N ops`; the structured op list rides code-side in `result.delta`, not
  in `content`. Query `workspace_query` after apply when graph facts are needed.

## Editing

Use `intent="edit"` before a body edit. Only regions identified by
`byte_exact` are safe exact-match inputs. Prefer `apply_edit` for a supported
unit edit. On rejection, re-read and retry; never blind-retry. Use
`fidelity="verbatim"` for whole-document edits.

## Paths and legacy formats

The trailing `§PATHMAP` maps session aliases to paths. Do not reproduce it in
source edits. The reversible COMPACT-A codec and `compress_workspace` manifests
are code-side / legacy measurement formats; the SCHEMA-vNext presentation is the
model-visible content. When the presentation is not safely cheaper under the
local tokenizer estimate, `content` is the byte-exact raw source with no
wrapper or footer.
"#;

/// Return the list of available prompt definitions (for `prompts/list`).
pub(crate) fn prompt_list() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({
            "name": "cleanctx-notation",
            "description": "System instructions for reading Clean-CTX SCHEMA-vNext file context",
            "arguments": []
        }),
        serde_json::json!({
            "name": "dashboard",
            "description": "View the Clean-CTX token savings dashboard and per-file metrics.",
            "arguments": []
        }),
        serde_json::json!({
            "name": "clean-ctx-vocabulary",
            "description": "SCHEMA-vNext file-local presentation, exact-body, delta, raw-fallback, and path-map rules.",
            "arguments": []
        }),
    ]
}

#[cfg(test)]
#[path = "../tests/mcp/prompts.rs"]
mod tests;
