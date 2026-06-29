# Tree-Sitter Version Migration Plan

**Created:** 2026-06-29
**Status:** ✅ Complete (all 5 phases)
**Target:** v0.2.0 (Foundation item A-12)

> This document describes the migration from tree-sitter `=0.20.x` pinned versions to `^0.26.x` semantic version ranges. This is a **prerequisite** for F-20 (Rayon parallelization) because newer tree-sitter makes `Parser` `Send`, eliminating the need for per-thread parser initialization.

---

## Why Migrate?

### 1. `Parser` becomes `Send` (directly enables F-20)

| tree-sitter version | `Parser: Send` | Impact on Rayon |
|---------------------|---------------|-----------------|
| 0.20.x | ❌ No | Each `par_iter()` thread needs `thread_local!()` or `rayon::scope` parser init |
| 0.24+ | ✅ Yes | `Parser` can be created once and moved into the closure |

Without this migration, F-20 requires per-thread parser pools with `thread_local!()` or `rayon::scope` initialization. With it, the Rayon implementation is straightforward.

### 2. Security & supply-chain

- **Current:** `=0.20.10` pins never receive security patches. `cargo update` has no effect.
- **Target:** `^0.26` ranges with `Cargo.lock` for reproducibility. `cargo update` pulls patch releases automatically.
- **CI guard:** A simple check ensures all `tree-sitter-*` crates share the same `tree-sitter-sys` version.

### 3. Dependency island

The `=0.20.x` pins prevent Clean-CTX from coexisting with any other Rust tool that uses a newer tree-sitter. This blocks adding tree-sitter-based language servers, formatters, or linters as optional dependencies.

### 4. Language support

Newer grammars handle:
- TypeScript 5.x syntax (decorators, `satisfies`, `using` declarations)
- Modern C# features (records, primary constructors, file-scoped namespaces)
- Rust edition 2024 idioms
- Java 17+ features (sealed classes, records, pattern matching)

---

## Current State

| Crate | Pinned Version | Latest Available | Notes |
|-------|---------------|------------------|-------|
| `tree-sitter` | `=0.20.10` | `0.26.10` | Core library. Major ABI changes in 0.24+ |
| `tree-sitter-c-sharp` | `=0.20.0` | `0.23.5` | Must match core ABI |
| `tree-sitter-typescript` | `=0.20.5` | ~0.22+ | Now bundles TS + TSX in one crate |
| `tree-sitter-html` | `=0.20.0` | ~0.23+ | Used by Angular template parser |
| `tree-sitter-rust` | `=0.20.4` | ~0.22+ | |
| `tree-sitter-java` | `=0.20.0` | ~0.22+ | |

### Embedded queries (in `src/queries.rs`)

All four query constants (`TS_QUERY`, `CS_QUERY`, `RS_QUERY`, `JAVA_QUERY`) use 0.20.x grammar node names. These are simple `(node_type) @capture.name` patterns with one `(#match?)` predicate in the Rust query for panic macro detection.

### Language initialization (in `src/compression/language.rs`)

All language functions are wrapped in `OnceLock` for thread safety:
- `safe_typescript_language()`
- `safe_csharp_language()`
- `safe_rust_language()`
- `safe_java_language()`

These wrappers are defense-in-depth and should be preserved.

---

## Migration Phases

### Phase 1 — Version Selection & Cargo.toml Update

**Effort:** ~2 hours
**Risk:** Low (revertible with `git checkout`)

**Steps:**

1. **Determine compatible grammar versions.** For each grammar crate, check:
   - What `tree-sitter-sys` version it depends on
   - Whether it's compatible with tree-sitter 0.26.x
   - Whether the crate name or language function has changed

2. **Update `Cargo.toml` dependencies.** Change from:
   ```toml
   tree-sitter = "=0.20.10"
   tree-sitter-c-sharp = "=0.20.0"
   tree-sitter-typescript = "=0.20.5"
   tree-sitter-html = "=0.20.0"
   tree-sitter-rust = "=0.20.4"
   tree-sitter-java = "=0.20.0"
   ```
   To semantic ranges:
   ```toml
   tree-sitter = "0.26"
   tree-sitter-c-sharp = "0.23"
   tree-sitter-typescript = "0.22"
   tree-sitter-html = "0.23"
   tree-sitter-rust = "0.22"
   tree-sitter-java = "0.22"
   ```
   *(Exact versions TBD after compatibility check)*

3. **Run `cargo check`** to verify compilation. Fix any API changes:
   - `Parser::set_language()` signature may have changed
   - `Query::new()` error type may differ
   - `QueryCursor` API may have minor changes

4. **Run `cargo test`** to verify all existing tests pass.

**Rollback:** `git checkout -- Cargo.toml Cargo.lock`

---

### Phase 2 — Query Syntax Audit

**Effort:** ~4 hours
**Risk:** Medium (queries may produce different captures)

**Steps:**

1. **Check each grammar's `node-types.json`** for renamed nodes. Common renames in 0.24+:
   - TypeScript: `property_signature` → may be `public_field_definition` or similar
   - C#: `field_declaration` → may be split into `field_declaration` + `event_declaration`
   - Rust: `field_declaration` → may be `field_declaration` (likely unchanged)
   - Java: `record_declaration` → may be `record_declaration` (likely unchanged)

