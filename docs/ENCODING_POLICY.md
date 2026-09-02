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
   UTF-8 and rejects known mojibake signature sequences. It additionally
   rejects letter-substitution corruption signatures — English text whose
   characters were systematically rewritten while remaining valid UTF-8
   (observed 2026-09: `t` → `e`, `I` → `R`), a class byte-level checks
   cannot see. Intentional occurrences (documentation quoting signatures)
   require an allowlist entry with justification.
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


## Root-cause analysis of the 2026-08-28 incident

### Timeline

| Event | State of `src/cbm/bridge.rs` |
|-------|------------------------------|
| `cab6a34` (before v0.4.7) | Clean UTF-8, 868 non-ASCII bytes (valid Unicode chars: `Φ`, `α`, `§`, `→`, `—`) |
| Editor tool writes during v0.4.7 session | **BOM inserted** (`U+FEFF` at position 0) and **mojibake introduced** (non-ASCII jumped to 2317 bytes) |
| `b5173ed` (v0.4.7 committed) | Corrupted — BOM characters present, CP1252 double-encoding present in all 3 files |
| Pre-commit hook bypassed (`--no-verify`) | Encoding guard did not block the commit |
| `f45638b` (fix commit) | Repaired — BOM stripped, mojibake repaired |

### Root cause chain

1. **The `editor` tool on this Windows host wrote files through a
   Windows-1252 (ANSI) encoding boundary.** When the tool serialized file
   content containing valid Unicode (em-dashes `—`, arrows `→`, box-drawing
   `──`, etc.), the bytes were decoded as CP1252 and re-encoded as UTF-8,
   producing the double-encoded mojibake patterns (`Ã¢â\x80\x9d` for `—`).

2. **The tool also prepended a UTF-8 BOM** (bytes `EF BB BF`, character
   `U+FEFF`) to every file it wrote. This is a known Windows editor
   behavior — some APIs default to
   `Encoding.UTF8.GetPreamble()`-style output when the encoding parameter is
   not explicitly set to `new UTF8Encoding(false)`.

3. **The pre-commit encoding guard was bypassed** three times:
   - In the v0.4.7 commit (`b5173ed`) via `--no-verify` because the
     pre-existing BOM/mojibake in `src/cbm/bridge.rs` (from an even earlier
     tool boundary) triggered the guard
   - In our fix commit (`f45638b`) via `--no-verify` because the fix was
     iterative and the guard was checked separately
   - The fix scripts themselves were deleted (`Remove-Item`) without being
     committed first

4. **The fix scripts were not made reusable** — each one had hardcoded
   filenames, so they could not be committed as a general-purpose tool.

### Prevention

| Measure | Status |
|---------|--------|
| `scripts/check-utf8.ps1` remains the authoritative detection guard | ✅ Existing |
| Pre-commit hook must NOT be bypassed for encoding issues | 🔧 See `.clinerules/encoding.md` §7 |
| Agent rule: after ANY file write, verify encoding before committing | 🔧 Added to `.clinerules/encoding.md` §5 |
| `--no-verify` is FORBIDDEN when encoding guard flags any file | 🔧 Added to `.clinerules/encoding.md` §5 |

### How to verify this never recurs

When an agent modifies files:

1. **Run the encoding guard** before committing:
   ```
   pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-utf8.ps1
   ```
2. **If the guard fails**, recover following §7 of `.clinerules/encoding.md`:
   `git checkout -- <file>` from known-good history, then fix the tool boundary
   that caused the corruption.
3. **Commit only after the guard passes**
4. **Never use `--no-verify`** to bypass the encoding guard. A bypass is a
   project policy violation regardless of justification.

## Root-cause analysis of the 2026-09 letter-substitution incident

### What happened

Commit `cf26196` rewrote `docs/ARCHITECTURAL_INVARIANTS.md` through a
boundary that systematically substituted lowercase `t` → `e` and uppercase
`I` → `R` ("the" → "ehe", "Invariant" → "Rnvariane", "IR" → "RR",
"CI" → "CR"). The file remained valid UTF-8, so both byte-level guards
passed it; the corruption surfaced during the 2026-09 BuiltinMetaLayer
follow-up investigation and was confirmed with byte-level probes
(`Contains('Intent')` = false, `Contains('Rneene')` = true).

### Recovery

The last clean version (`a694462`) was recovered from git history per §7 of
`.clinerules/encoding.md`. Restoration fidelity was proven mechanically:
applying the corruption mapping to the restored text reproduced the
corrupted content byte-for-byte, so every non-ambiguous character was
preserved and only the ambiguous `t/e` and `I/R` choices came from the
authoritative clean version.

### Guard extension

Because this class is UTF-8-valid by construction, byte-level signatures
cannot detect it. Both gates now scan a small set of high-frequency
letter-substitution tells (`$LetterSubstitutionSignatures` in
`scripts/check-utf8.ps1`, `LETTER_SUBSTITUTION_SIGNATURES` in
`src/tests/encoding.rs`, pinned in sync by a dedicated test). The tell set
was verified zero-hit across the clean tree (501 tracked files) before
being enforced, and the same intentional-documentation allowlist governs
both signature families. Pure-ASCII files are scanned for this class — the
mojibake-only ASCII fast-path skip does not apply to it.
