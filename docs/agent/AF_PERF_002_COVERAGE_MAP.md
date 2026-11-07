# AF-PERF-002: production coverage map and pathology register

Agent: Agent MaxHeadRoom
Status: first-pass coverage skeleton and index/persistence trace; discovery INCOMPLETE.
Generation 78 remains the accepted semantic baseline. No broader repair approved.

## Coverage interpretation

INSPECTED means the named paths were read, not every path in the subsystem.
CHARACTERIZED applies only to named measured work units. PROTECTED applies only
to a declared invariant. NOT INSPECTED explicitly reserves the remaining work.
REVIEW REQUIRED blocks architectural acceptance of the affected unresolved scope;
the current finite AST gate does not yet mechanically enforce whole-system drift.
No clean test or repaired helper establishes whole-system completeness.

| Subsystem / source anchor | Current coverage | Delegated work and next obligation |
| --- | --- | --- |
| Workspace membership: index/edges.rs -> membership.rs -> remove.rs | PROTECTED, finite registration subset | Three membership dimensions measured to 256; order/ownership/refinement invariants and real mutation proof. Declaration/reconciliation costs excluded. |
| Declaration authority: index/declarations.rs | CHARACTERIZED, two membership dimensions | add_edges self-Defines carrier -> register_declaration -> occurrence iter/any plus publisher Vec contains. Both dimensions measured through 256: 32640 comparisons each; enclosing costs remain distinct. Queries sort the returned declaration occurrences; cost may be justified by deterministic answers. |
| NgRx: index/ngrx_refinement.rs | CHARACTERIZED, finite rebuild counts | add_edges/remove_file -> rebuild -> retract refined edges/entities -> clone complete candidates -> authority joins/export reads. Populated insertion is measured; all delegated object-member joins not characterized. |
| TypeScript witnesses: ir/typescript_resolution/host.rs | CHARACTERIZED, finite lifecycle census | Capture multiplicity vs replay vs required boundary checks remains REVIEW REQUIRED. No shared validation approved. |
| Retained admission: ir/typescript_resolution/relation_evidence.rs | PROTECTED, fourteen complete-set slots | Per-consumer construction bounded; structural inner joins and witness sweeps remain REVIEW REQUIRED. |
| Angular acquisition/extraction: angular_meta, layers/meta | INSPECTED, prior finite evidence | Route/provider/guard identity covered; exhaustive default pipeline and all other Angular consumers NOT INSPECTED in this audit. |
| Durable lifecycle: mcp/durable_semantics.rs | CHARACTERIZED, small fixture | Publication/restore/snapshot/query counters measured. Lock-held validation remains DESIGN REVIEW. Other failure/race workloads unresolved. |
| SQLite: mcp/sqlite_store/semantic_state.rs, replay.rs | CHARACTERIZED, finite canonical/durable replay slice; CLOSED / NO REPAIR | Bounded measured lookups ACCEPT. Instruction/delta work and seven refusal controls retained; full validation/prefix questions deferred MONITOR. Physical IO, other entry points and full publication coverage remain open. |
| Query/hydration: mcp/tool_handlers/query/prepare.rs, hydration | INSPECTED, prior finite census | Catalog/discovery reuse differs from retained owner currentness. Need per-entry-point graph and negative/unchanged/adversarial workloads. |
| CBM: cbm/bridge/indexing.rs | INSPECTED, project indexing invocation only | Project-level external indexing and caller multiplicity need characterization. External provider contract unchanged. |
| Canonical IR: ir/compiler, pipeline, hierarchical, replay, binary_wire | PARTIALLY CHARACTERIZED; hierarchy placement repair PROTECTED | Two hierarchy placement sinks protected; sequence replay packet closed without repair. Other compilation/edit/restore, encoding and ownership dimensions remain uninspected; no subsystem-wide acceptance. |
| Provenance/graphs: ir/program_graph, workspace/index/traversal.rs | PARTIALLY CHARACTERIZED, index traversal only | Unrelated/invocation and reachable-density/scoped lifecycle packets measured; direct-key alternative remains DESIGN REVIEW. Program graph and complete public-query amplification remain open. |
| C#/EF: dotnet_meta, ir/csharp_resolution; publication/reconciliation | PARTIALLY CHARACTERIZED, named A1/A2/A3 boundaries | A1 cold/dirty/warm copy/sink sites measured; A2 no-repair decision closed; A3 protected. Distinct owner copies ACCEPT/MONITOR, handler-only redundant durable IR clone FIX / PROTECTED (65-test final gate). EF and broader dependency/proof/concurrency coverage remains open. |
| Spring/Java: spring_meta, layers/language | PARTIALLY CHARACTERIZED, default Spring compile slice | Class/method axes and exact endpoint facts measured; inner scanning MONITOR. Other roles/properties, failure/fallback and Java consumers remain open. |
| Compression/compaction: compression/pipeline.rs, compaction | PARTIALLY CHARACTERIZED | Successful-IR C1 recount repair PROTECTED; C2 alias work MONITOR. Remaining compaction/capture/rendering consumers and independent dimensions remain open. |
| MCP/protocol/proxy: mcp, protocol, proxy_spawner | PARTIALLY INSPECTED; two bounded proxy scrub repairs PROTECTED | Exact change detector and eight ASCII prefilter roots protected. Unicode equivalence, transport/queue/cancellation, logging and other proxy/MCP paths remain open. |
| Other edit/diff/cache/analytics/intelligence/dictionary/gitdiff paths | NOT INSPECTED | Enumerate external/default callers before classifying work. No blanket historical exemption. |
| Observability: context_stats; observability/metrics, tracing, startup | PARTIALLY CHARACTERIZED; session-wide stats JSON format guard PROTECTED | Thirty complete response oracles, work budget/static and real mutation protection. Other logging serialization/allocation, diagnostics, startup/error and transport/concurrency paths remain open. |

