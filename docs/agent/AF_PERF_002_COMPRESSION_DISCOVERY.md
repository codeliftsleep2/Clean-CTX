# AF-PERF-002: compression/compaction first production trace

Agent: Agent MaxHeadRoom
Status: C1 approved local correction FIX / CLOSED / PROTECTED;
C2 characterization remains open. Historical discovery/evidence retained below. Previous NgRx collector FIX remains protected.

## Entry points and distinct paths

registry.rs routes compress_code_context to core/compress.rs. Normal IR success
compiles a candidate artifact, validates/reconciles its semantic dependencies,
checks hierarchy, counts raw source, clones a durable IR candidate, renders
presentation, applies economics, persists/publishes and builds a response.
Verbatim bypasses IR/economics; stale/excluded/unsupported/failure branches differ.
Failed compilation returns ir_unavailable (legacy fallback retired).
IR cloning is a separate ownership concern; do not infer it is redundant.

core/content.rs::economical_presentation_document is also called by core restore,
delta, delta_apply and persistence replay. Their source/currentness/lifecycle
boundaries are independent; one caller correction cannot be assumed valid for all.

compression/pipeline.rs exports compress_file_with_source/compress_text/compress_source
and capture assembly/type aliases/micro opcodes/symbol compression. These are not
the normal IR-success handler path. The fallback/default caller distinction must
be preserved in any performance ranking. compaction method/field/import/class
helpers service these paths; their full delegated growth is not yet characterized.

## C1 candidate: repeated raw-source token count in one public operation

compress.rs computes raw_tokens with count_tokens_with_tokenizer(source_text,
tokenizer_ref), then calls economical_presentation_document with the same immutable
source_text and tokenizer_ref. That helper calls select_with_local_tokenizer, which
counts raw again before calling select_with_local_tokenizer_and_raw_tokens. The
latter existing helper already accepts a caller-supplied raw count. Candidate/raw
selection, tie behavior, calibrated approximate-token margin and fail-safe raw
passthrough are semantic obligations, not optional overhead.

This is inspected duplicate immutable computation, not executed measurement or a
new authorization to change interfaces/callers. First measure requests at actual
count boundaries. No tokenizer output/candidate policy or fallback is changed.

New cfg(test) observations in tool_helpers raw/general count entry and economics
raw count entry record input content hashes, request counts and total bytes per
boundary. Hash is for test observation identity only, never semantic authority.
Counts are synchronous/thread-local and do not cover tokenizer-internal allocation,
encoding work outside these boundaries or async workers. Timing includes observation
hash/map overhead, so do not compare with prior uninstrumented measurements.

