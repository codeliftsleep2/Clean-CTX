# Clean-CTX — Changelog

**All notable changes to this project will be documented in this file.**

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
Historical releases are archived per version as `CHANGELOG_<version>.md`; the
version-history registry lives in
[`../CHANGELOG_VERSIONING.md`](../CHANGELOG_VERSIONING.md).

---

## [0.10.0] - 2026-10-16

`0.10.0` consolidates the complete 80-commit branch lineage after `0.9.3`:
the Angular template and invocation-evidence foundations, the pinned current-CBM
frontend upgrade, and the post-upgrade native correctness campaign through
Issues #132–#142. Across 434 changed files, CBM remains an optional acquisition
provider while Clean-CTX source evidence owns semantic truth.

### Pinned current-CBM frontend and wire upgrade

* **Owned frontend transport** — CBM communication now runs through an isolated
  frontend client with explicit request/reply correlation, bounded newline
  framing, launch-root binding, and coordinated retirement. A frontend must
  prove its protocol and capabilities before it becomes usable; admitted
  sessions can be replayed safely after recovery instead of silently reusing an
  unproven process.
* **Lifecycle and recovery correctness** — startup, disconnect, replacement,
  account/environment recovery, and child-process ownership are modeled
  explicitly. Concurrent or stale frontends cannot take authority merely by
  finishing last, and provider completion is never falsely reported while
  lifecycle work remains outstanding.
* **Pinned typed wire epoch** — adapter payload caches are isolated by the
  admitted wire epoch. Typed JSON normalization accepts only the pinned
  contract, provider limits survive the adapter boundary, indexing outcomes
  are validated, and paginated provider results are collected within the
  approved bounds without unbounded continuation.
* **Model-facing provider truth** — proxy errors, partial/indexing state,
  continuation observations, and provider availability remain structured and
  truthful. Provider pages and graph hints remain candidate acquisition
  evidence; they never become WorkspaceIndex relationships or bypass current
  source validation.
* **Acceptance assets** — A0–A5 contract, admission, lifecycle, recovery,
  continuation, current-wire, provider-state, cache-epoch, environment
  recovery, Phase B, and fresh-process verification artifacts were added.
  Phase B also reconciled the native Issues #133–#141 boundary rather than
  treating a successful provider upgrade as proof of Clean semantic behavior.

### Angular template authority and linkage (#132, #141)

* **Metadata-owned external templates** — components resolve external templates
  from exact `templateUrl` metadata rather than filename convention,
  proximity, or warm index state. Inline and external templates converge on
  canonical template identity while preserving their distinct source
  ownership.
* **Structure across Angular generations** — ordered template extraction
  preserves nested, sequential, control-flow, binding, directive, and custom
  element structure across legacy and modern Angular syntax, including
  repeated occurrences that must not collapse by spelling.
* **Occurrence-aware component linkage** — component-to-template and
  template-to-child relationships retain source occurrences and exact selector
  evidence. Ambiguous selectors remain multiple candidates; refinement does
  not manufacture a component declaration.
* **Diagnostics and fidelity honesty** — template diagnostics, ownership,
  declaration-only behavior, fidelity/degradation state, and model-facing
  linkage are exposed consistently through the production MCP path. The #132
  and #141 reconciliations close the original filename-derived and
  partial-linkage assumptions.

### Source-bound invocation evidence and RxJS

* **Generic invocation substrate** — canonical IR now carries source-bound
  invocation occurrences with callee spelling, receiver evidence, written
  argument-node count, spread evidence, owner/source identity, spans, and
  source hash. Evidence follows compilation and publication lifecycle instead
  of being reconstructed from compressed text.
* **TypeScript and C# producers** — both languages collect invocation evidence
  through parser-owned structure. The generic consumer boundary proves that a
  framework can consume this evidence without adding framework meaning to the
  language layer.
