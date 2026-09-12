---

## [0.1.7] — 2026-06-20

### Added

#### Multi-Platform Proxy Support
- **Platform-agnostic proxy**: The proxy now supports any AI provider (Anthropic, OpenAI, DeepSeek, etc.) via the `PlatformAdapter` trait.
  - `proxy/src/platform/mod.rs` — `PlatformAdapter` trait with `is_tool_result()`, `extract_tool_result()`, `intercept_path()`, `platform_headers()`, `is_platform_model()`, `platform_name()` methods
  - `proxy/src/platform/anthropic.rs` — Anthropic API adapter (`type: "tool_result"` blocks, `cache_control`, `anthropic-beta` header)
  - `proxy/src/platform/openai.rs` — OpenAI API adapter (`role: "tool"` messages, `tool_call_id`)
  - `proxy/src/platform/generic.rs` — Generic fallback adapter (heuristic detection, multiple content field locations)
  - `platform::detect_platform()` — Auto-detection from model name in request body
  - `PLATFORM` env var — Manual override (`anthropic`, `openai`, `generic`)
  - Server now intercepts `/v1/messages` (Anthropic), `/v1/chat/completions` (OpenAI), and `/chat` (Generic)

#### Tool Output Filtering (R-38)
- **TOML-based filter engine**: 7 built-in filters that compress verbose tool output by 70-90%.
  - `proxy/src/filters.rs` — 7-step filter pipeline: `replace → match_output → strip/keep_lines → group_by → head/tail → max_lines → on_empty`
  - `proxy/src/filter_rules.rs` — TOML parsing, `CompiledFilter` struct, `compile_filter_file()` with regex validation
  - `proxy/src/filter_registry.rs` — Most-specific-match-wins filter selection with priority tiebreaker
  - `proxy/src/filter_loader.rs` — Built-in filter loading from `filters/` directory at startup
  - `proxy/src/filter_stats.rs` — Per-program token/line savings tracking with dashboard summary
  - `proxy/src/community_filters.rs` — Community filter loading from `.clean-ctx/filters/`
  - `filters/cargo.toml` — Compact cargo build/test/check/clippy output
  - `filters/npm.toml` — Compact npm/yarn/pnpm/bun install/build output
  - `filters/git-diff.toml` — Compact git diff/show output
  - `filters/pytest.toml` — Compact pytest output
  - `filters/tsc.toml` — Compact TypeScript compiler output
  - `filters/dotnet.toml` — Compact dotnet build/test/run output
  - `filters/ng.toml` — Compact Angular CLI build/test/lint output
  - `TOOL_FILTERS` env var — Enable/disable tool output filtering

#### Secret Scrubbing (R-37)
- **Platform-agnostic secret scrubbing**: Detects and redacts secrets in tool results before they reach the LLM.
  - `proxy/src/scrub.rs` — `scrub_secrets()` engine with `ScrubResult`, `ScrubHit`, `ScrubFailClosed` semantics
  - `proxy/src/scrub_patterns.rs` — Compiled `OnceLock<Regex>` statics for AWS keys, GitHub tokens, JWTs, PEM keys, etc.
  - `might_contain_secret()` pre-filter — Cheap literal-substring check before expensive regex passes
  - `SCRUB_SECRETS` env var — Enable/disable secret scrubbing
  - Now uses `PlatformAdapter::is_tool_result()` for cross-platform detection

#### Pluggable Transform Pipeline
- **`Pipeline` abstraction**: Makes the transform chain pluggable and testable (OCP compliance).
  - `proxy/src/pipeline.rs` — `Pipeline::build()` returns composed transforms, `Pipeline::run()` executes all
  - New transforms added via closure without modifying existing code
  - Each transform receives `&dyn PlatformAdapter` for format-aware operations

#### FAANG Audit & Regression Tests
- **Comprehensive code audit**: 243 total tests (112 lib + 112 bin + 18 audit regression + 1 integration)
  - `proxy/tests/audit_regression.rs` — 18 regression tests covering all audit findings
  - Critical bugs fixed: hardcoded Anthropic `tool_result` detection in `strip_ansi` and `scrub_secrets`
  - Security fix: exact path matching instead of `ends_with` for intercept routing
  - All transforms now platform-agnostic via `PlatformAdapter`

### Changed
- `ANTHROPIC_BASE_URL` env var renamed to `UPSTREAM_URL` conceptually (backward-compatible)
- `server.rs` now uses `Pipeline::build()` and `Pipeline::run()` instead of inline transform calls
- `transform::strip_ansi()` now takes `&dyn PlatformAdapter` parameter
- `transform::scrub_secrets()` now takes `&dyn PlatformAdapter` parameter
- `transform::apply_tool_filters()` now uses `PlatformAdapter::is_tool_result()` for cross-platform detection
- All filter modules (filters, filter_rules, filter_registry, filter_loader, filter_stats, community_filters) exported from `lib.rs`
- Documentation updated at `docs/PROXY.md` with multi-platform, filtering, scrubbing, and IDE integration details

### Test count
- 243/243 tests pass (112 proxy lib unit + 112 proxy bin unit + 18 audit regression + 1 integration)
- 0 new clippy warnings (pre-existing warnings only: dead-code on API surface types)

