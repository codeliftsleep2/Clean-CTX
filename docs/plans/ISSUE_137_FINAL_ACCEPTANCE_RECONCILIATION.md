# Issue #137 Final Acceptance Reconciliation

**Agent:** Agent MaxHeadRoom  
**Date:** 2026-10-15  
**Issue:** #137, `diff_commits: member-level misses, mis-attribution, noise and ref handling`

## 1. Executive verdict

**#137 COMPLETE REPAIR IMPLEMENTED — VERIFICATION REQUIRED**

The original issue contract was recovered directly from GitHub. It contains no
comments or later amendments. Every bounded acceptance residual is implemented
inside the existing diff acquisition, snapshot, comparison, projection, and ref
validation boundaries. No architectural approval gate was triggered.

This verdict is not `ACCEPTANCE GREEN`: the operator-run bounded regressions and
zero-warning Clippy gate are green, but the authoritative final gate has not
been reported complete. Historical evidence for the unchanged semantic-
signature slice remains separately identified from the new operator runs.

## 2. Original contract

The authoritative Issue #137 body requires:

1. return-type-only and attribute-only changes reported;
2. a last-member body change reported on that member, never its neighbor;
3. unchanged members omitted as `=` lines by default;
4. `^1`/`^2` accepted, or a clear error naming supported grammar;
5. an explicit flag when comparison is lossy.

The Expected section additionally names return type, attributes/decorators,
modifiers, body evidence, unchanged suppression, and disclosure of anything not
compared. The issue's “Not tested” list is not promoted to acceptance.

## 3. Acceptance matrix

Statuses use only the required vocabulary. `SATISFIED` here means the current
source path implements the criterion; rows changed in this reconciliation still
require the verification in section 15.

| # | Criterion | Status | Evidence / disposition |
|---:|---|---|---|
| 1 | Return-type-only | SATISFIED | Landed typed C# `DeclarationEvidence`; historical 5/5 native and 25/25 live evidence. |
| 2 | Attribute/decorator-only | SATISFIED | Landed span-owned C# attribute evidence; historical evidence retained. |
| 3 | Modifiers if promised | SATISFIED | C# `modifier` nodes now join the existing owner-span evidence and participate in equality. |
| 4 | Parameter/signature | SATISFIED | Existing compact signature comparison retained; unique one-to-one signature continuity remains modification. |
| 5 | Body changes | SATISFIED | Existing normalized declaration-owned body fingerprint retained. |
| 6 | Final-member body | SATISFIED | Existing historical final-member control retained; new first/middle/final tracked regression added. |
| 7 | No next-member mis-attribution | SATISFIED | Marker evidence now joins only captures contained by that method's declaration span. |
| 8 | Member add/remove | SATISFIED | Existing actions retained; ambiguous overload remainders now truthfully remove/add. |
| 9 | Method moves | SUPERSEDED | Original result remove+add is explicitly acceptable; no cross-file move-link protocol required. |
| 10 | Unchanged-member suppression | SATISFIED | Default formatter omits unchanged child methods/fields while retaining changed class context. |
| 11 | Loss/capability disclosure | SATISFIED | `§COMPARISON` and `_meta.comparison` expose `lossy`, scope, compared dimensions, and omissions. |
| 12 | Binary handling | SATISFIED | Git blob/disk bytes are preserved; NUL/non-UTF-8 content is labeled binary and never parsed as source. |
| 13 | Working-tree modified | SATISFIED | Existing Git-vs-disk path retained; corrected body/marker comparison applies to disk bytes. |
| 14 | Staged | SATISFIED | Existing `git diff <from>` acquisition covers index plus worktree. |
| 15 | Untracked | SATISFIED | Existing `git ls-files --others --exclude-standard` path retained. |
| 16 | Deleted | SATISFIED | Existing deletion truth retained; deleted binary blobs are now labeled. |
| 17 | Rename | SATISFIED | Git `--find-renames` remains authority; old/new bytes are compared without extension-based binary claims. |
| 18 | Merge/range | SATISFIED | Git remains range/status authority; parent refs are admitted and member fabrication paths are removed. |
| 19 | CRLF/LF artifact | SUPERSEDED | Original item was a one-sample hypothesis, not acceptance; body comparison already collapses semantic whitespace and no global byte normalization was introduced. |
| 20 | `^1` / `^2` | SATISFIED | Decimal parent selectors are admitted, including chained `HEAD~2^1`. |
| 21 | Invalid ref messaging | SATISFIED | Every validation failure names `<name-or-sha>[~N|^N ...]`; unsafe syntax remains rejected. |
| 22 | Fidelity-specific behavior | SATISFIED | Snapshots remain comparison-stable at Medium; fidelity changes projection only. Historical five-fidelity signature evidence retained. |
| 23 | File vs member completeness | SATISFIED | File truth stays Git-owned; admitted semantic scope and omissions are explicitly separate. |
| 24 | Invented/misattributed lines | SATISFIED | Span ownership replaces adjacency; exact/stable-signature overload matching replaces name-index pairing; binary text fabrication removed. |

