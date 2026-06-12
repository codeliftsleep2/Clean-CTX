# Clean-CTX — Future Roadmap

**Last updated:** 2026-06-12

> **Living document.** Items are reviewed and pruned every release. Status legend: 📋 proposed · 🚧 in-progress · ✅ done · ⏸️ deferred

---

## At a Glance

| Horizon | Target Release | Theme | Items |
|---------|----------------|-------|------:|
| **Now** | v0.2.0 | Real-world ready | 4 |
| **Next** | v0.3.0 | Advanced capabilities | 6 |
| **Later** | v1.0.0+ | Ecosystem & integrations | 7 |
| **Architectural** | Continuous | Code health & tooling | 6 |
| **Community** | Continuous | Docs & marketing | 4 |

---

## Now (v0.2.0) — "Real-world ready"

These items address the most common user requests and unlock adoption on larger codebases.

| ID | Title | Description | Effort | Priority |
|----|-------|-------------|-------:|---------:|
| **R-22** | Angular Meta-Layer | Framework-annotation layer for Angular files. Phase 1 ✅ (Tier 1: decorator extraction). Phase 2 ✅ (file-triplet bundling). Phase 2.5 ✅ (modern Angular 17–21 syntax: `@if`/`@for`/`@switch`/`@defer`/`@let` detection + `Φmodel:` marker). Phase 3 ✅ (cross-file DI + selector graph). | 6.5 days | 🟡 Medium |
| **R-19** | Pluggable tokenizers | Today the binary hard-codes cl100k via `tiktoken-rs`. Add a `Tokenizer` trait with `o200k_base` (GPT-4o), Claude, and Llama-3 implementations selectable via tool argument or config. | 2 days | 🔴 High |
| **F-20** | Rayon parallelization for `compress_workspace` | Per-thread tree-sitter `Parser` pool, shared `DashMap` for the path dictionary, `par_iter().try_for_each`. Expected ~4× speedup on 16-core boxes. | 3-5 days | 🔴 High |
| **F-19** | Streaming workspace walk | Replace `collect_source_files` collect-then-sort pattern with a `walkdir` streaming visitor. Required before F-20. | 1 day | 🟡 Medium |
| **A-07** | Property-based tests with `proptest` | Fuzz-style input tests for the decompressor, the config glob matcher, and the modifier stripper. Would have caught F-06 (Unicode) and F-12 (substring match) regressions. | 1-2 days | 🔴 High |

---

## Next (v0.3.0) — "Advanced capabilities"

Smaller scope than the Now list, but each item provides significant user value.

| ID | Title | Description | Effort | Priority |
|----|-------|-------------|-------:|---------:|
| **R-01** | Additional language support — Python | Most-requested language. Follows the 4-step guide in `DEVELOPER_DOCUMENTATION.md`. | 1-2 days | 🔴 High |
| **R-01b** | Additional language support — Go | Second-most requested. | 1-2 days | 🟡 Medium |
| **R-01c** | Additional language support — Rust | Common in LLM-context scenarios (LLM code reviewers, AI IDEs). | 1-2 days | 🟡 Medium |
| **R-01d** | Additional language support — Java | Enterprise staple. | 1-2 days | 🟢 Low |
| **R-02** | Type-aware compression | Inline `type_aliases` from config: `UserId` → `$uid`, `JsonObject` → `$jo`. Currently the type table is loaded but not injected into the capture pipeline. | 2-3 days | 🔴 High |
| **R-07** | MCP `resources` support | Expose compressed snapshots as MCP resources in addition to tools, enabling LLM clients to read prior state without re-invoking tools. | 1-2 days | 🟡 Medium |
| **R-08** | Improved diff: rename detection | Detect class/method renames (same signature, different name) and emit as `~` with a `renamed from X` hint instead of a delete+add pair. | 1 day | 🟡 Medium |
| **R-10** | Tool input validation via `schemars` | Replace hand-rolled `args.get("foo")` chains with `schemars`-derived JSON Schema; tool schemas in `tools.rs` auto-generate from Rust types. | 1-2 days | 🟡 Medium |
| **R-12** | Multi-file / git-commit diff | Diff an entire workspace between two git commits; emit per-file deltas in one tool call. Powers "what changed in this PR?" workflows. | 3-5 days | 🔴 High |
| **A-06** | `cargo-bench` benchmark suite | Currently we have timing data in `docs/PERFORMANCE.md` but no automated benchmarks. Add a `benches/` directory with criterion-style benchmarks for compression, decompression, and diff. | 1 day | 🟡 Medium |

---

## Later (v1.0.0+) — "Ecosystem & integrations"

Items that add value but require demand signal before investing. **YAGNI applies** — these should not be built speculatively.

