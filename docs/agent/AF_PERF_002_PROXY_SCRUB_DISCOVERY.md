# AF-PERF-002: proxy request pipeline and length-only scrub shortcut

Agent: Agent MaxHeadRoom
Status: owner-approved scrubber correctness repair FIX / PROTECTED.
Original RED,52 focused GREEN, isolated predicate RED/restored GREEN and final
composed55 GREEN verified. Historical authored statuses below are chronological evidence.
No server/network/real credential used.

## Real production path and operation boundaries

proxy/src/server.rs intercepts supported POST message endpoints, reads bounded body,
parses JSON, snapshots config/adapter under a read lock, builds/runs Pipeline without
the state lock, injects cache breakpoints where configured, commits statistics under
a write lock, optionally clones for blocking logging, serializes then forwards.
Other endpoints forward through their separate path. Request/response buffering,
rate limiting, serialization and logging remain independent unmeasured obligations.
No lock, size-limit, streaming or scheduling change is proposed.

Pipeline orders tool exclusion, ANSI stripping, Bash trimming, model override,
secret scrubbing, configured tool filtering and optional aging. Multiple whole-body
passes have different policies; they cannot be collapsed merely because they loop
over the same JSON. Logging's fail-closed scrub boundary is distinct from request
transforms and must be traced independently. This first trace does not certify it.

## Concrete optimization-risk counterexample

scrub::scrub_secrets applies regex replace_all, then accepts that returned content
and records hits only if new_result.len() != result.len(). Its comment identifies
length comparison as avoiding string equality and separate match passes. Different
strings can have equal lengths, so that shortcut is not a valid change detector.
The synthetic string password=abcdefghij and expected password=[REDACTED] are both
19 bytes; the assignment rule explicitly matches the first. The keyword prefilter
admits it. No actual secret is used or printed.

transform::scrub_secrets only updates a JSON field if returned hits are nonempty.
Therefore the length shortcut can also suppress mutation at the pipeline consumer.
This is a source-grounded predicted correctness failure, not an executed security
incident or completed RED. No claim about historical exposure or live deployments.