## 4. Current diff authority chain

### Committed range

`fromRef`/`toRef` → bounded grammar validation → Git `diff --name-status
--find-renames` → Git status/path classification → byte-exact `git show` for
before/after blobs → source-text admission → language capture → declaration
span ownership → comparison → change-only formatter → manifest and MCP metadata.

### Working tree

`fromRef` → validation → Git tracked status plus non-ignored untracked discovery
→ baseline bytes from Git and current bytes from disk → the same admission,
semantic comparison, projection, and disclosure path.

Git owns refs, file status, rename classification, historical bytes, and range
truth. Source parsers own admitted declarations and spans. Neither authority
manufactures facts belonging to the other.

## 5. Semantic comparison contract

For admitted C# method declarations, comparison covers:

- compact written signature/parameter shape;
- structured written return type;
- structured attributes in source order;
- structured modifiers in source order;
- declaration-owned behavior markers;
- declaration-owned, whitespace-collapsed body content.

The contract does not claim unsupported declaration forms, compiler-resolved
type equivalence, cross-file move linking, or full added/deleted source. These
omissions are emitted in-band and in `_meta.comparison`. Consequently an
unchanged class/member result is never presented without the surrounding loss
contract.

## 6. Declaration matching

Methods remain grouped by the existing canonical name derivation. Inside an
overload family the comparator now:

1. preserves exact declaration matches independent of order;
2. pairs unmatched methods with the same visible signature for body/marker/
   declaration-evidence changes;
3. treats a single remaining before/after pair as the existing bounded
   signature change;
4. emits ambiguous overload remainders as remove/add.

This removes name-plus-index inference without introducing a new symbol model.
Moves and ambiguous renames are not guessed.

## 7. Body attribution

`method.root` already supplies the declaration-owned raw body used by the body
fingerprint. The residual defect was marker ownership: descendant marker
captures arrive after their `method.root`, but a pending positional buffer gave
them to the next method and discarded final-member markers. The repair derives
each method's markers only from capture spans contained by that method span.

Tracked cases cover first/middle/final bodies and a first-method conditional
change followed by an unchanged member. Exact overload preservation also guards
against neighboring same-name attribution.

## 8. Unchanged-member projection

Comparison still emits unchanged actions internally for class equality and
summary accounting. Default formatting suppresses unchanged child methods and
fields. Changed members and their class context remain visible. An unchanged
top-level class line remains allowed as scope context, subject to the explicit
loss contract.

## 9. Loss/completeness disclosure

Every manifest now contains `§COMPARISON lossy=true`, the
`admitted-declarations` scope, compared dimensions, and omitted dimensions.
The same values appear at `result._meta.comparison`. A public MCP regression
pins this machine-readable contract.

The current summary is deliberately always marked lossy: it does not include
full added/deleted source, does not understand every declaration form, and does
not provide compiler-equivalence or move linking. This is a truthful bounded
contract, not a global #142 coverage schema.

## 10. Binary handling

`run_git_bytes` and `show_file_bytes` retain blob bytes. Semantic admission
requires UTF-8 without NUL; otherwise output preserves the file-level Git change,
labels it binary, and states that semantic comparison was not attempted.
Extension never overrides byte truth. A tracked `.cs`-named binary regression
guards against empty or fabricated source output.

## 11. Working-tree/newline behavior

Committed content remains Git-owned; working-tree content remains disk-owned.
Tracked, staged, untracked, deleted, and rename classification paths were not
redesigned. No global source-byte or hash normalization was added. Existing
body fingerprints ignore whitespace runs locally, which covers CRLF/LF inside
the promised normalized-body dimension without altering Git/file truth.

## 12. Git ref grammar and safety

The accepted grammar is an alphanumeric-leading name/SHA using
`[A-Za-z0-9._/-]`, followed by optional decimal `~N`/`^N` selectors. Examples:
`HEAD~2`, `HEAD^1`, `HEAD^2`, `abc123^1`, `HEAD~2^1`.

Empty selectors, peel braces, nondigits, whitespace, option-leading strings,
shell metacharacters, and arbitrary revision syntax remain rejected. Git still
runs through `Command::arg`, with `--end-of-options`; no shell interpolation or
pathspec ambiguity was introduced.

## 13. Repairs implemented

- declaration-span-owned markers replace positional pending markers;
- exact/stable-signature overload matching replaces name-index pairing;
- unchanged child actions are suppressed by default;
- C# modifier nodes join typed declaration evidence;
- Git/disk content acquisition is byte-preserving;
- binary changes are labeled and excluded from semantic parsing;
- manifest and MCP metadata disclose comparison loss/capabilities;
- safe decimal `^N` parent selectors are accepted with precise errors;
- the user guide documents the resulting contract.

## 14. Regression inventory

Tracked under `src/tests/**`:

- `src/tests/gitdiff/issue_137.rs`: marker ownership, first/middle/final body
  attribution, overload insertion, modifier-only change, unchanged suppression,
  `.cs` binary handling, and manifest loss disclosure;
