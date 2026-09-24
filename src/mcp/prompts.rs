// Prompt content for the MCP server.

/// The Clean-CTX model-visible context guide.
pub(crate) const SYSTEM_PROMPT: &str = r#"# Clean-CTX Context Guide

`provide_code_context`, `compress_code_context`, `restore_context`, and replay
responses use the SCHEMA-v5 presentation in `content`, or byte-exact raw source
when the presentation is not safely cheaper under the local tokenizer estimate.
Workspace graph facts are retrieved on demand with `workspace_query`; they are
not repeated in file context. `_meta` is application-facing state, not model
context.

## SCHEMA v5

Every presentation opens with this header, which also declares the
single-character markers used below:

`// SCHEMA v5  @=meta X=extends I=implements F=field M=method $=import →=scope mod:=method-modifiers cmod:=class-modifiers ctl:=control-summary pf:=pattern-facts fl:=legacy-flags cl:=class-metadata P=pattern T=type-alias`

Structure:

- `// ── ClassName ──` opens each class; interfaces open with `// Q=interface`
  followed by `Q Name`.
- `X Parent` extends, `I Iface` implements, `F name:type` declares a field.
- `M name` declares a method; overloads are disambiguated by parameter count as
  `M name(+N)`.
- `$ alias module [named]` imports; `T alias = original` aliases a type;
  `P name args` records a pattern.

A method line continues after a `→` scope arrow with `p:name:type` parameters,
the declared return type after a second `→`, then `mod:`, `ctl:`, `pf:`,
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
are code-side / legacy measurement formats; the SCHEMA-v5 presentation is the
model-visible content. When the presentation is not safely cheaper under the
local tokenizer estimate, `content` is the byte-exact raw source with no
wrapper or footer.
"#;

/// Return the list of available prompt definitions (for `prompts/list`).
pub(crate) fn prompt_list() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({
            "name": "cleanctx-notation",
            "description": "System instructions for reading Clean-CTX SCHEMA-v5 file context",
            "arguments": []
        }),
        serde_json::json!({
            "name": "dashboard",
            "description": "View the Clean-CTX token savings dashboard and per-file metrics.",
            "arguments": []
        }),
        serde_json::json!({
            "name": "clean-ctx-vocabulary",
            "description": "SCHEMA-v5 file-local presentation, exact-body, delta, raw-fallback, and path-map rules.",
            "arguments": []
        }),
    ]
}

#[cfg(test)]
#[path = "../tests/mcp/prompts.rs"]
mod tests;
