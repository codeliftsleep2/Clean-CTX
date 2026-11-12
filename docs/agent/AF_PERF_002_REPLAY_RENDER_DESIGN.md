# AF-PERF-002: replay selected-candidate presentation reuse design

Agent: Agent MaxHeadRoom
Status: executed design evidence complete; REJECT proposed Candidate reuse.
Tracked tests GREEN includes positive semantic counterexamples, not equivalence.
Owner approves bounded design only. Production behavior remains original.
No production optimization, staging, commit or push.

Active apparatus note: owner subsequently authorized retirement of the rejected
candidate switch and benchmark/comparison machinery. The reports and executed
receipts remain historical evidence; source snapshots are preserved beside the
design receipt. The old design command below is retired. Active protection is now
replay_footer_observation_af_perf_002.rs, with cleanup verification tracked in
AF_PERF_002_GITDIFF_DISCOVERY.md. No Candidate reuse implementation remains active.

## Exact rendering, state and authority trace

Default registry replay_history -> persistence::handle_replay_history -> replay.rs.
The handler starts its existing publication ticket, recovers pending edits, selects
durable context and validates historical compatibility against acquired current
source. Historical canonical IR and retained semantic authority come from that
validated artifact. Alias adoption precedes hierarchy construction. source_matches
compares the acquired source hash against the selected historical hash.

economical_presentation_document always calls presentation_document once before
tokenizer selection. The renderer consumes local immutable IR/hierarchy/fidelity,
then calls state.format_dict_footer_for_aliases(&[&ir.file_id]). That wrapper takes
dict_lock only for PathDictionary::format_footer_for_aliases. The renderer does
not hold this guard through tokenizer selection, optional second presentation or
canonical publication. economic.text owns the selected String after the guard
has released. Source, IR and hierarchy inputs remain local immutable snapshots;
the request-scoped dictionary footer is the live mutable input between renders.

PathDictionary::get_or_create_alias preserves existing path/alias pairs. An unrelated
alias insertion does not affect this required-alias-only footer. remove_path_alias
can remove that exact pair; it does not itself hold or advance the semantic
publication ticket. state.dict_lock exposes the dictionary, whereas the traced
registered deletion path additionally deletes durable state and forgets canonical,
semantic/cache and alias ownership. These mutation surfaces are not interchangeable.
Do not infer a legal registered deletion outcome from a direct dictionary mutation.

Original selection: matched-source raw consumes economic.text; all other cases call
compact, hence presentation_document again. Proposed TEST-ONLY selection adds only
Candidate -> economic.text; both raw branches retain their original expressions.
The release cfg keeps the original selection, renderer/economics/publication code.
Historical raw with mismatched current source still requires compact historical IR.

After selection, publication validates the existing ticket/retained authority,
clones IR into canonical ownership, publishes compatible semantics and clones the
selected String into the text cache. The same rendered String then enters the
unchanged complete response; original IR/hierarchy still supply wire output and
metadata. No guard extension, new lock/cache, retry or authority policy is proposed.

## Test apparatus and expected semantic counterexample

The default test mode remains original. A per-thread RAII mode enables only the
approved Candidate reuse and restores mode/observer/hook state on unwind. New hooks
are cfg(test,typescript), operation-local and one-shot. They never compile into
release builds. Discovery observations remain available in default mode; equality
of two presentations is not asserted while a deliberate dictionary fault is active.
Instead the design test retains and compares the exact original/candidate outputs.
No response/error/path/hash/footer normalization occurs.

Nine deterministic paired interleavings run through the actual registered handler:

