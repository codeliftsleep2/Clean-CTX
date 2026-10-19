# Issue #147 Contract-Gap Architecture Investigation

**Agent:** Agent MaxHeadRoom  
**Date:** 2026-10-19  
**Repository:** codeliftsleep2/Clean-CTX  
**Authoritative branch:** `(fix)Performance-Regression-Issues`  
**Phase:** Architecture investigation only; no production implementation

No Cargo command, build, test, Clippy invocation, binary, MCP runtime session,
or live verification was run. Static inspection used the repository source,
tracked tests and prior investigation records. Clean-CTX graph discovery was
attempted first, but CBM launch timed out; the documented `rg` fallback and
`provide_code_context` were then used.

## 1. Executive verdict

Issue #147 contains three independent authority questions and should not be
implemented as one coupled repair.

- **147-A: A2 — DBCONTEXT IDENTITY AND RELATIONSHIP EVIDENCE ARE IMPROPERLY
  COUPLED; STRUCTURED FRAMEWORK IDENTITY SHOULD BE DURABLE INDEPENDENTLY.** A
  supported, unambiguous structured EF Core base-class classification proves
  the framework role. `DbSet<T>` and supported `OnModelCreating` evidence prove
  relationships, not the role itself. A framework identity with zero
  `HasEntity` edges is valid.
- **147-B: B5 — COMPOSED DESIGN REQUIRED (B2 + B3 + B4).** Exclude a source
  kind only through an explicit active-producer incapability proof (B4); use a
  certified name-bearing predicate where one exists (B2); otherwise process
  the complete admitted unit authoritatively (B3). An uncertified predicate
  returning zero candidates has no negative authority.
- **147-C: C2 — CURRENT WORDING IS MISLEADING; ADDITIVE DESCRIPTION CHANGE
  ONLY.** The existing value can denote healthy CBM search plus lack of
  authoritative scope coverage. Preserve the public string for compatibility,
  but define “scope unavailable” as “CBM could not supply authoritative
  coverage for this request,” not “CBM was unavailable.” Existing provider and
  project diagnostics already carry availability distinctions.

The new field evidence causally explains Issue #146 under the current durable
producer contract: Root A had no supported relationship payload and therefore
published no durable `dotnet/DbContext/ContextA` endpoint; Root B had nineteen
`DbSet<T>` relationships and therefore did. Issue #146 should close as expected
under the current contract while 147-A changes that contract deliberately.

## 2. Issue #147 evidence summary

The field report is consistent with current code:

1. `dotnet_meta::efcore::analyze_efcore` recognizes textual `: DbContext`,
   renders an EF marker, then creates a `dotnet/DbContext/<class>` only as the
   subject of one or more `HasEntity` edges derived from textual `DbSet<T>`
   members. With zero detected sets, the marker survives but no durable
   framework occurrence enters `WorkspaceIndex`.
2. The request-local C# `EfCoreCatalog` has stronger structured base/import
   checks, but emits only supported `OnModelCreating` `HasEntity` edges and is
   consulted only by exact EF edge requests. It does not publish an independent
   durable framework occurrence.
3. Exact reverse `builtin/TypeRef` acquisition creates one required
   `CoverageUnit` per inventoried root/source-kind group. Only C# currently has
   a certified name-bearing predicate. A TypeScript unit is therefore residual
   with `candidate_predicate_uncertified`, even when all observed candidates
   compile and the returned positive edges are correct.
4. Hydration selects `cbm_scope_unavailable` for `(saw_unavailable=false,
   saw_search_failure=false)` whenever filesystem coverage is still required.
   This includes a healthy successful CBM search that lacks the authority
   needed for exact incoming-TypeRef proof.

These are contract boundaries, not evidence that the returned positive facts
were false.

## 3. Shared authority principles

The three questions share these rules:

- Positive-fact authority and exhaustive-negative authority are separate.
- Structured syntax may establish the proposition it actually represents; a
  relationship payload must not become a prerequisite for an independent
  identity proposition.
