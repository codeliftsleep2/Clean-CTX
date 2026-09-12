---

## [0.4.2] - 2026-08-25 - Non-CBM Tool Audit Fix Cycle

Evidence-driven fix cycle over the 2026-08-25 non-CBM tool audit. Every fix was reproduced with a failing regression test before implementation.

### Fixed

- **`diff_commits` emitted access modifiers as changed-class labels** (`~ class internal`, `~ class public`). Two compounding label-derivation defects: (1) `MODIFIERS_CLASS` lacked `internal `, so `strip_modifiers` stopped immediately on `internal static class Foo` and the first whitespace token became the "name"; (2) the diff snapshot builder routed `struct.root`/`enum.root` through `extract_rust_struct_name` for ALL languages — that helper only strips Rust visibility prefixes, so `public enum Foo` labeled as `public`. Fixes: `internal ` added to `MODIFIERS_CLASS`; `enum `/`struct ` keyword stripping added to `extract_class_name`/`extract_class_meta`; `diff::builder::try_build_with` now receives its parser label and routes non-Rust struct/enum/trait/impl through the shared class-name extractor while Rust keeps byte-identical behavior. Unchanged-class rendering untouched.
- **`compress_workspace` at Low fidelity lost method identifiers** for C# signatures like `internal static async Task<(A section, …, Guid requestId)> CreateRecordWithDefaults(…)`, rendering them as `internal static async Task(scope)`. Mechanism: with a named tuple return type whose last element is lowercase, `is_csharp_return_type(tokens[len-2])` misfires and the fallback split the WHOLE signature at the first `<`, yielding the type prefix as the "name". Fix: when the parameter list is located structurally (`find_method_params`), the name is the last whitespace token before it regardless of naming convention; the no-parameter-list legacy fallback is preserved; Medium/High/Edit outputs are pinned unchanged by test.
- **Alias registry path fragmentation**: the same physical file could hold two aliases (visible as duplicate `α` entries in `§PATHMAP`) because alias keys used the raw caller-supplied string while handlers mix absolute and workspace-relative spellings. Fix: `PathDictionary::get_or_create_alias` now resolves keys via canonicalize-or-fallback (Windows verbatim `\\?\` prefix stripped for readable PATHMAP output); exact-string repeats fast-path without filesystem access. Alias-keyed state (IR context versions, text-delta baselines, LLM text cache) converges onto one identity per file.
- **`decompress_code_context` destroyed class boundaries on round-trip**: the blanket skip-all-`//`-comments rule removed the IR renderer's structural `// ── ClassName ──` markers, turning a multi-class skeleton into an unattributed flat field list. Boundary markers are now preserved verbatim; opcode expansion around them is unchanged.
- **`list_sessions` returned a static `"Persistence DB active."` status line** despite promising an enumeration. The persistence model has NO session concept (the `sessions` table is dead schema nothing writes), so instead of inventing session data the tool now lists what genuinely exists: one row per persisted file context (path, fidelity, token counts, delta count, last-update timestamp) via new `SqliteStore::list_contexts()` + `BufferedStore` passthrough; description corrected accordingly.
- **`context_stats` clamped negative savings to `0.0%`**: compression can legitimately COST tokens (small file / unfocused edit fidelity). Per-file, domain-aggregate and session-total percentages are now signed — the dashboard shows the true negative figure while raw/compressed token measurements are untouched.
- **`diff_commits` attributed enclosing-class methods to nested declarations**: language queries capture class/struct/enum/record declarations at ANY nesting depth, but the snapshot walker attached every method via `classes.last_mut()` — so after a nested `enum`/`class`, ALL subsequent methods of the enclosing class were owned by the nested type and rendered as `~ class OrderStatus` instead of `~ class OrderService`. Fix (structural): `CapEntry` now carries `end_byte`; the walker keeps an open-scope stack keyed by source span and attaches members to the INNERMOST declaration whose span CONTAINS them (a nested type that closed before the member starts can never own it). Nested types keep their own correctly-labeled entries; top-level/nested labels from finding #1 are pinned unchanged.
- **`SessionStats` fragmented per-file tracking across path spellings**: `files` was keyed by the RAW string passed to `record_compression()`, so absolute / workspace-relative / redundant-segment spellings of ONE physical file produced separate rows and double-counted session totals (the long-documented CONTEXT_STATS plan issue #2). Fix at the choke point: `record_compression`/`file_stats` now resolve through the shared `dictionary::path::canonical_identity_key` (exact-string hits fast-path without filesystem access), giving stats the same canonical identity as the alias registry.

### Changed

- **`delta_text_context` contract made explicit**: the tool is a text-based diff strategy for the SAME source-code registry as `delta_code_context` — not a generic text tool. Description and README row updated to state the code-only scope; no behavior change.
- Documented (no code change) that `diff_code_context` maintains its own baseline local to itself (canonical path + fidelity), independent of `provide_code_context`/`compress_code_context` (audit finding #8), and that bare `F` field markers at low fidelity remain intentional compression semantics, not a defect (audit finding #7).

### Added

- Regression suites: gitdiff end-to-end label tests (changed internal class + unchanged sibling in one file); compaction tests for modifier/keyword extraction and compound-signature fidelity pins; diff-builder language-split tests (C# internal class, C# enum, Rust enum guard); alias identity convergence + unresolvable-fallback tests; decompression multi-class boundary round-trip; `SqliteStore::list_contexts` store tests; `src/tests/mcp/tool_contracts.rs` pins locking both corrected tool descriptions and the enumeration behavior.

### Verification

- Focused regression suites green under default features AND `--features rust` (the diff-builder module is feature-gated).
- Full gate: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, complete workspace test suite.

---