| Boundary | Mutation | Required control |
| --- | --- | --- |
| Before economics helper/first render | Add unrelated alias on scoped thread | Both complete results equal; mutation precedes selection |
| Between economics result and compact | Add unrelated alias on scoped thread | Required-alias footer and complete results equal |
| After historical alias adoption | Remove that alias on scoped thread | Both first/second renders see removal; complete results equal |
| Before economics helper/first render | Remove that alias on scoped thread | Both see removal before selection; complete results equal |
| Between economics result and compact | Direct exact-alias removal on scoped thread | EXPECT response and text-cache divergence; canonical/provenance/durable identities remain exact |
| After final presentation, before publication | Direct exact-alias removal on scoped thread | Both retain already-selected text and equal resulting identities |
| Between economics result and compact | Actual registered delete_context | Preserve full nested/outer responses and state; report any difference explicitly |
| Between economics result and compact | Write new source and actual registered compression | Preserve the original successful historical adoption; compression does not advance replay ticket clock |
| Between economics result and compact | Compression followed by a newer actual registered replay | Newer replay commits its ticket; older replay is rejected identically |

Direct removal between renders is a deliberately stronger dictionary-API mutation
than registered deletion. Original can re-read an absent alias while candidate keeps
the first footer; no new ticket policy is added to force equivalence. This expected
counterexample is not yet runtime evidence. Tests require the specific divergence
and emit both complete results rather than calling it GREEN equivalence. Registered
deletion's outer outcome is observed and compared, not assumed to share direct
dictionary semantics; any difference is retained in its interleaving row.

If the divergence executes, a passing tracked test means the counterexample was
observed, NOT that reuse is universally equivalent. Whether direct mutation is
excluded by an authoritative production contract remains a substantive owner-review
question. Do not manufacture that exclusion from current call-site convenience.
New synchronization/snapshot/authority policy is outside scope. Return DEFER or
REJECT if exact observation semantics cannot be retained within the approved slice.

## Complete paired public, persistence and error controls

The six discovery populations retain High/Edit, matched/comment-changed source,
cl100k/llama3, cold and four warm registered replays. Each original/candidate mode
has a fresh database over the SAME fixture path and source bytes. Two additional
phases restart state, call registered restore_context, then replay_history again:
48 cases x seven phases = 336 paired complete semantic rows. Mismatched-source
restore must refuse; matched-source restore must succeed. Both full responses and
post-operation identities are compared without error normalization.

Rows retain full response/metadata, canonical version/tuples/hash/alias/owner,
required-alias footer, text cache, source cache, provenance, semantic edges, fidelity
and complete loaded durable Debug context. Durable identity stays unchanged across
ordinary replay/restore sequences. Positive Candidate and BOTH raw branches are
required. In ordinary cases economics text/token counts/source eligibility and
necessary publication/cache observations match; only the Candidate second-render
work differs. No IR ownership or save snapshot correction is attempted.

Missing argument and missing durable context compare exact complete errors with
zero economics/compact work. Five existing compatibility tests retain generation,
missing-identity, current/historical source, later historical adoption and clean
refusal behavior. Existing older-in-flight restore versus newer publication race
runs separately. Controlled interleavings are bounded, not exhaustive concurrency
proof or a new dictionary authority contract.

## Symmetric work and construction-inclusive costs

All 48 discovery dimension combinations have original/candidate local cost rows.
Durable checkpoint setup, file writes, state creation and initial cache/alias warmup
are outside timing. Each mode retains three warmups then 24 fixed alternating
samples. Actual registered dispatch, source/compatibility acquisition, first render,
economics/tokenizer work, optional compact render, publication, response construction
and operation-local/output teardown are inside. State/cache-owner teardown is outside,
matching repeated request ownership. No extra samples or favorable subset selection.

Enabled complete response/state and work observations are collected separately before
timing. During samples both modes disable receipt/diagnostic updates symmetrically;
stage hooks have no mutation and no extra dictionary observation reads. Existing
debug output and test-mode branching remain. Both modes use the actual handler;
the candidate moves its owned selected String without a second clone or new cache.
Original nonconsumed economic.text retains its original handler-scope drop boundary.

Observations distinguish economics calls (first candidate render SOURCE-TRACED),
actual optional compact calls and logical output lengths, selected text/token counts,
dictionary/footer stage observations and actual successful cache-copy/publication
sites. IR clone counts/lengths are offered necessary work, not instructions walked
inside cloning. First renderer internals remain untouched; no independent internal
traversal profiler is claimed. Cache-clone bytes are String lengths, not allocator
events. Full observer JSON/string copies are test overhead excluded from named work
and timed samples. No copied-source, peak-memory, allocator or public/release latency
claim follows local output bytes or test-build timings. Historical discovery counts
remain their own executed receipt.

