# AF-PERF-002: bounded signature-array refinement investigation

Agent: Agent MaxHeadRoom
Status: executed refinement GREEN; bounded production FIX PROPOSED for owner review.
Residual narrow-group cost and timing variation remain explicit; not FIX / PROTECTED.
FIFO remains unaccepted. All original/FIFO/alternative receipts remain frozen.

## Exact authorized changes

The baseline test-only signature array is preserved as Design::Partition. New
Design::Capacity constructs its index Vec with capacity equal to current group
length, extends directly with original indices and retains the SAME signature/
index sorting, ranges, exact matching and signature cursors. It removes the old
filtered collection for this all-current setup; no filtering is needed before the
global exact phase. The cursor Vec retains one slot per input index as before.
Do not interpret capacity equality as a measured allocator improvement.

Design::Refined adds only the authorized structural one-current-occurrence case.
When current.len() == 1, no retrieval index/cursor vectors are constructed. Exact
queries check the original used flag and full equality against current index zero.
The caller still completes every baseline exact query before finish_exact and the
signature loop. Signature queries then check the same used flag and complete visible
signature equality at index zero. The helper does not consume flags itself: original
comparator code marks them at the existing action emission sites.

Thus a later exact baseline occurrence can reserve the sole current occurrence
before an earlier unmatched baseline reaches signature selection. Multiple baseline
duplicates can consume current zero only once. Sole/ambiguous remainder behavior
and output order stay in the unchanged test comparator adapter. Empty or one-sided
groups take original comparator arms, outside retrieval setup. This is cardinality
specialization explicitly authorized by the owner, not a chosen performance threshold.
For groups larger than one, Refined uses Capacity without another fallback.

No production source matcher/helper algorithm changes, hashing redesign, identity
normalization, rejection filter, persistent cache or adaptive selection is made.
Dense same-signature repeated full equality remains unresolved and is not disguised
as discrimination work removed by this refinement.

## Semantic and component isolation

Existing bounded exhaustive semantic helpers are shared without changing the frozen
original matcher/oracle or public budget. New refinement tests run Capacity and
Refined against original occurrence traces, complete ordered actions/reasons/previous
details, summaries and all five fidelity outputs for class and orphan paths.
Additional one-current adversarial cases explicitly require a later baseline exact
match to reserve current zero and cover duplicate baselines and failed equality.
Both candidates compare all 45 frozen original public-response/cache projections.
This remains test-only public-oracle consistency, not new production reachability.

Costs compare five fixed designs, independently:

1. The same field-observed original reference.
2. Rejected production FIFO comparator, only as a cost reference.
3. Existing signature-partition arrays.
4. Exact-capacity arrays alone.
5. Exact capacity plus one-current structural specialization.

All fourteen original fixtures and source generation are reused unchanged. This
isolates capacity effects from specialization rather than attributing both to one
aggregate outcome. There is no production winner or automatic design dispatch.

## Observations and timing

Complete operations include grouping, construction, queries, action construction
and teardown. Input preparation and fixed test-design selection are outside timing.
Three warmup rounds precede 24 rotating-order samples per fixture/design. Sorted
sample vectors are emitted as well as medians so the receipt preserves spread;
execution order can be reconstructed from the fixed rotation but is not encoded
in those sorted vectors. No confidence or public/release latency claim follows.

Enabled observations separately report original predicate visits/equalities,
signature discrimination string comparisons/offered lengths, actual whole-method
equality calls, payload field/sequence comparisons/offered lengths, selection/used-
flag visits and retained index/cursor capacities. Actual inspected string bytes
remain unknown; std equality can reject lengths or use optimized operations. The
original and candidates share the detailed equality observer and retain their
diagnostic sites with updates disabled during timing. FIFO legacy category zeros
remain unavailable attribution, not zero equality work.

Capacity arrays report actual Vec capacities; single-current retrieval should show
zero retained index/cursor slots. Fixed inline lookup/header sizes are still reported
by the shared layout row. Container headers, capacity bytes and actual allocator
events are different quantities. This packet measures neither allocation calls nor
RSS, so it cannot claim allocation reductions. Cumulative capacities across sequential
groups are not peak live memory. Equality costs inside dense same-signature ranges
must be reported even if signature construction becomes cheaper.

