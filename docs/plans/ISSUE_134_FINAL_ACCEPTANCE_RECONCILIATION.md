# Issue #134 Final Acceptance Reconciliation and Residual Slice Design

**Author:** Agent MaxHeadRoom  
**Date:** 2026-10-15  
**Repository state:** `6a04ace83d59ec358bb2eddc7dcb81ee7f7730f8`  
**Scope:** Read-only reconciliation and residual design; no implementation or runtime verification

## 1. Executive verdict

#134 ACCEPTANCE RECONCILED — ADDITIONAL RESIDUALS FOUND

LGR-001 and LGR-004 are genuine blockers, but the live ledger is not exhaustive.
Static inspection proves two more original-contract residuals:

1. EF `HasEntity` still derives only from `DbSet<T>`; configuration forms do not
   author equivalent semantic edges.
2. `transitive_dependencies` and `has_cycle` do not consume exact, declaration-
   resolved C# inheritance or constructor dependency facts.

Repairing only LGR-001 and LGR-004 would not make #134 closure-ready. Four separate
slices remain: declaration-proven test subjects, indirect ASP.NET ancestry, EF
configuration entity publication, and declaration-resolved C# traversal/cycles.

The historical observation that one direct controller exposed only one action has
no retained fixture, tracked regression, or current live failure. It is a
preservation concern, not a fifth evidenced residual.

## 2. Original #134 acceptance contract

GitHub issue #134 (opened 2026-10-10) defines four families:

1. **Controllers/routes/actions:** a controller two project levels below the
   framework base receives `Controller`, `Route`, and `ControllerAction`; guard
   already-working direct controllers.
2. **EF entities:** `IEntityTypeConfiguration<T>`, `modelBuilder.Entity<T>()`,
   `ApplyConfiguration`, and assembly scanning are recognized; ten configurations
   yield ten `HasEntity` edges; preserve `DbSet<T>`.
3. **Tests:** never link to a nonexistent subject; a differently named test class
   links to the real subject; preserve legitimate relationships.
4. **Dependencies/cycles:** a four-level inheritance chain returns 1/2/3 hops at
   depths 1/2/3 and the whole chain at depth 0; a two-class constructor cycle is
   detected with compiled/indexed scope disclosed.

Naming fallback and Moq extraction were discussed as approaches/evidence, not
separate acceptance requirements. Name resemblance cannot be authority.

## 3. Acceptance matrix

| Original criterion | Current implementation / evidence | Coverage / live evidence | Status |
| --- | --- | --- | --- |
| Direct Controller/Route/Action | `analyze_aspnet` admits attributes or directly written framework bases and publishes route/action facts | `src/tests/dotnet_meta/{aspnet,semantic}.rs`; direct live control passed | SATISFIED |
| Two project bases below framework base | Base facts reach the index; refinement produces builtin `Extends`/`Implements`, not dotnet classification | LGR-004 reproduces Orders chain failure | UNSATISFIED (LGR-004) |
| Ten EF configurations -> ten entities | Semantic edges are built only from `dbsets`; Fluent Entity is rendered metadata only | Tests cover `DbSet<T>`; dedicated live probe unestablished | UNSATISFIED |
| No test edge to absent class | Suffix candidate is published before any declaration join | LGR-001 reproduces `GhostTests -> Ghost` | UNSATISFIED (LGR-001) |
| Differently named constructed SUT | Fallback finds one persistent field assigned one constructed type | `red_s2_construction_and_s4_reverse_index_integration` | SATISFIED; preserve and validate |
| Four-level inheritance traversal | Generic BFS excludes `HasBaseType`/`Extends`; C# resolution is edge-query-only | Generic depth tests; no accepted C# traversal proof | UNSATISFIED |
| Constructor cycle | Cycle allowlist is only `Injects`/`ImportsModule`; constructor type facts are invisible | Generic cycle tests; no accepted C# constructor-cycle proof | UNSATISFIED |
| Neutral base facts | Structured headers plus exact query-time C# declaration resolver | C# base/cross-file suites; LGR-004 chain publication | SATISFIED reusable authority |

## 4. Current .NET semantic authority architecture

```text
C# source -> tree-sitter captures -> canonical declarations/type references
  -> MetaLayerPass exact per-class captures -> DotNetMetaLayer interpretation
  -> SemanticEdge + canonical provenance -> WorkspaceIndex publication
  -> workspace_query hydration/exact identity/refinement -> public response
```

