## [0.6.2] - 2026-09-08

### Fixed

* **IR nested-type ownership (span-aware) — `apply_edit` "unit not found" for methods after a nested type** — `CoreIRPass` used a single `current_class` slot, so a nested `enum`/`class` declaration overwrote it with no scope restoration; every member lexically after the nested type was reparented to it and `UnitTable` materialized `SomeStatus.After` instead of `SomeService.After`, so `apply_edit replace_body` reported `unit not found`. `PassContext` now carries a span-keyed `TypeScope` stack: type roots push `[start_byte, end_byte)` scopes, member captures (`method`/`constructor`/`func`/`arrow`/`field.root`) refresh ownership to the innermost scope containing them (mirroring the proven `diff/builder.rs` containment contract), and `impl.root` reuses the struct's `class_id` (no duplicate `DefClass`) so methods attach and emit their Flags. Also routes C# `enum.root` naming through `extract_class_name` instead of the Rust-only `pub`-stripper (a `public enum Foo` no longer became `DefClass "public"`). (`src/ir/pipeline.rs`)
* **C# static-flag contamination — non-static classes rendered `cl: EXPORT STATIC`** — `CSharpLayer::extract_class_flags`/`extract_method_flags` scanned the FULL declaration node (head + body) with `contains("static")` substring matching, so a `static` token inside any body, comment, or string marked the enclosing class/method `STATIC`. Both now inspect only the declaration head (attribute-stripped, cut at the first `{`/`;`/depth-zero `=>` outside string/char literals) with word-boundary token matching via `has_head_modifier`; legitimate `public static class` and static methods remain flagged. (`src/ir/layers/csharp.rs`)

### Tests

| Area | Count |
|------|------:|
| `src/tests/ir/pipeline.rs` — nested-enum / nested-class span-containment ownership; enum members stay inside the enum | 3 tests |
| `src/tests/ir/layers/mod.rs` — C# static-flag isolation (body calls/comments/strings) + legitimate static class/method recognition | 5 tests |
| `src/tests/edit/spans.rs` — `apply_edit` end-to-end (read identity == write identity, no lookup fallback); render check (`cl: EXPORT` without `STATIC`, enum renders its own section) | 2 tests |
| `src/tests/ir/rust_integration.rs` — struct-following-impl methods attach and emit Flags (CI `--all-features` regression) | 1 test |

### Verification

Focused suites green: `ir::pipeline` 21, `ir::layers` 51, `ir::` 648, `edit::` 28, `diff::` 40, `compaction` 95, `compression` 125, `mcp::` 209 (8 ignored), `gitdiff::` 50, `layers::` 85, `rust_integration` 39. `cargo clippy --all-targets --all-features -- -D warnings` clean. `scripts/check-utf8.ps1` PASS (501 files, 0 BOMs).

---

