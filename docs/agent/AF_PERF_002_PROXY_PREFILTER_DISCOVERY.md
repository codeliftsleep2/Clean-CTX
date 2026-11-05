# AF-PERF-002: scrub prefilter versus existing rule vocabulary

Agent: Agent MaxHeadRoom
Status: approved eight-root correction FIX / PROTECTED.
Original two-test/31-case RED, focused66 GREEN, eight isolated root-removal REDs/
exact restored GREEN and final composed59 GREEN verified. Historical statuses below
remain chronological evidence. Protected comparator and rules unchanged.

## Independent obligation and concrete source evidence

The scrub prefilter claims a superset of all regex triggers. Returning false skips
all rules, so false negatives affect correctness, not merely token/performance counts.
This gate precedes the protected exact-content comparison; repairing that comparator
does not imply the gate covers every existing rule alternative.

scrub_patterns supports pwd and optional underscore/hyphen/no separator in api-key,
access-key and private-key families. KEYWORD_ROOTS contains api_key/access_key/
private_key but not all alternate spellings or pwd. Authorization regex accepts any
scheme beginning with a letter, while prefilter roots include bearer but not the
literal authorization header name. Basic/Digest/Custom examples therefore require
explicit admission checks. These are source-grounded predicted failures, not executed
RED or a claim of real credential exposure. No redaction scope expansion is proposed.

New tracked tests under src/tests/proxy are loaded from proxy/src/lib.rs. One table
covers23 existing assignment/flag names, lower and uppercase forms, and four existing
Authorization schemes:96 bounded cases. Each case first requires the actual production
regex to match, preventing the fixture from silently inventing a new policy. The test
reports every admission and accumulates all omitted triggers before its final invariant
assertion. The second test asserts exact complete request JSON and hit propagation
through actual Pipeline::build/run for synthetic apikey=abcdefghij.

Only synthetic values are used. Production prefilter, rules, comparator and pipeline
remain unchanged. No new cache, regex, normalization, generic registry or runtime is
introduced. The table is a finite contract slice, not generated-regex completeness.
URL/token/PEM anchors, quoting and Unicode regex policy remain independent obligations.

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-proxy-prefilter-budget.ps1 -Phase Before
```

Expected two intended correctness failures, zero ignored. The wrapper preserves
scrub/pattern/test bytes and hashes, selects only these tests, and checks both exact
failure markers. Compilation or fixture-rule mismatch is not accepted RED. Naming
follows earlier drivers but this is correctness, not an optimization work budget.
Agent only performed bounded source, UTF-8/size and script syntax checks; no test,
build, formatter, lint or runtime was agent-started.

After actual evidence, return a bounded correction design for owner approval. Any
candidate should preserve actual regex eligibility and merely make the cheap gate
a superset; avoid changing matching rules, accounting, publication, logging or the
protected comparator. Extra false positives cost work but must not change redaction
policy. No production correction is implemented under discovery authorization.

Syntax accounting remains9763 preserved/13 scoped/9750 pending plus96 callback
supplements (unrelated to this96-case test table). Public HTTP/logging and remaining
proxy behavior remain uncharacterized. Broader audit discovery stays incomplete.


## Actual existing-rule omissions verified

proxy-prefilter-before-20261104-195317 executes two intended correctness failures,
zero passes/ignored,0.09s. The rule-prerequisite table reaches all96 cases (no invalid
fixture input);31 are excluded by the prefilter. Pipeline retains synthetic apikey
value instead of the exact existing-rule redaction. All 31 source hashes
match before this documentation update; inspection.json retains omissions/counts.
No source algorithm change or agent runtime followed the run.

| Existing matching spelling | Excluded tested cases |
| --- | ---: |
|pwd|4 assignment/flag and lower/uppercase|
|apikey, api-key|8|
|accesskey, access-key|8|
|privatekey, private-key|8|
|Authorization Basic, Digest, Custom|3|

The65 other table cases pass admission. This establishes31 finite omissions, not31
independent production defects or complete regex-language coverage. All values are
synthetic. Protected exact-content comparator tests remain unchanged; this defect
is earlier in the gate, not evidence that its completed repair is missing.

### Smallest proposed correction, owner disposition required

Add eight exact lowercase trigger substrings to existing KEYWORD_ROOTS only:
pwd, apikey, api-key, accesskey, access-key, privatekey, private-key, authorization.
Existing ASCII case folding admits uppercase forms. Keep all old roots/anchors,
rule syntax, replacements, ordering, hit accounting, pipeline and comparator code.
The roots only admit regex evaluation; they do not themselves redact text. Inputs
that contain a keyword but match no existing rule must remain byte-exact with zero
hits. No regex generator, new matching engine, normalization or cache is proposed.

This bounded table repair covers the demonstrated ASCII spellings and scheme
examples. The regex engine's Unicode case-insensitive language versus ASCII-only
prefilter folding remains a separate untested equivalence obligation; do not call
these eight additions proof of a universal superset. Return any additional concrete
mismatch separately rather than silently broadening matching policy.

After approval: keep both original assertions unchanged, apply roots-only edit,
require GREEN, add keyword-present/nonmatching and exact per-family byte/hit/order
controls, existing scrub/pattern/pipeline/transform suites and protected comparator
controls; prove isolated removal of the added roots RED and exact restored GREEN.
Retain original source/hash evidence and update mandatory selection only with real
tracked regressions. No prefilter correction is implemented in this return.

Disposition: confirmed prefilter correctness defect / FIX PROPOSED. Approval is
needed for this separate production redaction behavior correction under the
repository's architectural approval gate; discovery authorization alone does not
approve it. Broad proxy performance and source-wide inventory stay incomplete.


## Approved eight-root correction authored

Only production diff is eight KEYWORD_ROOTS entries:pwd,apikey,api-key,accesskey,
access-key,privatekey,private-key,authorization. Removing exactly those added lines
reconstructs the frozen195317 patterns-source.rs text. Current scrub.rs hash still
matches that baseline, proving the protected comparator was not edited. No regex,
normalization, replacement, order, public contract or other prefilter logic changed.

Both original failing tests remain unchanged. New exact96-case control checks the
same23 names/lower-uppercase/assignment-flag and four authorization schemes. Each
must yield exact standalone bytes, one ordered appropriate rule hit with existing
replacement/line metadata, identical complete pipeline JSON and one statistic, and
no new hit on repeat. A separate16-case negative control includes every added root
in lower/uppercase documentation text, requires prefilter admission but byte-exact
standalone/pipeline output and zero hits. Admission alone never grants redaction.

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-proxy-prefilter-focused.ps1
if ($LASTEXITCODE -eq 0) {
    pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-proxy-prefilter-mutation.ps1
}
```