`MetaLayerPass` attaches canonical provenance. `WorkspaceIndex` owns occurrence and
declaration lifecycle per file. Self-`Defines` becomes declaration authority without
entering adjacency; relationship endpoints do not prove declarations.

**ASP.NET:** `aspnet.rs` performs source-local admission from attributes or a
directly written framework base, then extracts routes/actions. The query catalog
resolves unique Class/Interface declarations by namespace, nesting, name, and arity,
but no .NET consumer walks transitive ancestry.

**EF:** `efcore.rs` publishes `HasEntity` only from `DbSet<T>`.
`modelBuilder.Entity<T>()` affects High-fidelity rendering only; explicit
configuration and assembly scanning publish no entity semantics.

**Tests:** `[TestClass]` selects a suffix candidate first, otherwise one constructed
persistent-field SUT, then immediately emits `Tests`. There is no declaration join.

**Traversal:** the index uses fixed dependency/cycle allowlists and generic
deterministic algorithms. The missing boundary is a request-local exact C#
dependency view, not global admission of unresolved structural relations.

## 5. LGR-001 root-cause analysis

`extract_testing_semantic_edge` strips `Tests`, gives that convention priority over
constructed-SUT evidence, and publishes a builtin Class target. The index registers
the target as an occurrence, but an occurrence is not declaration authority.

```text
GhostTests -> suffix proposes Ghost -> no declaration check
  -> Tests(TestClass/GhostTests, Class/Ghost) -> public false positive
```

Both convention and constructed-SUT evidence are candidates only. Existing
declaration authority must decide whether an exact target exists.

## 6. LGR-001 smallest repair slice

**Authority chain:** structured TestClass -> candidate from persistent constructed-
SUT evidence, then convention -> exact scoped builtin Class declaration -> exactly
one authoritative target -> `Tests` relationship.

**Owner:** MSTest candidate interpretation stays in `dotnet_meta::testing`; the
cross-file declaration join belongs at the existing scoped query/refinement or
publication-preparation boundary. Prefer ephemeral refinement if durable cross-file
ownership cannot retract correctly.

**Reuse:** declaration authority, current scoped C# identity resolution/hydration,
`infer_constructed_sut`, and suffix convention as candidate evidence.

**Forbidden:** name-only publication, Ghost special case, endpoint-as-declaration,
second registry, CBM authority, ambiguity guessing, warm-state truth.

**Positive:** real conventional target; differently named persistent constructed
SUT; same/cross-file exact declarations; qualified/generic cases supported by the
existing resolver.

**Negative:** absent target; ambiguous same-name classes; relationship-only target;
multiple constructed candidates; transient local construction; removed target.

**Tracked regression:** production MCP path under `src/tests/**`: no Ghost edge;
differently named and conventional positives; ambiguity; endpoint-only negative;
replacement/removal; deduplication.

**Focused live assertion:** rerun `Invoke-NativeCausalControls.ps1` case
`134 / test-target-must-exist`, retaining a differently named positive control.

**Generation/version:** changing durable .NET semantic output requires incrementing
`dotnet_meta::SEMANTIC_PRODUCER_GENERATION` and compatibility coverage. Purely
ephemeral refinement does not, but no public surface may leak stored fabrication.

**Non-goals:** mock call graph; MSTest redesign; #135 fidelity; #142 completeness.

## 7. LGR-001 regression matrix

| Case | Expected |
| --- | --- |
| `GhostTests`, no declaration | No `Tests` edge |
| Conventional class plus exact target | Exactly one edge |
| Differently named class constructs persistent `_sut` | Edge to declared constructed type |
| Convention conflicts with unique structured construction | Structured construction wins |
| Two plausible constructed targets | Fail closed |
| Same terminal name is ambiguous | Fail closed |
| Target only occurs as relationship endpoint | No edge |
| Target removed/replaced | Edge retracts/recomputes |
| Repeated publication | Exactly one current edge |

## 8. LGR-004 root-cause analysis

Direct controller admission is a substring decision inside `analyze_aspnet`. A
project base does not satisfy it even when structured C# proves
`OrdersEndpoint -> SecondHttpBase -> SharedHttpBase -> ControllerBase`. The exact
C# resolver handles one edge at a time and has no ASP.NET endpoint predicate.

