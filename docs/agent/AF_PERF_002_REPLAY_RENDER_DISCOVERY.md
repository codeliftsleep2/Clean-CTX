# AF-PERF-002: next default-operation discovery, registered history replay

Agent: Agent MaxHeadRoom
Status: executed discovery GREEN; one bounded design investigation proposed.
Discovery only. No production optimization, lifecycle transition or publication.
All completed repairs stay CLOSED / FIX / PROTECTED. Second delta read, startup
projection and dense equality remain deferred; startup delta-count removal rejected.

Active apparatus note: after owner accepted the design rejection, its candidate
switch, comparison/timing tests and discovery counters/drivers were retired.
Historical receipts and source evidence remain; the command below is historical.
The focused replay_footer_observation_af_perf_002.rs regression now protects the
later dictionary observation. See AF_PERF_002_GITDIFF_DISCOVERY.md for cleanup gate.

## Scope and registered reachability

Owner's continuation authorizes broad discovery, not another correction. Registry
inspection covers context_history, save_context, replay_history and diff_commits
before narrowing instrumentation to one new production path. These are actual
default registry operations; workload frequency in real installations is unknown.
CBM reports degraded frontend_retired/timeout; unavailable fallback capabilities
require bounded host reads of registry-returned and directly referenced paths.

create_default_registry registers replay_history to persistence::handle_replay_history,
re-exported from persistence/replay.rs. The handler selects durable context and
validates historical compatibility, acquires the current source Arc, adopts the
historical alias, constructs hierarchy, computes source_matches, and calls
economical_presentation_document. That helper ALWAYS constructs one full candidate
using presentation_document before its tokenizer-based selection.

If economics selects raw AND source_matches, replay consumes economic.text. Otherwise
it calls its compact closure, invoking presentation_document a second time. When
economics selected Candidate, its already-owned text appears to equal the second
result. When economics selected raw but source does not match, replay must not expose
the current raw source as the historical body; compact historical output remains
necessary. Absence of raw selection alone cannot bypass historical compatibility.

Publication orders historical adoption using the existing authority ticket. It
clones IR into the canonical owner, publishes compatible semantics and clones the
selected presentation into the text cache. The original IR remains consumed by
hierarchy_to_wire_reduced and response metadata. These consumers make a wholesale
IR-clone deletion unjustified. No adoption, compatibility, tokenizer, source hash,
ownership, response or durable-write policy is changed by this investigation.

## Provisional shortlist before measurement

| Candidate | Current disposition | Boundary and uncertainty |
| --- | --- | --- |
| Replay's second presentation when economics already selected Candidate | UNMEASURED, highest-priority observation | Registered handler has an owned selected candidate and constructs another. Need exact equality and actual branch/population work before recommending design. |
| Historical raw selection with mismatched current source | REJECT blanket omission of compact historical presentation | Current raw bytes cannot stand in for historical output. Any narrower reuse must retain this fallback and error boundary. |
| Replay IR clone into canonical owner | UNMEASURED; no correction proposed | Live IR ownership and subsequent public wire construction both consume it. Removing ownership without a new bounded design would change lifetime/order. |
| save_context canonical snapshot and binary construction | UNMEASURED; leave intact | Snapshot outlives its IR guard and feeds tuple validation, hierarchy, binary encoding and transactional durable equality. A short-lived clone is not evidence of redundancy. Encoder implementation stays protected. |
| context_history all-file-stat ownership | REJECT alleged full collection clone | all_file_stats returns &HashMap; the handler borrows entries under its existing guard. Formatting/output ownership remains required. No new measured performance disposition follows from this source check. |
| diff_commits processing | UNMEASURED, not expanded here | Registry trace confirmed; no loop/clone promoted without consumer or representative-path measurements. |

This is a provisional source-based list, not measured ranking. No new syntax-census
disposition is recorded. At most one future design investigation may be recommended
after executed results: branch-local reuse of the already-selected replay candidate,
keeping historical raw fallback and the unchanged rendering/economics helpers.

## Actual-site observations and controls

