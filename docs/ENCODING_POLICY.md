# Encoding Policy: UTF-8 as a Project Invariant

**Authoritative rule:** [`.clinerules/encoding.md`](../.clinerules/encoding.md).
This document explains *why* the invariant exists and how it is enforced. The
rule file governs; this page never overrides it.

> **Location note:** `.clinerules/` is intentionally gitignored in this
> repository ("local AI assistant configuration, user-specific"). This tracked
> document therefore serves as the canonical, versioned statement of the same
> policy for collaborators, CI, and non-Cline contributors, while the
> `.clinerules` copy remains the live instruction source for agent sessions on
> machines where it exists. Keep the two in sync when the policy changes.

## Why encoding is an explicit invariant

Clean-CTX compresses, marks, and renders source code containing real-world
Unicode: Greek opcodes (`Φ`, `α`, `§`), path aliases (`§PATHMAP`), scope
headers (`// ── Name ──`), arrows in legends, and user text of any script.
Corrupting one byte changes behavior: test assertions fail, compression
markers stop matching, rendered IR misleads the consumer.

Historically this repository suffered exactly such corruption
(`docs/CHANGELOG.md`: mojibake `0xCE+0xA6` sequences replaced valid `Φ`/`α`/`§`
in test assertions), so the risk is demonstrated, not hypothetical.

## Why LLM-assisted development makes this urgent

Agent workflows add failure-prone boundaries that human-only development does
not:

```
LLM  ->  tool call  ->  serialization  ->  filesystem  ->  parser  ->  Git / CI
```

Typical observed failures on Windows hosts:

| Boundary | Failure mode |
|----------|--------------|
| PowerShell 5.1 pipe | UTF-8 bytes decoded through ANSI/cp1252 defaults, re-encoded with damage |
| Shell output shown to the model | Console mangling leads the agent to "repair" clean files |
| Editor default without explicit encoding | BOM inserted or legacy codepage used |
| Naive find-and-replace | Agent "fixes" suspicious characters, destroying intentional Unicode |

The defense principle: **do not rely on the model remembering how to handle
encoding correctly.** Establish the invariant in project rules and enforce it
mechanically.

## How enforcement works

Two deterministic guards back the rule file (no new dependencies):

1. **`scripts/check-utf8.ps1`** — CI step (`.github/workflows/ci.yml`) and
   pre-commit gate. Strictly decodes every tracked text file as BOM-less
   UTF-8 and rejects known mojibake signature sequences. Intentional
   occurrences (documentation quoting signatures) require an allowlist entry
   with justification.
2. **`src/tests/encoding.rs`** — runs under plain `cargo test`. Asserts the
   repo tree passes the same scan and that the Unicode canary fixture
   ([`src/test_files/unicode_canary.txt`](../src/test_files/unicode_canary.txt))
   survives byte-for-byte across every boundary above, including Git checkout
   (`eol=lf` pinned in `.gitattributes`).

Both guards keep their own sources pure ASCII where feasible (escapes,
codepoint construction) so they cannot be corrupted by the very problem they
detect.

## What to do when mojibake appears

Follow §7 of [`.clinerules/encoding.md`](../.clinerules/encoding.md): stop,
identify the boundary, recover intended text from git history or known intent,
fix the boundary (not just the bytes), re-run both guards. Never silently
replace suspicious characters, and never assume non-ASCII means corruption.

## Suggested pre-commit usage

```powershell
powershell -NoProfile -ExecutionPolicy Bypass ./scripts/check-utf8.ps1  # local Windows (PS 5.1)
# On Linux/macOS or CI (PowerShell 7+ / `shell: pwsh` in ci.yml):
# pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-utf8.ps1
cargo test encoding
```