Two tracked tests under src/tests/proxy are loaded by proxy/src/lib.rs: direct scrub
requires exact replacement and one hit; actual Pipeline::build/run with scrubbing
enabled requires exact complete request JSON and one scrub statistic. These reach
the production transform but do not exercise HTTP transport, upstream or logging.
They do not weaken fail-closed policy or introduce a credential fixture.

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-proxy-equal-length-budget.ps1 -Phase Before
```

Expected two intended correctness failures, zero ignored. The driver selects the
proxy package with all features, preserves production/test bytes and hashes, and
requires both exact failure markers. Compile/fixture failures are not valid RED.
No agent Cargo/test/formatter/lint/runtime was started. Bounded UTF-8/size and wrapper
syntax review only. This packet is correctness regression evidence, not a performance
budget despite the wrapper's naming convention.

## Conditional next decision after evidence

If confirmed, prioritize the narrow scrub change detector correction before proxy
performance optimization. Preserve rule order, replacements, match/hit accounting,
prefilter and all existing consumers. Review regex replacement ownership semantics
or exact content comparison as bounded alternatives; do not replace matching policy,
merge scrub boundaries or introduce caching. Report semantic changes explicitly,
including equal-length and already-redacted matches, before owner-approved repair.

Classification: REVIEW REQUIRED correctness risk attributable to an invalid
performance shortcut. No production fix is authorized by this discovery return.
Proxy filtering/aging, platform shapes, fail-closed logging and independent growth
remain unmeasured. Source accounting9763 preserved/13 scoped/9750 pending plus96
callback supplements is unchanged. Traversal and C2 remain DESIGN REVIEW/MONITOR;
completed hierarchy, C1, NgRx and A3 scopes remain protected.


## Actual correctness RED verified; bounded correction proposed

proxy-equal-length-before-20261104-180252 executes two tracked regressions:
zero passes, two intended assertion failures, zero ignored,0.06s. Direct result
retains synthetic password=abcdefghij; complete pipeline JSON also retains it,
instead of expected password=[REDACTED]. Both fixture strings are19 bytes.
No compilation or fixture failure is counted as RED. All 30 receipt hashes
match at inspection; original production/test bytes and logs remain frozen.
No real credentials, HTTP traffic, server or field incident was involved.

Confirmed defect: length inequality is insufficient to detect content replacement.
Suggested narrow correction, requiring owner disposition: compare replacement
content with the original content, rather than their lengths, before counting hits
and committing it. Retain all existing regex rules, order, prefilter, replacement
strings, hit enumeration, stats and consumer gating. Exact equality also preserves
old no-change behavior for already-redacted matches; replacement-ownership alone
must not accidentally start reporting unchanged content as new hits.

This adds no regex pass or retained state; it may compare string bytes. A borrowed
no-match shortcut may be evaluated if its ownership contract is established, but
is not necessary to correctness and must not complicate this repair. Prioritize
restoring redaction over retaining an invalid length-only micro-optimization.

After approval, replace only the change detector and explanatory comment. Require
unchanged two-test GREEN; expand controls for shorter/equal/longer values, UTF-8,
already-redacted idempotence, multiple matches with cancelling total length changes,
rule ordering and full request JSON/statistics. Run existing proxy scrub/pipeline/
transform controls and affected warning-fatal lint/format/size/encoding. Prove isolated
length-shortcut restoration RED and exact restored GREEN. Do not modify oversized
transform/server modules or unrelated filtering/cache/transport behavior.

Disposition: confirmed correctness defect / FIX PROPOSED, not repaired/protected.
The performance audit has found an invalid shortcut, not evidence to weaken matching
policy for speed. Broader proxy coverage, fail-closed logging, prefilter equivalence
and independent work growth remain open. No broad rewrite or secret-policy redesign.


## Owner-approved exact comparison implemented; acceptance authored

Owner expressly authorizes the correctness repair, not a performance optimization.
Exact production diff in proxy/src/scrub.rs: replace
new_result.len() != result.len() with new_result.as_ref() != result.as_str(),
and correct the explanatory comment. No other production executable code changes.

Each rule's result is still the owned current input, including earlier rule output.
replace_all borrows that input; exact comparison occurs before captures_iter and
before result = new_result.into_owned(). Thus comparison uses the correct rule-local
snapshot, not the original pre-pipeline input or an already-mutated baseline.
Same-content replacement skips hit reporting exactly as before. Changed content
runs the same captures_iter on that same input, records the same ordered rule hits,
then transfers replacement ownership. No rule, prefilter, regex, replacement,
consumer gate, logging policy, API, cache or pipeline order is changed.

The original two tests are retained unchanged. Four added tracked controls cover
shorter/equal/longer replacement sizes, equal-byte UTF-8 content, no match, empty,
already-redacted matches, cancelling total length changes across two matches,
rule-ordered hit kinds and fail-closed wrapper result, exact complete pipeline JSON
across text/string/nested tool-result shapes, statistics and repeat idempotence.
Only synthetic values are used. Tests remain under src/tests/proxy and are loaded
by proxy/src/lib.rs; wrappers are evidence drivers, not substitutes for tests.

| Canonical question | Boundary / oracle | Current evidence |
| --- | --- | --- |
| Is it on the real production path? | server request pipeline -> transform -> scrub | source traced; actual Pipeline test original RED |
| Is the baseline current? | per-rule input before replacement ownership transfer | source proof; new behavioral controls UNRUN |
| Are rules/identity/order unchanged? | exact output bytes, hit types/order, unchanged matches | original controls retained; expanded tests UNRUN |
| Does output propagate? | exact complete request JSON and scrub statistics | original pipeline RED; repaired tests UNRUN |
| Are failure/lifecycle contracts retained? | fail-closed wrapper and repeat idempotence | authored controls; no injected panic or HTTP field claim |
| Is regression protected permanently? | tracked tests selected in composed fitness gate | authored; isolated real predicate mutation UNRUN |

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-proxy-scrub-focused.ps1
if ($LASTEXITCODE -eq 0) {
    pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-proxy-scrub-mutation.ps1
}
```