Only replay.rs receives cfg(test,typescript) observations. Its compact closure still
calls the same renderer and returns the same text; an inactive observer returns
immediately. The economics return is observed separately. No alternate selection,
test-only optimization, cache or authority state is introduced. All protected delta,
diff, source-cache, publication, SQLite, manifest and encoder bytes are preserved.

Tracked tests run 48 independent cases: six method/body/comment populations, High
and Edit fidelity, matched versus comment-changed current source, and cl100k versus
llama3 tokenizers. Each performs actual registered compression to produce durable
history, drops the producer, then uses a fresh state for one cold and four warm
registered replays: 240 complete response/authority/work rows. Dimensions include
0/1/8/64 methods, independent 4096 six-byte Unicode body/comment repetitions,
source mismatch and request repetition. These are controlled fixtures, not enterprise
frequency or scalability estimates.

Every replay must retain historical source hash, canonical tuples/version/alias,
selected response content, text-cache content and complete loaded durable identity.
The before/after durable context must remain exact. Candidate-selected second renders
must equal the first selected candidate byte-for-byte. Positive cases must reach all
three branches: Candidate, matched-source raw, mismatched-source raw. No desired
branch is inferred solely from tokenizer configuration. Errors with missing arguments
and absent durable context must produce no economics or compact rendering.

Five existing historical replay compatibility controls and the existing older-restore
publication race run separately. They preserve generation/missing-identity rejection,
historical H1 versus current H2 authority, deliberate later historical adoption,
current replay and supersession behavior. No new concurrency equivalence claim.

Observations distinguish economics calls, optional compact calls, offered instruction
population, selected/raw/candidate token counts, full selected/compact text and logical
output bytes. The economics helper's first render is source-traced, not independently
profiled inside the protected helper. Offered instructions are not renderer traversal
counts. Rendering bytes are constructed output, not copied-source bytes, allocator
events, capacity measurements, CPU time or peak live memory. Test-only JSON/string
receipt construction adds work and is not counted as production work. No timing or
alternative comparison is included in this discovery; a later design would require
symmetric construction-inclusive cost accounting and teardown.

