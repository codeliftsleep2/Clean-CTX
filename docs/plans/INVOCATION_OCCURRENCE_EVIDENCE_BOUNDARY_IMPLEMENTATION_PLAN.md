# Invocation Occurrence Evidence Boundary Implementation Plan

**Author:** Agent MaxHeadRoom  
**Date:** 2026-10-10  
**Status:** Phases 0–3 implemented; focused verification recorded below

**Authoritative predecessor:** docs/architecture/SEMANTIC_EVIDENCE_SUFFICIENCY_AND_RECOVERABILITY_INVESTIGATION.md

## 1. Executive design verdict

**PROPOSED — freeze a private, typed invocation-evidence side channel assembled from the existing capture walk and valid only for one source identity and SHA-256 source hash.**

The design preserves CoreOp::Call, CallEvidence, canonical encodings, semantic edges, WorkspaceIndex, snapshots, producer generations, and every public MCP schema. It adds no resolution or framework meaning.

The existing PassContext is the assembly owner because it already owns the source, captures, callable scopes, and CallProducer during one compilation (src/ir/pipeline.rs, PassContext; src/ir/pipeline/core.rs, CoreIrPass::run). Durable CompiledIR is explicitly rejected as the owner because it participates in replay, delta, wire, and persistence contracts (src/ir/compiler.rs, CompiledIR; src/ir/replay.rs, load_ir).

After compilation, occurrences move into a separate InvocationEvidenceSet carrying the canonical source owner, exact SHA-256 hash, and occurrence records. It remains an IRCompiler side output in the first slice, analogous in lifecycle but not meaning to semantic_edges. It is never inserted into CompiledIR or durable state.

The two proof consumers are:

1. **Generic:** internal tests consume receiver, spans, arguments, and nesting without framework knowledge.
2. **Domain:** RxJS .pipe(...) discovery and top-level operator-argument boundaries consume the shared evidence. RxJS retains operator interpretation, rendering, filtering, and pipe-owner behavior.

The implementation-level falsification passes. TypeScript and C# need different adapter captures but the same written-evidence contract; generic and RxJS consumers need the same root/receiver/argument/span facts; no second production parse, persistence, public API, or semantic field is required.

## 2. Authoritative predecessor findings

These are baseline, not reconsidered:

- **PROVEN BY CODE:** tree-sitter encounters richer invocation structure than CoreOp::Call retains.
- **PROVEN BY CODE:** CallProducer temporarily uses callee byte spans and individual argument captures, then emits only caller, terminal callee, count, and spread (src/ir/calls.rs: PendingCall, record_callee, record_argument, settle).
- **PROVEN BY CODE:** NgRx and RxJS reconstruct invocation mechanics from raw text (src/angular_meta/ngrx/extract.rs, extract_selectors.rs; src/angular_meta/rx/pipes.rs).
- **AUTHORITATIVE PREDECESSOR:** Decision C requires this narrow structural boundary before limited resolution extraction.
- **AUTHORITATIVE PREDECESSOR:** generalized synthesis, durable occurrence graphs, compiler-grade resolution, protocol bridges, and broad producer migration remain unauthorized.

## 3. Exact scope

The first implementation slice will:

- define private structural occurrence types;
- add TypeScript and C# capture adapters to the existing combined query;
- assemble occurrences from the same Vec<CapturedNode> returned by run_capture_pipeline_nodes;
- bind completed evidence to one source owner and SHA-256 hash;
- prove a generic internal consumer;
- migrate only RxJS .pipe(...) occurrence and ordered operator-argument discovery;
- retain existing RxJS semantic and rendered results;
- stop for architectural review.

## 4. Non-goals

- No declaration, receiver-type, import, overload, or member resolution.
- No NgRx Dispatches, HTTP, DI, endpoint, or persistence implementation.
- No Java or Rust production adapter in the first slice.
- No generalized occurrence registry, AST model, evidence bus, or synthesis engine.
- No durable occurrence identity across edits.
- No canonical opcode, encoding, snapshot, delta, producer generation, graph, or public MCP change.
- No migration of all RxJS or NgRx text extraction.
- No assertion that a method name implies framework meaning.

## 5. Current call-pipeline trace