- Candidate selection is not absence proof. A narrowing predicate can exclude
  files only when its producer-specific exhaustiveness is certified.
- Source-kind capability and predicate capability are separate dimensions.
- Providers may supply candidates and acquisition economics. They do not own
  Clean-CTX semantic truth or completeness.
- Process completion, successful parsing, and `discovered == compiled` do not
  establish semantic completeness.
- Public detail may explain a conservative result but must never strengthen
  the legacy completeness tuple.
- Warm state and batching may reuse proof; they may not change truth.

## 4. 147-A current DbContext authority paths

| Path | Evidence consumed | Form | Identity authorized today | Relationships authorized | Lifetime | Payload dependency | Framework/import authority |
|---|---|---|---|---|---|---|---|
| C# canonical/base capture | Parsed class and base-list captures | Structured, unresolved | `builtin/Class` plus neutral base facts; no durable EF identity | None by itself | Durable canonical artifact | None | Preserves written base; does not bind EF framework |
| .NET detector/meta admission | File regions and .NET patterns | Mixed/textual gate | None | None | Compile-local | N/A | Applicability only |
| `analyze_efcore` marker | Isolated class text containing `: DbContext` | Textual | Marker says DbContext, but no indexed identity carrier | Textual `DbSet<T>` creates `HasEntity` | Marker/edge artifact | Durable identity exists only when an edge exists | Does not inspect imports, aliases, project references, or ambiguity |
| DbSet extraction | `DbSet<T>` spellings | Textual | Indirectly creates subject occurrence | `HasEntity(DbContext, Entity)` | Durable | Yes | No binding proof |
| Fluent marker extraction | `modelBuilder.Entity<T>` spelling | Textual | None | Rendered configuration marker, not the durable edge authority audited here | Durable render | No identity carrier | No binding proof |
| Request-local `EfCoreCatalog` | Structured declarations, base references, imports, `OnModelCreating` AST | Structured with a textual written-name resolution policy | Recognizes a context internally | Supported direct/configuration/assembly `HasEntity` edges | Request-local | Emits no context result without relationship evidence | Fully qualified EF base, or unqualified base plus exact namespace import |
| Exact EF edge handler | Exact `dotnet/DbContext` request plus catalog | Structured request-local | Uses identity as an edge selector | Adds request-local EF edges | Request-local | Yes | Inherits catalog rules |
| WorkspaceIndex publication | Published `SemanticEdge` endpoints | Structured semantic edge | Registers the subject occurrence | Stores edge | Durable owner projection | Yes under current producer | Trusts upstream producer |
| `find_entities` | Indexed occurrences | Indexed | Returns durable endpoint occurrences | N/A | Query-time over durable index | Therefore yes today | No special DbContext refinement |
| Declaration queries | Source declaration carriers | Structured canonical | `builtin/Class`, not `dotnet/DbContext` | N/A | Durable | No | Deliberately preserves #145 boundary |

No current production path durably publishes a framework-role occurrence from
structured base classification independently of an EF relationship.

## 5. 147-A identity-vs-relationship analysis

These are distinct propositions:

```text
P1: class C is authoritatively classified as an EF Core DbContext
P2: context C exposes/manages entity E through supported source evidence
```

`P2` entails that the producer has treated C as a context, but the absence of
`P2` does not negate `P1`. Infrastructure contexts, generic accessors, contexts
whose model comes from conventions or unsupported configuration forms, and
partially implemented contexts can all satisfy `P1` while yielding no supported
`HasEntity` relation.

The current durable representation uses an edge endpoint as the only
registration carrier, making `P1` accidentally depend on `P2`. This is an
implementation coupling rather than an essential authority boundary.

The correct separation is:

```text
supported structured EF base classification -> durable framework occurrence
supported DbSet/model-building evidence      -> HasEntity edges
```

No relationship is inferred from identity alone. No absence of DbSet or model
configuration is allowed to negate an independently proven identity.

## 6. 147-A structured framework authority requirements

