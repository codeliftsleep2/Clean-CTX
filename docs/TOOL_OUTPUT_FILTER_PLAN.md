# Clean-CTX — Tool Output Filter Plan (R-37 + R-38)
**Version:** 0.2.0 (merged — Claude plan + ctx-wire insights)
**Status:** 📋 Proposed · Last updated: 2026-06-14

---

## Core Principle

The Tool Output Filter is **purely additive** — it runs as a post-processing pass inside the existing reverse proxy pipeline before forwarding responses to the LLM. Existing users see no change. Users who opt in get dramatically smaller tool result payloads with secrets automatically scrubbed before they ever reach the LLM.

---

## Background: The Problem

When an LLM coding agent runs shell commands during a session, the output floods the context window with noise that was designed for human consumption, not LLM input:

```
Agent runs: cargo test
→ 500 lines: progress bars, timing data, deprecation warnings, 
             test counts, feature flags, platform info
→ LLM needs: "47 passed, 2 failed: test_auth, test_cache"
→ Waste: ~490 tokens of pure noise per cargo test call

Agent runs: npm install  
→ 2,000 tokens: deprecation warnings, funding notices, 
                audit results, package counts
→ LLM needs: "added 247 packages" or the actual error
→ Waste: ~1,980 tokens per npm install call

Agent runs: git diff
→ 800 tokens: index hashes, file mode lines, context lines,
              chunk headers, trailing whitespace markers
→ LLM needs: the actual changed lines
→ Waste: ~600 tokens per git diff call
```

This is a different problem than source code compression. Source code is signal — Clean-CTX compresses it semantically. Shell output is mostly noise — it should be filtered, not compressed. The mechanism is different, the savings are real, and the implementation lives in the same reverse proxy that already handles cache control injection.

**Current state:** The proxy already strips ANSI codes (`strip_ansi: true`) and trims git/bash output (`trim_bash_git: true`). These are blunt instruments. This plan adds a precise, declarative, per-program filter system on top of that foundation.

---

## Merged Insights: ctx-wire Comparison

