# ANG-R2 Phase A incremental performance disposition

Agent: **Agent MaxHeadRoom**. Design obligations; no measurement or optimization PASS.

## Standing and scope

Acquired PERF-004-DOCTRINE and PERF-005-BOUNDED with applicable qualifiers/paths.
Correctness, currentness and complete accepted support membership are mandatory.
New semantic representation work needs bounded deterministic coverage; optimization
needs demonstrated enclosing benefit and proportional complexity. No token-savings
criterion, witness cache, lock change, global index repair or P0 re-investigation.

Accepted generation-78 P0 projection batching and sibling/delegated lazy slots
remain the baseline. ANG-R1's seven scoped passes remain their exact prior evidence;
they are not new ANG-R2 or complete performance results. The proposed shared resolver
projection must preserve the existing per-admission complete-set bound while adding
registration output, not replace it with per-key reconstruction.

## Proposed operation and work units

Let F be nominated source owners, C owners needing compilation, M retained mapping
occurrences, R admitted registrations, T distinct route/target memberships and B
returned records. R can exceed T; preserving that information is necessary work.
Count known omission records and their serialized provenance separately as well.
These are independent dimensions, not a claim that the entire existing projector
is linear: current field duplicate checks and retained proof joins already scan.

| Boundary | Required accounting / bound for the new work |
| --- | --- |
| Producer | One shared resolver projection traversal per applicable Angular combined evaluation. Each existing candidate mapping is proved once at that operation's existing proof boundary, then contributes registration plus membership dedup. No second capture or registration-only parse. |
| Admission | One operation-local resolver projection shared by membership and full registration-collection validation. New records must not cause one complete projection per record or validation of the full collection per edge. No reuse across mutation/publication/later calls. |
| Payload | Report actual R/T and serialized registration bytes, including repeated route/declaration/provenance values. Do not call logical payload bytes allocator savings. |
| Source-owned index | Retain records in existing SemanticCoverage owner entry. Whole-source replacement/retirement removes that owner's collection; no maintained inverse index or new coordination state. Count record transport/copies separately from unrelated existing rebuild work. |
| Query acquisition | One query-operation nomination/preparation using existing scoped source inventory; distinguish F, current owners and C. Existing current selection can avoid recompilation; an empty cache does not prove no registrations. |
| Forward/reverse filtering | Inspect each prepared candidate owner's admitted records once for that evaluation, then emit B matches. Do not reconstruct every owner's complete mapping authority per candidate edge or returned row. |
| Query sorting | O(B log B) deterministic output order for this new operation; retain necessary identity and multiplicity. |
| Persistence/replay | Count logical bytes, serialization/decode and projection/admission calls separately. The same required witness checks remain; no assumption that duplicated paths are equivalent observations. |
| Batch preparation | Preserve existing final common generation and request-local sharing. Different direction/selector fields are not equivalent query keys. Repreparation after relevant publication is not falsely counted as needless stable-operation duplication. |

The conservative nomination cost is a material tradeoff of a correct cold reverse
query over aliases. It is not a universal free operation. Owner acceptance of that
query-specific policy and focused cold/warm/incomplete acquisition observations are
required; no new global nomination rule is proposed. If consequential costs appear,
return the exact measurements and smallest justified alternative before optimizing.

## Required focused evidence after contract approval

Extend existing test-only resolver/admission counters where sufficient. Grow keys
on one target independently of route and target counts; also grow independent
routes, same-name targets and nominated source owners. Use 1/2/4/8/16 and an agreed
larger bounded point when needed to expose the mechanism. Assert the work invariant
at the actual producer/admission/query boundary, not a mirrored helper mock.

Positive/refusal/currentness assertions must remain independent of counts. Require
exact registrations with two keys/one target while old membership stays one; stale,
forged, missing and contradictory proof must still refuse. Count complete projection
construction once per stable operation; preserve lazy zero work for unrelated paths
where that remains the existing behavior. Keep cold compilation, warm payload reads,
SQLite restoration, query filtering and provider/transport latency separate.

Use the existing mandatory AF-PERF-001 finite gate for changed semantic paths and
its source/static coverage review. Retain all established required tests; no blanket
legacy exception, ignored required test or allow-failure substitution. Obtain exact
operator authorization before each required gate invocation. Do not start a full
repository gate locally.

Record construction, serialization, maintenance, retirement and complete query costs;
do not declare acceptance from lower counters alone. This is a new capability,
not a speculative optimization claim or a requirement to eliminate all historical
witness/snapshot/index costs before key registration.

## Disposition

**DESIGN REVIEW pending owner contract acceptance and actual incremental evidence.**
No new failing P0 invariant was observed or asserted. Performance acceptance for
the proposed representation is UNRUN. A failing applicable established gate would
block Phase B acceptance; an unrelated optimization opportunity does not.

## Current implementation work scope and pending measurements

This records the implemented work, not performance acceptance or a new optimization.

- One resolver production traversal produces registrations and membership before
  target deduplication. Added owned record/diagnostic data is O(J+D) for J admitted
  mappings and D detected refusals. Existing native mapping proof and P0 projection
  reuse remain; no per-registration full-set replay is added.
- One complete admission lazily shares the resolver result with membership checks.
  Record/omission sets and matching membership support are built once locally.
  They expire at the call boundary; no cache, store or lifecycle policy is changed.
- Decoded-context scope binding invokes the existing applicability predicate once
  for that owner with borrowed source/config/native context. It can include existing
  lexical/detection work; no zero-cost or no-additional-parse claim is made. It does
  not run Angular semantic evaluation or retain another complete source snapshot.
- The new query shares one configured-root nomination per preparation context,
  including mixed F/R items. With U required source owners, J inspected registration
  records and D diagnostics, final source/currentness checks are once per owner and
  record inspection is once per record, followed by deterministic sorting. Existing
  dependency-witness costs remain explicit; withinPath narrows answers, not work
  needed to acquire prerequisites. No maintained inverse index is introduced.
- Existing snapshot/artifact/index copies now include the collection. The controlled
  2/16/64-registration test records actual response bytes, the collection field's
  compact-JSON contribution in the real SQLite payload, and enclosing cold/warm
  durations. Owner-validation count must stay constant as J grows at fixed U;
  record count grows with J. These assertions are authored and currently UNRUN.

These are scoped deterministic checks and descriptive timing/size observations,
not allocator profiling, enterprise efficiency evidence or a causal before/after
performance improvement claim. Actual results must be inspected before acceptance.
The established AF-PERF-001 CI gate remains required. No P0 reinvestigation, blanket
witness/snapshot-lock/index optimization, exception or waived invariant is proposed.

## Observed controlled growth (2026-11-15)

Operator repair capture `registration-green-20261115-100307-289` passed the tracked controlled-growth test. At 2/16/64 registrations, owner validations remain 21 and nominations remain one; inspected records are respectively 2/16/64. Response bytes: 2353/15842/62114. Durable collection JSON bytes: 2008/15496/61768. Enclosing cold query seconds: 10.1780565/12.1058122/38.6528444; warm seconds: 2.4078558/2.4138884/2.8897333.

This verifies the asserted bounded owner/record work and records substantial existing enclosing costs. It does not establish an enterprise efficiency claim, causal optimization benefit, allocator cost, or complete production-operation coverage. No per-registration owner reconstruction was observed by these counters; accepted P0 batching is not reopened. Applicable AF-PERF-001 CI acceptance remains pending, not waived. Remaining optimization opportunities do not independently block public registration; an actual applicable failing invariant would.