The minimum sound durable classifier is not textual `: DbContext`. It must
consume parser-owned declaration/base-reference structure and fail closed when
the written base cannot be bound unambiguously to EF Core under the supported
resolution contract.

| Base form | Minimum authority decision |
|---|---|
| `Microsoft.EntityFrameworkCore.DbContext` or supported `global::` form | Authoritative when parsed as the actual class base and arity is zero |
| Unqualified `DbContext` plus exact `using Microsoft.EntityFrameworkCore` | Authoritative only if the supported catalog proves no local/import/alias ambiguity or shadowing |
| Namespace/type alias to EF DbContext | Unsupported until alias resolution is explicitly implemented and tested; do not guess |
| Ambiguous unqualified `DbContext` | Fail closed; retain `builtin/Class` and neutral base evidence only |
| User-defined/local `DbContext` | Not EF identity |
| Indirect subclass of a proven EF context | Unsupported for the first slice unless a complete, ambiguity-safe inheritance chain is resolved inside the admitted authority scope |
| Generic `DbContext<T>` | Unsupported for EF Core `DbContext` because current exact contract requires arity zero |
| Malformed or parser-recovered declaration | No framework identity unless the structured capture remains valid under an explicit recovery contract |
| Partial declarations | Identity may be emitted once if one admitted part proves the base and no conflicting part invalidates the declaration; otherwise fail closed |

The existing request-local `is_ef_framework_type` is a useful starting rule,
not automatically sufficient for durable authority: its unqualified
base-plus-import branch must be audited for local shadowing and alias ambiguity
before reuse. Package/project binding is not presently part of the contract; a
first implementation should state that limitation rather than imply compiler-
grade symbol resolution.

`dotnet/DbContext/<class>` should mean “authoritatively classified EF Core
DbContext under the supported structured resolver,” not merely “subject
endpoint of a detected EF relationship.” That broader meaning matches the
public name and makes the current relationship-endpoint meaning the historical
implementation limitation. A new identity kind is unnecessary if the stronger
classification gate is enforced.

## 7. 147-A scenario matrix

| Scenario | Durable DbContext identity after approved separation | `HasEntity` | Status |
|---|---:|---:|---|
| EF context, many supported `DbSet<T>` members | Yes | One per distinct supported entity | Supported |
| EF context, zero DbSets and no supported model evidence | Yes | Zero | Required regression |
| EF context, supported `OnModelCreating` only | Yes | Supported relationships only | Supported if existing catalog proves targets |
| User-defined `DbContext` | No | Zero | Must fail closed |
| Fully qualified EF base | Yes | Evidence-dependent | Strongest first-slice authority |
| Unqualified base plus exact import, no ambiguity | Yes | Evidence-dependent | Supported after ambiguity guard |
| Ambiguous/shadowed unqualified base | No EF identity | Zero | Unsupported/fail closed |
| Indirect inheritance from proven EF context | Not in first slice | Zero unless separately proven | Future resolution contract |
| Alias-qualified EF base | Not until alias resolver is certified | Zero | Unsupported |
| Malformed declaration | No | Zero | Fail closed |

## 8. 147-A relationship to #145

Issue #145 remains unchanged. A source declaration is still
`builtin/Class/<name>`. The new framework occurrence is an additive semantic
classification with the same file provenance; it does not rewrite declaration
identity, change `declarations_only`, or cause declaration filters to pretend a
framework role is the source declaration kind.

`find_entities` may return both the builtin declaration occurrence and the
framework occurrence. Exact forward/reverse EF edge queries continue to return
only proven relationships. Framework filters may find the independent role
even when the edge set is empty.

## 9. 147-A resolution of #146

The new field evidence supplies the causal discriminator missing from the
earlier investigation:

- Root A: zero textual DbSets and no reported supported relationship-producing
  model evidence -> durable producer emits zero `HasEntity` edges -> no durable
  `dotnet/DbContext/ContextA` endpoint.