| Stage | Current authority | Evidence |
|---|---|---|
| src/queries.rs call queries | Terminal callee and individual arguments; TS spread is distinct; receiver omitted | PROVEN BY CODE |
| capture_pipeline::run_capture_pipeline_nodes | One parse; raw/processed text, byte spans, per-walk match identity, stable source order | PROVEN BY CODE |
| CoreIrPass::run | Appends call query and walks all captures once | PROVEN BY CODE |
| PassContext | Owns source, file identity, captures, callable scopes, CallProducer | PROVEN BY CODE |
| CallProducer | Joins duplicate matches by callee span, counts args, records caller | PROVEN BY CODE |
| CoreOp::Call | Caller id, written callee, count, spread only | PROVEN BY CODE |
| semantic_projection | Neutral written-name Calls edge with count/spread evidence | PROVEN BY CODE |
| calls_in_file | Recompiles current source; returns callee/count/spread and ordinal | PROVEN BY CODE |
| Angular MetaLayerPass | Gets source/captures; RxJS scans .pipe( and splits text | PROVEN BY CODE |

The side channel is assembled after run_capture_pipeline_nodes returns and before MetaLayerPass executes. It does not alter the sequential CallProducer path.

## 6. Frozen v1 occurrence contract

The crate-private production shape is:

    SourceSpan {
        start_byte: usize,
        end_byte: usize,
    }

    ArgumentQualifier =
        Plain
        | Spread
        | Named { name: SourceSpan }

    ArgumentOccurrence {
        span: SourceSpan,
        qualifier: ArgumentQualifier,
    }

    WrittenCallee =
        Free { terminal: SourceSpan }
        | Member { receiver: SourceSpan, terminal: SourceSpan }
        | Unsupported { function: Option<SourceSpan> }

    DeclarationContextKind =
        Callable
        | VariableInitializer
        | FieldInitializer
        | PropertyInitializer

    DeclarationContext {
        kind: DeclarationContextKind,
        declaration: SourceSpan,
        written_name: Option<SourceSpan>,
    }

    ParentSite =
        Receiver
        | Argument { index: usize }

    InvocationParent {
        invocation: SourceSpan,
        site: ParentSite,
    }

    InvocationOccurrence {
        span: SourceSpan,
        callee: WrittenCallee,
        arguments: Vec<ArgumentOccurrence>,
        type_arguments: Vec<SourceSpan>,
        declaration_context: Option<DeclarationContext>,
        parent: Option<InvocationParent>,
    }

    InvocationEvidenceSet {
        source_id: String,
        source_hash: String,
        occurrences: Vec<InvocationOccurrence>,
    }

Production types use constructors and read-only accessors where necessary to prevent invalid spans or unsorted occurrences.

Frozen semantics:

- Spans are zero-based half-open UTF-8 byte offsets [start_byte, end_byte).
- They are not character, Unicode scalar, UTF-16, column, or LSP positions.
- Snapshot-local identity is source_id + source_hash + invocation span.
- Sort by start_byte then end_byte descending, so an equal-start containing call precedes its child.
- No durable cross-edit occurrence identity exists.
- Unsupported records an invocation without inventing a terminal name.
- Declaration context is source containment, not semantic ownership.
- Parent records only immediate invocation containment through receiver or argument.
- V1 has no general expression-kind enum. Plain, Spread, and C# Named are the only justified argument distinctions.
- Records own no source substring. Text is sliced lazily after source validation.

## 7. Field-by-field authority table

| Field | Source authority | Capture | Consumer | Why required |
|---|---|---|---|---|
| source_id | canonical path or file id | PassContext | lifecycle | Prevent same bytes under another owner |
| source_hash | exact bytes | existing SHA-256 | all | Prevent stale span use |
| invocation span | invocation node | invoke.root | generic/RxJS | Occurrence identity and slicing |
| free terminal | identifier | adapter | generic | Written callee |
| receiver | object/expression | adapter | generic/RxJS | Written subject |
| member terminal | property/name | adapter | generic/RxJS | Written pipe/save |
| arguments | grammar nodes/order | adapter | generic/RxJS | Retire body splitting |
| qualifier | spread/named syntax | adapter | generic | Preserve syntax truth |
| type arguments | grammar nodes | adapter | generic | Preserve generic call evidence |
| declaration context | enclosing declaration spans | adapter/containment | generic | Avoid synthetic Method owner |
| parent/site | smallest containing invocation | containment | generic/RxJS | Preserve nesting/argument ownership |

No field carries a resolved target, receiver type, framework kind, confidence, endpoint, persistence operation, or causal relation.

## 8. Resolution of B1-B5

### B1 — Argument representation

**FROZEN: Option A, span plus qualifier, with lazy slicing from validated source.**

Reasons:

- CapturedNode and tree-sitter already use UTF-8 byte spans.
- Owned text would duplicate potentially large closures, objects, arrays, and templates.
- RxJS needs complete ordered slices and can obtain them after validation.
- Diagnostics can render exact source without normalization.
- Range validation is cheap and allocation remains consumer-controlled.

InvocationEvidenceSet::view(source_id, source) verifies source identity and SHA-256 before returning a borrowing InvocationEvidenceView. Slice helpers exist only on that validated view. Invalid boundaries return a typed error and never panic or truncate.

### B2 — Required v1 context

| Candidate | Decision | Reason |
|---|---|---|
| Nested invocation | REQUIRED V1 | T2 and RxJS nesting |
| Invocation argument index | REQUIRED V1 | T2/T3 and pipe argument ownership |
| Initializer context | REQUIRED V1 | Truthful createEffect/inject-shaped ownership and T9 |
| Await context | DEFER | Neither proving consumer requires it |
| Return context | DEFER | No shared proof need |
| Ordinary assignment | DEFER | Direct initializer is sufficient |
| Callback/lambda ancestry | DEFER | Parent argument containment suffices for first proof |
| Control-flow ancestry | REJECT V1 | Would create a generic context model without a proving consumer |

### B3 — Initializer ownership

**FROZEN: Option<DeclarationContext>, never a synthetic callable.**

The nearest supported containing declaration is recorded by span containment. Direct variable, field, and property initializers also carry the target written-name span. Calls inside methods, functions, and supported bound arrows carry Callable declaration context. Calls without a supported declaration carry None and remain valid.

No DefMethod is created for an initializer. No semantic entity is encoded.

### B4 — Invocation-root capture

**FROZEN: extend language adapter queries and add a parallel occurrence collector over the same returned capture vector. Do not turn CallProducer into the evidence model.**

Each language adapter adds:

1. One broad root-only pattern for every grammar invocation node as invoke.root, including unsupported callee forms.
2. Detailed free/member callee, receiver, argument, qualifier, type-argument, and initializer captures.
3. Root capture on existing detailed call patterns where match identity is necessary.

The collector merges repeated details by invocation root span in a BTreeMap. Existing queries already create an argument-less match plus argument matches. Root merging keeps the same asymptotic behavior. The broad pattern adds one match per invocation, not per argument.

After merging, a containment pass assigns the smallest enclosing invocation as parent only when the child is within the parent receiver or one argument. It then assigns the nearest supported declaration context.

CallProducer remains sole CoreOp::Call authority. The collector remains sole structural-occurrence authority. Both consume the same parse.

### B5 — Ownership and lifecycle

- **Creator:** CoreIrPass invokes the collector after capture collection.
- **Assembly owner:** PassContext.invocation_occurrences during the pipeline.
- **In-pass readers:** MetaLayerContext borrows occurrences and the same state.source.
- **Post-compile owner:** IRCompiler.invocation_evidence: Option<InvocationEvidenceSet>, cleared at compile start and replaced only on success.
- **Identity:** canonical path where present, otherwise file_id.
- **Hash:** repository SHA-256 byte hash, factored into one pure helper shared by cache and evidence.
- **Movement:** the set owns no source borrow; a reader must bind matching identity and bytes before slicing.
- **Refresh/replacement:** no workspace store receives the set. An old caller-local set fails binding to new bytes/hash.
- **Deletion:** there is no current source to bind and no persisted set to retrieve.
- **Compile failure:** the prior compiler-local set was already cleared; no new set is exposed.
- **Mismatch:** a typed error exposes no source slices.

CompiledIR is not changed. Replay cannot accidentally present nonserialized evidence as current.

## 9. Ownership and lifecycle design

A new src/ir/invocation_evidence.rs owns spans, occurrence types, validated views, assembly, and tests registration. PassContext holds Vec<InvocationOccurrence>, not the bound set, while source is already owned by that exact compilation. IRCompiler forms the bound set at compile completion.

The existing LocalStateCache::compute_hash implementation is extracted mechanically into one crate-private pure SHA-256 helper. LocalStateCache delegates to it byte-for-byte. This avoids two hash authorities. It does not change hash format or cache semantics.

A validated view requires both source identity and bytes:

    evidence.view(expected_source_id, source) -> Result<InvocationEvidenceView, EvidenceMismatch>

The view lends exact slices for spans. Consumers cannot call a raw slicing method on InvocationEvidenceSet.

**PROPOSED:** computing the hash once in compile_inner and reusing it at the MCP helper is preferable, but changing the existing helper tuple is unnecessary for the first proof. If avoiding a duplicate hash would require widening production return contracts, accept one additional SHA-256 pass in v1 and measure before refactoring.

## 10. TypeScript capture design

TS_CALL_QUERY gains private invoke.* captures:

- call_expression root;
- identifier function terminal;
- member_expression object receiver and property terminal;
- arguments and spread_element spans;
- type_arguments children;
- direct variable_declarator initializer owner/name;
- direct public_field_definition initializer owner/name.

Existing call.callee, call.argument, call.spread, arrow.name, and arrow.root captures remain byte-for-byte authoritative for CoreOp::Call.

Unsupported examples such as computed nonliteral calls still receive invoke.root. If the adapter cannot prove a terminal name, WrittenCallee::Unsupported is emitted. Literal computed member support is not added merely because it is recoverable; it is a separate coverage decision.

Optional chains are included only if the TypeScript grammar adapter can capture root, receiver, and terminal without guessing. Otherwise they remain root-only Unsupported in v1.

Declaration context is found from supported captured declaration spans, not textual backward scanning.

## 11. C# capture design

CS_CALL_QUERY gains equivalent private captures for invocation_expression:

- root;
- identifier/generic-name free terminal;
- member_access_expression receiver expression and identifier/generic terminal;
- argument nodes in grammar order;
- argument name-field identifier for named arguments (tree-sitter-c-sharp 0.23);
- generic type-argument nodes;
- direct variable/field/property initializer declaration/name where grammar patterns prove them.

Existing call captures remain unchanged for canonical behavior.

Conditional access, casts, or parenthesized targets receive root-only Unsupported unless a specific adapter pattern proves receiver and terminal. No terminal is inferred from raw text.

C# named arguments are accepted into v1 because the grammar exposes the name separately and T6 provides direct preservation value. The name span excludes the colon.

## 12. Generic consumer proof

The generic proof is internal and framework-free:

- compile one TypeScript and one C# source through IRCompiler;
- take InvocationEvidenceSet;
- bind it to the matching source id and bytes;
- inspect exact invocation, receiver, terminal, args, qualifier, type arguments, parent, and declaration context;
- prove mismatched source hash and source id reject slicing;
- simultaneously assert existing CoreOp::Call and CallEvidence output.

No MCP response changes. calls_in_file is the likely later public consumer, but public occurrence fields require separate approval after this boundary gate.

## 13. Selected RxJS consumer proof

**Selected slice: RxJS .pipe(...) occurrence discovery and ordered top-level operator argument boundaries.**

Current authority in src/angular_meta/rx/pipes.rs:

- source.find(".pipe(");
- lexical-region filtering;
- collect_call_body;
- split_top_level;
- parse_operator over each textual segment.

Migrated authority:

1. Select InvocationOccurrence values whose WrittenCallee is Member with terminal slice exactly pipe.
2. Use the receiver span as written pipe subject evidence.
3. Use ordered argument occurrence spans as the operator-expression boundaries.
4. Slice each operator expression and retain RxJS-owned parse_operator/operator_to_kind interpretation.
5. Preserve existing owner resolution, min_pipe_operators filtering, RxShape, and rendering.

This proves reuse without imports, receiver-type resolution, action binding, HTTP, or persistence.

### Standalone RxJS compatibility

The source-only extract_rx_shape API has no upstream capture walk. It must not remain a second text authority.

**FROZEN:** factor the TypeScript occurrence collector behind two inputs:

- production: consume captures from the existing compiler parse;
- standalone compatibility adapter: perform one TypeScript parse solely because no parse exists, then invoke the same collector.

The production compilation path still performs exactly one parse. The standalone API performs one parse, not a second parse. Both use one occurrence authority. If this cannot be implemented without duplicating capture semantics, the Phase 5 stop condition fires.

## 14. Raw-text authority retirement plan

During RED/GREEN implementation only, tests may compare legacy and structural RxShape results.

Final Phase 5 state:

- Remove .pipe( substring discovery for pipe chains.
- Remove collect_call_body and split_top_level use from pipe-chain extraction.
- Retain these shared utilities for other still-authorized consumers.
- Retain parse_operator and operator_to_kind as RxJS domain interpretation.
- Retain pipe-owner behavior initially. If it still uses textual ownership, document it as outside the migrated slice rather than claiming it was retired.
- Do not fall back to textual pipe discovery when occurrence evidence is incomplete. Unsupported evidence yields no pipe chain plus explicit internal diagnostic/status where the caller supports it.
- Source-only API obtains structural occurrences through the same collector, not the old parser.

## 15. RED test matrix

All required regressions are tracked under src/tests and registered via normal path modules.

| ID | Source | RED assertion |
|---|---|---|
| T1 | this.userService.save(user) | full span, receiver, save terminal, ordered arg, callable context, no resolution |
| T2 | foo(bar(baz())) | three spans, immediate parents, argument indexes, deterministic order |
| T3 | this.userService.getAll().pipe(map(...)) | distinct calls, receiver containment, no flattened receiver/target |
| T4 | TS http.post<User>(...), C# generic call | type-argument spans only, no HTTP meaning |
| T5 | foo(a, ...items) | two ordered args, Spread qualifier, unchanged CoreOp spread |
| T6 | Save(user: dto) | Named qualifier and name span |
| T7 | await service.save(user) | occurrence preserved; await specifically asserted absent from v1 |
| T8 | return service.save(user) | occurrence preserved; return specifically asserted absent from v1 |
| T9 | const result = foo(); loadUsers$ = createEffect(...) | variable/field initializer contexts, no Method fabrication |
| T10 | pipe(switchMap(x => service.load(x))) | parent argument containment; callback ancestry specifically absent |
| T11 | foo(); foo(); | two source-span identities |
| T12 | if(a) foo(); if(b) foo(); | two occurrences; no generic control ancestry |
| T13 | FakeStore.dispatch(x) | structural call only; no NgRx edge caused by side channel |
| T14 | computed/dynamic target | root preserved, unsupported callee, no fabricated terminal |
| T15 | non-ASCII prefix/content | UTF-8 byte slicing round-trip |
| T16 | edited call | old set fails new hash binding |
| T17 | deleted source | no persisted current evidence; missing source cannot bind |
| T18 | existing call suite | CoreOp::Call, Calls, CallEvidence, calls_in_file unchanged |
| T19 | RxJS pipe parity | legacy expected RxShape equals new structural result |
| T20 | comment/string fake .pipe | no occurrence, no RxJS pipe |
| T21 | nested comma/object/template operator args | grammar boundaries remain intact |
| T22 | source-only Rx API | same result via one-pass compatibility collector |
| T23 | parse counter in production | exactly one tree-sitter parse |
| T24 | C# and TS contract parity | same shared fields without language-specific enum leakage |

T7, T8, and T10 deliberately pin deferral, preventing accidental claims that v1 preserves contexts it does not.

## 16. Existing regression preservation matrix

| Authority | Existing tests to preserve | Required outcome |
|---|---|---|
| CallProducer joining/count | src/tests/ir/calls.rs | unchanged |
| TypeScript calls/arrows/callbacks | calls_typescript.rs, calls_arrows.rs, calls_arrows_callbacks.rs | unchanged |
| C# calls | calls_csharp.rs | unchanged |
| Semantic Calls evidence | src/tests/ir/semantic_projection.rs | unchanged |
| Wire/binary/hierarchy/replay | existing IR round-trip suites | no new operand or serialized data |
| calls_in_file | MCP call-query tests | schema/results unchanged |
| RxJS extraction/rendering | src/tests/angular_meta/rx.rs, rx_lexical.rs | equal output |
| Angular meta production integration | pipeline_meta_layer and Angular MCP tests | equal output/edges |
| Hash/cache behavior | src/tests/cache.rs | identical SHA-256 strings |
| File-size guard | scripts/check-file-sizes.ps1 | every active code file <=615 lines |

## 17. Performance and memory analysis

No source substrings are stored per occurrence.

Approximate 64-bit v1 cost before allocator/layout optimization:

- Invocation fixed data and enum payloads: roughly 80–144 bytes.
- Each argument: roughly 24–32 bytes plus Vec allocation.
- Each type argument: 16 bytes.
- One source id/hash allocation per evidence set.
- Parent/declaration records are inline optional payloads.

At 10,000 calls and two arguments per call, expect low single-digit megabytes, not source-size multiplication. Exact size must be measured with size_of and a production-shaped fixture during implementation; no hard global cap is introduced in this phase.

CPU:

- No second production parse.
- Capture events increase by O(invocations + arguments).
- Assembly is O(captures log invocations) with BTreeMap, followed by deterministic containment. The containment algorithm must use a span stack or indexed sweep, not O(n²) pair scanning.
- SHA-256 may be computed one extra time in v1 if avoiding tuple churn is more invasive; measure before optimizing.
- Lazy slicing is O(1) validation/range access after one view-level hash check.

Compilation-result lifetime bounds memory. No workspace accumulation occurs.

## 18. Compatibility and generation analysis

| Surface | Impact |
|---|---|
| CoreOp::Call | None |
| CallEvidence | None |
| named/binary/hierarchical wire | None |
| delta/replay/snapshot | None |
| semantic edges/identities | None |
| WorkspaceIndex | None |
| producer keys/generations | None |
| MCP schemas | None |
| source hash format | None; shared helper preserves SHA-256 output |
| RxJS rendered behavior | Must be identical |

No producer generation changes because no durable semantic or marker output changes. Any observed output change stops the preservation migration unless separately approved as a bug correction.

## 19. Failure and unsupported behavior

| Condition | Frozen behavior |
|---|---|
| Partial/error parse | Emit only roots/details tree-sitter safely captures; make no completeness claim |
| Unsupported target | Unsupported callee with optional function span |
| Receiver absent | Free callee, not Unsupported |
| Receiver shape unsupported | Unsupported; never infer terminal/receiver from text |
| Malformed argument | Retain safely captured span; otherwise omit it; no count fabrication in side channel |
| Slice boundary invalid | typed EvidenceSliceError |
| Source id mismatch | typed SourceIdentityMismatch |
| Hash mismatch | typed SourceHashMismatch |
| Adapter lacks field | field remains absent/unsupported as defined; no cross-language guess |
| Compilation failure | no evidence set published |
| RxJS evidence unavailable | no textual fallback in migrated pipe slice |

No confidence score is introduced. The collection is evidence, not a completeness report.

## 20. Dependency-ordered implementation phases

### Phase 0 — Preservation freeze

Add RED tests and record current CoreOp/Calls/RxJS outputs. No production code.

### Phase 1 — Evidence types and lifecycle

Add private types, validated view, shared SHA-256 helper, PassContext storage, and IRCompiler side output. No language population and no consumer migration.

### Phase 2 — TypeScript collection

Extend TS query captures and populate occurrences in the existing parse. Cover T1–T5 and T7–T15. Preserve current call output.

#### Phase 2 working record — 2026-10-11

The owner explicitly authorized beginning Phase 2 on
`(feat)Semantic_Intelligence_Architecture_Boundaries`. The branch already
contains the Phase 0 frozen regressions and Phase 1 source-bound lifecycle
(commits `bbe82e2` and `fbbde3a`). This authorizes the local TypeScript collector,
not C# collection, RxJS migration, resolution, or durable/public contract changes.

Five additional tracked production-path regressions now live in
`src/tests/ir/invocation_evidence_typescript.rs`. They pin generic member calls,
plain/spread arguments, UTF-8 spans and field context; immediate receiver and
argument parents with variable context; callable/bound-arrow context and one
production parse; repeated/dynamic calls and lexical decoys; and exact rich
argument boundaries independently of presentation fidelity.

The owner authorized focused RED/GREEN verification. The TypeScript run observed
nine assertion failures and one canonical preservation pass. Only the regression
module was stashed during implementation and restored unchanged: all ten tests
now pass. Six source-bound lifecycle and 24 canonical TypeScript call tests also
pass. The later-phase C# and RxJS RED modules remain unchanged and outside scope.

TS_CALL_QUERY now supplies private invoke.* captures. The collector merges roots
by exact span, retains written callee/argument/type-argument evidence, and assigns
immediate invocation parents and nearest declaration contexts using containment
stacks. It consumes the existing capture walk without another parse. Private
captures bypass canonical emission and scope refresh. No public contract,
producer generation, semantic edge, or persistent index changed.

The existing arrow preservation comparison now excludes invoke.* alongside
call.* and arrow.* when projecting base captures; its exact base-capture equality
assertion is retained. This accounts for the approved private query extension
without changing the frozen TypeScript evidence assertions.

Final preservation checks: 33 arrow/callback tests passed; package Clippy with
all targets/all features and warnings denied passed. The full repository suite
was not run and remains reserved for CI. Phase 3 C# collection is next.

### Phase 3 — C# collection

Add C# adapter, named arguments, generic forms, and parity coverage. No Java/Rust expansion.

#### Phase 3 working record — 2026-10-11

The owner authorized C# collection and focused verification. Seven tracked C#
regressions failed their intended assertions before implementation. Only that
test module was stashed during production changes and restored unchanged.

The C# adapter extends CS_CALL_QUERY with private structural captures for all
invocation roots, supported free/member terminals and generic type arguments,
ordered whole arguments, named-argument names, callable declarations, and direct
local/field/property initializers. Initializer patterns require the grammar's
written equals token; property expression bodies are not initializer contexts.
Wildcard initializer children avoid rejected expression-supertype query patterns
while retaining the structural equals/name/parent proof. No source-text inference
is used. Conditional, cast, parenthesized and dynamic targets remain Unsupported.

The existing shared collector joins named arguments by query match and argument
span, preserving the colon-free name separately from the whole argument. C# now
uses the same single production parse, root merge, containment sweep, source hash
binding, and private side output as TypeScript. Canonical producers, generations,
public schemas, semantic edges and durable state are unchanged.

Verification: all seven unchanged C# evidence regressions passed after the fix;
six lifecycle, 19 canonical C# call, and ten TypeScript evidence tests passed.
Package Clippy passed with all targets/all features and warnings denied. The
single-parse assertion and shared occurrence types establish adapter parity.
The full suite remains CI-only. Phase 4 generic consumer proof is next.

### Phase 4 — Generic consumer proof

Consume the hash-bound set internally and prove stale-source rejection. Do not change MCP.

### Phase 5 — RxJS domain proof

Thread borrowed occurrences through MetaLayerContext, replace pipe discovery/argument splitting, add standalone compatibility collector, compare behavior, then remove migrated text authority.

### Phase 6 — Fresh-state and preservation readiness

Exercise edit/delete/hash lifecycle, production one-parse invariant, existing call and RxJS regressions, file-size policy, and architecture audit.

### Phase 7 — Decision gate

Classify the boundary as PROVEN, PROVEN WITH REVISIONS, TOO BROAD, TOO NARROW, LANGUAGE-LEAKING, or NOT WORTH SHARING. Stop before limited resolution extraction.

## 21. File and module change table

| Phase | File/module | Change | Preserved authority | New authority | Tests | Compatibility |
|---|---|---|---|---|---|---|
| 0 | src/tests/ir/invocation_evidence*.rs | RED contract | Existing call tests | None | T1–T18 | None |
| 0 | src/tests/angular_meta/rx_invocation_evidence.rs | RED parity | RxJS expected behavior | None | T19–T23 | None |
| 1 | src/ir/invocation_evidence.rs | Types, view, assembly helpers | Canonical IR | Source-proven occurrences | unit tests | Private |
| 1 | src/cache.rs or small hash module | Extract pure hash helper; delegate existing method | SHA-256 strings | Shared hash authority | cache tests | None |
| 1 | src/ir/pipeline.rs | Add ephemeral occurrence Vec | Pass ordering | Compilation-local storage | lifecycle tests | None |
| 1 | src/ir/compiler.rs | Clear/take bound side output | CompiledIR | Hash-bound set | compiler tests | None |
| 2 | src/queries.rs | TS invoke.* captures | call.* captures | TS structural evidence | TS RED | None |
| 2 | src/ir/pipeline/core.rs | Invoke collector on same captures | CallProducer flow | Side-channel population | parse-count tests | None |
| 3 | src/queries.rs | C# invoke.* captures | call.* and C# semantic captures | C# structural evidence | C# RED | None |
| 4 | src/tests/ir/* | Generic proof consumer | no public query | internal read proof | T1–T18 | None |
| 5 | src/layers/meta/context.rs | Borrow occurrences | existing meta inputs | shared structural input | integration tests | Private |
| 5 | src/ir/pipeline/meta_layer.rs | Thread borrow | meta ordering | no new meaning | production test | None |
| 5 | src/angular_meta/rx/extract.rs | Evidence-aware pipe extraction | RxJS results | structural input use | parity | None |
| 5 | src/angular_meta/rx/pipes.rs | Retire .pipe text discovery/splitting | RxJS interpretation | one structural authority | Rx tests | None |
| 5 | src/angular_meta/mod.rs / angular_semantic.rs | Thread precomputed invocation evidence | existing Angular results | none | Angular tests | None |

File-size discipline:

- New files stay under 600 lines.
- pipeline.rs and tool_helpers.rs are already near the active-file target; avoid adding large implementations there.
- Put types/assembly in the new invocation_evidence.rs.
- If queries.rs approaches 600 lines, split invocation adapter query constants into a semantically named submodule rather than exceeding 615.

## 22. Candidate invariant decisions

| Invariant | Decision | Reason |
|---|---|---|
| INV-1 CoreOp::Call remains compact neutral summary | ACCEPT | Different concern and compatibility boundary |
| INV-2 occurrences record syntax, never declaration/framework meaning | ACCEPT | Rust types exclude resolved/domain fields |
| INV-3 valid only for producing source identity/hash | ACCEPT | Validated view enforces both |
| INV-4 domain producer stops reconstructing equivalent parser-known structure once available | ACCEPT WITH QUALIFICATION | Applies only after equivalent shared evidence reaches that producer; legitimate domain parsing remains |
| INV-5 domain interpretation remains domain-owned | ACCEPT | RxJS still maps operators; future NgRx owns meaning |
| INV-6 spans are not durable identity across edits | ACCEPT | Identity includes hash and no persistence |
| INV-7 unsupported is omitted/explicit, never guessed | ACCEPT WITH QUALIFICATION | Root occurrence remains; unavailable substructure is explicit, safely uncaptured malformed children may be absent |
| INV-8 no durable/index/snapshot/public relation changes | ACCEPT | Structural scope of first slice |

If implementation succeeds, document these in docs/ARCHITECTURAL_INVARIANTS.md only at finalization, using the existing invariant process. Do not create a new invariant registry.

## 23. Cargo and test authorization plan

The agent must not run these commands. Every command below **REQUIRES USER AUTHORIZATION BEFORE EXECUTION** and should be handed to the user at its phase.

Targeted RED/GREEN examples, finalized after test module names exist:

    cargo test --all-features ir::calls::invocation_evidence
    cargo test --all-features ir::calls::typescript_invocation_evidence
    cargo test --all-features ir::calls::csharp_invocation_evidence
    cargo test --all-features angular_meta::rx::invocation_evidence

Focused preservation:

    cargo test --all-features ir::calls
    cargo test --all-features ir::semantic_projection
    cargo test --all-features angular_meta::rx
    cargo test --all-features ir::pipeline::meta_layer

File-size and formatting checks are also user-run under repository policy. The authoritative final gate must be read from docs/agent/verification.md at finalization rather than copied from this planning artifact.

Do not run the full suite after every phase. Run focused RED/GREEN per phase, then the complete authorized gate once after migration cleanup and architectural audit.

## 24. Stop conditions

Stop and return for architectural approval if:

1. TypeScript and C# require incompatible shared fields.
2. RxJS needs framework fields inside the shared occurrence.
3. Production requires a second parse.
4. Lifecycle safety requires durability or WorkspaceIndex storage.
5. CoreOp::Call or canonical output must change.
6. RxJS proof requires broad imports/type resolution.
7. Public MCP changes are required for proof.
8. The contract grows toward a generic AST.
9. Source-only RxJS compatibility cannot use the same collector without a second authority.
10. Existing RxJS results cannot be preserved exactly.
11. Capture merging becomes nondeterministic or O(n²).
12. Any modified code file would exceed 615 lines without semantic decomposition.

## 25. Open questions

No architectural blocker remains. These implementation checks stay explicit:

- **UNRESOLVED until RED prototype:** exact tree-sitter-c-sharp query field names for property/field initializer and named-argument captures.
- **UNRESOLVED until RED prototype:** whether TypeScript optional-chain grammar can provide full member details in v1; root-only Unsupported is acceptable.
- **PROPOSED:** use a sweep/stack for parent containment; exact data structure may be chosen locally if deterministic and nonquadratic.
- **PROPOSED:** one extra SHA-256 computation is acceptable in v1 if reusing the MCP-computed hash would widen unrelated return contracts.
- **DEFERRED:** import binding, DI property/type binding, await/return/ordinary assignment, callback identity, Java/Rust adapters, public calls_in_file exposure.

These do not change the frozen public/architectural boundary. If the C# grammar cannot prove the accepted named/initializer fields with narrow query patterns, stop with READY WITH EXPLICIT OPEN DECISIONS rather than use text extraction.

## 26. Architectural falsification and final implementation gate

### Strongest case against the side channel

Extending CoreOp::Call would put all call evidence in one result; producer-local access would avoid a new compiler output; spans require source coordination; language grammars differ; occurrence volume may be large; and RxJS can already parse text successfully.

### Why it does not defeat this design

- CoreOp enrichment would immediately alter canonical arity, wire/binary/hierarchical encoding, replay, delta, snapshots, compatibility, and storage.
- Producer-local access perpetuates the proven duplicated reconstruction and cannot serve generic calls.
- Identity-plus-hash validated views make span/source coupling explicit.
- Shared types contain only common written facts; grammar mechanics remain adapters.
- No source strings or workspace persistence are added, bounding memory.
- Both proving consumers require the same roots, receiver, ordered arguments, nesting, and spans.
- RxJS parity tests prevent architectural cleanup from silently changing behavior.

### Exact first implementation phase to authorize

Authorize **Phase 0 — Preservation freeze only**:

1. Add tracked RED tests under src/tests for the frozen occurrence contract, lifecycle mismatch, TypeScript/C# adapter expectations, existing Calls preservation, and RxJS pipe parity.
2. Register them through the normal path modules.
3. Make no production changes.
4. Hand the focused commands to the user for valid RED evidence.
5. Stash only the RED tests and registrations before Phase 1, following docs/agent/architecture.md.

No later phase is implicitly authorized by approving Phase 0.

READY FOR IMPLEMENTATION WITH EXPLICIT OPEN DECISIONS
