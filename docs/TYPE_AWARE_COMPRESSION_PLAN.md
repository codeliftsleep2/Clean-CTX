# R-02 — Type-Aware Compression Plan

**Status:** 📋 Proposed → 🚧 In-progress (approved 2026-08-04)
**Target release:** v0.3.0
**Effort:** 2-3 days
**Priority:** 🔴 High

---

## Problem Statement

The `CleanCtxConfig.type_aliases` field (`BTreeMap<String, String>`) is loaded from `.clean-ctx.json` but **never injected into the compression pipeline**. Type names appear verbatim in compressed output at Medium/High fidelity:

- **Method signatures** — `getUser(id:string):Promise<User>`
- **Field types** — `userId:string`, `userMap:HashMap<string,User>`
- **Class extends/implements** — `class AdminUser extends User`
- **Rust type aliases** — `type.root` captures
- **IR `FieldType`/`ReturnType` ops** — the IR path also carries full type names

The type table is dead configuration. This plan wires it into both the legacy text-compression pipeline and the IR path, replacing configured type names with short alias tokens and emitting a reversible footer so the LLM can resolve them.

---

## Current State (verified)

| Location | What exists | Gap |
|----------|-------------|-----|
| `src/config.rs` | `CleanCtxConfig.type_aliases: BTreeMap<String, String>` with `#[serde(default)]` | Loaded but never consulted |
| `src/compression/pipeline.rs` | `compress_file_with_source`, `compress_text`, `compress_source` all take `config: Option<&CleanCtxConfig>` | Config only used for `resource_limits` — `type_aliases` ignored |
| `src/mcp/tool_helpers.rs` | `compress_text_body` delegates to `compress_text` | No alias parameter threaded through |
| `src/mcp/workspace.rs` | `compress_pass` / `compress_pass_with_global_symbols` pass `Some(&state.config)` | Aliases not applied |
| `src/ir/render_llm.rs` | `HierarchicalIR.type_aliases: Vec<Vec<String>>` rendered in output | IR compiler never emits config-driven aliases |
| `src/ir/hierarchical.rs` | `CoreOp::TypeAlias(alias, original)` wire format exists | Only used for source-level `type X = Y` declarations |

---

## Design

### Core principle: additive + reversible

- **Additive guarantee**: existing opcodes, markers, and structural output are never modified. Type-alias substitution is a pure text pass applied *after* structural assembly and *before* micro-opcodes/symbol compression.
- **Reversibility**: a `§TA` footer maps `$uid → UserId` so the LLM can resolve every alias. In the IR path, the existing `CoreOp::TypeAlias` op and `HierarchicalIR.type_aliases` carry the mapping.
- **Deterministic**: aliases are applied longest-key-first to prevent partial matches (`User` before `UserService`).

### Alias token rules

| Rule | Rationale |
|------|-----------|
| Alias must be ≥ 2 chars | 1-char aliases collide with the symbol-dictionary `$1` opcode space |
| Alias must start with `$` | Distinguishes from structural markers (`⊕`, `Φ`, `§`) and symbol refs |
| Alias must be `[A-Za-z0-9_]+` after `$` | Keeps the token space clean |
| Original type must be ≥ 4 chars | Avoids replacing trivial types (`int`, `str` are exactly 3 chars) where savings are negligible |
| Longest key first | Prevents `User` matching inside `UserService` |

### Token-boundary matching

Replacement only occurs when the type name is a **whole token** in a type position. A type position is any occurrence bounded by:

```
: < > | ( , ; { } [ ] space tab newline
```

So `User` matches in `id:User`, `Map<string,User>`, `Promise<User>`, `A | User`, but NOT in `UserService`, `GitUserProfile`, or `user_id`.

---

## Implementation Phases

### Phase 1 — New module: `src/compression/type_aliases.rs`

```rust
/// Apply configured type aliases to a compressed body.
///
/// Replaces whole-token occurrences of configured type names with their
/// alias tokens (longest key first). Returns the substituted body and a
/// `§TA` footer mapping each *used* alias back to its original type.
pub fn apply_type_aliases(
    body: &str,
    aliases: &BTreeMap<String, String>,
) -> (String, String)
```

- `pub fn is_valid_alias(alias: &str) -> bool` — enforces the token rules above.
- `pub fn substitute_type_token(body: &str, original: &str, alias: &str) -> String` — single-pair substitution with boundary checks.
- Footer format:
  ```
  §TA $uid→UserId $jo→JsonObject
  ```
  Only aliases actually used in the body are emitted (avoids dead footer entries).

### Phase 2 — Wire into legacy pipeline (`src/compression/pipeline.rs`)

1. **`compress_file_with_source`** — after `assemble_body` + meta-block injection, before `apply_micro_opcodes`:
   ```rust
   if let Some(cfg) = config
       && !cfg.type_aliases.is_empty()
   {
       let (substituted, ta_footer) = apply_type_aliases(&body_content, &cfg.type_aliases);
       body_content = substituted;
       type_alias_footer = ta_footer;
   }
   ```