- Root B: nineteen textual DbSets -> nineteen relationship candidates -> the
  durable subject endpoint registers `dotnet/DbContext/ContextA`.
- The earlier synthetic case gave both roots relationship-producing bodies and
  both were present, falsifying root placement, order, warm state, and same-name
  collision as the current cause.

Therefore #146 is expected behavior under the current contract. This does not
make the contract desirable; it isolates the improvement to 147-A.

## 10. 147-A architecture verdict

**A2 — DBCONTEXT IDENTITY AND RELATIONSHIP EVIDENCE ARE IMPROPERLY COUPLED;
STRUCTURED FRAMEWORK IDENTITY SHOULD BE DURABLE INDEPENDENTLY.**

The change is externally observable and therefore requires explicit approval
before implementation. It must be additive, structured, ambiguity-safe, and
must not fabricate `HasEntity` edges.

## 11. 147-B current TypeRef proof pipeline

```text
exact reverse builtin/TypeRef request
  -> AcquisitionTarget::exact_type_ref
  -> configured-root inventory
  -> group manifest by root and SourceKind
  -> CoverageUnit(root, source kind, BuiltinTypeRefIncoming,
                  producer generation,
                  HasConstructorParameterType + HasBaseType)
  -> clean_type_ref_proof_claim
  -> complete inventory check
  -> source-kind predicate certification check
  -> literal candidate-owner selection
  -> claim validation
  -> candidate semantic publication
  -> AcquisitionSatisfaction
  -> residual units
  -> completeness generation/certificate
  -> public exact or lower-bound coverage detail
```

`CoverageUnit` currently expresses root, source kind, producer family,
producer generation, and relationship families. It does not yet express an
explicit “this active source producer cannot emit this family” capability
disposition, nor an alternate full-unit acquisition proof mode.

In the reported topology the statically identified residual is the TypeScript
unit because `has_certified_type_ref_name_bearing_predicate` returns true only
for C#. The exact runtime root name is field evidence; no new live execution is
needed to decide the architecture.

## 12. 147-B empty-candidate falsification

An uncertified predicate returning zero candidates cannot prove authoritative
absence. Formally, let `P(file, name)` be the cheap predicate and `E(file,
name)` mean the active producer can emit a matching TypeRef edge. Negative
authority needs:

```text
for every admitted file: E(file, name) -> P(file, name)
```

Without that certified implication, `count(P) == 0` says nothing about
`count(E)`. Treating the empty result as proof assumes the proposition that is
missing and is circular.

Adversarial forms include imported aliases, renamed imports, namespace or type
aliases, qualified versus normalized names, generic wrappers, generated
canonical forms, escape/Unicode normalization, and indirect references whose
emitted identity is not a verbatim occurrence of the queried canonical name.
Which forms actually emit depends on each active language producer; that is
why certification is producer-specific. An inventory being complete does not
certify its narrowing predicate.

The empty-candidate proposal is rejected.

## 13. 147-B producer-capability vs predicate-capability model

Two independent questions are required for each `(source kind, active producer
generation, relationship family)`:

1. **Producer capability:** can this producer emit an edge to the queried
   identity family at all?
2. **Predicate capability:** if it can, does the cheap predicate select every
   owner that could emit the exact requested identity?

The sound decision table is:

| Producer capable? | Predicate certified? | Required treatment |
|---:|---:|---|
| No, with generation-bound proof | Irrelevant | Exclude unit from requirement |
| Yes | Yes | Narrow to predicate candidates, then validate/publish all |
| Yes | No | Full-unit authoritative acquisition |
| Unknown | Any | Treat as capable; certified predicate may narrow, otherwise full unit |

Current construction conservatively creates units for every inventoried
supported source kind. That is safe but cannot distinguish an incapable active
producer from a capable producer whose predicate is uncertified. The capability
catalog must be direct, versioned, and tied to the actual canonical/semantic
producer generation; file extension alone is insufficient authority.

## 14. 147-B candidate architecture options

### B2 — certify more predicates