* **RxJS pipe semantics** — RxJS pipe-chain markers consume structural
  invocation evidence, retain operator order and duplicates, preserve complete
  UTF-8 operator text, and avoid comment/string false positives. Cross-layer
  and live MCP verification covers extraction, IR, meta-layer consumption,
  rendering, freshness, and phase boundaries.
* **Precise file-local calls** — `workspace_query(type="calls_in_file")`
  answers from fresh canonical High-fidelity file IR rather than Model C
  WorkspaceIndex identity. It selects typed owners and overloads, preserves
  call order/duplicates/spread evidence, and fails closed on duplicate owners.

### Declaration authority and shared semantic evidence

* **Independent declaration publication** — recognized Angular, NgRx, and
  related framework declarations publish independently of whether another
  relationship happened to mention them. Isolated declarations remain
  discoverable without self-edge fabrication.
* **Decorated TypeScript owner identity** — Component, Directive, Pipe, Module,
  and standalone owner names come from declaration fields, not decorator
  proximity or incidental text. Ambiguous/missing declaration proof emits no
  authoritative framework edge.
* **Structured Angular injection evidence** — injection tokens retain their
  structured kind and source identity across constructor parameters,
  `inject(...)`, qualifiers, string/symbol/class tokens, and supported
  framework declarations. Complete recognized DI registrations survive
  projection instead of being reduced to one convenient spelling.
* **Provider-registration vocabulary** — proven Component/NgModule provider
  contexts publish durable `RegistersProvider`, `ProviderUsesClass`,
  `ProviderUsesFactory`, and `ProviderUsesValue` relationships. Registration
  is not misrepresented as dependency consumption, unresolved endpoints publish
  nothing, and the Angular semantic producer generation rejects stale snapshots.
* **Signals and functional DI** — exact property initializers bound to
  `@angular/core` imports publish `HasInput`/`HasOutput` for `input`,
  `input.required`, `model`, and `output`. Proven functional guard,
  resolver, and interceptor owners may consume direct `inject(...)` evidence;
  receiver calls, local shadows, wrappers, and ambiguous targets remain
  negative.
* **Classic member identity** — setter inputs, optional/definite-assignment
  fields, and ordinary Input/Output declarations use their canonical property
  or accessor name. Decorator aliases remain metadata and never replace
  declaration identity.
* **Pipe constructor DI** — a Pipe participates in constructor dependency
  semantics only after exact Pipe declaration proof. Ordinary same-shaped
  TypeScript classes and ambiguous decorators fail closed.
* **C# diff declaration evidence** — diff comparison retains structured
  declaration identity instead of falling back to lossy labels where exact
  declaration evidence exists.

### Exact TypeScript and NgRx binding (#138–#140)

* **Language-level binding primitives** — exact same-file TypeScript identifier,
  `this.property`, and named type-import binding preserve symbol, module,
  source span, source identity, and source hash. They reject aliases, namespace
  forms, defaults, collisions, missing imports, unsupported qualification, and
  other ambiguous evidence rather than guessing.
* **Declaration-backed NgRx identities** — class Actions, modern and legacy
  Selectors, feature selectors, Services, reducers, effects, and relevant
  constants publish only from recognized declaration evidence. Direct imports,
  namespace Action references, qualified constructor dispatch, and one-hop
  named re-exports resolve to exact declarations.
* **Exact selector ownership** — Store receivers and Service selector owners
  are proven structurally. Selector references in calls and property
  initializers bind to exact declarations; same-spelled selectors in unrelated
  modules cannot acquire each other's ownership.
* **Action/reducer transitions** — legacy discriminants, multi-action reducer
  handlers, cross-file object dispatch, constructor dispatch, and modern
  service dispatch preserve exact Action identity and source ownership.
  Dynamic, missing, aliased, ambiguous, or unsupported forms emit no invented
  transition.
* **Recall verification** — the unrecoverable historical #140 benchmark was
  replaced with a pinned public Angular/NgRx corpus, deterministic
  normalization/scoring, 84 independently reviewed relationship pairs, a
  versioned oracle checksum, provider-disabled public-MCP execution, and
  reusable runners that preserve valid empty results.