Focused expects52 tracked passes: six original/extended regression tests plus24
existing scrub, five pipeline and17 transform controls. Includes proxy-package
all-target/all-feature warning-fatal Clippy, scoped format/format-check, active-file
size and strict encoding gates with source hashes. It does not start a server or
call an upstream. Existing test code is not relocated or rewritten.

Mutation creates an isolated current-source clone, requires all six regressions
GREEN, restores only the old length predicate, requires both original cases RED,
restores exact bytes and requires all six GREEN. Active source hashes must remain
unchanged. Permanent protection is provided by behavioral tests and mandatory gate
composition; no additional generic static registry or framework is introduced.

Both packets are UNRUN. Agent performed bounded script parsing, source-diff review,
UTF-8/no-BOM and line-ceiling checks only. The composed gate now also selects these
six proxy-package regressions; final composed receipt remains pending after focused
and mutation verification. Do not label FIX / PROTECTED until execution confirms it.


## Focused semantic and isolated predicate sensitivity verified

proxy-scrub-focused-20261104-181341 passes52 tracked tests: six direct/pipeline/
byte/order/idempotence regressions0.08s,24 existing scrub controls0.07s, five pipeline
controls0.00s,17 transform controls0.03s. Zero failed/ignored. Proxy all-feature/
all-target warning-fatal Clippy49.09s, empty format-check log, size13 active files
and strict encoding2104 files pass. All 30 recorded source hashes match
at inspection before documentation update.

proxy-scrub-mutation-20261104-181501 establishes clean six-case GREEN0.07s, changes
only the production comparison back to length inequality, and observes both original
equal-length assertions fail0.05s. Exact source restoration returns all six GREEN
0.03s. No compile/fixture errors substitute for mutation RED. Active scrub bytes equal
restored isolated bytes; all 2111 inventory hashes match at inspection.
inspection.json records identities. No production/test source edits follow these runs.

The expanded tests confirm exact changed bytes for short/equal/long/UTF-8 values,
unchanged text and already-redacted hit behavior, cancellation across replacements,
ordered rule hits, fail-closed wrapper output, exact JSON and hit propagation across
three supported request text shapes, and zero new hits on repeat. This does not
certify all platforms, HTTP transport, live exposure or injected panic behavior.

Narrow correction has executed semantic GREEN and permanent behavioral/mutation
protection. Final mandatory composed gate remains pending before FIX / PROTECTED
acceptance. No new production API, rule, scope, cache or matching engine introduced.

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-semantic-work-fitness.ps1
```

Expected at least55 tracked passes: prior49 plus six proxy-package regression tests.
Original two-case RED stays frozen separately. Agent inspected evidence only, with
no test/build/lint/runtime started. Broader proxy/audit coverage remains incomplete.


## Final correctness acceptance: FIX / PROTECTED

Final composed af-perf-001-20261104-193130 verifies 55 tracked passes across
12 selected groups, zero failed/ignored, including six proxy-package regressions.
All 219 receipt source hashes match at inspection; inspection.json retains the
per-group counts. No production/test changes follow this verification. Agent only
inspected files and evidence; no runtime was agent-started.

Accepted scope: the one rule-local exact-content predicate in proxy/src/scrub.rs,
plus its explanatory comment. new_result.as_ref() != result.as_str() compares the
replacement against the current pre-mutation input before hit enumeration and
ownership transfer. Rules, precedence, replacements, prefilter, unchanged-content
behavior, hit reporting and pipeline consumers remain intact.

Evidence chain: original proxy-equal-length-before-20261104-180252 two-case RED;
proxy-scrub-focused-20261104-18134152 semantic/existing control passes with clean
proxy lint/format/size/encoding; proxy-scrub-mutation-20261104-181501 clean baseline,
real old-predicate two-case RED and exact restored six-case GREEN; final composed
receipt above. Permanent tracked tests are selected by the mandatory gate. Generated
receipts are evidence, not replacement regression assets.

No broad scrubber redesign, performance speedup, live incident attribution, HTTP
field certification or whole-proxy acceptance is claimed. Other proxy behavior,
prefilter completeness, logging fail-closed lifecycle and architectural performance
discovery remain separate. Original syntax accounting remains unchanged. Protected
hierarchy, C1, NgRx and A3 repairs are not reopened.