This is valuable where the producer guarantees every emitted target has a
sound searchable source representation. TypeScript certification must audit
the exact canonical producer behavior for aliases, imports, qualified names,
type aliases, generics, and normalization. A boolean flip without producer
contract tests is unsound. Rust and Java require separate audits.

### B3 — full-unit acquisition for uncertified predicates

This is the universal sound fallback: enumerate every admitted owner in the
unit, obtain current adequate semantic projection, publish successfully, and
only then prove the unit. It does not use the uncertified predicate for
negative authority. It can be expensive, so it should run only for units that
remain relevant after capability scoping and only when an exact certificate is
requested.

### B4 — capability-scoped exclusion

This is sound only when the active source-kind producer contract proves it
cannot emit either requested relationship family. The exclusion must be
family-specific and generation-bound. It must be revoked when a new producer
generation gains capability.

### B4 language-qualified query contract

An explicit future query could ask for a language-qualified proof universe,
but the existing language-neutral `builtin/TypeRef` request must never be
silently interpreted as C#-only. This option is additive and not required for
the core repair.

### Preserve lower-bound

If full-unit work exceeds an explicit budget, processing fails, or producer
capability remains unknown without adequate acquisition, preserve the current
structural lower-bound result. Exactness is optional; truthfulness is not.

## 15. 147-B withinPath / proof-scope analysis

Today `withinPath` narrows final occurrence projection inside the configured
workspace scope. It does not narrow acquisition roots, cache identity,
inventory manifests, required units, or the proof fingerprint. This is
conservative: proof over the broader universe implies exactness for its
projection, while a residual unit anywhere keeps the result lower-bound.

Projection-only behavior is not a correctness defect. Overloading `withinPath`
to redefine proof scope would change public exactness and cache semantics. If
narrow proof is desired, add an explicit proof-scope contract (or explicitly
version `withinPath` as such) and bind acquisition, admission fingerprint,
cache key, certificate scope, and final filtering to the identical canonical
subtree. No response-layer hiding of residual units is acceptable.

## 16. 147-B performance analysis

The composed design protects #144 performance as follows:

- capability exclusion removes proven-irrelevant units before inventory work;
- certified predicates retain K-candidate compilation for common languages;
- full-unit processing is limited to relevant uncertified units and only when
  exactness is requested;
- retained current artifacts satisfy owners without reread/reparse;
- one inventory manifest and one generation-bound proof are reused per unit;
- equivalent warm requests reuse compatible satisfaction in O(1)-style
  currentness checks rather than global reconciliation;
- budget or publication failure returns lower-bound instead of weakening the
  source universe;
- exact-call reconciliation remains independent and must not be triggered by
  TypeRef preparation.

If an uncertified capable unit contains N admitted owners and K predicate
candidates, the cold worst case is necessarily O(N) authoritative owner
validation/processing for that unit. Any design claiming exhaustive absence
with O(K) work requires a certified narrowing proposition. The implementation
must avoid per-query repeated inventories and N-squared dependency validation.

## 17. 147-B scenario matrix

| Workspace/proof state | Required result | Reason |
|---|---|---|
| C# only; complete inventory, certified predicate, publication succeeds | Exact | Every required unit proven |
| C# + TS; active TS producer proven incapable of requested families | Exact if C# unit proven | B4 excludes irrelevant TS unit |
| C# + TS; TS producer capable and predicate certified | Exact if both units proven | B2 narrowing is authoritative |
| C# + TS; TS capable/unknown, predicate uncertified, zero candidates | Lower-bound | Zero from uncertified predicate is not absence proof |
| Same, but every TS owner receives current authoritative processing | Exact if all publication/proof checks succeed | B3 full-unit proof |
| Rust/Java producer proven incapable | Unit excluded | Capability proof, not language assumption |
| Rust/Java capable or unknown with uncertified predicate | Full unit or lower-bound | No unsupported exclusion |
| `withinPath` narrowed under current projection-only contract | Broad proof obligation remains | Safe current behavior |
| Explicit future narrow proof scope | Exact only if all units inside that bound are proven | Scope must bind cache and certificate |
| Budget exhausted during full-unit processing | Lower-bound | Budget cannot redefine authority |

