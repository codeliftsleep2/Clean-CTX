# Cline Hooks Evaluation

**Conclusion: NO new Cline lifecycle hook/plugin infrastructure for now.**
Existing git hooks + CI are the preferred deterministic enforcement mechanism.
This document records the candidates evaluated and the adopted/rejected
decision.

## Context

The repository has zero Cline hook/plugin infrastructure today: there is no
`cline.hooks`/plugin config in `.vscode/settings.json`, no local
`~/.clinerules`, and no plugin files. `.clinerules/` is gitignored (local agent
instruction surface) while `.githooks/` and `.github/workflows/` are tracked
and versioned. Cline's lifecycle hooks (SDK plugin `beforeTool`/`afterTool`/
`run_end`/etc., with `fail_closed`/`fail_open` policies) are a separate
mechanism from git hooks and CI.

## General evaluation criteria

For each candidate: (1) can it be reliably detected mechanically? (2) does it
belong in a Cline hook, a git hook, or CI? (3) should it block
(`fail_closed`) or merely report? (4) could it create false positives or
interfere with normal development? (5) would it reduce prompt/context
complexity?

Prefer deterministic enforcement over prompt instructions, but do not add
infrastructure that is not clearly justified.

## Candidates

| Candidate | Reliably detected? | Best layer | Block? | Risk | Reduces prompt? | Decision |
|---|---|---|---|---|---|---|
| Encoding enforcement after text writes | ✔ (guard on tracked text) | git pre-commit + CI (already exist) | ✔ fail_closed (commit abort) | none | ✔ | **Adopted (existing, no new hook)** |
| Zero-warning gates (clippy `-D warnings`) | ✔ | CI (exists) | gate at PR | per-commit full gate is slow | partial | Keep in CI + prompt timing |
| Full test suite before commit | ✔ but expensive | CI (exists) | no | would destroy velocity | — | No hook |
| Formatting (`cargo fmt --all -- --check`) | ✔ cheap/deterministic | CI — ADD | yes at gate | low (idempotent) | ✔ | **Adopted (CI only, via this task's ci.yml change)** |
| Dirty / untracked temp files | partial; WIP is legitimate | — | no | high false-positive noise | low | Rejected |
| Architectural-migration work detection | ✖ not machine-reliable | — | — | — | — | Rejected; prompt discipline remains |
| Changelog must change when src/ does | diff-based, partial | CI advisory | no | medium (docs-only commits) | low | Optional advisory; not added now |
| Cline lifecycle hooks (Session/PreToolUse/postToolUse plugins) | costly new infra | — | — | session noise + latency; new framework | minimal | **Rejected for now** |

## Adopted

- **Encoding guard** — already enforced by `.githooks/pre-commit` + CI via
  `scripts/check-utf8.ps1`; nothing new required. Prompt only points at it.
- **Formatting gate** — `cargo fmt --all -- --check` added to CI so formatting
  is enforced mechanically rather than by prompt alone.

## Rejected and why

- **Cline lifecycle hooks / plugins** — no existing infrastructure; a hook
  layer would be new framework, not simplicity, and per-session hook output
  adds noise/latency. The encoding guard proves the better model: a versioned
  guard + git hook + CI + a short prompt pointer.
- **Per-commit clippy/full-test hooks** — too expensive and would destroy
  implementation velocity.
- **Dirty/untracked file checks** — WIP is legitimate; too many false
  positives.
- **Migration detection** — not reliably detectable mechanically.
- **Changelog-on-release enforcement** — only advisory and version correctness
  is not safely checkable.

## How mechanical enforcement is achieved without Cline hooks

- `.githooks/pre-commit` → `scripts/check-utf8.ps1` (fail-closed on commit when
  wired via `scripts/install-git-hooks.ps1`, i.e. `core.hooksPath = .githooks`).
- `.github/workflows/ci.yml` → check, clippy `-D warnings`, fmt, tree-sitter,
  UTF-8 guard, full tests, `cargo audit` (fail-closed at PR merge/release).
- `src/tests/encoding.rs` → `cargo test encoding` (Rust-side twin).

This matches the engineering goal: deterministic enforcement outside prompt
context, with rules kept small and routing to docs for conditional detail.