# Clean-CTX: Add Symbol/Method Targeting to `edit` Fidelity

## Problem

`provide_code_context` with `fidelity: "edit"` is meant to give a caller the
exact, byte-accurate body of the method(s) it's about to modify — enough
fidelity to safely write an `Edit` diff against. In practice it currently
renders **every method's full body** in the target file, not just the one(s)
the caller actually needs. That means `edit` fidelity often costs *more*
tokens than just doing a plain file `Read`, because on top of every method
body it also carries fixed overhead (schema legend, per-method compressed
signature line, re-wrapped imports, PATHMAP footer) that a plain `Read`
doesn't have.

Measured example (`OrgUnitValue.cs`, a 26-line file with 2 methods):
- Plain `Read`: 174 tokens
- `provide_code_context` `edit` fidelity: 326 tokens (+87%)

Another example (`ImportPreLoadService.cs`):
- Plain `Read`: 2,401 tokens
- `edit` fidelity: 2,726 tokens (+13.5%)

So today, `edit` fidelity unlocks nothing over `Read` — it's strictly worse
for any file where you only care about one or two methods, which is the
common case.

## Goal

Let the caller say "give me `edit`-level fidelity, but only for methods X and
Y" — full body only for the named symbols, signature-only (current
`overview`/`refactor`-style compression) for everything else in the file.
This makes `edit` fidelity actually valuable: full byte-accuracy where it's
needed, without paying for full bodies of unrelated methods.

**Backward compatibility requirement**: if the new parameter is omitted, behavior
must be byte-for-byte identical to today (render everything, as `edit`
fidelity currently does). This is strictly additive.

## Where to make the change

These are the four touch points, in the order you should implement them.

### 1. Schema — `src/mcp/tools.rs`, `tool_list()` (~lines 137–151)

Add an optional parameter to `provide_code_context`'s input schema:

```jsonc
"focusMethods": {
  "type": "array",
  "items": { "type": "string" },
  "description": "Optional. When set alongside fidelity: \"edit\", only these method/function names get full verbatim bodies; all other methods in the file are rendered signature-only. Omit to render every method's body (current default behavior)."
}
```

Naming note: call it `focusMethods` (plural, array) rather than a single
`symbol` string — callers frequently need bodies for more than one method in
the same file (e.g. a constructor + the method it initializes for), and a
plural param avoids a second N+1 tool-call round trip.

### 2. Handler — `src/mcp/tool_handlers/core.rs`, `handle_provide_code_context`

Parse the new field into a `Option<HashSet<String>>`:

```rust
let focus_methods: Option<HashSet<String>> = args
    .get("focusMethods")
    .and_then(|v| v.as_array())
    .map(|arr| arr.iter().filter_map(|s| s.as_str().map(String::from)).collect());
```

Thread `focus_methods` down into whatever render call this handler currently
makes (see #3). If the existing call is something like
`render_hierarchical_for_llm(hir, fidelity)`, either:
- add an overload/new fn `render_hierarchical_for_llm_focused(hir, fidelity, focus_methods.as_ref())`, or
- add `focus: Option<&HashSet<String>>` as a new parameter to the existing fn and update all call sites to pass `None`.

Prefer the new-fn approach (`_focused` suffix) to keep every other call site
in the codebase untouched and avoid needing to audit every existing caller.

### 3. Render-time gate — `src/ir/render_llm.rs`, `render_methods` (~line 157; the body-append check is ~line 273)

This is the actual fix. Today the body-append logic is a blanket check with
no per-method awareness:

```rust
// current (~line 273)
if fidelity == Fidelity::Edit {
    if let Some(body) = &method.body {
        // ... append full body
    }
}
```

This sits inside a `for method in &class.methods` loop, so `method.name` is
already in scope. Change the condition to also require symbol membership:

```rust
if fidelity == Fidelity::Edit
    && focus.map_or(true, |f| f.contains(&method.name))
{
    if let Some(body) = &method.body {
        // ... append full body (unchanged)
    }
}
```

`focus.map_or(true, ...)` is the key: when `focus` is `None` (no
`focusMethods` supplied), every method matches — identical to current
behavior. When `focus` is `Some(set)`, only named methods get bodies;
everything else falls through to whatever signature-only rendering already
happens for non-`Edit` fidelities (that code path already exists — reuse it,
don't duplicate it).

Thread `focus: Option<&HashSet<String>>` as a new parameter into
`render_methods` and its caller chain up to the new `_focused` entry point
from #2.

### 4. Optional optimization — `src/ir/compiler.rs` (~line 300)

There's also a compile-time gate that extracts method bodies into the IR in
the first place:

```rust
if fidelity == Fidelity::Edit {
    extract_method_body(&cap.raw_text) // ...
}
```

Mirroring the same `focus` check here avoids extracting/storing body text for
methods that will just get filtered out at render time. This is a memory/CPU
optimization, not required for correctness — the render-time filter in #3
alone achieves the token-count goal. Do this only after #1–#3 are working and
verified; skip it if you want the smaller, lower-risk change.

## Non-goals / things not to touch

- `IRQueryEngine::get_fan_in` / `get_fan_out` in `src/ir/query.rs` already do
  exact-string method-name matching elsewhere in this codebase — useful as a
  reference for the matching convention (exact name match, not fuzzy/regex),
  but there's no shared function to extract; the `focus.contains(&name)`
  check above is simple enough to inline.
- `MethodNode` (`src/ir/hierarchical.rs` ~line 85) already has `name`,
  `params`, and `body: Option<String>` — no IR schema change needed.
- Don't touch `overview`/`refactor`/`debug`/`implement`/`verbatim` fidelity
  paths at all. This change is scoped entirely to `Fidelity::Edit`.

## Verification plan

Test against files whose exact byte content is already known (do a plain
`Read` first to get ground truth):

1. Pick a file with 3+ methods (e.g. `ImportPreLoadService.cs`).
2. Call `provide_code_context` with `fidelity: "edit"`, no `focusMethods` —
   confirm output is byte-identical to before this change (regression check).
3. Call again with `focusMethods: ["GetOrgUnitDic"]` — confirm:
   - `GetOrgUnitDic`'s body appears in full, matching the `Read` ground truth
     exactly (correct braces, indentation, comments).
   - Every other method in the file appears signature-only (no body).
   - Total token count (via `context_stats` or comparing response size) drops
     relative to the no-`focusMethods` call, and ideally drops below a plain
     `Read` of the same file.
4. Call with a `focusMethods` entry that doesn't exist in the file (e.g.
   `["NoSuchMethod"]`) — confirm it degrades gracefully to all-signatures, no
   crash, no partial match.
5. Call with multiple valid names — confirm all named methods get bodies,
   only those methods.
6. Re-run `context_stats` and confirm "Files Tracked" still increments
   correctly and delta-transport on repeat calls still works (this change
   must not break the existing delta/caching path in `apply_delta` /
   `delta_code_context`).