### C# and .NET exact semantics (#133–#135)

* **Cold-query C# constructor dependencies** — large cold workspaces hydrate
  declaration candidates and return exact constructor dependency evidence
  without relying on prior queries or CBM semantic authority.
* **Declaration-proven test subjects** — MSTest/xUnit/NUnit-style test
  classification requires a real declaration and supported test evidence;
  mentions, filenames, and nearby attributes cannot fabricate a test class.
* **ASP.NET ancestry** — controller classification resolves exact declaration
  ancestry through structured C# base evidence. Same-name, ambiguous,
  unresolved, or incompatible candidates remain unclassified.
* **Direct controller declaration authority** — current structured
  `[ApiController]`/`[Controller]` evidence remains discoverable independently
  of derived relationship survival and optional provider or hydration state.
  Ancestry-only classification still requires exact, unambiguous authority.
* **Dependency traversal and cycles** — C# dependency traversal uses exact
  declaration catalogs and reports truthful cycle evidence instead of joining
  same-spelled types. The generic semantic-tuple cycle surface separately
  reports identity ambiguity and occurrence witnesses.
* **EF configuration identity** — Entity Framework configuration relationships
  resolve exact entities/configuration owners, preserving namespace, nesting,
  arity, source, and ambiguity boundaries.
* **DI residuals** — registration facts, interface-to-implementation evidence,
  qualifiers/tokens, constructor dependencies, and framework-independent
  declarations survive the complete production query path without
  first-match selection.
* **Exact C# Method identity** — proven callees publish
  `Owner::Method(ParameterTypes)->ReturnType` identities. Receiver/type
  binding covers proven `this`, static, constructed, parameter, local, field,
  overload, and extension-method forms; spread, missing, unresolved, or
  ambiguous targets fail closed while generic written-call evidence remains.
* **Two-pass exact-call reconciliation** — a C# declaration-authority change
  first reconstructs the current catalog from every admitted C# source owner,
  then recompiles admitted caller-bearing sources against that catalog. Source
  hashes are revalidated before one index replacement, making add/remove,
  signature change, edit/delta, restore/replay, recovery, and deletion
  independent of file order and query history.
* **Reconciliation fidelity preservation** — exact-call reconciliation may
  strengthen evidence but republishes each file at its existing semantic
  fidelity. A Low or Medium publication is never silently upgraded to High;
  only a later explicit higher-fidelity request can authorize that transition.
* **C# projection truthfulness** — Low/Medium/High skeletons expose stable
  C#-scoped omission categories and declared semantic loss; Verbatim/raw stays
  exact. The C# canonical and semantic-input producer generations advance to 3
  and semantic projection to 4 so older durable output cannot masquerade as
  the repaired identity model.
* **Request and file diagnostics** — batched `provide_code_context` rejects
  top-level `fidelity` with `-32602` because fidelity is item-owned.
  `entities_in_file`/`declarations_in_file` distinguish admitted empty
  files, `excluded_by_within_path`, `outside_authorized_roots`,
  `unsupported_file_type`, and `file_not_found` without echoing
  unauthorized paths.

### Safe mutation and Git change fidelity (#136–#137)

* **Declaration-owned insert-before** — `apply_edit` anchors insertion at the
  complete declaration start, including owned decorators/attributes, and
  rejects unknown comment/declaration ownership instead of placing code inside
  metadata or against the wrong nested declaration.
* **Overload and nested-owner selection** — signature-qualified unit identity
  distinguishes overloads, constructors, accessors, nested types, and
  same-named members. Legacy bare selectors are accepted only when unique;
  ambiguous or stale selection fails before mutation.
* **Source-fidelity transaction** — byte-exact expected text, UTF-8 validation,
  EOL preservation, non-overlap, syntax gating, live/durable source
  hash/version checks, physical alias guards, same-directory staging, atomic
  replacement, and recoverable durable edit intent remain one closed
  transaction.