## 9. LGR-004 smallest repair slice

**Authority chain:** candidate Class -> exact declared base -> exact project Class ->
repeat -> authoritative ASP.NET framework endpoint -> classify original class ->
reuse its source-local route/action extraction.

The endpoint must be qualified/import-resolved framework identity. A project class
merely named `ControllerBase` is insufficient.

**Owner:** domain-neutral mechanics may walk exact identities; the endpoint
predicate and Controller projection stay in .NET query/refinement code. Prefer
query-local facts to avoid stale cross-file ownership.

**Reuse:** C# resolution catalog, staged hydration, effective scope, neutral base
facts, declaration authority, and existing ASP.NET route/action extractors.

**Forbidden:** `*Controller` naming; terminal-name-only endpoint; whole-repository
substring authority; persistent derived edges without invalidation; ASP.NET rules in
`WorkspaceIndex`; classification from incomplete/ambiguous evidence.

**Termination:** visited exact identities; stop on repetition; fail closed on zero
or multiple next declarations; bound by the finite admitted declaration set rather
than a new global hop cap.

**Positive:** direct guard; one/multiple project-base hops; full original-class
route/actions; supported qualified/generic bases.

**Negative:** project-owned fake ControllerBase; missing/ambiguous hop; cycle;
unrelated endpoint; namespace/project collision; partial discovery; target removal.

**Tracked regression:** registered MCP three-level chain plus every negative above,
direct preservation, exact provenance, and lifecycle retraction.

**Focused live assertion:** rerun `Invoke-NativeIssueReconciliation.ps1` case
`134 / controller-two-project-bases-below-framework`, requiring Controller,
HasRoute, and the complete expected action set.

**Generation/version:** query-time refinement needs no semantic generation bump.
Persisting framework edges would require a bump and cross-file invalidation and is
not recommended.

**Non-goals:** naming fallback; Minimal API redesign; #133, #135, #142; generic
framework registry.

## 10. LGR-004 regression matrix

| Case | Expected |
| --- | --- |
| Direct framework base | Existing Controller/Route/Action unchanged |
| Required three-level chain | Original class classified once; full route/actions |
| Project class named ControllerBase | No classification |
| Missing or ambiguous middle declaration | Fail closed |
| `A -> B -> A` | Terminates; no Controller |
| Unrelated endpoint | No Controller |
| Namespace/project collision | Only exact chain may classify |
| Partial acquisition | No positive classification |
| Target removal/kind change | No stale classification |

## 11. Additional residual: EF configuration registrations

### Root cause

`analyze_efcore` constructs every `HasEntity` edge from `dbsets`. Its Fluent API
routine recognizes `modelBuilder.Entity<T>()` only for a marker. It never joins
`ApplyConfiguration(new XConfiguration())` to
`IEntityTypeConfiguration<T>` or expands an authoritative assembly scan.

### Smallest repair slice

**Authority chain:** structured DbContext/OnModelCreating -> exact direct Entity,
explicit configuration, or assembly registration evidence -> exact configuration
declaration implementing `IEntityTypeConfiguration<T>` -> exact entity declaration
-> deduplicated `DbContext --HasEntity--> Entity`.

**Owner:** .NET EF semantic producer/refinement. Shared code may expose structured
generic/interface/call evidence but must not interpret EF APIs.

**Reuse:** structured declarations/invocations, exact C# catalog, declaration
authority, workspace scope, existing publication/deduplication.

**Forbidden:** `*Configuration` naming; assuming generic arguments are EF entities;
assembly expansion beyond proven scope; use-as-declaration; textual repository grep
as authority; CBM-required truth.

**Positive:** ten exact configurations; direct `modelBuilder.Entity<T>()`; explicit
ApplyConfiguration; proven assembly scan; deduplication with `DbSet<T>`.

**Negative:** configuration-shaped name without interface; missing/ambiguous target;
outside-scope configuration; unrelated generic interface; duplicates; lexical
decoys; removal/replacement.

**Tracked regression:** production MCP ten-configuration fixture plus every form,
provenance, deduplication, ambiguity, decoys, replacement, and refresh.

**Focused live assertion:** provider-disabled issue-reconciliation case publishing
one context, ten configuration declarations, and entity declarations; require
exactly ten named current `HasEntity` edges.

**Generation/version:** new durable edges require incrementing the .NET semantic
producer generation and compatibility coverage. An ephemeral design avoids the bump
but must still support public forward/reverse queries and lifecycle.