| ID | Title | Description | Effort | Priority |
|----|-------|-------------|-------:|---------:|
| **R-03** | Compression ratio statistics per project | New tool `get_compression_stats(workspacePath)` that returns avg savings per file/class/method. | 1 day | 🟡 Medium |
| **R-04** | Incremental decompression | Don't re-decompress unchanged sections; use the baseline-snapshot pattern from `diff_code_context`. | 2 days | 🟡 Medium |
| **R-05** | Semantic opcode suggestions | When a custom opcode appears 5+ times across the session, surface a "promote to built-in primitive?" suggestion in the response metadata. | 1 day | 🟢 Low |
| **R-06** | Config hot-reload | File-watcher on `.clean-ctx.json`; debounce + atomic reload of `McpState.config` without restart. | 1 day | 🟡 Medium |
| **R-09** | Custom query strings | Allow users to override the tree-sitter query via `.clean-ctx.json` for project-specific AST extraction. | 1 day | 🟢 Low |
| **R-11** | Cross-file symbol resolution | Track imports across files in a workspace; emit `this method calls UserService.authenticate() in user_service.ts`. | 5+ days | 🟡 Medium |
| **R-13** | Compress-to-compress diff | Diff two compressed outputs directly (without re-parsing either source) — uses the `§MAP` footer + per-class UUIDs. | 2-3 days | 🟡 Medium |
| **R-14** | LLM-aware prompt optimization | Analyze the LLM client's prompts and auto-tune fidelity/markers for that specific model. | 3 days | 🟢 Low |
| **R-15** | Python bindings via PyO3 | Expose the compression engine to Python for integration with non-Rust tooling. | 2-3 days | 🟢 Low |
| **R-16** | WebAssembly build target | Browser-based compression for in-IDE previews (no server needed). | 3-5 days | 🟢 Low |
| **R-17** | VS Code extension (native) | First-party VS Code extension that wraps the binary; sidebar with live savings stats. | 1-2 weeks | 🟢 Low |
| **R-18** | Markdown / JSON / YAML compression | Extend compression to data formats using the same opcode framework with format-specific markers. | 1-2 days | 🟡 Medium |
| **R-20** | Persisted baseline cache | Currently `LocalStateCache` is in-memory only; optionally persist to disk so baseline diffs survive server restarts. | 2-3 days | 🟢 Low |

---

## Architectural improvements (continuous)

Code health work that has no user-facing feature but improves long-term maintainability.

| ID | Title | Description | Effort | Priority |
|----|-------|-------------|-------:|---------:|
| **A-01** | Remove `src/helpers.rs` shim | F-42 was partial; the 18-line re-export shim should be removed once all internal callers are confirmed to use `crate::compaction::*`. | 1 hour | 🟢 Low |
| **A-02** | Migrate to `walkdir` (precondition for F-20) | Required before parallelism can be added — streaming input is a prerequisite for `par_iter`. | 1 day | 🟡 Medium |
| **A-03** | `schemars` for tool schemas | Replaces hand-written JSON Schema in `src/mcp/tools.rs`; less drift between Rust types and advertised schema. | 1-2 days | 🟡 Medium |
| **A-04** | `tracing` + `metrics` | Structured logging + OpenTelemetry-compatible spans; useful for diagnosing slow compressions in production. | 1-2 days | 🟢 Low |
| **A-05** | Workspace-aware language detection | When `compress_workspace` runs, cache language detection results across files. TS file next to another TS file is, predictably, TS. | 1 day | 🟢 Low |
| **A-07** | Property-based tests with `proptest` | See Now list. | 1-2 days | 🔴 High |

---

## Documentation & community (continuous)

Work that improves discoverability and onboarding for new users and contributors.

| ID | Title | Description | Effort | Priority |
|----|-------|-------------|-------:|---------:|
| **D-01** | Landing-page screenshot / GIF | Top of README showing side-by-side raw vs compressed output for a real class. Increases GitHub star rate. | 1 hour | 🟡 Medium |
| **D-02** | Blog post: "Why we wrote a tree-sitter-based LLM compressor" | Marketing + technical deep-dive; good for HackerNews launch and SEO. | 1 day | 🟢 Low |
| **D-03** | Real-world benchmark against alternatives | Compare token savings to literal source code, `gpt-tokenizer`, and any competitor tools. Cite real numbers. | 2-3 days | 🟡 Medium |
| **D-04** | Interactive web playground | Browser-based tool that compresses pasted source code; drives adoption. | 1 week | 🟢 Low |
| **D-05** | Architecture decision records (ADRs) | Track WHY each major decision was made (tree-sitter, stdio, no-DB, etc.). | 2-3 hours per ADR | 🟢 Low |

---

## Carry-over from FAANG Audit

These were intentionally deferred during the original 5-phase audit because they required larger refactors than fit the "fix one bug per finding" structure. They're explicitly tracked here so they don't get lost.

| ID | Title | Description | Status |
|----|-------|-------------|--------|
| **F-19** | Streaming workspace walk | See Now list. | 📋 Proposed |
| **F-20** | Rayon parallelization | See Now list. | 📋 Proposed |

---

## Prioritization rationale

The Now list was chosen by balancing three factors:

1. **Unblocks other work** — R-19 (tokenizer abstraction) unblocks every model-specific feature. A-02 (walkdir migration) unblocks F-20.
2. **Adoption blocker** — Python (R-01) is the single most-requested language. F-19/F-20 are required for any user with >1K files in a repo.
3. **Regression insurance** — A-07 (proptest) is cheap insurance against the kind of input-validation bugs the FAANG audit found.

The Next list focuses on the highest-value features per engineering-hour: R-02 (type-aware) and R-12 (multi-file diff) are the two features most likely to drive daily usage.

The Later list contains items that are valuable but require demand signal. None of R-14 through R-17 should be started speculatively.

---

## How to add/remove items

1. **Adding:** Open a GitHub issue using the `roadmap` label, then add the item here with status 📋 proposed
2. **Promoting:** When a proposed item is scheduled for a release, change status to 🚧 in-progress and link the issue
3. **Completing:** When shipped, change status to ✅ and link the PR
4. **Removing:** If an item is no longer relevant, delete the row and explain in the next CHANGELOG entry
5. **Pruning:** Every release (every minor version bump), review this document and remove anything that has been 📋 for >2 releases

---

## Tracking

Each item should eventually link to a GitHub issue. Until issues exist, the IDs (R-01, F-19, A-07, etc.) are stable references for discussion in PRs and code comments.

---

## License

[CC0-1.0 Universal](https://creativecommons.org/publicdomain/zero/1.0/) — Dedicated to the public domain.