* **Semantic Git signatures** — `diff_commits` and related Git change
  projection retain structured signatures for constructors, overloads,
  generic/tuple returns, nested declarations, expression bodies, and
  declaration-only changes. Body-only and semantic-signature changes are no
  longer hidden by lossy display labels.
* **Comparison disclosure** — model-facing `§COMPARISON` and
  `_meta.comparison` report comparison scope, dimensions, omissions, and
  whether the semantic projection is lossy. Unchanged members remain
  suppressed while their changed owning declaration remains visible.
* **Binary and Git-ref safety** — NUL/non-UTF-8 blobs are labeled binary and
  never parsed into invented source; deleted and renamed binary paths preserve
  Git truth. Admitted refs include decimal `~N`/`^N` parent selectors while
  unsafe syntax remains rejected with explicit accepted-grammar guidance.

### Angular composition, routes, and lifecycle (#138–#139)

* **Exact NgModule composition** — declarations, imports, and exports retain
  their distinct relations while endpoint kind comes from an exact decorated
  declaration. Slot position never invents Component/Directive/Pipe/Module
  identity.
* **Structured routes** — nested/full-path routes, guards, resolvers,
  `RouterModule.forRoot/forChild`, lazy components, lazy modules, and exported
  Routes arrays preserve their structured ownership. Dynamic, external,
  missing, or ambiguous targets fail closed.
* **Lazy-target dependency fingerprints** — derived route semantics record the
  source identity/hash of every direct-relative target consulted, including an
  explicit missing state. Changed, deleted, restored, or recreated targets
  invalidate derived snapshots without invalidating unrelated canonical IR.
* **Refresh/republication correctness** — `refresh_workspace`, unchanged
  asserting sources, durable restore, and later `provide_code_context`
  publication cannot resurrect a relationship invalidated by an external
  target change.

### Workspace query authority, optionality, and history independence (#142)

* **Acquisition before interpretation** — heterogeneous query batches acquire
  the union of required source evidence before final C# or Angular
  interpretation. Result meaning is independent of item order, repeated calls,
  and unrelated warm-up queries.
* **Source/fidelity certificates** — WorkspaceIndex coverage records canonical
  file identity, semantic fidelity, and source hash. A stale, insufficient, or
  missing projection recompiles read-only source and atomically replaces the
  owner's entities, edges, and coverage, including an empty result.
* **Provider-independent truth** — CBM may improve candidate discovery cost or
  breadth, but provider absent, ready, indexing, partial, failed, recovered, or
  restarted states cannot author Clean semantic facts. Filesystem fallback and
  sparse diagnostics preserve the declared query contract.
* **Root-scoped refresh and multi-root isolation** — requested-root refresh
  retracts only facts owned by that root, invalidates the matching discovery
  generation, and leaves unrelated configured roots current.
* **Truthful result contracts** — exact identity/capability responses,
  declaration-only queries, file certificates, lower-bound graph families,
  ambiguity, unsupported producers, and next-action diagnostics remain
  distinct. Result cardinality never substitutes for completeness.
* **Restart and persistence equivalence** — restored state must pass current
  source, compatibility, dependency, and ownership checks before adoption;
  durable presence or provider recovery cannot improve truth.
* **Model-facing envelopes** — `workspace_query` single and batch responses
  include a portable model-facing answer envelope while retaining authoritative
  `structuredContent`. Occurrence arrays stay ordered and each result remains
  failure-isolated.

### Identity-collision and release-blocker reconciliation

* **Same-name evidence remains occurrence-aware** — Model C semantic identity
  remains `(domain, entity_type, name)`, while declaration occurrences and
  edge evidence retain canonical file provenance. An explicit exact-tuple edge
  query returns each occurrence with endpoint `file` identity; it does not
  silently render several declarations as one declaration's unproven
  relationships.