2. **`compress_text`** — add a new parameter `aliases: Option<&BTreeMap<String, String>>`. Signature change; update callers in `tool_helpers.rs`.
3. **`compress_source`** — same injection (workspace path).
4. **`BuildOutputResult`** — add `type_alias_block: Option<String>` (default `None`) so the footer is carried through `format_compacted_body`.

### Phase 3 — Wire into IR path

- After IR compilation, a new pass replaces type-named operands (`FieldType`, `ReturnType`, `ClassField` type positions) with alias tokens.
- Emits `CoreOp::TypeAlias(alias, original)` for each **used** alias (wire format already exists).
- `HierarchicalIR.type_aliases` already renders in `render_llm.rs` — no renderer change needed.
- **Invariant**: C1-C4 (CBM never modifies Core IR) and B1-B5 (behavioral enrichment never changes structural meaning) are preserved — this is a pure token substitution, not a semantic change.

### Phase 4 — Config + docs

- `.clean-ctx.json` example gains:
  ```json
  "type_aliases": {
    "UserId": "$uid",
    "JsonObject": "$jo",
    "HttpClient": "$http"
  }
  ```
- `docs/CONFIGURATION.md` documents the section with the token rules and reversibility guarantee.

### Phase 5 — Tests (in `src/tests/`)

**New: `src/tests/compression/type_aliases.rs`**
- Token boundary: `User` vs `UserService` vs `GitUserProfile`
- Nested generics: `Map<string,User>` → `Map<string,$uid>`
- Union types: `A | User` → `A | $uid`
- Optional/array: `User[]`, `User?`, `Promise<User>`
- Collision avoidance: 1-char alias rejected
- Longest-key-first: `User` + `UserService` both configured
- Footer: only used aliases emitted; reversibility (`$uid → UserId`)
- Determinism: same input → same output

**Pipeline tests (`src/tests/compression/pipeline.rs`)**
- `compress_text` with aliases at Medium/High fidelity
- `compress_file_with_source` with aliases
- Delta consistency: aliases applied deterministically across calls

**IR tests (`src/tests/ir/round_trip.rs`)**
- `CoreOp::TypeAlias` emitted for used aliases only
- Round-trip preserves alias mapping

---

## Fidelity Scope

| Fidelity | Type substitution | Rationale |
|----------|-------------------|-----------|
| Low | No (types already stripped) | `compact_method_low` / `extract_field` Low drop types entirely |
| Medium | Yes | `getUser(id:string):Promise<User>` → `getUser(id:string):Promise<$uid>` |
| High | Yes | Full signatures benefit most |
| Footer | Emitted at all fidelities if aliases exist | Keeps output self-describing |

---

## Token Savings Estimate

Type-heavy files (services, models, DTOs) at Medium/High fidelity:

- Method signatures: `Promise<User>` → `Promise<$uid>` saves ~4-6 tokens per occurrence
- Field types: `userMap:HashMap<string,User>` → `userMap:HashMap<string,$uid>` saves ~3-5 tokens
- Class hierarchies: `class AdminUser extends User` → `class AdminUser extends $uid` saves ~2-3 tokens

**Estimated 5-15% additional savings** on type-heavy files at Medium/High fidelity. Low fidelity is unaffected (types already stripped).

---

## Files Touched

| File | Change |
|------|--------|
| `src/compression/type_aliases.rs` | **New** — substitution pass + footer |
| `src/compression/mod.rs` | Export new module |
| `src/compression/pipeline.rs` | 3 call sites + `BuildOutputResult.type_alias_block` |
| `src/mcp/tool_helpers.rs` | `compress_text_body` passes config aliases |
| `src/ir/compiler.rs` | IR alias pass (post-compile) |
| `.clean-ctx.json` | Example `type_aliases` entries |
| `docs/CONFIGURATION.md` | Document the section |
| `src/tests/compression/type_aliases.rs` | **New** — unit tests |
| `src/tests/compression/pipeline.rs` | Pipeline integration tests |
| `src/tests/ir/round_trip.rs` | IR alias round-trip tests |

---

## Acceptance Criteria

1. `type_aliases` from `.clean-ctx.json` are applied to compressed output at Medium/High fidelity.
2. Token-boundary matching prevents partial-type false positives (`User` ≠ `UserService`).
3. `§TA` footer emitted with only used aliases; reversible.
4. 1-char aliases rejected (no `$1` collision with symbol dictionary).
5. IR path emits `CoreOp::TypeAlias` for used aliases; round-trip preserves mapping.
6. Zero clippy warnings: `cargo clippy --all-targets -- -D warnings`.
7. All tests pass: `cargo test --workspace --all-targets --all-features`.
8. `docs/CONFIGURATION.md` documents the section with token rules.