## User-owned verification

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-replay-render-discovery.ps1
```

Expected eight tracked passes: two discovery controls, five existing compatibility
controls and one existing publication-race control. Require 240 actual public rows,
positive selection branches, exact protected identities, warning-fatal all-target/
all-feature Clippy, formatting, size and strict encoding. Executed source inventory
is frozen before Cargo and checked unchanged at completion. Local final closed-slice
source receipts are checked when present; no generated evidence is called portable.

All runtime UNRUN. Agent performed bounded source/parser/UTF-8/line review only;
no Cargo, formatter, lint, benchmark, binary or server launch. No optimization RED,
restoration mutation or composed closure is appropriate for discovery only. Return
measured INVESTIGATE/DEFER/REJECT/UNMEASURED ranking after receipt inspection; do not
implement the candidate. No staging, commit, push or memory update.

## Executed discovery and owner return

Agent MaxHeadRoom inspected replay-render-discovery-20261109-171537: eight tracked
passes, zero failed/ignored, all 240 public/work rows complete. New public discovery
passes in 154.34s, error control in 1.14s; five historical compatibility controls
and the publication-race control pass separately. Warning-fatal all-target/all-feature
Clippy passes in 62s, both formatting logs empty, size passes three active files,
strict encoding passes 2309 text files. All 1410 recorded source hashes match at
inspection before this documentation update. Protected implementation identities
remain exact. inspection.json retains checked branch totals; original logs and
receipt remain untouched. No optimization, alternative, timing or closure executed.

| Actual replay branch | Requests | Second compact render calls | Logical second-render output bytes |
| --- | ---: | ---: | ---: |
| Economics selected Candidate | 30 | 30 | 19640 |
| Economics selected raw, current source matches | 105 | 0 | 0 |
| Economics selected raw, current source differs | 105 | 105 | 2070470 |

Every one of the 240 calls enters economics, whose first candidate rendering is
source-traced. The optional second rendering is observed directly at its caller.
Thus 375 presentation invocations are source-accounted, not an independently
profiled renderer traversal total. The potentially redundant subset is only 30
second renders/19640 constructed bytes. Do not label the much larger historical
compact output removable merely because economics initially selected current raw.

All thirty candidate-selected second results equal the already-selected candidate
byte-for-byte. Ten observations each come from High/eight long-body methods,
High/eight methods with long comments, and Edit/eight methods with long comments.
Their second-render lengths are respectively 607, 607 and 750 bytes per request.
Both matched and mismatched source cases occur; each case includes cold replay
and four warm replays. Warm history adoption does not remove this work. In contrast,
long Edit body payloads are not automatically a large avoidable-render population:
economics and historical-source eligibility decide the actual branch.

All complete response text equals the selected historical/raw result, text cache
equals that same output, and adopted canonical hash remains H1 even with current
H2 source. Complete durable context remains identical before/after every request.
Generation and legacy-identity failures preserve clean rejection; separate existing
controls preserve later historical adoption and reject an older in-flight restore
after newer publication. Error controls reach no rendering. No production consumer,
validation, ordering, persisted identity or source authority is replaced.

### Ranked disposition

1. **INVESTIGATE — selected-candidate replay presentation reuse.** Actual reachable
   requests construct and discard a second equal presentation despite retaining
   an owned candidate from economics. The branch has a narrow potential substitution
   and independently observed repeated work. Observed avoidable output is modest;
   public frequency, renderer CPU share and allocator impact are unknown. This is
   an engineering priority, not a demonstrated latency or high-impact correction.
2. **REJECT — blanket second-render removal on historical raw selection.** The 105
   mismatched-source/raw-selected requests need historical compact output rather
   than current raw bytes. Preserve this consumer and its authority boundary.
3. **UNMEASURED / no correction proposed — replay IR ownership and save snapshot.**
   Source tracing finds real live/public/persistent consumers. Clone-specific work
   and a behavior-preserving alternative were not measured; do not promote them
   from syntax or assume that shortened lifetime makes ownership redundant.
4. **REJECT — context_history all-file-stat clone hypothesis.** The collection is
   borrowed; this source hypothesis does not exist as an owned full collection copy.
   Formatting remains necessary. **UNMEASURED** diff_commits processing stays outside
   the measured shortlist; no benefit or correction is invented for it.

No DEFER timing conclusion is asserted because timings were not collected. Protected
and previously deferred/rejected scopes retain their existing dispositions.

### One proposed bounded design, requiring owner authorization

Compare only reusing economic.text when economics selected Candidate against the
original second presentation. Retain matched-source raw behavior and the complete
mismatched-source/raw-selected compact fallback. Keep renderer, economics/tokenizer
policy, hierarchy, IR clone/publication, persistence and public contracts unchanged.
Do not introduce caching or a new dictionary snapshot/lock policy.

The renderer calls state.format_dict_footer_for_aliases, which takes a dictionary
guard per call. Equality in these fixtures does not prove that two render times
observe identical alias/footer state under concurrent activity. A design must trace
that ownership/currentness boundary and preserve the original operation's observation
semantics, including error/supersession behavior. If the earlier candidate cannot be
substituted exactly, return DEFER or REJECT rather than invent synchronization.

Require paired complete registered public/canonical/cache/durable/provenance results,
positive Candidate and both raw branches, tokenizer/fidelity controls, historical
H1/current H2, restart, malformed/compatibility refusal and publication/dictionary
interleavings. Compare symmetric construction-inclusive costs with render/result
teardown, separating output bytes from allocation events and public latency. Reuse
the discovery dimensions; do not retime simply to obtain a favorable result.

No FIX recommendation or implementation authorization follows discovery GREEN.
No optimization RED, restoration mutation or composed closure is run. All protected
repairs stay closed; census unchanged. Agent only inspected user-run evidence and
performed bounded documentation/encoding work. No staging, commit or push.

## Subsequent design return

The separately authorized test-only comparison is executed in
AF_PERF_002_REPLAY_RENDER_DESIGN.md. Recommendation: REJECT earlier-Candidate
presentation reuse. An actual registered deletion between renders changes the
original footer and text cache while reused economic.text retains the earlier
mapping. Discovery's sequential equality remains valid historical evidence; it
does not establish currentness equivalence. No production correction follows.