* **Precise operations use stronger identity** — file-local call inspection
  uses canonical IR, framework refinement requires declaration proof, and
  `apply_edit` uses explicit file/unit/source authority. Coarse WorkspaceIndex
  identity is never promoted into mutation authority.
* **Final adversarial audit** — the release-blocker audit found no serious
  current correctness, lifecycle, provider, warm-state, mutation, panic, hang,
  or process-lifecycle defect. Issue #40 is superseded by the current
  occurrence-aware public authority model; the open issue was not mutated by
  the audit.

### Documentation and reusable verification

* Added final reconciliation records for Issues #132–#142, CBM A0–A5 and Phase
  B evidence, authority-boundary investigations, implementation plans,
  production-path traces, and the 2026-10-16 release-blocker audit.
* Added tracked regressions under `src/tests/**` for every repaired
  correctness boundary and kept live/operator harnesses separately under
  `verification/**`, including issue-reconciliation drivers, invocation
  evidence, Angular template linkage, CBM lifecycle, workspace-query behavior,
  TypeScript binding, lifecycle refresh, and NgRx recall.
* Updated architectural invariants, tool schemas, runtime prompts, Claude
  integration guidance, discovery records, and verification ledgers to match
  the production architecture. No live harness is represented as a Rust test
  or CI result.

### Compatibility and explicit non-changes

* This minor-version release intentionally includes new public query/envelope
  behavior, exact semantic producers, producer-generation changes, and the
  pinned provider frontend contract. Consumers should rebuild the binary and
  allow incompatible cached semantic projections to regenerate.
* CBM remains optional and advisory. No provider graph edge becomes core
  semantic truth, no eager global workspace compilation was introduced, and no
  global completeness claim was added to lower-bound query families.
* WorkspaceIndex Model C identity is unchanged. Precision is supplied by
  occurrence provenance, query scope, declaration catalogs, canonical
  file-local IR, and mutation-specific source authority rather than a global
  identity redesign.
* No new global timeout, candidate cap, truncation policy, background watcher,
  or repository-wide semantic inference framework was introduced.

### Verification

* The branch records unchanged RED→GREEN tracked regressions for the repaired
  boundaries and operator-reported focused tests/live acceptance for the
  individual CBM, Angular, NgRx, C#/.NET, edit, Git-diff, lifecycle, and #142
  campaigns. Those historical results are attributed in their reconciliation
  documents.
* Agent MaxHeadRoom performed static ancestry, diff, manifest, changelog,
  encoding, file-size, and release-surface inspection for this metadata pass.
  No Cargo build, check, test, Clippy, binary, provider, live harness, or model
  pilot was run by the agent.
* The consolidated `0.10.0` authoritative final verification gate,
  post-changelog encoding gates, and
  `cargo pkgid -p clean-ctx` package-identity confirmation remain
  maintainer-run release requirements. They must not be reported green until
  the maintainer supplies their results.

---

## [0.9.3] - 2026-10-10

### Resolved C# generic and qualified base types

* **Structured query-time resolution** — neutral C# Class base-list facts now
  resolve qualified and constructed-generic references through parser-owned
  namespace, nesting, terminal-name, and generic-arity identity. A uniquely
  proven target adds the existing ephemeral `Extends` or `Implements` edge;
  ambiguous, missing, or arity-incompatible targets remain neutral.
* **Source-true durable facts** — the written qualified or generic spelling
  remains queryable as the authoritative `HasBaseType` relationship to its
  `TypeRef`. Resolution consumes only the exact cached source snapshots that
  produced the current workspace index, so externally edited bytes cannot be
  mixed into the active semantic generation.
* **Truthful query guidance** — successful reverse Class/Interface queries no
  longer advertise an `alternative_query` as though their typed capability
  were unavailable. Unresolved queries retain the neutral `TypeRef` /
  `HasBaseType` guidance.
