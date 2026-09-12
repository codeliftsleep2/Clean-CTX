---

## [0.4.4] - 2026-08-27 - git_diff Signature-Bleed Fixes

### Fixed

- **Signature bleed from expression-bodied members in `git_diff` output.** Diffing a file whose expression-bodied member interpolates a primary-constructor parameter name (`public string Display() => $"Value: {Value}";` under `record Example(string Value)`) emitted corrupted change-set lines — the method name duplicated with `=>` and interpolated-string fragments welded onto the signature (`method Display Display():=> $"Value:`), plus a corrupted `~ class Example(string` label — while raw `git diff` was correct. Root cause: `extract_method_sig` treated the first `{` as end-of-header; expression-bodied members have none, so the interpolation hole became the boundary, truncating mid-literal. The signature span now ends at min(first `{`, first depth-0 `=>` outside string/char literals) via new `find_depth_zero_arrow` + `skip_quoted_literal` helpers (`src/compaction/method.rs`); paren depth keeps TS callback-typed parameters intact and unmatched quotes fail safe to the legacy behavior. Byte-identical output whenever no arrow exists.
- **C# primary-constructor parameter lists leaked into class labels.** After modifier/keyword stripping, whitespace tokenization turned `Example(string Value)` into the label `Example(string`, surfacing corrupted `~ class …` rows for every record or class with a primary constructor. New `strip_trailing_param_list` peels a trailing balanced `(…)` group using the EXISTING last-depth-0 locator `find_method_params` — activated only when the group closes out the declaration (empty tail or lone `;`), leaving `: Base(args)` and `where` clauses untouched — wired into both `extract_class_name` and `extract_class_meta` so name and base metadata derive from identical declaration text (`src/compaction/class.rs`).

Renderer/differ untouched; no fixture special-casing; no output deduplication.

### Tests

- RED→GREEN end-to-end regression `gitdiff_interpolation_does_not_bleed_into_signature_line` (`src/tests/gitdiff/engine.rs`): established failing against the unfixed tree (asserts the added member surfaces yet no `=>` or `$"Value:` fragment reaches any diff/signature line), promoted GREEN by these fixes with assertions unchanged.
- Unit-level regressions: `extract_method_sig("public string Display() => $\"Value: {Value}\";", Medium)` returns `Display()` with a brace-bodied control proving preserved legacy extraction; `extract_class_name("public sealed record Example(string Value)")` yields `Example`, and `class Foo<T>(int x) : Base` yields `Foo` with `:Base` carried in `class_meta` (`src/tests/compaction/method.rs`, `src/tests/compaction/class.rs`).

### Verification

- `cargo fmt --all -- --check` clean; `cargo clippy --all-targets -- -D warnings` clean for all changed targets (two pre-existing lints remain in unrelated `src/tests/encoding.rs`); compaction suite 86 passed; focused `compaction:: diff:: gitdiff::` suites 143 passed including the end-to-end regression; UTF-8 guard PASS (473 files, strict UTF-8, 0 BOMs, 0 mojibake).

---