**Non-goals:** executing EF models/assemblies; migrations; navigation inference;
global acquisition/completeness policy.

### Regression matrix

| Case | Expected |
| --- | --- |
| Ten supported configurations | Exactly ten expected edges |
| Direct Entity call | One exact edge |
| Explicit ApplyConfiguration | Edge after exact config/entity resolution |
| Assembly scan | Only proven admitted configurations |
| Same entity via multiple paths | Deduplicated current relationship |
| Fake/missing/ambiguous configuration | No edge |
| Comment/string decoy | No edge |
| Deletion/replacement | Current projection retracts/recomputes |

## 12. Additional residual: C# dependency traversal and constructor cycles

### Root cause

The generic algorithms work for approved relations, but their semantic view excludes
inheritance and constructor type facts. Exact C# base refinement is edge-query-only.
Globally adding structural TypeRefs would be wrong: unresolved references are not
exact dependencies and unrelated domains would change.

### Smallest repair slice

**Authority chain:** structured C# Class -> written base or constructor parameter
type -> exact scoped Class declaration -> request-local C# dependency adjacency ->
existing deterministic BFS/cycle algorithm -> existing depth/scope/provenance
response envelope.

**Owner:** C# query/refinement prepares exact adjacency; generic graph utilities
traverse caller-supplied identities without C# meaning.

**Reuse:** structured headers/constructor facts, C# Catalog, declaration authority,
scope/hydration, generic BFS, deterministic cycle mechanics.

**Forbidden:** terminal-name joins; unresolved TypeRef traversal; global allowlist
expansion; Calls traversal; endpoint-as-declaration; provider-only/warm truth.

**Positive:** four exact inheritance classes at depths 1/2/3/0; P(Q) and Q(P)
constructors; qualified/generic identities; scoped provenance.

**Negative:** missing/ambiguous declaration; kind mismatch; unrelated structural
relations; lexical decoys; cross-workspace composition; removed half-cycle;
unsupported external type.

**Tracked regression:** registered MCP four-hop matrix and P/Q cycle plus ambiguity,
missing declarations, root isolation, deterministic witness, and removal. Preserve
existing generic graph-policy tests.

**Focused live assertion:** provider-disabled #134 live case requiring exact ordered
dependencies for every depth and a closed P/Q witness with fixture provenance.

**Generation/version:** request-local refinement needs no durable generation bump.
Persisted resolved edges would require a C# producer bump and cross-file
invalidation and are not recommended.

**Non-goals:** global dependency semantics; Calls; #133 cold discovery; #142;
ASP.NET classification.

### Regression matrix

| Case | Expected |
| --- | --- |
| `A:B`, `B:C`, `C:D` | depth 1=`B`; 2=`B,C`; 3/0=`B,C,D` |
| Missing/ambiguous B | Stop at proven frontier |
| Qualified same names | Traverse only exact identity |
| P(Q), Q(P) | Closed deterministic cycle witness |
| Unresolved constructor target | No fabricated cycle |
| Same names in other root | No cross-workspace chain/cycle |
| HasRoute/Tests/Calls | Not traversed |
| Replacement/removal | Current result only |

## 13. Issue-boundary analysis (#133/#135/#142)

**#133:** owns closed constructor-injection cold recall. #134 may consume already
published constructor facts for its explicit cycle criterion, but must not reopen
acquisition, large cold recall, or provider discovery.

**#135:** owns skeleton fidelity, DI truncation, call ownership, and context
parameters. #134 may reuse internal structured evidence but does not require new
Moq/EF rendering or omission disclosure.

**#142:** owns history independence, global coverage/completeness metadata,
refresh/index policy, and shared diagnostics. #134 must use the current lifecycle
and fail closed. It may merge existing staged-hydration diagnostics but must not
redesign global acquisition or freshness semantics.

## 14. Exhaustiveness determination

LGR-001 and LGR-004 are not exhaustive. EF configuration registration and C#
traversal/cycles are original, statically proven residuals. No other #134-specific
residual is supported by current evidence: direct controllers, direct DbSet facts,
differently named construction inference, neutral bases, and generic algorithms all
exist and become preservation guards.

**Required answer: No.** Green LGR-001 and LGR-004 alone would leave the original EF
and traversal/cycle criteria unsatisfied.

