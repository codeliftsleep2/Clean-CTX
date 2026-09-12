---

## [0.1.5] — 2026-06-08

### Changed

#### FAANG Audit — Compiler IR Phase E (F-30 through F-47)
- `src/ir/compiler.rs` — **F-30:** `compile()` now returns `Result<CompiledIR, CompileError>` (typed enum with `Capture`, `Layer`, `NoCaptures` variants) instead of `Box<dyn std::error::Error>`. **F-31:** `id_counter` changed from `u32` to `u64` to prevent arithmetic overflow.
- `src/ir/patterns.rs` — **F-33/F-34:** Documented that `PatternOp::consumed()` is a heuristic approximation; actual count is computed by `try_compress_pattern`.
- `src/ir/render.rs` — **F-35/F-36/F-37:** Documented ASYNC `$a` backward compatibility, unknown flag handling, and fidelity match arm rationale.
- `src/ir/positional.rs` — **F-39:** Documented `PositionalConfig` struct rationale. **F-40:** Documented `verify_round_trip` return semantics. **F-41:** Fixed misleading "tokens" docstring (function returns char counts). **F-42/F-43:** Documented encoding naming and `+ 12` envelope estimate.
- `src/ir/layers/angular.rs` — **F-44/F-45:** Documented text round-trip through `parse_phi_line` as known design debt.
- `src/ir/layers/mod.rs` — **F-46:** Documented `LayerContext` ownership semantics for `GlobalSymbolTable`.
- `src/ir/layers/typescript.rs` — **F-47:** Replaced byte-level parsing with char-level iteration in `extract_class_relationships` to prevent issues with multi-byte UTF-8 and non-ASCII whitespace.
- `src/ir/mod.rs` — Exported `CompileError` for downstream consumers.

### Test count
- 318/318 IR tests pass (0 failures, 0 regressions)
- 0 clippy warnings (`cargo clippy --all-targets -- -D warnings`)

---