- `src/tests/gitdiff/refs.rs`: positive `^1`/`^2`/chained selectors and negative
  grammar/security cases;
- `src/tests/mcp/issue_137_diff_commits.rs`: public `_meta.comparison` contract;
- historical `src/tests/mcp/semantic_signature_evidence.rs`: return type,
  attribute, parameter, final body, unchanged, and neighbor controls.

## 15. Verification evidence

### Historical evidence retained

From the landed semantic-signature slice (`a1ae555b` and its report): focused
native 5/5, existing diff filter 94/94, live immutable-history 25/25, Clippy and
candidate build green. This evidence applies to the materially retained return/
attribute path only. It does not verify the new residual repairs.

### Executed in this reconciliation

- direct GitHub Issue #137 read (body/state/comments; no mutation);
- bounded source/history/document/test inspection;
- `git diff --check` (passed; Git emitted expected line-ending notices);
- `git status --short` and complete patch review;
- active-file line-count inspection before regression additions.
- operator-reported focused regression result: the first run of
  `csharp_modifier_only_change_is_compared_as_declaration_evidence` failed
  because the C# grammar represents method modifiers as anonymous keyword
  tokens rather than a named `(modifier)` node. The producer query was repaired
  to capture the explicit structured keyword tokens. The second operator run
  remained RED because the fixture's bare `class Probe` was admissible to the
  TypeScript fallback and therefore never exercised C# declaration evidence.
  The fixture now uses unambiguous `public class` syntax and asserts that both
  snapshots contain C# modifier evidence before testing the diff. The third
  operator run proved that the expanded anonymous-token query itself caused the
  C# candidate to be discarded before evidence production. That query extension
  was removed. Modifier evidence is now derived inside the existing C# producer
  from the span-owned method declaration, using the structured return capture
  as the boundary and the shared modifier vocabulary. The operator reran
  `cargo test --locked -p clean-ctx --lib --all-features
  csharp_modifier_only_change_is_compared_as_declaration_evidence`; the
  corrected focused regression is GREEN.
- the operator's subsequent zero-warning Clippy run failed on
  `clippy::collapsible-match` in the refactored `field.root` arm. The nested
  condition was replaced with an equivalent guarded match arm. The operator
  reran the same zero-warning Clippy command and reported it GREEN.
- the operator then ran the bounded broader regression suite and reported all
  three commands GREEN:
  `cargo test --locked -p clean-ctx --lib --all-features gitdiff::`,
  `cargo test --locked -p clean-ctx --lib --all-features
  issue_137_diff_commits`, and
  `cargo test --locked -p clean-ctx --lib --all-features
  semantic_signature_evidence_`. This establishes the wider Git diff surface,
  public #137 MCP loss metadata, and the retained semantic-signature controls.

### Not executed by Agent MaxHeadRoom

Agent MaxHeadRoom did not start Cargo, binaries, servers, or verification
scripts. The operator-run focused Cargo tests and Clippy results are recorded
above. No full workspace test gate, formatting gate, file-size script, encoding
script, Clean-CTX binary, MCP server, live harness, CBM/provider test, model
pilot, or complete authoritative final gate has been reported.

## 16. Cross-issue boundary audit

#133–#136 and #140 remain closed baselines. No behavior from #138, #139,
#141, or #142 was redesigned. The disclosure is local to `diff_commits`; it is
not a global coverage schema. No CBM/provider authority, durable semantic index,
global canonical IR change, global hash policy, or cross-language move protocol
was introduced.

## 17. Remaining residuals

No known bounded implementation residual from the original #137 acceptance
contract remains. Verification remains outstanding. Unsupported declarations,
compiler semantic equivalence, move linking, and full added/deleted source are
declared non-goals rather than silently represented as complete.

## 18. Closure recommendation

Do not close #137 yet. Run the focused regression commands and authoritative
gate below. If all are green, the implemented matrix supports:

**#137 COMPLETE REPAIR — ACCEPTANCE GREEN**

No Claude/model-host pilot is currently required: all original acceptance
criteria have deterministic local regressions and metadata assertions. A live
pilot remains last-resort only if local MCP behavior diverges unexpectedly.

## 19. Commands run

Read-only commands used: `Get-Content`, `rg`, `git status --short`, bounded
`git log`, `git show --stat`, `git diff`, `git diff --check`, line counts, and
`gh issue view 137 --repo codeliftsleep2/Clean-CTX --json ...`. Source and docs
were changed only through `apply_patch`. No GitHub mutation occurred.

### Recommended bounded verification commands

```powershell
cargo test --locked -p clean-ctx --lib --all-features issue_137
cargo test --locked -p clean-ctx --lib --all-features semantic_signature_evidence_
cargo test --locked -p clean-ctx --lib --all-features gitdiff::refs::tests
cargo test --locked -p clean-ctx --lib --all-features gitdiff::
```

### Authoritative final gate

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/tests/check-file-sizes.tests.ps1
pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1
pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-utf8.ps1
cargo test --all-features encoding
```

After successful focused and final verification, #137 would be closure-ready.
Stop before GitHub mutation and before #138.