This plan incorporates architectural patterns from [ctx-wire](https://github.com/pivanov/ctx-wire), a Go-based command output filter for AI agents. The two tools solve complementary problems (ctx-wire wraps the shell; Clean-CTX proxies the API), but ctx-wire's filter engine is battle-tested and its patterns are directly transferable.

### What We Adopt from ctx-wire

| Concept | ctx-wire Implementation | Clean-CTX Adaptation |
|---------|------------------------|---------------------|
| **Declarative TOML filters** | `filters/*.toml` with `match_command`, transform stages, inline `[[tests]]` | Same TOML schema adapted for proxy-based detection (see below) |
| **Filter selection** | Most specific match wins (longest matched span) | Same: when multiple filters match, longest match wins |
| **JSON guard** | Complete JSON never truncated; opt-out via `reduce_json` | Same: detect complete JSON in tool results, pass through whole |
| **Failure handling** | On non-zero exit, suppress synthetic success summaries; keep tail | Adapted: detect exit codes from tool_result metadata, adjust filtering |
| **Secret scrubbing** | Regex-based with `ScrubFailClosed` semantics | Same patterns + `ScrubFailClosed` for the proxy path |
| **`on_empty` fallback** | If all output stripped, emit summary like "cargo: ok" | Same: `on_empty` field in filter rules |
| **`match_output` collapse** | If output matches success pattern, replace with one-liner | Same: collapse stage before line filtering |
| **`group_by`** | Bucket lines by regex key, cap per bucket | Same: useful for lint output grouped by file |
| **`replace` stage** | Line-by-line regex substitution before filtering | Same: normalize output before filtering |
| **`filter_stderr`** | Route stderr through same pipeline | Adapted: proxy sees combined output; filter both streams |
| **Inline tests** | `[[tests]]` blocks in TOML files | Same: self-documenting, portable, runnable via `ctx-wire verify` |
| **Dedup** | Byte-identical repeated output → short reference | Considered but deferred (proxy sees tool results, not raw shell) |
| **Gain tracking** | Per-program, per-agent, per-period stats | Integrated into `context_stats` dashboard |

### What We Skip from ctx-wire

| Concept | Why We Skip |
|---------|------------|
| **PATH shims** | Proxy-based; agents call tools, not shell directly |
| **Hook/rewrite system** | Proxy intercepts at API level, not shell level |
| **Tee/spool to disk** | Proxy already logs full content before filtering |
| **`discover` command** | Not needed — proxy sees all tool results |
| **`learn` command** | Not needed — different integration model |
| **Agent-specific init** | Proxy is agent-agnostic |

---

## Decisions Locked

| Question | Decision |
|----------|----------|
| Integration point | Reverse proxy response pipeline — same layer as cache hints, ANSI stripping |
| Filter approach | Declarative TOML rules — keep/drop patterns, collapse rules, line caps |
| Full log preservation | Always log the unfiltered response to `.clean-ctx/proxy-logs/` before filtering |
| Secret scrubbing | Runs before filter rules, on every tool result, when `scrub_secrets: true` |
| Default state | Both R-37 and R-38 off by default. Opt-in via `.clean-ctx.json`. Zero overhead when disabled |
| Built-in rule sets | Ship with pre-built patterns for cargo, npm, git, pytest, tsc, angular-cli, dotnet |
| Community rules | `.clean-ctx/filters/` directory — user can add custom `.toml` rule files |
| Filter selection | Most specific match wins (longest `match_command` span); ties broken by priority |
| JSON guard | Complete JSON never truncated; opt-out per filter via `reduce_json` |
| New dependencies | None — regex engine (`regex` crate) already in dependency tree |
| Savings tracking | Extend `context_stats` dashboard with per-program filter savings |
| Failure handling | On failed commands, suppress synthetic success; keep error tail |

---

## Two Sub-Features

**R-37 — Secret Scrubbing:** Detect and redact secrets in tool results before they reach the LLM. Runs independently of R-38.

**R-38 — Shell Output Filtering:** Declarative per-program filter rules that reduce noisy command output to signal. Depends on R-37 (scrubbing runs first in the pipeline).

---

## R-37: Secret Scrubbing

### Goal

Intercept tool results in the reverse proxy and redact any detected secrets before forwarding to the LLM. The full unredacted content is preserved in the local log. The LLM never sees a secret. Enterprise environments get a meaningful additional trust guarantee.

### What Gets Scrubbed

Adapted from ctx-wire's `scrub` module with additional patterns:

| Secret Type | Pattern | Replacement | Source |
|-------------|---------|-------------|--------|
| AWS Access Key | `AKIA\|ASIA[0-9A-Z]{16}` | `[REDACTED:aws_key]` | ctx-wire |
| GitHub Token | `gh[pousr]_[A-Za-z0-9]{36,}` | `[REDACTED:github_token]` | ctx-wire |
| GitHub Fine-grained PAT | `github_pat_[A-Za-z0-9_]{22,}` | `[REDACTED:github_pat]` | ctx-wire |
| Google API Key | `AIza[0-9A-Za-z_\-]{35}` | `[REDACTED:google_key]` | ctx-wire |
| Slack Token | `xox[baprs]-[A-Za-z0-9-]{10,}` | `[REDACTED:slack_token]` | ctx-wire |
| Stripe Key | `(sk\|rk)_(live\|test)_[A-Za-z0-9]{16,}` | `[REDACTED:stripe_key]` | ctx-wire |
| OpenAI/Anthropic Key | `sk-(ant-)?[A-Za-z0-9_\-]{20,}` | `[REDACTED:api_key]` | ctx-wire |
| HashiCorp Vault Token | `hvs\.[A-Za-z0-9_-]{20,}` | `[REDACTED:vault_token]` | ctx-wire |
| PyPI Token | `pypi-[A-Za-z0-9_-]{16,}` | `[REDACTED:pypi_token]` | ctx-wire |
| JWT | `eyJ[A-Za-z0-9_-]+\.eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+` | `[REDACTED:jwt]` | ctx-wire |
| PEM Private Key | `-----BEGIN .* PRIVATE KEY-----.*-----END .* PRIVATE KEY-----` | `[REDACTED:private_key]` | ctx-wire |
| Bearer Token | `(?i)(authorization\s*[:=]\s*[A-Za-z][A-Za-z0-9._-]*\s+)\S+` | `${1}[REDACTED]` | ctx-wire |
| Secret Flag Value | `(?i)(--(?:password\|secret\|token\|api_key)\s+)\S+` | `${1}[REDACTED]` | ctx-wire |
| URL UserInfo | `([a-zA-Z][a-zA-Z0-9+.\-]*://[^\s:/@]+:)[^\s@/]+(@)` | `${1}[REDACTED]${2}` | ctx-wire |
| Secret Assignment | `(?i)((?:password\|secret\|token\|api_key)\s*[:=]\s*)\S+` | `${1}[REDACTED]` | ctx-wire |
| Database URL | `(postgres\|mysql\|mongodb)://[^@]+@` | `[REDACTED:db_credentials]@` | Claude plan |

### Key Design: `ScrubFailClosed`

From ctx-wire: if scrubbing panics for any reason, **withhold the output** rather than risk leaking a secret. The proxy logs the error and returns a safe error message to the LLM.

```rust
pub fn scrub_fail_closed(content: &str) -> Result<String, ScrubError> {
    std::panic::catch_unwind(|| scrub_secrets(content))
        .map_err(|_| ScrubError::Panicked)
}
```

### Key Design: `might_contain_secret` Pre-filter

From ctx-wire: a cheap literal-substring check before expensive regex. Skip all regex passes when the content cannot possibly contain a secret. This is critical for performance on large tool results.

```rust
fn might_contain_secret(s: &str) -> bool {
    // Check literal anchors first (case-sensitive)
    const ANCHORS: &[&str] = &[
        "eyJ", "AKIA", "ASIA", "AIza", "ghp_", "gho_", "ghu_", "ghs_", "ghr_",
        "github_pat_", "xox", "sk_", "rk_", "sk-", "-----BEGIN", "://",
        "hvs.", "hvb.", "hvr.", "pypi-",
    ];
    for anchor in ANCHORS {
        if s.contains(anchor) { return true; }
    }
    // Check keyword roots (case-insensitive)
    const KEYWORDS: &[&str] = &[
        "password", "passwd", "secret", "token", "api_key", "access_key",
        "private_key", "auth_token", "client_secret", "credential",
    ];
    let lower = s.to_ascii_lowercase();
    for kw in KEYWORDS {
        if lower.contains(kw) { return true; }
    }
    false
}
```

### Scope

| Action | File | Purpose |
|--------|------|---------|
| Create | `proxy/src/scrub.rs` | `scrub_secrets(content: &str) -> ScrubResult` — applies all patterns, returns redacted content + count of hits |
| Create | `proxy/src/scrub_patterns.rs` | Compiled `OnceLock<Regex>` statics for each secret type — compiled once, reused per request |
| Modify | `proxy/src/transform.rs` | In tool result processing path: run `scrub_secrets()` before any other transform |
| Modify | `proxy/src/config.rs` | Add `scrub_secrets: bool` (default false) + `scrub_patterns: Vec<CustomPattern>` for user-defined patterns |
| Modify | `proxy/src/server.rs` | Pass scrub config through to transform pipeline |
| Modify | `proxy/src/lib.rs` | Export scrub modules |
| Create | `proxy/tests/scrub.rs` | Unit tests: each pattern type (positive + negative), custom patterns, no false positives on code |
| Modify | `docs/SECURITY.md` | Document scrubbing behavior, pattern list, custom pattern config |

### Key Structs

```rust
// proxy/src/scrub.rs

/// Result of secret scrubbing.
pub struct ScrubResult {
    pub content: String,        // redacted content
    pub hits: Vec<ScrubHit>,   // what was found and replaced
}

/// A single redaction event.
pub struct ScrubHit {
    pub secret_type: SecretType,
    pub line: usize,
    pub replacement: String,
}

/// Types of secrets that can be detected.
pub enum SecretType {
    AwsKey,
    GitHubToken,
    GitHubPat,
    GoogleApiKey,
    SlackToken,
    StripeKey,
    OpenaiKey,
    VaultToken,
    PypiToken,
    Jwt,
    PrivateKey,
    BearerToken,
    SecretFlag,
    UrlUserInfo,
    SecretAssignment,
    DatabaseUrl,
    Custom(String),
}

/// A compiled scrub rule.
struct ScrubRule {
    name: SecretType,
    re: Regex,
    replacement: String,
}

/// Custom user-defined scrub pattern.
pub struct CustomPattern {
    pub name: String,
    pub pattern: String,
    pub replacement: String,
}
```

### Completion Criteria — R-37

**Functional**
- A tool result containing `AKIAIOSFODNN7EXAMPLE` produces `[REDACTED:aws_key]` in the forwarded response.
- A tool result containing a GitHub token `ghp_abc123...` produces `[REDACTED:github_token]`.
- A JWT token in tool output is redacted before reaching the LLM.
- The full unredacted content is preserved in `.clean-ctx/proxy-logs/` regardless of scrubbing.
- `context_stats` shows `secrets_scrubbed: N` for the session.
- Normal code content (variables named `api_key_input`, connection string comments) does not produce false positives.
- `might_contain_secret` pre-filter skips regex for clean content (verified via benchmark).

**Non-regression**
- `scrub_secrets: false` (default) produces byte-identical proxy output to current behavior.
- All existing proxy tests pass.
- `cargo clippy --all-targets -- -D warnings` is clean.

**Tests**
- At least 15 new tests: one per secret type (14 positive cases), 2 false-positive tests on real code, 1 custom pattern test, 1 `ScrubFailClosed` panic test.

**Effort:** ~1.5 days. **Risk:** Low (additive transform pass, isolated behind config flag).

---

## R-38: Shell Output Filtering

### Goal

Reduce noisy shell command output to signal before it reaches the LLM. Declarative per-program filter rules define what to keep, what to drop, and how many lines to allow. The full output is preserved locally. Token savings from filtering are tracked in the stats dashboard alongside compression savings.

### Filter Pipeline

The pipeline order is adapted from ctx-wire's proven transform sequence, simplified for the proxy context:

```
1. strip_ansi        — Remove ANSI escape codes (already exists)
2. replace           — Line-by-line regex substitution (normalize before filtering)
3. match_output      — Collapse to summary if output matches success pattern
4. strip/keep_lines  — Drop or keep lines matching patterns
5. group_by          — Bucket lines by key, cap per bucket (for lint output)
6. head/tail         — Keep first/last N lines
7. max_lines         — Hard cap on total lines
8. on_empty          — Emit fallback if all output was stripped
9. §FILTERED marker  — Append reduction summary
```

### Key Design: Filter Selection

From ctx-wire: when multiple filters match, the **most specific** wins (longest matched span). Priority only breaks ties between equal-length matches.

```rust
fn select_filter<'a>(command: &str, filters: &'a [FilterRule]) -> Option<&'a FilterRule> {
    filters.iter()
        .filter(|f| f.matches(command))
        .max_by_key(|f| f.matched_span(command))
        // Priority breaks ties for equal spans
}
```

### Key Design: JSON Guard

From ctx-wire: **complete JSON is never truncated.** If a filter would cut a valid JSON document, the proxy emits it whole instead. This prevents breaking downstream parsers.

```rust
fn json_guard(content: &str, filtered: &str, truncated: bool) -> (String, bool) {
    if truncated && is_complete_json(content) {
        if content.len() <= MAX_JSON_PASSTHROUGH {
            return (content.to_string(), false); // pass whole document
        } else {
            return (format!("[JSON document omitted: {} bytes]", content.len()), true);
        }
    }
    (filtered, truncated)
}
```

### Key Design: Failure Handling

From ctx-wire: on a **failed command** (detectable via tool_result metadata), suppress synthetic success summaries (`match_output` and `on_empty`). Keep the tail of the output, not the head, because the error is usually at the end.

```rust
fn apply_failure_handling(filtered: &str, original: &str, failed: bool) -> String {
    if failed {
        // Suppress "ok" summaries
        // Keep tail instead of head
        // Never replace error output with success message
    }
    filtered
}
```

### Key Design: `match_output` Collapse

From ctx-wire: if the output matches a success pattern (e.g., "Finished" + "test result: ok"), replace the entire output with a one-liner. This is the biggest single savings source.

```toml
# Example: cargo test success collapses to one line
match_output = [
  { pattern = "(?m)(Finished .* target\\(s\\)|test result: ok)", 
    message = "cargo: ok", 
    unless = "(?i)(error|warning|panicked|[1-9][0-9]* failed)" },
]
```

### Key Design: Inline Tests

From ctx-wire: each filter TOML file includes `[[tests]]` blocks for conformance testing. Tests are self-documenting, portable, and runnable.

```toml
[[tests.cargo]]
name = "success collapses"
input = """
   Compiling app v0.1.0 (/repo)
    Finished test [unoptimized + debuginfo] target(s) in 2.34s
running 3 tests
test result: ok. 3 passed; 0 failed
"""
expected = "cargo: ok"

[[tests.cargo]]
name = "compiler error preserved"
input = """
error[E0425]: cannot find value `x` in this scope
  --> src/main.rs:2:5
"""
expected = "error[E0425]: cannot find value `x` in this scope\n  --> src/main.rs:2:5"

[[tests.cargo]]
name = "failed exit with success-looking output is not collapsed"
failed = true
input = """
   Compiling app v0.1.0 (/repo)
    Finished test [unoptimized + debuginfo] target(s) in 1.45s
running 3 tests
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
"""
expected = "    Finished test [unoptimized + debuginfo] target(s) in 1.45s\nrunning 3 tests\ntest result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out"
```

The `failed = true` flag is critical: it tests the failure path where synthetic success must be suppressed.

### Key Design: `group_by`

From ctx-wire: bucket lines by a regex key, then cap per bucket and total buckets. Essential for lint output grouped by file.

```toml
[filters.eslint.group_by]
key = "^(.+?):"           # capture group 1 is the file path
max_per_group = 3          # lines kept per file
max_groups = 10            # files kept total
omit_label = "... %d more in %s"
```

### Built-in Filter Rules

#### cargo / rustc

```toml
[filters.cargo]
description = "Compact cargo build/test/check/clippy output"
match_command = "^cargo\\s+(build|test|check|clippy)\\b"
strip_ansi = true
filter_stderr = true

match_output = [
  { pattern = "(?m)(Finished .* target\\(s\\)|test result: ok)", 
    message = "cargo: ok", 
    unless = "(?i)(error|warning|panicked|[1-9][0-9]* failed)" },
]

strip_lines_matching = [
  "^\\s*$",
  "^\\s*Compiling ",
  "^\\s*Checking ",
  "^\\s*Downloading ",
  "^\\s*Downloaded ",
  "^\\s*Fresh ",
]

max_lines = 100
on_empty = "cargo: ok"

[[tests.cargo]]
name = "success collapses"
input = """
   Compiling app v0.1.0 (/repo)
    Finished test [unoptimized + debuginfo] target(s) in 2.34s
running 3 tests
test result: ok. 3 passed; 0 failed
"""
expected = "cargo: ok"

[[tests.cargo]]
name = "compiler error preserved"
input = """
error[E0425]: cannot find value `x` in this scope
  --> src/main.rs:2:5
"""
expected = "error[E0425]: cannot find value `x` in this scope\n  --> src/main.rs:2:5"

[[tests.cargo]]
name = "failed exit not collapsed"
failed = true
input = """
    Finished test [unoptimized + debuginfo] target(s) in 1.45s
test result: ok. 3 passed; 0 failed
"""
expected = "    Finished test [unoptimized + debuginfo] target(s) in 1.45s\ntest result: ok. 3 passed; 0 failed"

[[tests.cargo]]
name = "clean cargo check collapses"
stderr = "    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.01s"
expected = "cargo: ok"

[[tests.cargo]]
name = "warnings preserved"
stderr = """
warning: unused variable: `x`
  --> src/main.rs:2:9
warning: `app` generated 1 warning
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.01s
"""
expected = "warning: unused variable: `x`\n  --> src/main.rs:2:9\nwarning: `app` generated 1 warning\n    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.01s"
```

#### npm / yarn / pnpm

```toml
[filters.npm]
description = "Compact npm install/build output"
match_command = "^(?:npm|yarn|pnpm|bun)\\s+(install|ci|run|build)\\b"
strip_ansi = true

match_output = [
  { pattern = "(?m)^added \\d+ packages?", message = "npm: ok", unless = "(?i)(error|ERR|vulnerabilit)" },
]

strip_lines_matching = [
  "^\\s*$",
  "^npm warn deprecated",
  "^npm notice",
  "^npm warn",
  "^\\s*\\d+ packages? (is|are) looking for funding",
  "^\\s*run `npm fund`",
]

max_lines = 30
on_empty = "npm: ok"

[[tests.npm]]
name = "clean install collapses"
input = "added 247 packages in 12s\n\n42 packages are looking for funding\n  run `npm fund` for details\n"
expected = "npm: ok"

[[tests.npm]]
name = "error preserved"
input = "npm error code ERESOLVE\nnpm error ERESOLVE unable to resolve dependency tree\n"
expected = "npm error code ERESOLVE\nnpm error ERESOLVE unable to resolve dependency tree\n"

[[tests.npm]]
name = "vulnerabilities preserved"
input = "added 247 packages\nfound 3 vulnerabilities\n  high: 2\n  moderate: 1\n"
expected = "added 247 packages\nfound 3 vulnerabilities\n  high: 2\n  moderate: 1\n"
```

#### git

```toml
[filters.git-diff]
description = "Compact git diff output"
match_command = "^git\\s+(diff|show)\\b"
strip_ansi = true

strip_lines_matching = [
  "^index [0-9a-f]{7,}",
  "^\\\\ No newline at end of file",
  "^Binary files",
]

keep_lines_matching = [
  "^diff --git",
  "^\\+\\+\\+ ",
  "^--- ",
  "^@@",
  "^\\+",
  "^-",
  "^new file",
  "^deleted file",
  "^rename ",
]

max_lines = 500

[[tests.git-diff]]
name = "index hashes stripped"
input = """
diff --git a/src/main.rs b/src/main.rs
index abc1234..def5678 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,3 +1,3 @@
 fn main() {
-    println!("hello");
+    println!("world");
 }
\\ No newline at end of file
"""
expected = """diff --git a/src/main.rs b/src/main.rs
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,3 +1,3 @@
 fn main() {
-    println!("hello");
+    println!("world");
 }"""
```

#### pytest

```toml
[filters.pytest]
description = "Compact pytest output"
match_command = "^(?:python -m )?pytest\\b"
strip_ansi = true

match_output = [
  { pattern = "(?m)\\d+ passed", message = "pytest: ok", unless = "(?i)(FAILED|ERROR|error)" },
]

strip_lines_matching = [
  "^platform ",
  "^rootdir:",
  "^plugins:",
  "^collecting ",
  "^={3,}\\s+(test session|short test summary)",
  "^-{3,}",
  "^\\s*$",
]

max_lines = 100
on_empty = "pytest: ok"

[[tests.pytest]]
name = "clean run collapses"
input = "platform linux -- Python 3.11.0, pytest-7.4.0\ncollecting ... collected 47 items\n\n47 passed in 2.34s\n"
expected = "pytest: ok"

[[tests.pytest]]
name = "failures preserved"
input = "FAILED tests/test_auth.py::test_login - AssertionError\n1 failed, 46 passed in 2.34s\n"
expected = "FAILED tests/test_auth.py::test_login - AssertionError\n1 failed, 46 passed in 2.34s\n"

[[tests.pytest]]
name = "failed exit not collapsed"
failed = true
input = "platform linux -- Python 3.11.0\n47 passed in 2.34s\n"
expected = "platform linux -- Python 3.11.0\n47 passed in 2.34s\n"
```

#### tsc (TypeScript compiler)

```toml
[filters.tsc]
description = "Compact TypeScript compiler output"
match_command = "^(?:npx |bunx )?tsc\\b"
strip_ansi = true

match_output = [
  { pattern = "(?m)^Found 0 errors", message = "tsc: ok" },
]

strip_lines_matching = [
  "^\\s+~+$",
  "^Version",
]

max_lines = 100
on_empty = "tsc: ok"

[[tests.tsc]]
name = "clean compile collapses"
input = "Found 0 errors. Watching for file changes.\n"
expected = "tsc: ok"

[[tests.tsc]]
name = "errors preserved"
input = "src/main.ts(5,1): error TS2322: Type 'string' is not assignable to type 'number'.\nFound 1 error.\n"
expected = "src/main.ts(5,1): error TS2322: Type 'string' is not assignable to type 'number'.\nFound 1 error.\n"
```

#### dotnet / C#

```toml
[filters.dotnet]
description = "Compact dotnet build/test output"
match_command = "^dotnet\\s+(build|test|run)\\b"
strip_ansi = true

match_output = [
  { pattern = "(?m)^Build succeeded", message = "dotnet: ok", unless = "(?i)(error|warning)" },
  { pattern = "(?m)^Test Run Passed", message = "dotnet: ok", unless = "(?i)(Failed|Error)" },
]

strip_lines_matching = [
  "^Microsoft \\(R\\)",
  "^Copyright \\(C\\)",
  "^\\s+Determining",
  "^\\s+Restoring",
  "^Build started",
  "^Time Elapsed",
  "^\\s*$",
]

max_lines = 100
on_empty = "dotnet: ok"

[[tests.dotnet]]
name = "clean build collapses"
input = "Microsoft (R) C# Compiler version 4.8.0\nBuild succeeded.\n    0 Error(s)\n    0 Warning(s)\nTime Elapsed 00:00:02.34\n"
expected = "dotnet: ok"

[[tests.dotnet]]
name = "errors preserved"
input = "error CS0246: The type or namespace name 'Foo' could not be found\nBuild FAILED.\n    1 Error(s)\n"
expected = "error CS0246: The type or namespace name 'Foo' could not be found\nBuild FAILED.\n    1 Error(s)\n"
```

#### Angular CLI (ng)

```toml
[filters.ng]
description = "Compact Angular CLI build output"
match_command = "^ng\\s+(build|test|lint)\\b"
strip_ansi = true

match_output = [
  { pattern = "(?m)^Build succeeded", message = "ng: ok", unless = "(?i)(error|warning)" },
  { pattern = "(?m)^Build failed", message = "ng: build failed" },
]

strip_lines_matching = [
  "^Browser application bundle",
  "^Generating browser",
  "^Processing assets",
  "^Output location:",
  "^\\s*$",
]

max_lines = 80
on_empty = "ng: ok"

[[tests.ng]]
name = "clean build collapses"
input = "Browser application bundle generation complete.\nBuild succeeded.\n"
expected = "ng: ok"

[[tests.ng]]
name = "errors preserved"
input = "Error: src/app/app.component.ts:5:1 - error TS1234\nBuild failed.\n"
expected = "Error: src/app/app.component.ts:5:1 - error TS1234\nBuild failed.\n"
```

### `.clean-ctx.json` Config Schema

```json
{
  "proxy": {
    "auto_cache": true,
    "strip_ansi": true,
    "trim_bash_git": true,
    "scrub_secrets": true,
    "tool_filters": {
      "enabled": true,
      "cargo": { "enabled": true, "max_lines": 100 },
      "npm": { "enabled": true, "max_lines": 30 },
      "git": { "enabled": true, "max_lines": 500 },
      "pytest": { "enabled": true, "max_lines": 100 },
      "tsc": { "enabled": true, "max_lines": 100 },
      "dotnet": { "enabled": true, "max_lines": 100 },
      "ng": { "enabled": true, "max_lines": 80 },
      "custom_program": {
        "enabled": true,
        "keep_patterns": ["^important"],
        "drop_patterns": ["^noise"],
        "max_lines": 50
      }
    }
  }
}
```

### Community Filter Files

Users can add custom filter files to `.clean-ctx/filters/`:

```
.clean-ctx/
  filters/
    jest.toml
    gradle.toml
    docker.toml
```

Format matches built-in files. Each file includes inline `[[tests]]` blocks.

### `§FILTERED` Marker Format

Every filtered tool result appends a summary line the LLM can see:

```
§FILTERED cargo-test: 487 lines → 23 lines (95.3% ↓) | full log: .clean-ctx/proxy-logs/2026-06-14T14:23:11Z_cargo.log
```

### Scope

| Action | File | Purpose |
|--------|------|---------|
| Create | `proxy/src/filters.rs` | `ToolOutputFilter` struct + `apply_filters()` — main filter engine |
| Create | `proxy/src/filter_rules.rs` | `FilterRule` struct, TOML parsing, inline test runner, rule loading |
| Create | `proxy/src/filter_registry.rs` | Program detection heuristics — identify cargo/npm/git/pytest from command string or output shape |
| Create | `proxy/src/community_filters.rs` | Load `.clean-ctx/filters/*.toml` user-defined rules; merge with built-in rules |
| Modify | `proxy/src/transform.rs` | After `scrub_secrets()`, run `apply_filters()` on tool results before forwarding |
| Modify | `proxy/src/config.rs` | Add `tool_filters: ToolFilterConfig` with per-program overrides and global settings |
| Modify | `proxy/src/server.rs` | Load community filter files on startup, pass filter registry to transform pipeline |
| Modify | `proxy/src/lib.rs` | Export filter modules |
| Create | `proxy/src/filter_stats.rs` | `FilterStats` struct — per-program savings accumulator for dashboard |
| Create | `filters/cargo.toml` | Built-in cargo filter rules (shipped with binary as embedded resource) |
| Create | `filters/npm.toml` | Built-in npm filter rules |
| Create | `filters/git-diff.toml` | Built-in git diff filter rules |
| Create | `filters/pytest.toml` | Built-in pytest filter rules |
| Create | `filters/tsc.toml` | Built-in TypeScript compiler filter rules |
| Create | `filters/dotnet.toml` | Built-in dotnet filter rules |
| Create | `filters/ng.toml` | Built-in Angular CLI filter rules |
| Create | `proxy/tests/filters.rs` | Filter engine tests — each built-in program, custom rules, max_lines, §FILTERED marker |
| Create | `proxy/tests/filter_registry.rs` | Program detection tests |
| Create | `proxy/tests/community_filters.rs` | Custom filter loading tests |
| Modify | `docs/TOOL_OUTPUT_FILTER.md` | Full documentation: built-in rules, custom rules, §FILTERED format, community filter guide |

### Key Structs

```rust
// proxy/src/filters.rs

/// A compiled filter rule for a specific program.
pub struct ToolOutputFilter {
    pub program: String,
    pub match_command: Regex,
    pub priority: i32,
    pub strip_ansi: bool,
    pub replace: Vec<ReplaceRule>,
    pub match_output: Vec<MatchOutputRule>,
    pub strip_lines: Vec<Regex>,
    pub keep_lines: Vec<Regex>,
    pub group_by: Option<GroupByConfig>,
    pub head_lines: Option<usize>,
    pub tail_lines: Option<usize>,
    pub max_lines: Option<usize>,
    pub on_empty: Option<String>,
    pub filter_stderr: bool,
    pub reduce_json: bool,
    pub tests: Vec<FilterTest>,
}

/// A line-by-line regex substitution rule.
pub struct ReplaceRule {
    pub pattern: Regex,
    pub replacement: String,
}

/// A collapse rule: if pattern matches, replace output with message.
pub struct MatchOutputRule {
    pub pattern: Regex,
    pub message: String,
    pub unless: Option<Regex>,
}

/// Group-by configuration for bucketing lines.
pub struct GroupByConfig {
    pub key: Regex,
    pub max_per_group: usize,
    pub max_groups: usize,
    pub omit_label: String,
}

/// Inline conformance test.
pub struct FilterTest {
    pub name: String,
    pub input: Option<String>,
    pub expected: Option<String>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub failed: bool,
    pub min_saved_percent: Option<u32>,
    pub draft: bool,
}

/// Result of applying a filter.
pub struct FilterResult {
    pub content: String,
    pub original_lines: usize,
    pub filtered_lines: usize,
    pub original_tokens: usize,
    pub filtered_tokens: usize,
    pub program: String,
    pub reduction_pct: f32,
    pub truncated: bool,
}

/// Registry of all loaded filters.
pub struct FilterRegistry {
    pub builtin: HashMap<String, ToolOutputFilter>,
    pub community: HashMap<String, ToolOutputFilter>,
    pub overrides: HashMap<String, ToolOutputFilter>,
}
```

### Completion Criteria — R-38

**Functional**
- A `cargo test` output of 500 lines is reduced to ≤ 100 lines containing only errors, warnings, and test results. A `§FILTERED` line is appended.
- A `npm install` output is reduced to the summary line and any errors. Deprecation warnings are gone.
- A `git diff` output retains added/removed lines and chunk headers. Index hash lines and no-newline markers are gone.
- A `pytest` run retains only failed test names, exception lines, and the final count.
- `match_output` collapses clean runs to one-line summaries (e.g., "cargo: ok").
- `failed = true` tests verify that failed commands are NOT collapsed to success.
- Custom filter files in `.clean-ctx/filters/` are loaded on proxy startup and applied alongside built-in rules.
- Per-program filter config in `.clean-ctx.json` overrides or extends built-in rules.
- The full unfiltered output is written to `.clean-ctx/proxy-logs/` before filtering.
- Complete JSON in tool results is never truncated (JSON guard).
- `context_stats` shows per-program token savings with line counts and reduction percentages.
- `tool_filters.enabled: false` (default) produces byte-identical proxy output to current behavior.

**Non-regression**
- All R-37 (scrubbing) tests pass.
- All existing proxy tests pass.
- Marker notation compression tests unaffected.
- `cargo clippy --all-targets -- -D warnings` is clean.

**Tests**
- At least 30 new tests: each built-in program positive case (7), each built-in program edge case (7), `match_output` collapse tests (3), `failed` path tests (3), custom filter loading (2), community filter loading (2), §FILTERED format (1), JSON guard (2), filter selection (2), stats accumulation (1).

**Effort:** ~3 days. **Risk:** Low (additive proxy pass, isolated behind config flag, no changes to compression pipeline).

---

## Combined Pipeline (post R-37 + R-38)

```
Proxy receives tool result from MCP tool call
        │
        ▼
Log full unfiltered content to disk ◄── always, regardless of config
        │
        ▼
Secret Scrubbing (R-37) ◄────────────── if scrub_secrets: true
  might_contain_secret() pre-filter
  → regex passes for each secret type
  → ScrubFailClosed: withhold on panic
  → AWS keys, GitHub tokens, JWTs → [REDACTED:type]
        │
        ▼
Program Detection ◄──────────────────── identify cargo/npm/git/pytest/etc.
  from command string in tool_result metadata
  Most-specific-match-wins selection
        │
        ▼
Tool Output Filtering (R-38) ◄────────── if tool_filters.enabled: true
  1. strip_ansi
  2. replace (line-by-line normalization)
  3. match_output (collapse to summary)
  4. strip/keep_lines (pattern matching)
  5. group_by (bucket + cap)
  6. head/tail
  7. max_lines (hard cap)
  8. on_empty (fallback summary)
  9. JSON guard (never truncate valid JSON)
  10. Failure handling (suppress fake success on errors)
  11. §FILTERED marker
        │
        ▼
Forward filtered + scrubbed result to LLM
        │
        ▼
Update filter stats in session dashboard
```

---

## Full Token Surface Coverage (post R-37 + R-38)

| Token Waste Source | Mechanism | Coverage |
|-------------------|-----------|:--------:|
| Source code verbosity | Marker notation compression (existing) | ✅ |
| API envelope overhead | Reverse proxy cache hints (existing) | ✅ Claude only |
| Build tool noise | Tool output filtering (R-38) | ✅ |
| Test runner noise | Tool output filtering (R-38) | ✅ |
| Git diff noise | Tool output filtering (R-38) | ✅ |
| Secrets in output | Secret scrubbing (R-37) | ✅ |
| Unused tools overhead | `drop_tools` in proxy (existing) | ✅ |
| ANSI escape codes | `strip_ansi` in proxy (existing) | ✅ |

**Clean-CTX covers the entire token surface in a single tool.** No need for ctx-wire, no need for Pino proxy, no need for separate secret scanning tools.

---

## Comparison: Clean-CTX vs ctx-wire

| Capability | ctx-wire | Clean-CTX (post R-37+R-38) |
|-----------|:--------:|:--------------------------:|
| Shell output filtering | ✅ | ✅ |
| Declarative filter rules | ✅ TOML | ✅ TOML + JSON |
| Secret scrubbing | ✅ | ✅ |
| Full log preservation | ✅ tee/spool | ✅ proxy logs |
| Community filter sharing | ✅ publish/pull | ✅ .clean-ctx/filters/ |
| Inline filter tests | ✅ `[[tests]]` | ✅ `[[tests]]` |
| Filter selection | ✅ most-specific | ✅ most-specific |
| JSON guard | ✅ | ✅ |
| Failure handling | ✅ | ✅ |
| match_output collapse | ✅ | ✅ |
| group_by bucketing | ✅ | ✅ |
| replace stage | ✅ | ✅ |
| Source code compression | ❌ | ✅ |
| Prompt cache optimization | ❌ | ✅ Claude |
| Framework meta-layers | ❌ | ✅ Angular/NgRx |
| IR delta transport | ❌ | ✅ |
| Air-gap certified | ❌ install.sh | ✅ |
| Single static binary | ✅ Go | ✅ Rust |
| Stats dashboard | ✅ `gain` cmd | ✅ `context_stats` |

---

## Implementation Order

1. **R-37 first** — Scrubbing is independent, low-risk, and the foundation
2. **R-38 core engine** — Filter pipeline, selection, JSON guard
3. **R-38 built-in filters** — cargo, npm, git, pytest, tsc, dotnet, ng
4. **R-38 community filters** — TOML loading, `.clean-ctx/filters/`
5. **R-38 stats** — Per-program savings in `context_stats`
6. **R-38 tune system** — Analyze filter effectiveness (future enhancement)

---

## Tracking

Each sub-feature ends with:
- A passing test suite (`cargo test`)
- A clean linter (`cargo clippy --all-targets -- -D warnings`)
- A ROADMAP status update (📋 proposed → 🚧 in-progress → ✅ done)
- An entry in `CHANGELOG.md`

A feature is not complete until the user signs off on its completion criteria. R-37 ships before R-38 since scrubbing runs first in the pipeline.

---

## License

[CC0-1.0 Universal](https://creativecommons.org/publicdomain/zero/1.0/) — Dedicated to the public domain.