## 18. 147-B architecture verdict

**B5 — COMPOSED DESIGN REQUIRED: B2 + B3 + B4.**

The order is capability proof, then certified narrowing where available, then
full-unit authoritative acquisition for remaining capable/unknown units.
Current lower-bound behavior remains the fail-closed outcome whenever that
composition cannot complete.

## 19. 147-C current diagnostic pipeline

Hydration records several independent observations:

- whether CBM was attempted and any search succeeded;
- project readiness/search failure exceptions;
- roots not covered authoritatively by CBM;
- whether filesystem discovery ran or was cached;
- the combined provider (`cbm`, `filesystem`, `cbm_and_filesystem`, `none`);
- completion/partial/unavailable discovery status; and
- one `fallback_reason` when filesystem coverage was needed.

The selection currently is:

```text
saw unavailable + saw search failure -> cbm_partial_failure
saw unavailable only                -> cbm_unavailable
saw search failure only             -> cbm_discovery_failed
neither                              -> cbm_scope_unavailable
```

For exact incoming TypeRef, a ready successful CBM search deliberately does not
mark the root complete because the query needs provider-neutral proof. The
filesystem inventory then supplies authority, producing
`provider=cbm_and_filesystem` and `fallback_reason=cbm_scope_unavailable` even
though CBM responded normally.

## 20. 147-C availability-vs-authority analysis

The state model is not fundamentally conflated: availability/readiness/search
failure, provider participation, discovery completion, and fallback reason are
already separate fields. The misleading part is lexical and descriptive. A
reader can interpret “CBM scope unavailable” as “CBM failed or lacked the
scope” when the real state can be “CBM searched successfully but its result is
not completeness authority for this query.”

No new provider-authority boolean is required for Issue #147. The existing
combination already expresses the state if the reason is defined precisely:

> `cbm_scope_unavailable`: CBM did not provide authoritative coverage for the
> request's required scope or proof contract; CBM may still have responded
> successfully and contributed advisory candidates.

This keeps provider evidence as acquisition advice and Clean proof as semantic
authority.

## 21. 147-C compatibility analysis

`fallback_reason` is a public enumerated string in the MCP schema and tracked
tests. Renaming it removes a value clients may branch on and is therefore a
behavioral contract change, plausibly semver-significant even if a replacement
is semantically clearer.

An additive description/schema-documentation clarification changes no JSON
shape or existing value. If a future schema version wants
`cbm_advisory_only`, it should add a versioned/additive reason or deprecate the
old value with a compatibility window, not silently rename it in this repair.

## 22. 147-C scenario matrix

| Provider state/request outcome | Scope authority | Filesystem fallback | Current reason | Recommended interpretation |
|---|---|---:|---|---|
| No CBM bridge/provider | None | Yes | `cbm_unavailable` | Provider unavailable |
| CBM initializing/not ready for a root | Incomplete | Yes | `cbm_unavailable` (plus readiness detail where emitted) | Provider unavailable for use now |
| CBM search errors | None/partial | Yes | `cbm_discovery_failed` | Provider operation failed |
| Some unavailable and some failed | Partial | Yes | `cbm_partial_failure` | Mixed provider failure |
| CBM search succeeds, but exact proof contract does not authorize its scope | Advisory candidates only | Yes | `cbm_scope_unavailable` | Healthy response, no authoritative scope coverage |
| CBM covers one root; filesystem covers another | Partial CBM authority | Yes | Often `cbm_scope_unavailable` absent failure | Combined coverage; not a CBM outage |
| CBM alone supplies accepted complete discovery for a legacy mode | Accepted for that discovery contract | No | Absent | Normal CBM-only path |
| Filesystem also unavailable | None | Attempted/failed | `filesystem_unavailable` | No authoritative fallback |

## 23. 147-C architecture verdict