## 15. Recommended repair order

1. LGR-001 declaration-proven test subjects: establish/reuse the declaration join.
2. LGR-004 exact ASP.NET ancestry: add bounded exact-identity walking with .NET-
   owned endpoint meaning.
3. C# dependency traversal/cycles: reuse exact walking without ASP.NET leakage and
   feed the generic algorithms.
4. EF configuration entities: separate EF interpretation and durable-version
   decision.

Keep all four as separate implementation/checkpoint slices. Sharing exact identity
mechanics does not justify sharing framework semantics or rollback boundaries.

## 16. Closure-readiness conditions

#134 reaches final closure reconciliation only when:

1. LGR-001 and LGR-004 have tracked production regressions and focused live GREEN.
2. Ten exact configuration registrations yield ten current `HasEntity` edges in
   tracked and live evidence.
3. The four-level depth matrix and P/Q constructor cycle pass tracked production
   tests and focused live assertions.
4. Direct controller, DbSet, legitimate test, neutral base, scoping, lifecycle, and
   generic traversal/cycle guards remain green.
5. Durable output changes carry producer-generation/compatibility coverage.
6. The maintainer runs the separately authorized final verification gate with zero
   warnings; live and tracked results remain separately reported.
7. No #133, #135, or #142 obligation is claimed as #134 evidence.

No Claude field pilot is requested. Batch local deterministic implementation and
evidence first; consolidate any genuinely Claude-only questions later.

## 17. Evidence references

- GitHub issue #134, retrieved read-only with `gh issue view`.
- `docs/plans/CROSS_FILE_CSHARP_BASE_TYPE_CLASSIFICATION_INVESTIGATION.md`
- `docs/plans/POST_CBM_133_141_NATIVE_DEFECT_RECONCILIATION.md`
- `docs/plans/ISSUES_133_141_POST_CBM_RECONCILIATION.md`
- `verification/LIVE_GATE_REGRESSIONS.md`
- `verification/issue-reconciliation/Invoke-NativeCausalControls.ps1`
- `verification/issue-reconciliation/Invoke-NativeIssueReconciliation.ps1`
- `src/dotnet_meta/aspnet.rs:33-113`
- `src/dotnet_meta/efcore.rs:29-98`
- `src/dotnet_meta/testing.rs:43-64,218-247`
- `src/dotnet_meta/mod.rs:197-246,314-351`
- `src/ir/pipeline/meta_layer.rs:81-95`
- `src/mcp/tool_handlers/query/{csharp_resolution,classification,edges}.rs`
- `src/workspace/index/declarations.rs`
- `src/workspace/index/traversal.rs:50-77,353-423`
- `src/tests/dotnet_meta/{aspnet,semantic,testing}.rs`
- `src/tests/mcp/workspace_query_csharp_base_types.rs`
- `src/tests/mcp/workspace_query_csharp_cross_file_classification.rs`
- `src/tests/mcp/workspace_query_csharp_constructor_types.rs`
- `src/tests/workspace/index_declaration_authority.rs`
- `src/tests/workspace/index_graph.rs`
- `src/tests/workspace/index_cycle_policy.rs`

## 18. Commands actually run

Only fast, bounded investigation commands and this report edit were run. No tests,
builds, checks, formatters, binaries, servers, live scripts, benchmarks, provider
calls, or GitHub mutations ran.

```text
Get-Content -LiteralPath <targeted attachment/files/ranges> [-Raw]
rg --files | rg <targeted #134/.NET patterns>
rg -n [-C N] <targeted symbols/phrases> <scoped paths>
git status --short
git rev-parse HEAD
git log --all ... <scoped paths>
git blame -L <range> -- <file>
gh issue view 134 --repo codeliftsleep2/Clean-CTX --json ...
```

Clean-CTX read-only calls: `get_cbm_status`; `graph_search` for relevant .NET,
traversal, and entity symbols; batched `provide_code_context` for targeted producer,
query, and index files. CBM reported available. Several searches had no symbol hit;
context was skeleton-only for several files, so exact known files/ranges were read
natively as required by the tooling protocol.

One web search did not locate the exact issue; GitHub CLI retrieved it. Initial
sandboxed reads were refused by the Windows restricted-token environment and were
repeated with approved read-only escalation. The normal patch tool could not write
through that sandbox, so its underlying patch executable was invoked in the
approved elevated shell for this report only.
