# AF-PERF-002: one bounded integrated source-form discrepancy investigation

Agent: Agent MaxHeadRoom
Status: investigation COMPLETE; minimal source form and residual tradeoff owner APPROVED.
Historical comparison below preserves its original source identities and limitations.
Current production acceptance is tracked separately in the overload repair report.

## Fixed investigation scope

Owner authorizes one controlled comparison of source-equivalent selection/helper
forms, not a new matching algorithm. Current integrated signature-range bounds,
original occurrence order, used flags, full equality, diagnostic locations and
returns remain fixed. No cardinality, container, cache, sorting or payload filter
changes. Previously protected corrections remain closed.

Production identities preserved when authoring:

- differ.rs: c8d5e698127c2690ca03ee36bc7cc8fb18eb32c3d9005b7f5393984e6b02e016
- overload_lookup.rs: 3f758c1f297528bc09abbb88669b7e357048da63970be7d5937ca7206fdcafd9

All new helpers/comparators live under src/tests/diff. Existing production functions
do not delegate to them. Only the existing test-only alternatives module registers
the new tests. This compares source-equivalent TEST copies, not an edited active
production variant. An exact source-control copy distinguishes source/module-layout
effects from the intended substitutions. No isolated production mutation is needed
to compile these test-only copies; actual active bytes are hash-guarded throughout.

## Finite candidate matrix

Complete comparators copy active production with only imports adapted and test
registrations removed. Four helper copies differ only as follows:

| Form | Same signature-range selection | Helper attributes |
| --- | --- | --- |
| Identical control | iter().copied().find | unchanged |
| Find then copied | iter().find(...).copied() | unchanged |
| Inline wrappers | current iterator | inline(always) on signature_cmp, method_equal, signature_equal |
| Combined | find then copied | same three attributes |

These are the entire authorized experiment set. Inline attributes explicitly test
function-boundary effects; no causal benefit is assumed. The direct-loop reference
is the existing approved test refinement, retained unchanged. A standalone direct
loop in the production helper previously failed manual_find; no Clippy suppression
or weaker lint gate is introduced to manufacture that production candidate.

One tracked identity test verifies each complete copied comparator against active
production and each helper against its exact declared substitution. It normalizes
whitespace and top-level import order only (rustfmt may reorder adapted imports),
never control-flow order. Thus accidental range/predicate/observer changes cannot
masquerade as source-form improvements. Clippy must accept all candidate files.

## Semantics and identical-work controls

Bounded sequences through length two over the existing nine method variants run
through original, approved direct-loop refinement, active production and all four
copies for both class/orphan paths. All fourteen larger fixtures are also checked.
Selected occurrence traces, complete actions/reasons/previous details, summaries
and all five fidelity renderings must equal the frozen original. Every source copy
must additionally have an IDENTICAL full OverloadWork observation to active
production, including visits, whole-method equality calls, signature/payload/sequence
observations and offered lengths, capacities, flags and pair traces.

Forty-five frozen public-oracle projections verify every form's exact actions,
indices, full response projection, source hash and snapshot identity. These are
test-copy projections, not new public production integration or edited source-cache
behavior. Existing actual production public GREEN remains its separate receipt.

The original reference is the same field-observed original, independently checked
against frozen original semantics. Shared equality/string observers are preserved.
This cannot equate diagnostic work with machine cost: it separates probe counts and
placement from the machine-code effects under investigation. No actual inspected
payload byte count or allocator-event claim is introduced.

## Paired execution and decision standard

All fourteen existing fixtures remain unchanged: singleton/narrow/wide, exact/body,
duplicates/same-signature, reordered/sparse, long common body and 512 singleton
groups. Each complete timed operation includes grouping, setup, selection, action
construction and teardown. Common snapshot preparation and fixed test-design
selection are outside timing. Observation updates are disabled while all diagnostic
call sites remain. Enabled observations are retained separately.