2. **Update `src/queries.rs`** with corrected node names.

3. **Test the `(#match?)` predicate** in the Rust query:
   ```scm
   (macro_invocation
       macro: (identifier) @_panic_macro
       (#match? @_panic_macro "panic|unreachable|unimplemented|todo|assert")
   ) @throw.root
   ```
   The `(#match?)` predicate syntax is stable across versions, but the `macro: (identifier)` field path may need updating.

4. **Run `cargo test`** with verbose output to verify captures match expected patterns.

**Verification:** Compare compressed output for a set of test files before and after migration. The output should be identical (same captures, same structure).

---

### Phase 3 — Language Function API Audit

**Effort:** ~1 hour
**Risk:** Low

**Steps:**

1. **Verify each language function exists** in the new crate version:
   - `tree_sitter_typescript::language_typescript()` — check if still exists
   - `tree_sitter_typescript::language_tsx()` — may be new, not needed currently
   - `tree_sitter_c_sharp::language()` — check if renamed
   - `tree_sitter_html::language()` — check if renamed
   - `tree_sitter_rust::language()` — likely unchanged
   - `tree_sitter_java::language()` — likely unchanged

2. **Update `src/compression/language.rs`** if any function names changed.

3. **Update `src/angular_meta/template.rs`** if `tree_sitter_html::language()` changed.

4. **Run `cargo test`** to verify.

---

### Phase 4 — OnceLock Verification

**Effort:** ~30 minutes
**Risk:** Very Low

**Steps:**

1. **Verify the `OnceLock` wrappers still compile** — they should, as `OnceLock` is a stdlib type.

2. **Consider removing the wrappers** if tree-sitter 0.26+ guarantees thread-safe initialization internally. **Recommendation:** Keep them as defense-in-depth. They add zero runtime overhead after the first call.

3. **Run `cargo test`** to verify.

---

### Phase 5 — CI Guard

**Effort:** ~1 hour
**Risk:** Low

**Steps:**

1. **Add a CI check script** (e.g., `scripts/check-tree-sitter-versions.sh` / `.ps1`) that:
   - Parses `Cargo.lock` for all `tree-sitter-*` packages
   - Extracts their `tree-sitter-sys` dependency version
   - Fails if any two crates depend on different `tree-sitter-sys` versions

2. **Add the check to CI configuration** (GitHub Actions, etc.).

3. **Document the check** in `CONTRIBUTING.md`.

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Grammar node names changed | Medium | High (wrong captures) | Phase 2 audit + test comparison |
| `(#match?)` predicate syntax changed | Low | Medium | Test the Rust query specifically |
| Language function renamed | Low | Medium | Phase 3 audit |
| `tree-sitter-sys` ABI mismatch between grammars | Medium | High (segfaults) | Phase 1 version selection + Phase 5 CI guard |
| Windows deadlock regression | Low | High | Keep `OnceLock` wrappers |
| Performance regression | Low | Low | Benchmark comparison |

---

## Dependency Chain

```
A-12 (Tree-sitter migration) — 2-3 days
    │
    ▼
F-19 (walkdir streaming workspace walk) — 1 day
    │
    ▼
F-20 (Rayon parallelization) — 2-3 days (simplified by A-12)
    │
    ▼
v0.2.0 release
```

**Why A-12 must come first:**
- Without A-12, F-20 requires `thread_local!()` parser pools or `rayon::scope` initialization
- With A-12, `Parser` is `Send` and can be created per-file in `par_iter()` closures
- A-12 also unblocks the supply-chain risk and enables future language grammar updates

---

## Test Plan

After each phase, run:
```bash
cargo check --all-targets 2>&1
cargo test 2>&1
```

For Phase 2 (query audit), additionally run a comparison test:
```bash
# Before migration: save baseline
cargo run --example fidelity_comparison > baseline.txt

# After migration: compare
cargo run --example fidelity_comparison > after.txt
diff baseline.txt after.txt
```

Expected: zero differences in compressed output for the same input files.

---

## Rollback Plan

If any phase fails:
1. `git checkout -- Cargo.toml` to restore original dependency versions
2. `cargo generate-lockfile` to regenerate `Cargo.lock` from original pins
3. File an issue with the error details

---

## Future-Proofing

After migration, the dependency specification changes from:
```toml
tree-sitter = "=0.20.10"  # Pinned — never updates
```
To:
```toml
tree-sitter = "0.26"  # Semver range — patch updates via cargo update
```

This means:
- `cargo update` will pull patch releases automatically
- Minor version bumps require manual `Cargo.toml` updates (intentional)
- The CI guard (Phase 5) prevents accidental ABI mismatches

---

## References

- [tree-sitter Rust crate changelog](https://github.com/tree-sitter/tree-sitter/blob/master/lib/binding_rust/CHANGELOG.md)
- [tree-sitter 0.24 release notes](https://github.com/tree-sitter/tree-sitter/releases/tag/v0.24.0)
- [tree-sitter query documentation](https://tree-sitter.github.io/tree-sitter/using-parsers#pattern-matching-with-queries)
- [ROADMAP.md](./ROADMAP.md) — Foundation item A-12