* **Explicit external-source reconciliation** — the new read-only, idempotent
  `refresh_workspace(workspaceRoot)` MCP tool reconciles the primary and
  configured additional roots, retracts indexed semantic state, invalidates
  source/discovery snapshots, and retires stale pending semantic transitions
  after editor, shell, Git, or other out-of-session writes. It reports affected
  roots and authority counts while preserving durable history and diff
  baselines. The next query hydrates current source without requiring CBM; `index_repository` remains the separate CBM graph-refresh
  operation.
* **Compatibility remains scoped** — canonical IR, durable semantic facts, and
  their C# producer outputs are unchanged, so no producer key or generation was
  added or incremented. The behavior change is confined to ephemeral query
  refinement and the explicit public refresh contract.

### Verification

* RED was established with tracked production-path regressions for qualified
  namespace selection, constructed-generic arity resolution, arity-mismatch
  neutrality, conditional alternative-query guidance, and CBM-independent
  external create/replace/delete reconciliation. The unchanged focused suites
  passed after implementation.
* The maintainer reported the complete final verification gate GREEN before
  release-metadata finalization. The v0.9.3 package-identity and post-changelog
  encoding checks remain the final release commands.

---

## [0.9.2] - 2026-10-09

### Structured C# declaration-header authority

* **One parser-owned declaration identity** — C# Class, Interface, Struct, Enum,
  and Record names now come from structured identifier captures shared by
  canonical lowering and builtin registration. Partial declarations, primary
  constructors, and positional records no longer corrupt or truncate the
  public owner identity.
* **Structured Class base-list lowering** — canonical C# Class relationships
  now consume direct parser base-list children. Multiline lists preserve every
  entry, qualified and generic written references remain intact, and comments
  or generic constraints cannot be misread as inheritance. The former
  text-derived Class relationship extractor was removed rather than retained
  as a competing authority.
* **Truthful exact-query capability** — an exact edge response containing a
  derived typed `Extends` or `Implements` edge now establishes that
  query's capability even when the edge is intentionally ephemeral. Coverage
  remains a lower bound when hydration is incomplete, and neutral
  `TypeRef / HasBaseType` alternative-query guidance remains available.
* **Scoped durable compatibility** — existing `CSharpCanonical`,
  `CSharpSemanticInput`, and `CSharpSemanticProjection` producer
  generations were incremented for their changed outputs. No producer key or
  unrelated-language compatibility identity was added or changed.
* **Struct/Record preservation** — C# Struct and Record base entries retain the
  v0.9.1 neutral `HasBaseType` behavior and distinct public owner kinds;
  this release only unifies their declaration-name authority with the
  structured C# identity path.

### Verification

* RED was established with seven tracked production-path regressions covering
  multiline and partial Class headers, constraints and comments, shared
  canonical/builtin identity, exact-query coverage, and producer generations;
  the qualified/generic neutral-reference control passed before implementation.
  The unchanged focused suite passed after implementation.
* The complete release gate and `cargo pkgid -p clean-ctx` remain
  maintainer-run requirements before the `v0.9.2` release tag.

---

## [0.9.1] - 2026-10-09

### Queryable neutral C# class, struct, and record base types

* **Cross-file C# base-type discovery** — unresolved or cross-file C# class, struct, and record base-list
  entries now project as source-true `HasBaseType` relationships from the
  correct `builtin / Class`, `builtin / Struct`, or `builtin / Record` owner to the written `builtin / TypeRef`. Forward
  queries can inspect written base types, and reverse queries on the neutral
  `TypeRef` can find declarations that name that type.
* **Cross-file C# class classification** — exact Class and Interface edge
  queries now refine neutral Class base-list facts into ephemeral `Extends`
  or `Implements` results after completed scoped hydration proves exactly one
  target kind. Cold forward Class queries hydrate each distinct neutral target
  name before classification. The durable `HasBaseType` fact remains
  authoritative and queryable; absent or ambiguous targets remain neutral, and
  Struct/Record facts are never consumed by the Class classifier.