**C2 — CURRENT WORDING IS MISLEADING; ADDITIVE DESCRIPTION CHANGE ONLY.**

Preserve the value, clarify that “scope unavailable” concerns authoritative
coverage for the request, and explicitly say that CBM may be healthy and may
have contributed candidates.

## 24. Required architectural invariants

After approval and implementation, document these durable invariants in
`docs/ARCHITECTURAL_INVARIANTS.md` using the existing classification model:

1. **Framework role/relationship separation:** an ambiguity-safe structured
   framework classification may register a durable role independently of
   relationship payload; role identity alone never fabricates relationships.
2. **Capability/predicate separation:** producer incapability may remove a
   coverage unit; certified predicate capability may narrow it; neither may be
   inferred from candidate count.
3. **Uncertified-zero prohibition:** zero candidates from an uncertified
   predicate has no exhaustive-negative authority.
4. **Full-unit fallback:** a capable/unknown unit without certified narrowing
   can be proven only by complete admitted-unit processing with adequate current
   semantic evidence and successful publication.
5. **Provider diagnostic semantics:** provider health and provider authority
   are independent; `cbm_scope_unavailable` does not assert provider failure.

Prefer structural types and the existing proof validator over a new generic
framework. Add narrow tracked regressions where the type system cannot enforce
the public behavior.

## 25. Required RED regressions

Do not implement these until the corresponding architecture slice is approved.

### 147-A first RED boundary

Under `src/tests/**`, compile a C# file containing an ambiguity-safe structured
EF Core base and zero `DbSet<T>`/supported model relationships. Assert:

- durable `dotnet/DbContext/<name>` occurrence exists in `find_entities`;
- zero `HasEntity` edges exist in both directions;
- `builtin/Class/<name>` declaration authority remains unchanged; and
- a shadowed/user-defined `DbContext` control does not gain EF identity.

The unfixed implementation must fail on the missing durable role assertion,
not on compilation or unrelated behavior.

### 147-B first RED boundary

Add two linked tracked tests:

1. capable/unknown source kind + uncertified predicate + zero predicate
   candidates remains residual/lower-bound; and
2. the same complete unit, after every admitted owner receives adequate current
   semantic processing and successful publication, can satisfy the unit and
   earn exactness.

Capability-exclusion and any newly certified TypeScript predicate require
their own producer-generation and adversarial syntax controls.

### 147-C first RED boundary

Test the public diagnostic contract: a successful ready CBM search that cannot
authorize the requested exact scope, followed by successful filesystem
coverage, reports `provider=cbm_and_filesystem` and the preserved
`cbm_scope_unavailable` value whose schema/tool description explicitly permits
healthy advisory CBM. Do not pin private branches beyond the public semantics.

## 26. Documentation/schema impact

- `docs/ARCHITECTURAL_INVARIANTS.md`: add the approved A/B authority
  invariants; clarify provider authority independently.
- `docs/agent/DISCOVERY_REGISTRY.md`: record the live DbContext coupling,
  mixed-language proof limit, and misleading diagnostic interpretation as
  separate discoveries.
- MCP tool schema in `src/mcp/tools.rs`: clarify `cbm_scope_unavailable`; if B
  adds public proof detail, keep it additive and non-strengthening.
- Coverage documentation and `docs/CLAUDE_INTEGRATION_RULES.md`: describe the
  capability -> certified predicate -> full-unit fallback order and the
  lower-bound escape.
- `.NET` meta-layer documentation: define `dotnet/DbContext` as a structured
  framework classification and separate it from `HasEntity` evidence.
- Issue #145 documentation: no semantic change; optionally cross-reference the
  role-versus-declaration distinction.
- Issue #146 record: update disposition to expected under the historical
  contract, with 147-A as the separately approved improvement.

Documentation must label current behavior, approved architecture, and future
implementation distinctly until code lands.

## 27. Smallest implementation slices for each approved change

The campaigns are independently reversible and should not share a commit or
feature gate merely for convenience.

### Campaign A — durable structured DbContext role

