## Description

<!-- Provide a clear and concise description of the changes in this PR. -->

Closes #(issue-number)

---

## Type of Change

<!-- Mark the relevant option(s) with an "x". -->

- [ ] Bug fix (non-breaking change that fixes an issue)
- [ ] New feature (non-breaking change that adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to change)
- [ ] Documentation update
- [ ] Refactoring (no functional changes)
- [ ] Performance improvement
- [ ] Test addition or improvement
- [ ] CI/tooling change

---

## PR Checklist

<!-- Mark completed items with an "x". If an item is not applicable to your changes,
     delete the checkbox line or replace it with "N/A". -->

### Code Quality

- [ ] `cargo check` passes without errors
- [ ] `cargo clippy --all-targets -- -D warnings` produces zero warnings
- [ ] `cargo test` passes — all existing tests (currently 121) still pass
- [ ] New tests added for new functionality, with at minimum:
  - Happy path test(s)
  - Error/invalid-input test(s)
  - Edge case test(s) (empty input, Unicode, boundaries)
- [ ] `cargo audit` shows no known security vulnerabilities
- [ ] `scripts/check-utf8.ps1` passes — text files remain valid UTF-8, no mojibake introduced ([policy & rationale](docs/ENCODING_POLICY.md))
- [ ] No new `#![allow(...)]` annotations without a `// SAFETY:` or `// Phase N:` comment
- [ ] No new `.unwrap()` calls without a `// SAFETY:` comment explaining why it cannot fail
- [ ] No `let _ = ...` dead-code suppression — unused variables are removed
- [ ] No `unsafe` blocks added (entire codebase is safe Rust)

### Code Style

- [ ] Follows the [Single Responsibility Principle](https://blog.cleancoder.com/uncle-bob/2014/05/08/SingleReponsibilityPrinciple.html) — each module/file owns one concern
- [ ] Uses `HashMap` over `BTreeMap` unless sorted iteration is explicitly required
- [ ] Error paths return `Result`, not panics
- [ ] Functions are documented with doc comments (`///`) explaining their purpose, parameters, and return value
- [ ] Inline comments explain non-obvious logic, especially around tree-sitter queries and opcode handling

### Documentation

- [ ] If adding a new language:
  - [ ] `Cargo.toml` updated with pinned tree-sitter grammar
  - [ ] Tree-sitter queries added to `src/queries.rs`
  - [ ] `src/compression/language.rs` updated with extension and heuristic
  - [ ] `docs/DEVELOPER_DOCUMENTATION.md` referenced (no changes needed — the guide covers the process)
  - [ ] Tests added for language detection
- [ ] If adding a new MCP tool:
  - [ ] Handler function added in `src/mcp/tools.rs`
  - [ ] Tool definition added in `get_tool_definitions()`
  - [ ] Dispatch arm added in `dispatch_tools_call()`
  - [ ] Tests added in `src/tests/mcp/tools.rs`
- [ ] If adding a new opcode:
  - [ ] Entry added to `src/compression/opcodes.rs`
  - [ ] Opcode table assertion count updated
  - [ ] Decompression marker added (if applicable)
  - [ ] README opcode reference updated
- [ ] If adding a dependency:
  - [ ] Dependency is MIT or Apache-2.0 licensed (no GPL/AGPL)
  - [ ] `cargo audit` re-run after dependency change
- [ ] If changing the public API or tool behavior:
  - [ ] README updated with new examples
  - [ ] `docs/CHANGELOG.md` updated

### Performance (if applicable)

- [ ] No unnecessary allocations in hot paths (`format!` → `write!`/`writeln!` in loops)
- [ ] Cache integration considered for repeat calls (content-hash registry, baseline snapshots, raw-token counts)
- [ ] Large files (>1 MB) and large workspaces (>1,000 files) tested for reasonable performance

### Migration / Backward Compatibility (if applicable)

- [ ] No breaking changes to existing tool signatures (parameter names and types unchanged)
- [ ] If breaking changes are unavoidable, they are documented in `docs/CHANGELOG.md` with migration instructions

---

## Additional Context

<!-- Add any other context about the PR here, such as screenshots,
     benchmark results, or links to related issues/PRs. -->

---

## Checklist for Reviewers

<!-- This section is for the PR author to leave notes for reviewers. -->

- [ ] Changes are scoped and focused — consider splitting into multiple PRs if too large
- [ ] Test coverage is adequate for the change
- [ ] Edge cases are handled (empty input, Unicode, boundary conditions)
- [ ] No regressions introduced in existing functionality