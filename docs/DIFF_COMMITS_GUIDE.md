# `diff_commits` — Multi-File Git Diff

> **Owner:** Git-combined diff engine (`fromRef`/`toRef`/`fidelity`) · **Status:** Living reference
>
> **R-12 (v0.3.0)** — Diff an entire workspace between two git refs in a single MCP tool call. Emits per-file **AST-level change-sets** (not raw file contents), so the LLM sees *what changed* with a fraction of the tokens.

## What it does

`diff_commits` answers: **"What changed in this PR / between these two commits?"**

When both `fromRef` and `toRef` are provided, it runs `git diff --name-status --find-renames` to enumerate every changed file between two refs, then for each file produces a **compact structural delta** instead of the full file content. When `toRef` is omitted (working-tree diff), it additionally discovers non-ignored untracked files via `git ls-files --others --exclude-standard` so that brand-new files appear alongside tracked modifications.

| Change type | What the output shows |
|---|---|
| **Modified** (ts/js/cs) | AST change-set: `+ method saveUser`, `- field userMap`, `~ method login ~ was: X now: Y` |
| **Modified** (html/css/json/md) | Line-count delta: `+2 lines (2 → 4)` |
| **Added** (ts/js/cs) | Skeleton: `+ import …`, `+ class NewClass` |
| **Added** (other) | Line-count: `+42 lines (0 → 42)` |
| **Deleted** | One-line entry: `- FILE αN: path (deleted)` |
| **Renamed** | Diff between old path at `from` and new path at `to` |

## How it saves tokens

The LLM doesn't need the full content of every file — it needs the **delta**. Consider a PR touching 3 files:

- `src/user.service.ts` — 40 → 45 lines (added a method)
- `src/auth.guard.ts` — 30 → 30 lines (changed a signature)
- `src/models/user.ts` — deleted

**Naive approach** (read all 3 files): ~115 lines of source sent to the LLM.

**With `diff_commits`:**

```
§GITDIFF HEAD~1..HEAD (3 files)
┌ FILE α1: src/user.service.ts (+1 -0 ~0)
+ method saveUser(u: User): void
┌ FILE α2: src/auth.guard.ts (+0 -0 ~1)
~ method login ~ was: login(token: Token) now: login(token: Token, ttl: number)
- FILE α3: src/models/user.ts (deleted)
```

That's **~6 lines instead of ~115** — roughly a **95% token reduction** for this diff. And the LLM gets *more* useful information: it sees exactly what changed semantically, not a wall of mostly-unchanged code it has to mentally diff.

### Why AST-level diff beats raw `git diff`

The tool reuses the existing `src/diff` machinery (`build_snapshot` + `diff_snapshots` + `format_diff`), which parses each file with tree-sitter and diffs the **AST structure** — classes, methods, fields, imports. So instead of a raw `git diff` (which shows every changed line including whitespace, renames, and unchanged context), it emits semantic markers. This is both **more token-efficient** (no unchanged context lines) and **more informative** (the LLM understands the *nature* of the change, not just that lines changed).

## How to use it

### Parameters

| Parameter | Type | Required | Default | Description |
|---|---|---|---|---|
| `fromRef` | string | **Yes** | — | The baseline git ref. e.g. `HEAD~1`, `main`, `abc123`, `v1.0`. Strictly validated. |
| `toRef` | string | No | working tree | The target ref. Omit to diff against uncommitted changes. |
| `workspaceRoot` | string | No | CWD | The repo root. Resolved against the trusted root (XPIA mitigation). |
| `fidelity` | string | No | config default | Compression fidelity: `low` (max compression), `medium` (balanced), `high` (minimal compression, preserves most detail). |

### Examples

**Diff the last commit against its parent:**
```json
{
  "fromRef": "HEAD~1",
  "toRef": "HEAD"
}
```

**Diff uncommitted working-tree changes against HEAD:**
```json
{
  "fromRef": "HEAD"
}
```

**Diff a branch against main:**
```json
{
  "fromRef": "main",
  "toRef": "feature/auth"
}
```

**Diff with high fidelity (preserve more detail):**
```json
{
  "fromRef": "HEAD~1",
  "toRef": "HEAD",
  "fidelity": "high"
}
```

### Output format

The tool returns a `content` string with this structure:

```
§GITDIFF <from>..<to> (N files)
┌ FILE α1: <path> (+A -D ~M)
<change-set body>
┌ FILE α2: <path> (+A -D ~M)
<change-set body>
- FILE α3: <path> (deleted)
~ FILE α4: <old> → <new> (+A -D ~M)
<change-set body>
```

- The header line `§GITDIFF <from>..<to> (N files)` gives the scope.
- Each file section starts with `┌ FILE αN:` (modified/added), `- FILE αN:` (deleted), or `~ FILE αN:` (renamed).
- The `(+A -D ~M)` counts are per-file: added/deleted/modified structural elements.
- Files exceeding resource limits are emitted as one-line skip entries and counted in `skipped`.

## Security

- **Ref validation**: `fromRef`/`toRef` are validated against `^[A-Za-z0-9][A-Za-z0-9._/\-~]*$` — rejects flag injection (`--upload-pack`), shell metacharacters, and whitespace.
- **Path validation**: paths returned by git are validated against the same allowlist (no absolute escapes, no flag-like paths).
- **`--end-of-options`**: all git subcommands use `--end-of-options` to prevent path/flag injection.
- **No shell**: git is invoked via `std::process::Command` (no `shell=True` equivalent).
- **Fail-closed**: `is_git_repo` check rejects non-repo directories; invalid refs return a structured error.

## Resource limits

The tool enforces two caps to prevent runaway output:

- **`max_files`** (default: 100) — caps the number of changed files processed. Files beyond the cap are counted in `skipped`.
- **`max_file_size`** (default: 10 MB) — caps per-file content size. Oversized files are skipped with an "exceeds size limit" marker.

A skipped file is counted in `skipped` **only** — it is never double-counted in `counts`. The invariant `counts + skipped == file_count` always holds.

## When to use it (vs. other tools)

| Tool | Scope | Use when you need… |
|---|---|---|
| `compress_code_context` | 1 file | Full structural skeleton of a single file |
| `diff_code_context` | 1 file | In-session AST delta (baseline → current) |
| `delta_code_context` | 1 file | IR-level delta for edit sessions |
| **`diff_commits`** | **whole workspace** | **"What changed in this PR?" — multi-file git-ref diff** |
| `compress_workspace` | whole workspace | Full skeletons of all files (not just changes) |

**Rule of thumb:** If you're asking "what changed between two commits/branches?", reach for `diff_commits` first. If you need the full content of a specific file that appeared in the diff, follow up with `compress_code_context` on that file.

## Limitations

- **Non-compressible files** (html/css/json/md) fall back to a **line-count delta** rather than a structural diff — still compact, but less semantic detail.
- **Added files** emit a skeleton (imports + class names), not the full new content — so if the LLM needs the *body* of a brand-new file, follow up with `compress_code_context`.
- It's a **summary tool** — for deep work on a specific file, use `compress_code_context` or `provide_code_context` on that file.
- `diff_commits` with `to` omitted (working-tree diff) includes tracked changes plus **non-ignored untracked files**. Ignored files remain excluded. Untracked files do not need to be staged or intent-to-added.

---

*See also: `docs/CONFIGURATION.md` (resource limits + security posture), `docs/ROADMAP.md` (R-12 status).*
