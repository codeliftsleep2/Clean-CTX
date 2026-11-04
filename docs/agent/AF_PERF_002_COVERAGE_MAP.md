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
| SQLite: mcp/sqlite_store/semantic_state.rs, replay.rs | INSPECTED, named save/load paths only | Save checks authority before begin transaction, clones snapshot authority, serializes/persists then commits/rolls back. Durable load calls metadata/context-ID/replay/snapshot queries; replay looks up context ID again and decodes/reconstructs IR. Counts/bytes not yet measured. Other SQLite entry points NOT INSPECTED. |
| Query/hydration: mcp/tool_handlers/query/prepare.rs, hydration | INSPECTED, prior finite census | Catalog/discovery reuse differs from retained owner currentness. Need per-entry-point graph and negative/unchanged/adversarial workloads. |
| CBM: cbm/bridge/indexing.rs | INSPECTED, project indexing invocation only | Project-level external indexing and caller multiplicity need characterization. External provider contract unchanged. |
| Canonical IR: ir/compiler, pipeline, hierarchical, replay, binary_wire | NOT INSPECTED system-wide | Module roots identified; enumerate real compilation/edit/restore callers, graph/clone/encode growth. SQLite replay observation alone is insufficient. |
| Provenance/graphs: ir/program_graph, workspace/index/traversal.rs | NOT INSPECTED system-wide | Cycles, repeated closure, sparse/wide/deep graph queries, attribution and output-sensitive work. |
| C#/EF: dotnet_meta, ir/csharp_resolution | NOT INSPECTED in current systematic pass | Historical correctness/performance evidence is not current exhaustive path coverage. |
| Spring/Java: spring_meta, layers/language | NOT INSPECTED | Default/delegated extraction, parsing, config and dependency multiplicity. |
| Compression/compaction: compression/pipeline.rs, compaction | NOT INSPECTED | Entry module exports located; per-candidate extraction, rendering, allocation and reduction remain untraced. |
| MCP/protocol/proxy: mcp, protocol, proxy_spawner | NOT INSPECTED system-wide | Transport filtering, envelopes, copies, cancellation, task queues, secret scrubbing and errors. |
| Other edit/diff/cache/analytics/intelligence/dictionary/gitdiff paths | NOT INSPECTED | Enumerate external/default callers before classifying work. No blanket historical exemption. |
| Observability: observability/metrics, tracing, startup | NOT INSPECTED | Logging serialization/allocation, repeated diagnostics and startup/error amplification. |

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
