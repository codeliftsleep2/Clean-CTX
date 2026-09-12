---

## [0.4.5] - 2026-08-27 - Constructor Base-Initializer Signature Truncation

### Fixed

- **Interpolation hole inside a constructor base-initializer terminated the signature.** A brace-bodied C# constructor whose base-initializer argument is an interpolated string (`: base($"Unexpected value: {value}, context: {context}")`) truncated at the FIRST `{` anywhere in the capture — the interpolation hole INSIDE the literal — rendering the diff member as `method value: …:base($"Unexpected value:` with both holes and the initializer tail destroyed while raw `git diff` was correct. The body-brace boundary now shares `find_depth_zero_arrow`'s literal-aware contract: new sibling `find_depth_zero_brace` skips string/char literals via the existing `skip_quoted_literal` and tracks paren/bracket depth, so only a structural-depth-0 `{` OUTSIDE quoted/interpolated literals ends the signature (`src/compaction/method.rs`). Renderer/differ untouched; no fixture special-casing; the expression-bodied min-boundary fix and normal brace-bodied legacy extraction are byte-identical.

### Tests

- Focused unit regression `high_fidelity_base_initializer_interpolation_keeps_full_header` (`src/tests/compaction/method.rs`): the High-fidelity signature span ends at the true body brace with the full initializer intact; the Medium-tier compacted label retains both interpolation holes.
- End-to-end regression `gitdiff_ctor_base_initializer_interpolation_not_truncated` (`src/tests/gitdiff/engine.rs`): established failing against the unfixed tree (manifest dropped `{value}`/`{context}` and mis-shaped the member marker), promoted GREEN with assertions unchanged; constructor body statements asserted absent from every signature line.
- Harness hardening (same build): `CAPTURED_RESPONSES` consumers are now poison-tolerant via `protocol::captured_responses()` and drain pop-first-then-assert, so a genuine test failure can no longer poison the shared sink while holding its guard; and both Phase A/B retirement suites gate their dispatching tests on one shared `protocol::HANDLER_RESPONSE_SERIAL` (Phase B previously had no serialization at all, and Phase A's gate covered only its own file) — eliminating the empty-pop → `PoisonError` cascade observed under full-suite parallelism, where one test's clear/dispatch/pop raced a sibling's and three unrelated tests died on a poisoned lock.

### Verification

- compaction 87 passed; diff 62 passed (2 feature-ignored); gitdiff 41 passed; `cargo fmt --all -- --check` clean (two EOF-newline fixes applied via the formatter); UTF-8 guard PASS (473 files, strict UTF-8, 0 BOMs, 0 mojibake); `cargo test encoding` 6 passed; `cargo clippy --all-targets -- -D warnings` 0 warnings.

---