Tracked compression_token_work_af_perf_002.rs dispatches actual compress_code_context
at High fidelity over1/2/4/8/16/64/256 source-comment repetitions with a fixed class/
method. It checks successful public completion and that both named boundaries saw
the exact raw source. It reports actual multiplicity without imposing a future
optimization budget. This is byte-growth, not independent AST/member growth.
Existing six economics tests retain exact/tie/approximate/fallback selection controls;
they are not substituted by the new response-success characterization.

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-af-perf-002-compression-work.ps1
```

Expected one new tracked public characterization plus six existing economics tests,
then focused warning-fatal all-target/all-feature lint/format/size/encoding. Exact
source hashes/logs are retained. All authored UNRUN; no agent Cargo/runtime started.
Fast bounded checks cover wrapper parsing, edited UTF-8/no BOM and line ceilings.

## C2 structural risk: configured alias population times scanned text

apply_type_aliases sorts valid pairs longest-key-first, scans UTF-8 text left-to-right,
and tests each pair at each current position until one matches. Text length T,
configured eligible alias count A, key lengths, boundary behavior and match density
are independent. Nonmatching inputs can inspect A pairs per text position. Alias
priority/reversibility and Unicode/token boundaries are correctness requirements.
No new matching structure or asymptotic speedup is assumed worthwhile before data.

Production callers include compression pipeline assembly and ir/type_aliases.rs
formatting of type opcodes; the latter may multiply matching across many small
rendered type strings. Per-call setup versus enclosing number of types must be
traced independently. No new alias instrumentation or test execution yet.

## Outcomes and next independent obligations

C1 DESIGN REVIEW pending actual public counts, semantic oracle review and narrow
construction-inclusive correction design. C2 STRUCTURAL RISK/MONITOR pending
configured-alias/type-opcode density characterization. Neither is a measured FIX.
No cache, global tokenizer state, new matching index or authority protocol added.

Source scope advances for the named handler/economics and alias helpers only.
Streaming/default capture, compaction internals, diff/edit, symbol registration/
encoding, rendering tree traversal, proxy transport and Java/Spring remain open.
CBM status was degraded (frontend-retired timeout); known local sources supplied
this trace. Original syntax accounting9763 preserved/13 scoped/9750 pending plus
96 callback supplements is unchanged. Production-path inspection does not classify
those syntax records or establish whole-system completeness.


## C1 measured return and smallest correction proposal

Operator compression-work-20261104-145210: one public compression characterization
passes10.76s, six existing economics controls pass0.00s, zero failed/ignored.
All-feature/all-target warning-fatal Clippy25.22s, empty format logs, size19 active
files and strict encoding2076 text files pass. All receipt source hashes match;
inspection.json preserves seven parsed rows. No agent runtime was started.

| Source bytes | Handler exact-source count requests | Economics exact-source count requests | Total observed boundary bytes | Public operation us |
| ---: | ---: | ---: | ---: | ---: |
| 71 | 1 | 1 | 142 | 1598835 |
| 93 | 1 | 1 | 186 | 839700 |
| 137 | 1 | 1 | 274 | 794561 |
| 225 | 1 | 1 | 450 | 830644 |
| 401 | 1 | 1 | 802 | 830710 |
| 1457 | 1 | 1 | 2914 | 854239 |
| 5681 | 1 | 1 | 11362 | 817616 |

Both boundaries observe the exact same immutable source content hash. These are
count requests/bytes, not tokenizer-internal CPU/allocator or syscall counts. The
packet does not separately report whether a local tokenizer or estimation fallback
served each request. Source tracing passes the same tokenizer_ref into both. Test
hash/map bookkeeping contributes to timing; total latency includes compilation,
semantic validation/publication, persistence and response assembly. No measured
latency fraction or enterprise speedup is attributable to the repeated count.

### Narrow C1 candidate, owner disposition required

In core/compress.rs's successful IR branch only, retain the already-computed
raw_tokens value. Render candidate through existing content::presentation_document,
then call existing content_economics::select_with_local_tokenizer_and_raw_tokens
with the same source, candidate, tokenizer kind/ref and that count. This replaces
only the local call to economical_presentation_document. No new helper, signature,
cache, shared context, tokenizer policy or other caller migration is needed.

Equivalence chain: existing economical_presentation_document renders that exact
candidate, calls select_with_local_tokenizer, computes raw with the same local
count/estimate semantics as count_tokens_with_tokenizer, then delegates to the
already-counted selector. Supplying the earlier count removes that middle count
without changing candidate counting, tie handling, approximate margins, fallback,
raw bytes, durable token metadata or publication. Neither source nor tokenizer_ref
is replaced between these two locations. Ref/tokenizer error and None behavior
must still receive explicit semantic controls before acceptance.

Only the normal successful public compression branch is in scope. Verbatim early
return, failed-compilation ir_unavailable, restore/delta/replay and focused Edit
semantics retain current behavior. Existing economical_presentation_document remains
for independent callers; no broad interface rewrite follows.

Four criteria: measured repeated immutable count work; one avoided source-size pass
with no new construction/state/maintenance; same established economics selector;
small local wiring with no API/authority change. This is a narrow FIX candidate,
not an implemented optimization or a claim of significant current wall-clock impact.
Larger fixtures are unnecessary to justify eliminating this clear duplicate if
proof and semantic controls hold; total workload remains honestly reported.

After owner disposition: add unchanged production-path budget requiring one handler
source count and zero redundant economics raw recounts; demonstrate genuine original
RED, implement narrow wiring, then unchanged GREEN. Compare complete public response,
raw/candidate token fields, candidate/raw decisions and bytes, exact/approximate/
missing tokenizer cases, source-growth counts/timing and semantic publication.
Keep six existing economics tests, add finite static caller/sink protection and
isolated real recount mutation RED/restored GREEN. Original characterization remains
separate frozen evidence. No production correction or new runtime was performed.

C2 alias-growth risk, remaining compression/compaction callers and broader system
inventory remain independent. Source syntax accounting does not advance from these
seven operation observations. Protected earlier repairs are not reopened.


## Owner approves C1; pre-repair baseline authored

AF_PERF_002_C1_EQUIVALENCE_AND_REPAIR.md records exact production tokenizer,
source and policy equivalence plus error-to-None behavior. Public one-count budget
and tokenizer/selection equivalence tests are authored UNRUN. Handler still uses
the original recount path. Run the documented Before command before production
editing. C2 remains bounded characterization only; no alias representation change.


## C1 final composed GREEN and bounded closure

Agent MaxHeadRoom inspected af-perf-001-20261104-153416:46 tracked passes,
zero failed/ignored. Groups:14 admission/AST/lifecycle,5 resolver P0,6 delegated,
4 A3,9 af174,4 NgRx collector and4 C1. All recorded source hashes match;
inspection.json retains exact count and outcome. No production/test edits follow
this verification; this update records evidence only.

C1: FIX / CLOSED / PROTECTED for successful public IR-branch raw-count reuse.
Original production-path RED, equivalence/semantic GREEN, exact public result/
fail-safe controls, before/after work counts, focused zero-warning/format/size/
encoding gates, static enforcement, isolated real recount mutation RED/restored
GREEN and composed architectural gate are established. No new helper/cache/state/
API/authority/lifecycle contract. Other compression branches/callers unchanged.
No universal latency or allocator-byte improvement claim follows.

C2 alias matching stays separate characterization-only. Remaining compression/
compaction/rendering and source-wide inventory are incomplete. A2 decision remains
closed; A3 and NgRx collector protected. No automatic broad campaign, binary rebuild
or Claude field switch is needed to record this bounded result.