* **Struct and Record identity preservation** — parser-structured C# base-list
  captures now publish every written qualified, generic, multiline, and nested
  base type from the existing `builtin / Struct` or `builtin / Record` owner,
  never through a fabricated Class identity. Republishing or removing an entry
  retracts the old fact without duplicates, and no unsupported `Extends` or
  `Implements` relationship is invented.
* **Scoped durable compatibility** — the changed edge projection is identified
  by a C#-specific semantic-projection producer generation. Pre-change C#
  semantic snapshots are rejected with the existing structured compatibility
  error and must be regenerated; unrelated language snapshots are unaffected.
* **Query guidance** — unsupported reverse lookup on a same-name
  `builtin / Class` or `builtin / Interface` now points callers to the neutral
  `HasBaseType` / `TypeRef` query.

### Verification

* RED was established through tracked production-path forward and reverse
  `workspace_query` regressions, both returning empty edge sets before the
  fix. The unchanged regressions and focused projection, compatibility,
  restore, coverage, Struct/Record publication and lifecycle, and cross-file
  Class classification suites passed after implementation. The final
  all-target/all-feature Clippy gate also passed with zero warnings.
* The complete release gate and live MCP pilot remain maintainer-run
  requirements before the `v0.9.1` release tag.

---

## [0.9.0] - 2026-10-05

`0.9.0` hardens the authority model beneath the correctness-oriented context
infrastructure introduced in `0.8.0`. State no longer becomes current truth
merely because it exists, deserializes, was persisted, or finishes last. It
must cross the structural, compatibility, ownership, source-precondition, and
owner-publication boundaries appropriate to that state.

### Authority and durable-state hardening