The same run rotates seven alternatives: observed original, approved loop refinement,
active production, identical copy, iterator substitution, inlined wrappers and
combined. Two FIXED blocks each retain 24 paired samples after three warmup rounds;
the second block offsets rotation. This is one bounded packet to inspect within-run
repeatability, not open-ended tuning or repeated confirmation runs. Sorted sample
vectors and medians are logged for each block. No statistics/release/public-latency
conclusion follows from debug comparator medians.

If the identical-source control itself diverges materially from active production,
module/code-layout effects limit causal attribution. A candidate must show the
long-duplicate and narrow improvements across both fixed blocks and preserve wide
gains; fewer work counts cannot be an explanation because work equality is required.
If none does, return explicit acceptance/rejection of the measured tradeoff. Do not
extend the matrix or silently try more inline attributes/iterator combinations.

## User-owned execution

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-overload-source-forms.ps1
```

Expected four tracked investigation tests: source identity, semantic/work equality,
45-row public-oracle projection and fourteen paired cost rows (two blocks each).
Wrapper runs warning-fatal all-target/all-feature Clippy, formatting of TEST files
only, size/encoding and source receipts; active production hashes must remain
identical before/after. All UNRUN. Agent performed bounded source, syntax, UTF-8
and diff review only, no tests/build/formatter/lint/runtime.

Return exact executed source identities/costs, functional/work controls and one
recommendation. If a Clippy-compliant source substitution resolves the discrepancy,
propose only that substitution for owner production approval. Otherwise return
for explicit acceptance/rejection, without closure. Original RED, frozen oracles,
all earlier FIFO/refinement/integration evidence stay preserved. No mutation or
composed gate runs during this unresolved cost decision. Census stays 13/9763
scoped, 9750 unresolved plus 96 supplements; dense same-signature equality separate.

## First execution rejected by import-format identity control

overload-source-forms-20261107-224838 runs three tests GREEN but fails the fourth,
source identity. The first mismatch is comparator control's adapted keys import:
rustfmt wraps it onto multiple lines and adds an optional trailing comma before
the use-list closing brace. Expected single-line production imports lack that comma.
The complete canonical strings differ by exactly that one character; normalizing
this optional import comma makes them identical. This is a guard-authoring formatting
error, not semantic failure, successful acceptance or performance RED.

Corrected normalization applies only to already-isolated import statements:
remove the optional comma immediately before their closing brace/semicolon. No
function-body punctuation, statement order, matching predicate or diagnostic site
is normalized away. Production/helper/comparator candidate bytes are unchanged.
The failed run's logs remain frozen; the other three passing tests do not override
the failed required guard or constitute a complete receipt.

Rerun the same source-forms command to obtain all four tracked GREEN and the
complete source/hash/encoding receipt. Agent performed bounded log/text/diff review
only; no test/build/Clippy/formatter rerun started. This correction does not extend
the fixed source candidate matrix or authorize production tuning/closure gates.

## Executed same-run source-form comparison

Agent MaxHeadRoom inspected overload-source-forms-20261107-225340: all four
tracked tests pass in 18.17s, zero failed/ignored. Source-substitution identity,
bounded semantic/complete-work equality, all 45 frozen public-oracle projections
and fourteen fixtures/two fixed cost blocks pass. Warning-fatal all-target/all-
feature Clippy passes 26.51s, format logs empty, size passes 37 active files and
strict encoding passes 2209 text files. All 1358 recorded hashes match at inspection
before this documentation update. Receipt confirms active production unchanged;
inspection.json preserves full blocks, work observations and exact source identities.
Failed earlier formatting-guard run remains separate and is not acceptance.

Microseconds below show block zero / block one. Loop is the approved direct-loop
test refinement; control is an identical-source helper/comparator copy. All seven
forms have complete retained samples in the receipt; this table shows the minimum
substitution and its controls rather than omitting the original reference.

| N / width / mode / payload repetitions | Observed original | Approved loop | Active production | Identical copy | Find then copied |
| --- | --- | --- | --- | --- | --- |
| 0 / 1 / exact / 0 | 1.9 / 1.9 | 2.0 / 2.0 | 2.0 / 2.0 | 2.0 / 2.0 | 2.0 / 2.0 |
| 1 / 1 / exact / 0 | 10.8 / 11.6 | 11.3 / 12.0 | 11.2 / 12.1 | 10.9 / 11.8 | 11.1 / 12.1 |
| 8 / 8 / body / 0 | 37.6 / 67.7 | 40.5 / 72.9 | 41.1 / 71.6 | 40.9 / 69.3 | 40.6 / 71.9 |
| 128 / 1 / body / 0 | 1163.8 / 1278.7 | 1154.0 / 1179.6 | 1150.8 / 1365.6 | 1174.6 / 1251.9 | 1138.5 / 1184.5 |
| 128 / 8 / body / 0 | 493.4 / 493.5 | 534.3 / 532.9 | 546.8 / 547.3 | 541.7 / 543.8 | 539.1 / 531.9 |
| 128 / 128 / body / 0 | 1786.4 / 1772.7 | 485.2 / 478.9 | 504.3 / 499.3 | 506.7 / 502.6 | 500.8 / 493.1 |
| 128 / 128 / exact / 0 | 460.5 / 523.0 | 344.9 / 405.2 | 357.5 / 416.4 | 361.8 / 420.7 | 356.4 / 419.3 |
| 128 / 128 / duplicates / 0 | 457.3 / 509.0 | 422.7 / 463.6 | 542.8 / 598.5 | 542.2 / 586.8 | 434.9 / 479.0 |
| 128 / 128 / same-signature / 0 | 3433.6 / 3426.0 | 3032.6 / 3044.5 | 3326.0 / 3566.7 | 3469.3 / 3618.3 | 3093.3 / 3147.7 |
| 128 / 128 / reordered / 0 | 1595.5 / 929.4 | 824.3 / 463.2 | 853.3 / 462.0 | 933.5 / 467.3 | 808.7 / 451.5 |
| 128 / 128 / sparse / 0 | 2509.5 / 2730.8 | 536.7 / 567.2 | 527.6 / 561.9 | 500.4 / 592.8 | 484.6 / 543.0 |
| 64 / 64 / duplicates / 4096 | 479.0 / 309.5 | 494.0 / 358.4 | 523.9 / 379.1 | 520.2 / 371.8 | 439.5 / 364.7 |
| 64 / 64 / same-signature / 4096 | 1747.1 / 1696.6 | 1653.8 / 1533.3 | 1800.0 / 1633.3 | 1842.0 / 1729.7 | 1655.4 / 1578.1 |
| 512 / 1 / exact / 0 | 2647.4 / 2781.9 | 2831.8 / 2863.3 | 2802.2 / 2717.3 | 2795.9 / 2650.5 | 2710.0 / 2659.8 |

### Interpretation, repeatability and limits

Find-then-copied has favorable long-duplicate median direction in both blocks:
523.9 -> 439.5us (16.1%) and 379.1 -> 364.7us (3.8%). Width-eight improves
546.8 -> 539.1us (1.4%) and 547.3 -> 531.9us (2.8%). Wide body changes retain
their large gain over original and improve modestly versus active production in
both blocks. Short duplicate/same-signature medians also improve in both blocks.
The small eight-body fixture differs by +0.3us in block one, not a demonstrated
consequential new regression. Exact-wide is essentially unchanged in block one.

This is repeated descriptive median direction, not a statistically established
causal effect. Width-eight candidate/active index-6..18 ranges overlap in both
blocks: candidate 529.3..612.5us versus active 536.9..819.2us, and candidate
530.7..563.3us versus active 538.9..607.6us. Long-duplicate ranges overlap too.
There are broad block-level shifts affecting references and variants together.

Identical-source control is close on width-eight/long duplicates, but source/module
layout can itself matter: e.g. reordered block-zero control is 933.5us versus
active 853.3us, and singleton-body block-one control 1251.9us versus active
1365.6us. Therefore do not assert that the iterator adapter alone explains the
entire original integrated discrepancy or that helper boundaries are proved causal.
Enabled work is exactly identical for all four source copies and active production.
This rules out observed matching-work differences, not codegen/layout/scheduler or
residual diagnostic costs. No machine-code/CPU-instruction proof was collected.

Residual cost has not disappeared. Width-eight candidate remains about 9.3% and
7.8% above original in these blocks, close to the same-run approved loop (8.3%
and 8.0%). Long duplicates are below original in block zero but about 17.8% above
original in block one, while the approved loop itself is about 15.8% above original
there. Candidate is about 1.8% above that loop in block one. This supports a small
source-form preference relative to the approved reference; it does NOT prove the
earlier absolute 6.2% cost boundary universally restored or eliminate owner review.

Inlining all three wrappers adds broader effects without a sufficiently consistent
advantage to justify attributes. Inline-only long duplicates are 486.9/408.5us;
combined 420.6/354.2us. Combined is best there, but exact-wide block one rises
to 470.7us versus active 416.4us and find-then-copied 419.3us. Inline-only also
regresses singleton-body block zero. Recommend no inline attributes: their extra
scope is not needed for the simpler candidate's favorable median pattern.

The direct-loop reference remains a test-only comparison; standalone production
manual-find lint is not bypassed. Dense same-signature full equality remains the
same work in every source form. No payload hashing, ordering or rejection-filter
claim is introduced, and no release/public latency or allocation benefit is claimed.

## Returned recommendation and exact proposed substitution

FIX PROPOSED for owner review: in the EXISTING signature slice only, change:

```rust
index.indices[a..b].iter().copied().find(|&i| {
    // Same selection observer and predicate.
    !used[i] && method_equal(self.current[i], base)
})
```

to:

```rust
index.indices[a..b].iter().find(|&&i| {
    // Same selection observer and predicate.
    !used[i] && method_equal(self.current[i], base)
}).copied()
```

No helper inlining, other call rearrangement or algorithm change is recommended.
All range/order/flags/equality/return and probe boundaries are preserved. Candidate
tests are Clippy GREEN, but its active production substitution has NOT been applied.
Source identities are frozen in the receipt for every helper/comparator copy and
the untouched active files. The exact-copy/work guards make this a bounded
source-level proposal rather than an inferred matching-policy improvement.

The one authorized investigation is complete. Owner should either approve this
minimal substitution WITH the disclosed residual/noisy cost profile for integrated
acceptance, or explicitly accept/reject the current production tradeoff. Do not
continue trying source forms or request more timing merely to obtain favorable
samples. If approved, require exact current-byte semantics/public/static protection
and remaining independent real scan mutation/restoration and composed gates before
closure. No mutation/composed execution, production edits, stage/commit/push in
this return. This remains not FIX / PROTECTED; all prior repairs stay closed.

## Owner disposition after the completed investigation

Owner approves only find-then-copied and explicitly accepts the disclosed residual
debug-cost tradeoff. The investigation is finished; no additional source forms or
timing searches follow. Active production is now allowed to differ from its old
control. The identity test therefore uses exact tracked pre-substitution helper/
comparator text fixtures to preserve the historical experiment baseline. Current
production static/public/mutation/composed acceptance is recorded in
[repair](AF_PERF_002_OVERLOAD_REPAIR.md) and cannot be inferred from this report.

```architecture-graph
node: DOC-182
kind: RESEARCH
status: INVESTIGATING
title: Owner-approved bounded overload source-form tradeoff
track: REPOSITORY
artifact: docs/agent/AF_PERF_002_OVERLOAD_SOURCE_FORMS.md
note: Source-attributed bounded subject; historical measurements and execution standing are not current-executable certification.
knowledge_scope: evidence
knowledge_content:
  - statement self#whole
```