## First-pass work-dependency graphs

Insertion: one add_edges invocation consumes E input edges, registers up to 2E
entities (before edge dedup), constructs owned keys, clones endpoints/edges into
forward/reverse storage, merges publisher keys, then invokes complete NgRx rebuild.
Self-Defines carriers additionally traverse declaration occurrence and publisher
key buckets. Over F publications the rebuild sees accumulated candidates; F and
candidate density are independent. One membership helper's expected-linear probes
do not bound declaration comparisons, candidate cloning or export IO.

Persistence: save_context_with_* -> save_context_transaction -> currentness ->
transaction -> context save -> snapshot construction/authority clone -> identity
JSON -> snapshot write -> commit, with rollback on failure. Trace enclosing batch
and BufferedStore callers before inferring transaction amplification. Restore:
load_durable_context -> load_latest + current_context_id + load_context_with_deltas
-> baseline lookup/decode + current_context_id + ordered delta query -> replay ->
instruction clone/reconstruction -> snapshot query/decode. Repeated owner lookup
is a structural observation, not proof that its cost warrants a new protocol.
SQLite ownership/transaction consistency cannot be weakened to reduce queries.

## Living pathology dispositions

| Finding | Classification / disposition | Evidence and decision |
| --- | --- | --- |
| Three ordered membership scans | MEASURED DEFECT / FIX, bounded repair verified | Original RED; growth GREEN; invariants; mutation static+counter RED and restored nine GREEN. Broader #174 remains open. |
| Declaration occurrence and per-file key scans | STRUCTURAL RISK / MONITOR pending new counts | Packet GREEN confirms both n(n-1)/2 series. Two-companion design proposed, no representation repair approved. |
| Per-mutation full NgRx reconstruction | MEASURED DEFECT / DESIGN REVIEW (#173) | Populated cumulative export counts 1/3/10/36/136; batch visibility/failure protocol unresolved. |
| Empty/duplicate insertion rebuilds | STRUCTURAL RISK / MONITOR | Unconditional call read in production; new no-op invocation packet UNRUN. Populated no-op export effects not yet measured. |
| Repeated witness observations and mutex-held IO | MEASURED WORK / DESIGN REVIEW | Existing census; duplication alone does not prove an observation can be omitted. |
| SQLite repeated owner lookup, IR/authority copies | STRUCTURAL RISK / MONITOR | Named functions read; no SQL, byte/allocation or enclosing-call counts yet. |
| Ordered result sorting and two adjacency representations | UNRESOLVED / MONITOR | Potentially justified ordering/query tradeoffs; characterize before ACCEPT or FIX. |

## Next focused characterization packet

Two tracked tests in index_enclosing_af_perf_002.rs grow 1/2/4/8/16/64/256
through real add_edges, separating physical declaration occurrences from unique
per-file declaration keys; count empty and duplicate call reconciliation as well.
Assertions characterize existing behavior, not permanent quadratic fitness budgets.
Counters are test-only synchronous observations, not runtime authority. No production
optimization, lifecycle change or deferred reconciliation is introduced.

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-af-perf-002-enclosing.ps1
```

The operator command formats changed files, checks sizes, runs only the two new
characterization tests and nine existing af174 controls, and retains hashes/logs.
UNRUN. Fresh instrumentation needs affected lint/encoding verification after the
packet stabilizes. Prior membership observations remain historical evidence;
new counters do not retroactively certify new source bytes.

## Discovery exit and adversarial second pass

First enumerate remaining public/default entry points and expensive delegated
sinks for every NOT INSPECTED row, refine rows as paths diverge, measure significant
independent dimensions and classify findings. Then conduct a separate adversarial
pass with its own evidence: negative/no-op, duplicate/ambiguity, cancellation/failure,
wide/deep/cyclic inputs, stale dependency changes, and relocated multiplication.
Neither pass is complete. Owner review of the completed coverage map is required.
Coverage-drift design must compare classified functions/call boundaries and block
new unresolved expensive paths; current known-writer AST coverage is only a finite
starting point. No whole-file hash approval or generic runtime registry is proposed.

## Enclosing packet and targeted pattern-search update

See [AF_PERF_002_ENCLOSING_RESULTS_AND_DECLARATION_DESIGN.md](AF_PERF_002_ENCLOSING_RESULTS_AND_DECLARATION_DESIGN.md)
for verified 2+9 passes, exact comparison/no-op counts, remaining lint boundary,
new delegated NgRx and graph findings and the proposed separate declaration design.
Targeted lexical census records 760 matching lines across 218 production source
files; this is a discovery queue, not 760 defects or completed classification.
Initial reviewed repeated-membership instances include selector resolution, CBM
trace dedup, config-composition pair dedup and forms normalization/validators.
No generalized rewrite or cross-subsystem persistent representation is approved.

Graph coverage is now INSPECTED only for named ProgramGraph scalar Vec queries,
optional graph-pass build and transitive-dependency global node-index preparation.
Default/live caller multiplicity, cycle/error/other graph paths remain unresolved.
NgRx binding collection's complete adjacency scan and candidate-by-binding joins
are traced but unmeasured. Prior finite coverage does not certify these paths.

Owner requires independent Agent Popper review after first-pass enumeration, with
concrete source-grounded counterexamples or evidence-backed acceptance. First pass
is incomplete; Popper review has not run. Both source-path coverage and behavioral
dimension coverage must be completed and owner-reviewed before discovery exit.

## Expanded membership-family enumeration

[AF_PERF_002_MEMBERSHIP_SOURCE_INVENTORY.md](AF_PERF_002_MEMBERSHIP_SOURCE_INVENTORY.md)
records 525 enumerated production/support Rust files, 736 explicit test/fixture
exclusions, 8334 expanded lexical candidate lines and 15 inspected operation
records. Counts have different units: lexical lines are not AST sites or defects;
operation records do not classify every nested site. Remaining sites are explicitly
unresolved. New AST census/call-boundary evidence is authored UNRUN. Proxy source
and auxiliary example targets are included; inline cfg(test) remains a known
superset limitation. CBM is degraded; no resolved whole-program call graph exists.

Source-path enumeration has broadened; behavioral coverage remains incomplete.
Do not upgrade NOT INSPECTED subsystem rows merely because their files appear in
the census. Manual/iterator/helper/macro/type dispatch requires classification.
The declaration proposal remains unapproved. Agent Popper's independent review
will follow completed first-pass enumeration, not substitute for missing inspection.

## Membership caller follow-up: source versus behavioral coverage

The membership ledger now contains 24 inspected operation records (nine added).
Additional exact bodies traced: GlobalSymbolTable register/unregister, ordered
StringTable intern, binary encode string cloning, IR subclass query, method diff
matching, reverse path-dictionary retirement, CBM grouped decoder, and proxy regex
filter selection/fallback. Each has its own classification and unresolved caller/
measurement constraints; no new defect is labeled measured.

Source-path progress: CoreIRPass capture registration and real snapshot/git diff
callers have named source anchors. Behavioral gaps remain repeated symbol overwrite,
wide overload groups, subclass graph density, grouped provider column rebuilding,
negative proxy regex fallback and retirement across large dictionaries. Local
StringTable membership is already keyed and preserves ordered storage; do not
generalize that finding to all IR work. Public IR query/unregister APIs lack
established default production caller multiplicity in this pass.

No membership-census run directory was present at inspection. Its tracked AST
census and affected lint packet remain UNRUN, not presumed GREEN from continue.
Raw source enumeration does not resolve macros, traits, aliases or inline test
code. First-pass inspection and independent Agent Popper review remain incomplete.

## Verified AST packet and remaining denominator

AF_PERF_002_CENSUS_RESULTS.md records the verified 525-file/9763-site census and
40863 raw calls, twelve tracked passes and clean affected lint/encoding. Thirteen
syntax records have explicit scoped dispositions; 9750 remain unresolved.
artifact.rs parser-error coverage is REVIEW REQUIRED, not silently excluded or
asserted compile-invalid. Source enumeration is now executed evidence; caller/type/
macro and behavioral inspection remains incomplete. Popper review has not run.
Next focused tests characterize populated NgRx empty/duplicate call work and
export-change retirement/repair; authored UNRUN, no reconciliation policy change.

## Owner completion boundary and populated characterization return

See [AF_PERF_002_INVENTORY_ACCOUNTING.md](AF_PERF_002_INVENTORY_ACCOUNTING.md)
for lossless reconciliation of all 9763 original records, seven reviewed logical
operations, 9750 blocking macro/review outcomes, 96 supplemental all-callback
obligations and explicit src/lib.rs drift. No automatic test-only exclusions or
blanket file-level approvals were used. Closed accounting is not completed review.

Agent Popper independently found bare-name owner collisions, omitted iterator
multiplicity and scalar-versus-iterator ambiguity; methodological scoped review
is recorded, final adversarial acceptance is outstanding. Populated NgRx packet
verified 2+9 passes, clean lint/format/encoding and export retirement/byte repair.
Empty input alone cannot justify skipping reconciliation. Exact parser-error
span/impl/function diagnostic is authored UNRUN under a narrow operator handoff.
No production optimization or declaration representation change was made.

## Scoped parser recovery verified

[AF_PERF_002_PARSER_RECOVERY_RESULTS.md](AF_PERF_002_PARSER_RECOVERY_RESULTS.md)
records the GREEN one-test diagnostic, clean affected lint/format/encoding, 1282
matching receipt hashes, three exact error spans confined to cfg-qualified field
destructuring and all 23 recovered function/impl boundaries. Agent Popper accepts
manual recovery of this syntax boundary only; artifact caller/cost classification
and final inventory acceptance remain open. Original parser flags/record identities
are retained; the accounting-003 supplement attaches recovered intervals without
automatically clearing any candidate classification. No production changes or
additional user-run gate are required for this metadata/documentation return.

## Owner reporting correction

The accepted parser syntax boundary is closed; its original flags remain historical
evidence, not an active blocker. Artifact operations remain separately reviewable.
See AF_PERF_002_OPERATION_PROGRESS.md for independent syntax-accounting and
production-operation indicators, four new artifact/public-caller traces and an
unrun real-query characterization. No raw syntax progress is equated to behavioral
coverage. No production optimization or declaration representation change followed.

## Actual artifact work return

AF_PERF_002_ARTIFACT_WORK_RESULTS_AND_A3_DESIGN.md records the completed authorized
artifact-work packet: three passes and clean affected gates. Independent C/D axes
measure direct filtering C*D; current warm generation avoids all instrumented
projection/resolution work. These are new production observations, not more syntax
classifications. The per-projection borrowed-bucket repair remains proposed only.
A1 publication-copy volume, A4 peer joins, extensions/ambiguity and A2 owner-count
growth remain unmeasured; no whole-system fitness or completed inventory claimed.


## Current disposition update after A2 owner review

This dated work-stage update supersedes earlier proposed/UNRUN statuses above
without rewriting their historical evidence:

- A3 bounded candidate selection: implemented; focused semantic/lifecycle, complete-
  operation work counts, finite static and isolated mutation proof, composed38-test
  fitness GREEN. Scope and latency/memory limitations remain explicit in its report.
- A2: characterization accepted for the selection decision; DESIGN REVIEW/MONITOR.
  Do not implement A2a; no automatic escalation to A2b. Broader coverage remains open.
- C# is partially traced/measured, not wholly NOT INSPECTED and not wholly certified.
  A1 copying and A4 footprints remain independent unresolved work units.
- Declaration membership: measured quadratic occurrence/publisher dimensions;
  representation still awaits separate disposition. No repair inferred from #174.
- NgRx populated no-ops are executed, not UNRUN: empty input revalidates consumed
  exports and retires stale dispatches; no no-op skipping/deferral is approved.

Next non-C# source-grounded priority: NgRx complete reconstruction and object-member
joins, extending the existing #173 investigation rather than creating a new issue.
Read anchors: WorkspaceIndex::rebuild_ngrx_import_refinements,
object_member_value_bindings, has_object_member/has_value_declaration/
has_reducer_declaration, one_hop_named_reexport and export_evidence helpers.

Additional static gaps found:

1. object_member_value_bindings scans all forward stored edges and clones four
   strings per eligible binding on every rebuild, before candidate-family dispatch.
   Zero object-map candidates does not currently bypass this collection. Measure
   unrelated E independently of bindings B and candidate kinds; do not assume
   removing it is valid until all consumers are traced.
2. Each MAP_LAYER candidate loops the complete value_bindings collection; switch-
   value candidates independently filter it. Candidate M and binding B dimensions
   can multiply. Existing direct-action no-op counts do not characterize these joins.
3. Exact declaration checks call declarations_by_identity, then filter/count by
   target; declaration lookup's clone/sort/occurrence costs are delegated work,
   not covered by a cheap outer candidate counter.
4. one_hop_named_reexport performs read_to_string and a TypeScript parse outside
   exports_const/exports_function counters. Existing export-read/parse totals are
   not whole-reconstruction IO totals. Record this missing observation boundary;
   do not retroactively reinterpret old counters as complete.

These are inspected structural risks, not new measured wall-time attributions.
Next characterization must distinguish collector visits/cloned payload, MAP and
switch join visits, direct versus reexport IO, candidate/export/declaration work,
negative/empty/dense input and independent survivors. Preserve existing export-
retirement/repair controls and immediate visibility. No optimization, batching,
cache, authority reuse or reconciliation deferral is authorized by this inventory.

Source-path coverage advances for these named helpers only. Behavioral dimensions
remain unmeasured here; original9763-record syntax accounting is unchanged.
Spring/Java, compression, protocol/proxy, graph traversal and other NOT INSPECTED
rows remain explicit queue items. Final independent adversarial review is outstanding.


## NgRx reconstruction measured return

ngrx-reconstruction-20261104-123958 passes four NgRx and nine af174 tests,
clean focused gates and matching source hashes.56 rows confirm E+B binding
collection even without consumers and M*B map/switch comparisons. One-hop reexport
retirement/repair exposes previously uncounted IO. Equal join counts have unequal
times; no latency attribution or broad join repair follows. See
AF_PERF_002_NGRX_RECONSTRUCTION_RESULTS.md for a proposed demand-local collector
slice and its required observation-position proof. No optimization is implemented;
owner approval and genuine pre-repair RED remain necessary. Broader discovery open.


## Latest baseline reconciliation and #173 proof gap

AF_PERF_002_NGRX_OBSERVATION_REVIEW.md records current source/receipt matching:
completed C# borrowed-bucket repair is A3 PROTECTED; A2 no-repair decision is closed
for its accepted scope, without fabricating an A2 optimization receipt. No repeat
A2 work is queued. #173 fact-multiset equivalence does not establish exact collector
order after earlier forward HashMap insertions. The directive's original-position
demand-prepass fallback is returned for owner review with explicit costs and a
narrower zero-no-family invariant. No production repair or runtime was performed.


## Collector slice final composed GREEN

Agent MaxHeadRoom inspected af-perf-001-20261104-144311:42 tracked passes,
zero failures/ignored across14 admission/AST/lifecycle,5 resolver P0,6 delegated,
4 A3,9 af174 and4 NgRx collector controls. All recorded source hashes match;
inspection.json records exact hash count and group results. No production/test
source changed after verification; this update is evidence prose only.

Approved no-family collector repair disposition: FIX / CLOSED / PROTECTED.
Original-position prepass removes proven unused collection while preserving demanding
construction, binding order/multiplicity, matching, export validation and immediate
reconciliation. Genuine original budget RED, unchanged GREEN, positive/negative/
duplicate/currentness controls, finite static enforcement, real isolated eager
mutation RED/restored GREEN and final composed fitness GREEN are all recorded.
No enterprise latency, allocator-byte or whole-system certification claim follows.

This closes only the approved collector slice. #173 M*B joins and reconstruction
scheduling remain unresolved; no deferral/index/cache change is authorized. A2's
no-repair decision remains closed and A3 protected. Source-wide discovery, A1/A4,
declaration membership and remaining inventory gaps are independent. No automatic
repeat campaign, binary rebuild or Claude field switch is needed for this result.


## Compression/compaction scoped discovery begins

AF_PERF_002_COMPRESSION_DISCOVERY.md traces the normal public IR compression path
separately from legacy capture/compaction exports. A same-operation raw-token recount
candidate is source grounded; a public characterization packet is authored UNRUN.
Configured alias matching is a separate T*A structural risk, including delegated
IR type formatting. No repair applied; full compression/compaction coverage remains
incomplete and syntax accounting unchanged. Protected A2/A3/collector work is not
reopened. CBM degraded; bounded known-source inspection used.


## Compression C1 measured duplicate-count candidate

compression-work-20261104-145210 passes one public-path growth test and six existing
economics controls, with clean focused gates and matching source hashes. Each raw
source has one handler count request and one economics recount:2*S observed bytes.
This is not measured tokenizer CPU or attributable total latency. Proposed local
successful-handler wiring reuses its existing raw_tokens via the already-counted
selector; no new API/cache/context. Owner disposition required before repair and
genuine regression RED. C2 alias matching and full compaction coverage remain open.


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


## C2 alias characterization packet authored

AF_PERF_002_C2_ALIAS_CHARACTERIZATION.md traces active MCP post-compilation operand
rewrites and distinguishes optional/retired capture callers. Test-only counters
measure setup/sort/scan/probe/key/match work and actual IR/public totals. Three
new tests and31 existing alias controls are selected by the new operator packet,
all UNRUN. No alias algorithm, state/API, generation or cache change. Existing
protected repairs are not reopened; source accounting remains independent.


Hierarchy owner-placement discovery: active checked projection still linearly scans
materialized classes for method/field placement. Independent class/member axes and
ordered identity characterization are authored UNRUN; no production repair. See
AF_PERF_002_HIERARCHY_DISCOVERY.md. Renderer overload counting has a distinct semantic
consumer. Whole-subsystem coverage and syntax disposition totals remain unchanged.


Hierarchy characterization hierarchy-work-20261104-163440 verified13 passes and
focused gates with matching source hashes. Method/field placement comparisons equal
K*C*(C+1), reaching65792 at256 classes. Existing-map lookup substitution proposed
only, pending owner disposition; public multiplicity and corrected costs unmeasured.
See AF_PERF_002_HIERARCHY_DISCOVERY.md. Syntax accounting remains unchanged.


Hierarchy placement repair:145 focused passes and clean affected gates verified
(hierarchy-focused-20261104-171027); isolated real one-site scan reintroduction
fails budget/static and exact restoration passes both (hierarchy-mutation-20261104-171239).
Approved scope only: two existing-map lookups; composed mandatory gate pending.
All other hierarchy/discovery gaps and syntax accounting remain explicit.


Hierarchy two-site placement scope FIX / CLOSED / PROTECTED: final composed
af-perf-001-20261104-172427 passes49 tracked executions with212 matching
source hashes, following focused145 and isolated mutation RED/restored GREEN.
See AF_PERF_002_HIERARCHY_DISCOVERY.md. Broader hierarchy coverage/discovery remains
open; original9763 syntax accounting and unresolved scope are unchanged.


Graph traversal slice: public dependency query preparation and answer traversal
straddle hydration; narrow reachability builds a complete sorted node table.
Independent unrelated/invocation and negative/depth controls authored UNRUN, no
repair. See AF_PERF_002_GRAPH_TRAVERSAL_DISCOVERY.md. Cycle witness determinism
remains a distinct complete-graph obligation; inventory totals unchanged.


Traversal-work-20261104-174619 verified38 tracked passes, focused gates and matching
source hashes. Fixed reachable output still pays1031 input keys/515 unique nodes
with256 unrelated edges; absent/sink starts do zero setup. Local direct-key BFS
candidate requires separate design approval and dense/scoped semantic comparison.
See AF_PERF_002_GRAPH_TRAVERSAL_DISCOVERY.md; no optimization implemented.
Syntax accounting and broader discovery incompleteness remain unchanged.


Traversal-density-20261104-175647 verified39 passes, clean affected gates and
matching hashes.42 shape/depth rows plus scoped removal/restoration controls pass.
Dense256/unlimited expands65280 edges for255 results while setup remains768 keys;
depth1 expands255 with the same setup. Direct-key replacement remains DESIGN REVIEW
for construction-inclusive hashing/memory tradeoffs; no repair/prototype implemented.
No new syntax dispositions or exhaustive graph/public coverage are claimed.


Proxy default request pipeline traced (server -> Pipeline -> transform -> scrub).
Length-only replacement acceptance is a source-grounded correctness risk: equal-
length redaction can be discarded. Two synthetic tracked regressions authored
UNRUN, no production correction. See AF_PERF_002_PROXY_SCRUB_DISCOVERY.md.
Transport/logging/other proxy paths and syntax classification remain open.


Proxy equal-length scrub defect confirmed:proxy-equal-length-before-20261104-180252
executes two intended direct/pipeline correctness failures, no real credentials.
Exact content change detection proposed; production unchanged pending disposition.
See AF_PERF_002_PROXY_SCRUB_DISCOVERY.md. This correctness finding does not close
proxy performance/transport/logging coverage or advance syntax accounting.


Proxy scrub exact-content comparison owner-approved and implemented at the one
change-detection predicate; rules/accounting/pipeline unchanged. Original two RED
cases retained; four additional byte/order/idempotence controls and focused52-test/
isolated predicate mutation packets authored UNRUN. No protected acceptance yet.
See AF_PERF_002_PROXY_SCRUB_DISCOVERY.md; other audit work remains independent.


Proxy scrub correction focused52 GREEN verified181341; isolated old-predicate
mutation181501 gives original two-case RED and exact restored six-case GREEN.
Source identities match; composed mandatory gate pending. Exact comparator only;
other proxy rules, scope and pipeline contracts unchanged. No audit completeness
or further syntax disposition is inferred.


Proxy scrub exact-content correction FIX / PROTECTED: final composed
af-perf-001-20261104-193130 passes55 tracked executions, zero failures/ignored,
219 source hashes match. Original two-case RED, focused52 and isolated
old-predicate RED/exact restored six-case GREEN retained. Scope is one predicate
and comment; no rule/policy expansion. See AF_PERF_002_PROXY_SCRUB_DISCOVERY.md.
Broader proxy/audit coverage and original syntax accounting remain incomplete.


Adjacent proxy prefilter obligation: existing optional-separator/pwd assignment/
flag and non-Bearer Authorization rules may be excluded by the cheap gate.
96-case finite regex-prerequisite table and exact pipeline regression authored
UNRUN; no production correction. Protected comparator remains unchanged. See
AF_PERF_002_PROXY_PREFILTER_DISCOVERY.md; syntax totals unchanged.


Prefilter195317 verifies two correctness REDs:31/96 actual existing-rule cases
blocked, exact pipeline propagation fails; source hashes match. Eight roots-only
additions proposed pending separate approval. No matching policy/comparator change.
Unicode/full-rule-language coverage stays open; inventory accounting unchanged.


Approved prefilter eight-root correction implemented, post-repair UNRUN. Frozen
source comparison confirms roots-only diff and unchanged protected comparator.
Original96 cases retained; exact standalone/pipeline96 and16 nonmatching-keyword
controls plus focused66/eight-root isolated mutation packets authored. Unicode
gap remains separate; no new rule or normalization. See proxy prefilter report.


Prefilter focused200149 verifies66 passes, all96 admissions/exact standalone-
pipeline outputs and16 negatives; clean focused gates. Mutation200235 removes each
of eight approved roots independently: eight intended REDs, exact restored four
GREEN; source hashes match. Final composed gate pending. Unicode scope remains
explicitly unresolved and syntax accounting unchanged.


Eight-root prefilter scope FIX / PROTECTED: final composed
af-perf-001-20261104-201445 passes59 tracked executions with220 matching
source hashes; original31-case RED, focused66 GREEN and eight independent root
mutation REDs/exact restored GREEN retained. Comparator/rules unchanged. Unicode
equivalence stays separately unresolved; broader discovery and syntax accounting
remain incomplete. See AF_PERF_002_PROXY_PREFILTER_DISCOVERY.md.


Java/Spring default compile path traced: existing once-detection/combined evaluation
protection preserved. Inner method-annotation calls/bytes and detection AST attempts
authored for independent class/method axes with exact Spring output/provenance and
public publication control. UNRUN, no optimization; see AF_PERF_002_SPRING_DISCOVERY.md.
Properties/other Java consumers and source-wide syntax classification remain open.


Spring-work-20261105-171440 verified13 tracked passes, clean affected gates and
matching source hashes. Positive High helper calls2*C, Low0; method growth increases
body bytes/output, not number of helper passes. Detection/evaluation remain once per
compilation; plain Java avoids detector parsing/evaluation. Inner scans MONITOR, no
measured multiplicative selection defect or repair proposal. Full Spring coverage
and9750 pending syntax records remain open; see AF_PERF_002_SPRING_DISCOVERY.md.


SQLite canonical/durable replay traced through metadata/owner/binary/delta/snapshot
loading and historical adoption. Independent instruction/delta axes and exact
version/IR controls authored UNRUN; helper query/decode/payload/tuple counters only.
Duplicate owner reads and possible presentation recount remain source leads with
consistency/observation obligations, no repair. See AF_PERF_002_SQLITE_REPLAY_DISCOVERY.md.


SQLite-work-20261105-172853 verifies27 passes, clean affected gates and matching
source hashes. Named canonical queries stay1 baseline/1 owner; durable baseline
1 baseline/2 owner. Delta rows/payload grow with history, output tuples with IR.
Per-delta candidate clones/validation/index rebuilds are outside output-copy counter
and remain separate measured-work obligations; whole replay MONITOR, no repair.
See AF_PERF_002_SQLITE_REPLAY_DISCOVERY.md; no syntax completeness advance.


Owner accepts bounded replay/durable lookups; no optimization queued. Independent
intermediate clone/validation/index/occurrence work and late-failure atomicity
packet authored UNRUN. SQLite temporary owner and public delta_apply outer candidate/
persistence/publication boundaries traced separately; no reuse or mutation-in-place.
See AF_PERF_002_REPLAY_INTERMEDIATE_DISCOVERY.md; final-output clones kept separate.


Replay-intermediate-work-20261105-174640 verified63 tracked passes, clean focused
gates and source identities. Intermediate cloning/full validation/index visits each
D*N, prefix checks2*D*P. Seven late/early refusal controls preserve exact owner/index/
version/hash state. Candidate isolation and index-maintenance obligations ACCEPT
with current representation; whole revalidation/prefix work MONITOR, no redundant
small fix proved. Final-output clone explicitly distinct; lookup acceptance unchanged.


## Replay owner closure and scheduling boundary

The executed intermediate replay packet is CLOSED / NO REPAIR. D*N evidence and
seven refusal controls remain retained, not a mandate to remove all complete-set
work. Future unchanged-region validation and identical occurrence-prefix questions
are deferred MONITOR obligations. Any stateful/incremental/copy-on-write/transactional
alternative needs separate architectural approval; no implementation queued.

Current discovery priority returns to other unexamined production paths. Top-level
rows now distinguish the named measured/protected slices from still-uninspected
subsystem scope; historical append-only evidence remains below its original dates.
Syntax accounting is unchanged and whole-system discovery remains incomplete.


Observability source trace separates tracing/startup/optional exporter from actual
context_stats SessionStats path. Public JSON appears to build unused full text
dashboard before format selection; file/invocation axes and read-only controls
authored UNRUN, no repair. See AF_PERF_002_OBSERVABILITY_DISCOVERY.md. Replay stays
closed; export/transport/concurrency and source-wide syntax accounting remain open.


Stats-work-20261105-201544 verified six passes, clean focused gates and matching
source hashes. Public session-wide JSON constructs one unused initial full text
dashboard per invocation (F256:20611 base bytes); Q growth multiplies it. Format-
guarded construction proposed only, preserving text lock placement and all JSON/
DB/proxy/cache observations. No repair approval inferred; other audit scope open.


A1 default publication characterization verified six passes / 63 operation rows,
1313 source hashes matched at inspection; real save/acquisition failure controls
pass. Copied item/string payload is measured at named clone sites, not allocator
traffic or whole graph memory. Current ownership-copy costs are ACCEPT/MONITOR;
handler durable_candidate elimination is separately proposed for owner review.
No repair implementation or syntax disposition follows from these observations.


A1 handler-only redundant durable-candidate clone is now FIX / PROTECTED after
original RED, exact durable/public GREEN, finite AST and real mutation RED/restored
GREEN, focused gates and final65-pass/230-hash composed verification. This closes
only that correction; distinct publication-owner copies retain ACCEPT/MONITOR and
uncounted proof/operand bytes, concurrency, A4 and broader source inventory stay open.

A4 original-input lookup correction CLOSED / FIX / PROTECTED: single-operation
borrowed seed/family retrieval replaces three full-input search sites, preserving
ordered duplicates, exact base predicate and path-sensitive ancestry. Original RED,
complete frozen/public semantics and work GREEN, finite static,3independent real
production mutations RED/exact restored GREEN, focused9plus checks and final
70pass/236hash composed receipt af-perf-001-20261106-152751 verified. Construction
comparisons/reference capacities measured; no public latency/allocator claim.
Peer matching, broader A4 paths/concurrency and source census remain separate.
No additional frozen syntax disposition follows from57operation rows or this fix.
Default structural edit relocation now source-traced, characterization authored
UNRUN: registered apply_edit uses UnitTable declaration attachment and syntax-to-
editable joins before staged durable commit. Actual U/D/S/Q work and ownership/
currentness observations scoped in AF_PERF_002_EDIT_RELOCATION_DISCOVERY.md.
No optimization or syntax disposition yet; A4 original retrieval remains closed.
Edit relocation characterization GREEN: edit-relocation-work-20261106-154640,
16tracked tests,15public rows,1323matching source hashes; all focused checks pass.
U256 single edit performs65536attachment and33152editability predicates. Field
population independently grows failed membership scans; Q changes resolution not
table count. Exact byte/span/durable/live and refusal/failure controls GREEN.
FIX PROPOSED: one call-local exact-end bucket map preserving ordered duplicate
captures plus maximum body start for exact existential editability. Construction,
small/same-end density costs and original RED/frozen oracle remain required before
production edit. No optimization applied or latency/allocator benefit claimed.
A4 stays closed; no reconfirmation runtime, source syntax disposition or commit.
Structural edit exact-end joins CLOSED / FIX / PROTECTED. Final composed receipt
af-perf-001-20261106-170706:75tracked passes/22groups/zero failed or ignored,
242recorded hashes match; both production identities unchanged from focused and
restored mutation. Original RED, exact frozen/public semantic/work GREEN, focused29,
real independent mutation4RED/2semantic passes/restored5GREEN, corrected paired-cost
review all retained separately. U256 eliminates65536/33152old predicates with full
setup/comparison/capacity accounting. No public latency/allocator claim. No code/test
change after acceptance, no new execution to reconfirm, no agent stage/commit/push.
Other edit paths and syntax census13/9763 scoped,9750unresolved remain independent.