## User-owned design gate

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-replay-render-design.ps1
```

Expected ten tracked passes: four new design/public/error/interleaving/cost tests,
five existing compatibility tests and one existing publication-race test. Require
336 paired semantic rows, nine complete interleavings and 48 fixed cost rows,
warning-fatal all-target/all-feature Clippy, formatting, active-file size, strict
encoding and matching executed/protected identities. The driver explicitly requires
the direct between-render removal counterexample; additional registered-deletion
differences must remain reported, never normalized away.

All runtime UNRUN. Agent performed bounded source/parser/UTF-8/diff review only;
no Cargo, formatter, test, lint, benchmark, binary or server launch. Return one
FIX/DEFER/REJECT only after complete executed evidence and cost/interleaving review.
No optimization RED, real restoration mutation or mandatory composed closure applies
to design-only work. All earlier repairs remain protected and all deferred/rejected
dispositions unchanged. No production implementation, staging, commit or push.

## First design attempt: compression is not replay-ticket supersession

replay-render-design-20261109-174412 passes warning-fatal Clippy and the complete
paired public/error tests, then fails an interleaving assertion that nested
compression must supersede the replay ticket. Original replay instead succeeds.
compress.rs uses its authority-update boundary; it does not commit a newer ticket
on replay's SemanticPublicationClock. This is a fixture assumption, not a production
policy repair. The incomplete packet is not full design acceptance; cost test has
not run and no completed timing packet is discarded or retimed.

The completed interleaving log already preserves two observation differences:
direct exact-alias removal and actual registered delete_context between renders.
For registered deletion, both outer replays succeed but full response and text-cache
contents differ. This is a reachable registered-operation counterexample, not merely
an unsupported raw dictionary fault. It must remain disclosed and blocks an
unqualified FIX recommendation; no normalization or new authority rule is proposed.

Keep compression-only historical adoption as its own equality control. Add a ninth
interleaving in which compression is followed by a NEWER actual registered replay,
which commits a later ticket before the outer replay attempts publication. Preserve
both complete nested responses. Nested control replay uses original selection with
observation updates disabled under an unwind-safe guard, so it cannot overwrite the
outer work receipt or test the proposed alternative recursively. The required older
superseded-operation rejection remains an explicit positive control.

No production or renderer/economics behavior changed. Rerun the same design command;
require nine interleaving rows. Agent performed bounded log/source/parser/encoding
review only, no tests/builds/formatter/lint/runtime. Prior incomplete evidence stays
separate. No implementation, staging, commit or push.

## Executed design: REJECT proposed Candidate presentation reuse

Agent MaxHeadRoom inspected replay-render-design-20261109-181332: ten tracked
passes, zero failed/ignored; all 336 ordinary paired rows, nine interleavings and
48 fixed paired cost rows complete. Public comparison passes in 364.23s,
interleavings in 47.69s, costs in 375.74s. Existing compatibility/publication-race
controls pass separately. Warning-fatal all-target/all-feature Clippy passes in
33.25s; both formatting logs empty; size passes seven active files; strict encoding
passes 2314 text files. All 1413 recorded hashes match at inspection before this
documentation-only update. Protected implementations remain exact. inspection.json
preserves checked totals, all cost rows and disposition. The earlier failed packet
remains distinct; no completed cost samples were repeated to seek favorable results.

<!-- pcg:REPLAY-executed-reason:begin -->
### Ordinary equality does not cover the executed currentness difference

All 336 ordinary original/candidate complete responses and canonical/owner/alias/
source-cache/text-cache/provenance/edge/fidelity/durable identities agree. Matched
restore succeeds; mismatched-source restore preserves refusal; subsequent historical
replay remains H1 while current source is H2. All three economics/source branches
are positively observed. Missing argument/missing durable errors remain exact and
perform no rendering. Existing generation/legacy-identity/historical-adoption and
older-restore publication protections remain GREEN.

| Executed interleaving | Original/candidate complete outcome |
| --- | --- |
| Unrelated alias insertion before economics or between renders | Equal |
| Own-alias removal after adoption or before economics | Equal; both renders see removal |
| Direct own-alias removal between renders | DIFFERENT response/footer and text cache |
| Own-alias removal after presentation/before publication | Equal; both retain completed presentation |
| Registered delete_context between renders | DIFFERENT response/footer and text cache |
| Registered compression only between renders | Equal; original historical adoption succeeds |
| Newer registered replay after compression | Equal rejection of older replay; newer ticket commits |

<!-- pcg:REPLAY-counterexample:begin -->
The registered-deletion case is decisive. Both original and candidate outer replays
succeed under the EXISTING implementation. The deletion removes the requested alias
before original compact rendering. Original reads the newer dictionary state and
returns a 542-byte presentation with no alias mapping; candidate reuses the earlier
607-byte presentation containing the deleted mapping. Their text-cache contents
also differ. All other captured identity fields agree across modes, including full
canonical/provenance and durable state. The direct dictionary-API mutation produces
the same footer/cache difference, but rejection does not rely only on that stronger
fault: the actual registered delete reproduces it.

<!-- pcg:REPLAY-counterexample:end -->
No field is normalized or removed, no divergent footer is declared equivalent, and
no synchronization or ticket change is added to hide the result. The tests pass
because they capture the counterexample explicitly. The observed successful replay
after deletion is original lifecycle behavior, not newly authorized policy; changing
it would be separate work and is not undertaken here. Compression-only adoption
and true newer-ticket supersession likewise remain distinct original contracts.

<!-- pcg:REPLAY-executed-reason:end -->
### Actual work retained versus omitted in sequential controls

There are 288 observed replay calls plus 48 restore calls in the 336 paired phase
rows. Compared with discovery's five replay phases, the design adds one post-restart
replay phase per case; these totals must not be relabeled as discovery's 240 rows.

| Replay branch | Calls per mode | Original / candidate second renders | Original second-render logical bytes |
| --- | ---: | ---: | ---: |
| Candidate selected | 36 | 36 / 0 | 23568 |
| Raw selected, source matches | 126 | 0 / 0 | 0 |
| Raw selected, source differs | 126 | 126 / 126 | 2484564 |

The first required candidate rendering remains source-traced inside every economics
call, with internal renderer traversal unprofiled. Successful publication/cache
sites retain one IR clone and selected-text cache clone per successful replay in
ordinary runs. Full selected presentations and offered instruction lengths remain
in receipts. Eliminating the 36 sequential second renders does not authorize losing
the later dictionary observation. Counts and constructed String lengths remain
logical work, not allocation events, peak memory, hash CPU bytes or workload frequency.

### Fixed construction-inclusive local costs

Milliseconds are original / test-only candidate medians. Every cell retains all
24 fixed alternating samples with three warmups per mode. The complete 48 rows are
shown, including unchanged-work raw controls and unfavorable cells. Timing includes
registered dispatch, compatibility/rendering/tokenizer/publication/response and
operation-local/output teardown; setup/state ownership boundaries remain as defined
above. These ordinary timing fixtures do not contain the divergent interleavings.

| Methods/body/comments | Fidelity | Tokenizer | Matched source ms | Mismatched source ms |
| --- | --- | --- | --- | --- |
| 0/0/0 | high | cl100k | 17.321 / 17.221 | 17.216 / 17.209 |
| 0/0/0 | high | llama3 | 17.026 / 16.977 | 17.161 / 17.228 |
| 0/0/0 | edit | cl100k | 14.320 / 14.134 | 13.937 / 13.975 |
| 0/0/0 | edit | llama3 | 13.703 / 13.686 | 14.175 / 14.196 |
| 1/0/0 | high | cl100k | 20.980 / 21.351 | 19.333 / 18.759 |
| 1/0/0 | high | llama3 | 18.170 / 18.168 | 18.479 / 18.523 |
| 1/0/0 | edit | cl100k | 15.122 / 15.284 | 15.323 / 15.268 |
| 1/0/0 | edit | llama3 | 14.634 / 14.562 | 14.845 / 14.880 |
| 8/0/0 | high | cl100k | 22.463 / 22.405 | 23.648 / 23.964 |
| 8/0/0 | high | llama3 | 22.286 / 22.517 | 22.002 / 22.002 |
| 8/0/0 | edit | cl100k | 19.197 / 19.216 | 19.269 / 19.066 |
| 8/0/0 | edit | llama3 | 18.533 / 18.550 | 18.927 / 18.836 |
| 64/0/0 | high | cl100k | 51.721 / 52.163 | 52.785 / 52.932 |
| 64/0/0 | high | llama3 | 51.785 / 51.473 | 50.525 / 50.577 |
| 64/0/0 | edit | cl100k | 51.672 / 51.855 | 51.481 / 51.941 |
| 64/0/0 | edit | llama3 | 49.427 / 51.731 | 47.364 / 47.505 |
| 8/4096/0 | high | cl100k | 276.602 / 276.026 | 280.434 / 282.675 |
| 8/4096/0 | high | llama3 | 296.655 / 296.956 | 289.187 / 286.846 |
| 8/4096/0 | edit | cl100k | 453.946 / 446.993 | 444.241 / 443.202 |
| 8/4096/0 | edit | llama3 | 464.271 / 466.273 | 462.324 / 461.858 |
| 8/0/4096 | high | cl100k | 55.246 / 54.696 | 54.061 / 53.908 |
| 8/0/4096 | high | llama3 | 58.178 / 57.069 | 55.144 / 55.272 |
| 8/0/4096 | edit | cl100k | 51.415 / 51.840 | 50.448 / 50.275 |
| 8/0/4096 | edit | llama3 | 53.617 / 53.703 | 50.839 / 51.021 |

All 48 central index-6..18 sample ranges overlap. Candidate-selected cases favor
reuse in four medians and disfavor it in two: High/long-body/mismatched source rises
280.434 -> 282.675ms (about 0.8%); Edit/long-comment/matched source rises
51.415 -> 51.840ms (about 0.83%). Unchanged-work raw controls also move both ways;
64-method Edit/llama3/matched source rises 49.427 -> 51.731ms (about 4.66%).
These observations establish no consistent/public/release latency improvement or
causal allocator/CPU benefit. No further samples or alternative tuning is justified.
Regardless of timing, a semantic counterexample prevents FIX under this authorization.

<!-- pcg:REPLAY-disposition:begin -->
### Owner return and boundary

**Recommendation: REJECT this bounded earlier-Candidate reuse substitution.**
Original's second render has an observable dictionary/footer read boundary that the
earlier owned candidate cannot preserve. A real registered deletion interleaving
changes complete response/cache bytes. Preserving that boundary by adding a new
dictionary snapshot, guard extension, lock, retry or publication policy would exceed
scope. No implementation slice or original optimization RED is proposed.

Production remains original; only cfg(test) comparison/hooks/observations were added.
Historical raw-selected/source-mismatched compact rendering stays required. Replay
IR ownership and save snapshots stay UNMEASURED; context_history collection-clone
hypothesis stays REJECTED; diff_commits stays UNMEASURED. Prior protected corrections
and all deferred/rejected dispositions remain unchanged. No new investigation follows
automatically. No optimization RED, restoration mutation, composed closure, staging,
commit or push. Agent inspected user-run evidence and edited documentation only.

<!-- pcg:REPLAY-disposition:end -->
```architecture-graph
node: DOC-188
kind: RESEARCH
status: INVESTIGATING
title: Executed replay reuse design and registered-delete counterexample
track: REPOSITORY
artifact: docs/agent/AF_PERF_002_REPLAY_RENDER_DESIGN.md
falsifies:
  - EXP-003-REPLAY
note: Source-attributed bounded subject; historical measurements and execution standing are not current-executable certification.
knowledge_scope: evidence
knowledge_content:
  - statement self#whole
  - counterexample:registered_delete self#REPLAY-counterexample
  - rejection_reason self#REPLAY-disposition
```