1. Add the narrow RED regression and ambiguity controls.
2. Extract/reuse one ambiguity-safe structured EF-base classifier at the C#
   framework boundary.
3. Publish a durable role-registration occurrence independent of edges.
4. Leave DbSet/model-building relationship producers unchanged except to use
   the shared classification result where appropriate.
5. Verify `find_entities`, framework filters, exact edges, zero-edge behavior,
   declaration behavior, owner lifecycle, deletion/refresh, and warm parity.
6. Update invariants and .NET documentation.

### Campaign B — composed TypeRef proof authority

1. Add RED tests for uncertified zero and full-unit success.
2. Introduce a narrow generation-bound producer-capability disposition for the
   existing `BuiltinTypeRefIncoming` families.
3. Exclude only proven-incapable units.
4. Retain certified C# narrowing; certify TypeScript only after its producer
   contract matrix passes.
5. Add full-unit authoritative processing for remaining capable/unknown units,
   reusing current artifacts and existing proof validation/publication gates.
6. Bind cache/proof generations to capability and predicate versions; preserve
   lower-bound on budget/failure.
7. Add performance counters/regressions protecting K-only certified work,
   O(N) full-unit fallback, warm reuse, and zero exact-call reconciliation.

### Campaign C — diagnostic clarification

1. Add the public semantics regression.
2. Clarify the MCP schema and operator documentation for the existing enum
   value without changing serialized output.
3. Update examples that currently imply provider outage.

## 28. Explicit non-goals

- No production implementation in this phase.
- No change to #145 declaration identity or `declarations_only` semantics.
- No relationship inferred from DbContext identity alone.
- No textual `: DbContext` authority where structured ambiguity matters.
- No compiler-grade C# binder claim beyond the explicitly supported resolver.
- No exactness from `discovered == compiled` or uncertified zero candidates.
- No implicit C# interpretation of language-neutral `builtin/TypeRef`.
- No CBM semantic authority and no provider-dependent truth.
- No global cache invalidation, whole-workspace reconciliation per query, or
  exact-call preparation for TypeRef.
- No overloading of `withinPath` as proof scope without an explicit versioned
  contract.
- No public enum rename in the additive C campaign.
- No live Claude field session until local implementation, deterministic RED /
  GREEN evidence, and batched local verification are complete.

## 29. Final recommendation

Approve three separate architecture decisions:

1. authorize an independent durable DbContext role only from supported,
   ambiguity-safe structured framework evidence;
2. authorize the composed TypeRef proof algorithm—generation-bound capability
   exclusion, certified narrowing, otherwise full-unit processing—with
   lower-bound as the failure/budget outcome; and
3. preserve `cbm_scope_unavailable` while clarifying that it describes missing
   authoritative scope coverage, not necessarily provider failure.

This distinguishes essential authority boundaries from accidental coupling:
the need for certified exhaustive proof is essential; requiring a
relationship before expressing an already proven framework role is accidental;
and treating a scope-authority diagnostic as a health diagnostic is avoidable
wording ambiguity.

147-A:
A2 — DBCONTEXT IDENTITY AND RELATIONSHIP EVIDENCE ARE IMPROPERLY COUPLED; STRUCTURED FRAMEWORK IDENTITY SHOULD BE DURABLE INDEPENDENTLY

147-B:
B5 — COMPOSED DESIGN REQUIRED (B2 + B3 + B4)

147-C:
C2 — CURRENT WORDING IS MISLEADING; ADDITIVE DESCRIPTION CHANGE ONLY

Issue #146 disposition:
CLOSE AS EXPECTED UNDER CURRENT CONTRACT

Production implementation authorized: NO

Smallest ordered implementation campaigns after approval:

1. Campaign C: additive diagnostic-description clarification and public
   semantics regression.
2. Campaign A: structured durable DbContext role with zero fabricated
   `HasEntity` edges.
3. Campaign B: producer-capability catalog, certified predicate dispatch, and
   full-unit authoritative fallback with #144 performance protections.