## User-owned execution

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-overload-refinement.ps1
```

Expected three tracked investigation passes: refined semantic controls, frozen
45-case public-oracle consistency and fourteen paired cost rows plus one layout
row. Then warning-fatal all-target/all-feature Clippy, format/size/encoding and
exact source receipts. All are UNRUN. Agent performed bounded source/text/UTF-8
and PowerShell syntax review only; no tests/builds/formatter/lint/runtime started.

Return executed comparison before seeking production authorization. Success must
reduce common singleton/narrow overhead while retaining wide benefits and avoiding
consequential long-payload regressions. If that fails, return measured tradeoffs;
do not introduce additional heuristics. If it succeeds, propose the smallest bounded
production change with preserved original RED/oracle/public-budget identity, exact
GREEN, finite static/phase protection, independent real scan-restoration mutation
RED/semantic GREEN/restored GREEN, focused checks and mandatory composed fitness.
No remaining closure gates are run on the rejected FIFO. No stage/commit/push.
All protected repairs stay closed; source census remains 13/9763 scoped, 9750
unresolved plus 96 callback supplements.

## Executed refinement comparison

Agent MaxHeadRoom inspected overload-refinement-20261107-155336: three tracked
tests pass in 11.14s, zero failed/ignored. Both component and combined candidates
pass original selected-pair/action/all-fidelity semantics, one-current exact
reservation controls and all 45 frozen public-oracle cases. Fourteen cost rows,
retained sample arrays and layout row are complete. Warning-fatal all-target/all-
feature Clippy passes 32.17s, format logs empty, size passes 20 active files and
strict encoding passes 2192 text files. All 1345 source hashes match at inspection.
inspection.json preserves complete parsed rows. No production algorithm changes
or agent runtime follow execution.

All times below are microseconds. These are same-run descriptive medians of 24
rotating debug comparator samples with symmetric disabled diagnostic sites, not
public/release latency or statistical certification. Original reference remains
the field-observed adapter independently checked against the frozen original.

| N / width / mode / payload repetitions | Original | FIFO | Prior arrays | Capacity only | Combined refinement |
| --- | ---: | ---: | ---: | ---: | ---: |
| 0 / 1 / exact / 0 | 1.6 | 1.6 | 1.6 | 1.5 | 1.5 |
| 1 / 1 / exact / 0 | 9.5 | 12.2 | 11.1 | 9.7 | 8.2 |
| 8 / 8 / body / 0 | 37.0 | 51.3 | 41.6 | 37.0 | 36.0 |
| 128 / 1 / body / 0 | 657.9 | 1028.7 | 831.6 | 806.9 | 667.3 |
| 128 / 8 / body / 0 | 412.2 | 586.7 | 463.9 | 438.8 | 437.8 |
| 128 / 128 / body / 0 | 1908.4 | 833.3 | 539.1 | 527.3 | 524.3 |
| 128 / 128 / exact / 0 | 623.8 | 706.1 | 539.0 | 626.9 | 608.5 |
| 128 / 128 / duplicates / 0 | 486.8 | 309.9 | 695.6 | 694.8 | 445.2 |
| 128 / 128 / same-signature / 0 | 4127.3 | 913.8 | 3520.9 | 4745.5 | 3286.1 |
| 128 / 128 / reordered / 0 | 1313.1 | 756.2 | 668.1 | 646.0 | 651.2 |
| 128 / 128 / sparse / 0 | 2479.4 | 671.5 | 454.1 | 442.5 | 440.9 |
| 64 / 64 / duplicates / 4096 | 540.5 | 548.5 | 522.5 | 520.0 | 524.9 |
| 64 / 64 / same-signature / 4096 | 1638.5 | 1910.4 | 1592.9 | 1666.0 | 1539.2 |
| 512 / 1 / exact / 0 | 4381.7 | 3899.0 | 3973.4 | 3876.2 | 4273.7 |

### What the refinement establishes and does not

At one occurrence the combined path is 8.2us versus prior arrays 11.1us and
original 9.5us. At 128 singleton body groups it falls 831.6 -> 667.3us, close to
original 657.9us (about 1.4% residual). Exact-capacity alone reaches 806.9us:
the specialization, not just retained capacity, removes most construction cost.

The width-eight population improves 463.9 -> 437.8us, but remains 25.6us / about
6.2% slower than original. Its central samples are separated: original order
statistics at indices 6/12/18 are 409.2/412.2/417.6us versus refined
433.9/437.8/444.9us. This is a real residual in the bounded descriptive samples,
not something to erase as generic noise. Additional signature discrimination
remains: 2314 signature comparisons versus original 1152, while full equality
calls fall 1024 -> 128. The owner must weigh this residual before production
authorization; the report does not unilaterally declare it inconsequential.

Wide all-body and sparse cases retain strong gains, roughly 72.5% and 82.2% lower
medians than original. Long-body duplicate and same-signature cases show no median
regression here (about 2.9% and 6.1% lower). Dense same-signature full equality is
still repeated; those costs have not been eliminated by the refinement.

The 512-singleton fixture is noisy: original index-6/12/18 samples are
2817.1/4381.7/5197.9us; refined 2868.7/4273.7/6018.6us. Its median is slightly
lower, but upper samples are higher. Do not claim a robust speedup or uniform
nonregression from this distribution. Duplicate fixtures also show broad spread.
Residual instrumentation and scheduler variation limit conclusions to this run.

### Work, equality and retained capacity

At one occurrence prior arrays retain five index/cursor capacity slots, capacity-
only two, combined zero. Across 128 singleton groups the corresponding cumulative
slots are 640 -> 256 -> 0; across 512 groups 2560 -> 1024 -> 0. These are logical
retained capacity reductions across sequential operations, not allocator-event or
peak/RSS measurements. Inline lookup state remains 120 bytes and signature-index
header 48 bytes on this target; zero array capacity does not mean zero state.

At 128 singleton body groups all designs perform 128 whole-method equality calls
and compare 676 offered payload bytes. Prior/capacity arrays make 640 signature
comparisons; combined and original make 256. At width eight, payload comparison
count/length stays 128 / 676 bytes while full equality calls fall from original
1024 to 128: the original rejects most unrelated signatures before body comparison.
This demonstrates the distinction between possible-match discrimination and payload
equality; a lower full-equality call count alone does not guarantee faster execution.
Offered lengths remain distinct from actual inspected bytes.

## Bounded production proposal for owner review

Recommend the combined refinement as the smallest candidate for review, with explicit
acceptance of the width-eight residual and noisy singleton distribution required.
It materially reduces the earlier narrow/singleton overhead and preserves wide and
long-payload behavior in this run, but does not prove universal cost nonregression.
No further heuristic or container is introduced. If the owner requires eliminating
the residual before authorization, disposition remains TRADEOFF REVIEW under the
current investigation bounds; do not extend the algorithm autonomously.

Proposed production delta, NOT IMPLEMENTED: replace rejected full-key FIFO retrieval
with one private local signature-index/cursor representation for groups with more
than one current occurrence, exact-capacity setup and the explicit one-current
structural case. Borrow existing groups and payloads. Reuse the same ordered
signature ranges through both global phases. Exact selection retains full equality
over unused candidates within that signature range; signature selection uses its
first-unused cursor only AFTER all baseline exact matching. Keep existing used
flags, action sites, sole/ambiguous remainder logic and remove/add order. Do not
rebuild name groups or change producers, caches, acquisition or formatting.

Acceptance after owner authorization:

1. Preserve genuine original overload-before-20261107-142854 RED and all frozen
   original/oracle/public-budget bytes. Do not relabel FIFO cost/semantic GREEN.
2. Integrate only the approved signature/one-current retrieval and require unchanged
   original budget GREEN, exact selected traces/actions/fidelities, actual public
   response/cache GREEN and source identities. Test-only projection consistency
   does not substitute for executing the integrated public path.
3. Add finite original-scan/global-phase/structural-case/full-equality protection.
   Independently restore each REAL original scan in isolated production, require
   intended budget/static RED while frozen semantics stay GREEN, then byte-exact
   restored GREEN. Dense same-signature original predicates cannot be prohibited
   by pretending the authorized partition scan is absent: protect unrelated-group
   searches and measure remaining within-partition selection work honestly.
4. Reconcile construction-inclusive work and focused semantic/public controls for
   the actual integrated bytes; warning-fatal Clippy, format/size/encoding and
   mandatory composed fitness/source receipts remain required. No closure gates
   run on the rejected FIFO to infer acceptance of a different implementation.

Return disposition: FIX PROPOSED with disclosed bounded cost tradeoffs, ready for
owner production decision. No production replacement, staging, commit, push or
additional runtime requested here. Prior protected repairs and source census stay
unchanged. Dense same-signature discrimination remains a separate unresolved unit.