* **Scoped durable compatibility (#116, #117, #119)** — durable canonical and
  semantic state now carry four independent identities:
  `CanonicalConfigIdentity`, `CanonicalProducerIdentity`,
  `SemanticConfigIdentity`, and `SemanticProducerIdentity`. Only dimensions
  capable of affecting the corresponding projection participate. Package,
  crate, Git, SQLite-schema, binary-wire, and whole-configuration fingerprints
  are not treated as semantic compatibility evidence. Missing legacy evidence
  is unproven rather than silently trusted.
* **Compatibility-aware publication (#116)** — restore, replay, recovered edit
  publication, and mutating delta application must produce a
  `CompatibleSemanticProjection` before semantic snapshot, coverage, or
  `WorkspaceIndex` authority changes. Handler-specific remove/add publication
  sequences were consolidated behind the shared typed boundary.
* **Durable semantic structural validation (#121)** — successful JSON decoding
  no longer suffices for adoption. Durable edges enforce intrinsic
  relation/evidence coherence (`Calls` if and only if call evidence exists)
  before entity restoration, interning, or live publication.
* **Coherent checkpoints (#123)** — `save_context` captures canonical IR,
  version, source identity, fidelity, semantic projection, and compatibility
  identities from one owner-authority epoch. Persistence I/O occurs after the
  immutable payload is captured, without imposing a global authority lock.

### Unified owner authority ordering

* **Source-ordered semantic publication (#113)** — same-owner compilation and
  publication use `SemanticPublicationClock` generations established before
  source selection. An older source candidate cannot overwrite a newer
  committed owner merely because it completes later; different owners remain
  independently processable.
* **Delta baseline epochs (#122)** — `delta_code_context` binds baseline
  capture, candidate compilation, delta derivation, pending-transition
  registration, and commit validation to one authority ticket. A transition is
  rejected or retried if its exact `from` baseline or source precondition is
  superseded.
* **Historical adoption ordering (#124)** — restore and replay retain their
  operation generation through final commit. Operation age is distinct from
  snapshot age: a stale in-flight operation cannot win by completion order,
  while a genuinely later operation may intentionally adopt compatible older
  history.
* **Commit-time edit compare-and-swap (#125)** — `apply_edit` re-proves the
  exact physical source bytes under the owner commit boundary before staging,
  durable intent, replacement, or live publication. Atomic replacement alone
  is not considered compare-and-swap.

### Current-source and restore authority

* **WorkspaceIndex source-universe reconciliation (#118)** — an authoritative
  refresh structurally retracts evidence owned by sources proven absent from
  the current semantic source universe. This protects hydrating and
  non-hydrating queries without per-query stale filters, filesystem watching,
  source-independent restore, or CBM authority over semantic truth.
* **Trusted restore admission (#120)** — `restore_context` resolves and admits
  the requested path through `workspaceRoot` and configured additional roots
  before durable identity registration, edit recovery, persistence loading,
  alias/canonical publication, semantic publication, index mutation, or cache
  publication. Nearest-existing-ancestor checks close dangling-symlink escapes.
* **Restored coverage truthfulness (#114)** — compatible restored projections
  reconstruct Low/Medium/High coverage only where certifiable. Edit and
  Verbatim states remain uncertified when equivalent semantic coverage cannot
  be proven.

### Semantic relationship and hydration correctness

* **Truthful inheritance projection (#98)** — `Extends`, `Implements`, and
  `InterfaceExtends` reach the semantic relationship model. Ambiguous C# base
  references remain neutral `BaseTypeRef` facts until resolution supplies
  classification evidence.
* **Fidelity-aware hydration (#99)** — hydration escalates fidelity when
  existing coverage is insufficient, and caches completion only after the
  required semantic projection becomes current.
* **Exact identity/capability reporting (#100)** — workspace queries distinguish
  genuine absence from identity or capability that could not be established;
  an unproven zero is no longer presented as a complete answer.
* **Publication-complete discovery (#112)** — candidate discovery is committed
  only after every selected candidate is already current or publishes the
  required projection successfully. Failed publication remains retryable.
* **Producer and physical-identity corrections (#108, #110)** — declaration
  extraction now handles post-parameter language syntax without corrupting
  identities, while reads may collapse hard-linked aliases and mutations reject
  multiply linked files whose path-based edit ownership would be ambiguous.
* **Verified non-changes (#102, #105)** — isolated declarations were confirmed
  to retain identity through self-`Defines` registration, and constructor-type
  plus inheritance relations remain intentionally outside automatic
  `transitive_dependencies` traversal policy. No speculative repair was added.

### Git change-set correctness

* **Pre-truncation accounting (#126)** — `file_count` records the complete
  changed-file set discovered before `max_files` limits processing. Limit and
  later processing/size skips contribute to `skipped`, preserving
  `added + deleted + modified + renamed + type_changed + skipped == file_count`.
* **File-type changes (#127)** — Git `T` status now maps to
  `FileChange::TypeChanged`, renders as `~ FILE αN: path (type changed)`, and is
  exposed as `_meta.counts.typeChanged`. The legacy four-element count tuple is
  unchanged, and Clean-CTX does not invent specific old/new physical types.

### Documentation

* Living architecture, IR, configuration, developer, diff, README, invariant,
  public MCP schema, roadmap, changelog, and version-registry surfaces now
  describe the 0.9.0 authority model. The September historical audit remains
  unchanged so it stays truthful to its original date.

### Verification

* Tracked regression coverage was added or strengthened for inheritance,
  identity/capability reporting, hydration completion, same-owner publication,
  restored coverage, durable compatibility, semantic snapshot validation,
  source-universe reconciliation, restore admission, delta/save/historical
  races, stale edits, Git truncation accounting, and real Git type transitions.
* Release metadata checks completed by the agent: strict UTF-8, file-size
  policy, whitespace validation, and targeted formatting checks. The complete
  Cargo/Clippy/test/encoding release gate and `cargo pkgid -p clean-ctx` remain
  maintainer-run requirements before the release commit and `v0.9.0` tag.

---

Historical release details:

* [`0.8.0` and `0.8.0-rc`](CHANGELOG_0.8.0.md)
* [`0.7.0`](CHANGELOG_0.7.0.md)
* Older archives are retained in this directory under their shipped versions.