Focused expects66 tracked passes: four prefilter tests, six protected comparator
regressions,10 existing pattern,24 scrub, five pipeline,17 transform controls. Then
proxy all-target/all-feature warning-fatal lint, scoped format/format-check, active
sizes and strict encoding; preserve source hashes. Agent does not run this packet.

Mutation first requires clean four-test prefilter GREEN in an isolated exact-source
clone. It removes one added root at a time from original current bytes, runs the
exact original96-case test and requires that specific root's omission and genuine
one-test RED. Each mutation restores exact bytes in finally before the next; all
eight independent root regressions are required. Final exact restored source must
pass all four tests and match active hash. No active workspace mutation or server.
Permanent four-test selection is added to existing mandatory gate; it does not
replace protected six-test comparator selection.

The simpler After evidence driver now expects four tests if reused; original Before
receipt and assertions remain frozen. New packets and tests are UNRUN. Agent only
checked source-diff identity, UTF-8/no-BOM, code ceilings and PowerShell syntax.
If any demonstrated case remains blocked or any test needs rule/policy changes,
stop for separate disposition. Final FIX / PROTECTED awaits executed evidence.
Unicode equivalence and universal regex-language coverage remain explicitly open.


## Focused complete table and eight-root sensitivity GREEN verified

proxy-prefilter-focused-20261104-200149 passes66 tracked tests: four prefilter
controls0.08s, six protected comparator regressions0.03s,10 pattern0.05s,24 scrub
0.11s, five pipeline0.01s,17 transform0.03s. Zero failed/ignored. All96 original
rule cases now admit; separate96-case oracle verifies exact standalone/pipeline
bytes, hit metadata and repeat behavior, including31 former omissions and65 former
successes. All16 keyword-only negatives retain exact bytes and zero hits. No existing
matching predicate was altered to produce these results.

Proxy all-feature/all-target warning-fatal Clippy8.45s, empty format-check output,
size18 active files and strict encoding2109 files pass. All 31 focused
source hashes match before documentation changes.

proxy-prefilter-mutation-20261104-200235 first passes all four prefilter controls.
Each of the eight independent real production root removals causes the exact original
table test to fail with that root's expected omission, zero ignored. Exact restoration
after each mutation and at completion gives four GREEN0.08s. Active patterns bytes
match restored isolated source; all 2116 inventory hashes match at inspection.
inspection.json retains receipt checks,96 admitted/exact cases and eight sensitivity
results. No production/test changes follow these verified executions.

Narrow FIX has focused semantic and permanent regression/mutation evidence GREEN;
final composed gate is pending before FIX / PROTECTED acceptance. Unicode/full-rule-
language equivalence remains separate and unresolved, as expressly scoped by owner.

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-semantic-work-fitness.ps1
```

Expected at least59 tracked executions: previous55 plus four prefilter controls.
Original195317 RED stays frozen; no claim of all-proxy or architectural discovery
completion. Agent performed evidence inspection only, no runtime execution.


## Final approved-scope acceptance: FIX / PROTECTED

Composed mandatory gate af-perf-001-20261104-201445 passes 59 tracked executions
across 13 selected groups, zero failures/ignored, including all four prefilter
controls and six protected comparator regressions. All 220 recorded source hashes
match at inspection; inspection.json retains per-group counts and source checks.
No production/test changes follow this final verification. Agent inspected evidence
only and did not start tests/builds/lints/runtime.

Exact production change remains eight missing KEYWORD_ROOTS entries only. Original
96-case table now admits every case; exact standalone/pipeline96-case oracle and16
keyword-only negatives pass. All31 demonstrated omissions are corrected; the65
previously admitted cases retain expected bytes and accounting. No matching rule,
replacement, normalization, precedence, comparator or pipeline contract changed.

Evidence chain: proxy-prefilter-before-20261104-195317 original two-test RED with31
omitted cases; proxy-prefilter-focused-20261104-20014966 passes and clean affected
gates; proxy-prefilter-mutation-20261104-200235 eight independent required-root
removal REDs followed by exact restored four-test GREEN; final composed receipt
above. Permanent behavioral regression selection is in the mandatory gate.

Accepted disposition is FIX / PROTECTED for the approved eight-trigger ASCII
correction. Unicode equivalence and universal rule-language coverage explicitly
remain unresolved, independently of this completed scope. Other proxy/audit paths
and source-wide discovery are incomplete. Original syntax accounting remains9763
preserved/13 scoped/9750 pending plus96 callback supplements; finite behavioral
cases are not syntax dispositions or whole-system acceptance.
