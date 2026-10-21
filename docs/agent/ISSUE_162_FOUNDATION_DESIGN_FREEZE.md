# Issue #162 foundation design freeze

Agent: Agent MaxHeadRoom

Date: 2026-10-21

Status: **APPROVED AND FROZEN by the maintainer**. Implementation is tracked in
[the working implementation plan](ISSUE_162_IMPLEMENTATION_PLAN.md), including
the mandatory Angular-wide consumer migration matrix and current incomplete
boundaries. The design-phase statements below describe the proposal at approval;
they do not claim implementation or verification. No production code, dependency,
public schema or frozen test changed during this design phase. No Cargo, binary,
MCP/provider or live process ran during that phase.
The established 3 PASS / 9 RED receipt is sufficient for this design phase.
The exact regression hash remains
`B63883E818FC70DF0990D23E2C31AC9D318FED458DA74B70B693C393DBA86379`.

## 1. Selected foundation

Recommend **native in-process Rust authority**: one project-aware TypeScript
file resolver, lexical/export-symbol binder and retained declaration graph,
consumed by Angular-owned interpretation/composition. Use existing tree-sitter
syntax/binding and source/publication mechanics. No Node/TypeScript runtime,
compiler subprocess, resolver crate, package install or compiler-plugin execution
is part of this freeze. Implementing this is conformance work, not an extension
of three existing relative-path helpers. The declared #162 envelope remains a
release requirement if any required mode/form is still unimplemented.

Select **producer-owned retained canonical identity**, not global EntityRef
equality migration. Retain exact TypeScript declarations and canonical Angular
facts before converting them to public semantic edges. Preserve existing public
EntityRef equality, shared EdgeKey, non-Angular producers and name-bucket query
contracts. Those buckets become candidate/presentation indexes for these Angular
facts; they must never become authority for a canonical join.

This is a concrete architecture decision, not a claim that native conformance or
consumer integration has already been demonstrated. Native correctness must be
earned by the acceptance matrix; no weaker compiler substitute or automatic
fallback to a different backend is authorized if it fails.

## 2. Repository evidence that determines the boundary

| Evidence | Architectural consequence |
|---|---|
| Cargo.toml: Rust binary/library, tree-sitter TypeScript, serde/serde_json; no canonical TS project resolver | Reuse syntax, not an imaginary existing compiler authority; no runtime/dependency addition is implicit |
| ir/typescript_value_import_binding.rs: exact reference spans/hash, aliases and namespace bindings | Extend neutral binding mechanics; do not reimplement local binding per Angular consumer |
| module_metadata/sources.rs, routing/structured.rs, ngrx/object_actions.rs: separate path policies | Converge file/export resolution in a language-owned boundary |
| layers/meta/semantic.rs: EntityRef equality/hash exclude file | Filling file provenance is necessary but insufficient for joins |
| workspace/index/edges.rs: EdgeKey deduplicates by asserting file and semantic names; endpoint declaration identity is absent | Two same-name endpoints in one asserting source can collapse; retain canonical facts before add_edges |
| query/edge_merge.rs: merge comparator also ignores endpoint files | Do not route authoritative Angular results through this lossy merge |
| query/identity.rs: exact public identity is domain/type/name | Existing public queries address name buckets, not one canonical declaration |
| semantic_compilation/artifact.rs: artifact already retains source and C# canonical evidence | Retain TS/Angular evidence in the real compilation artifact, not a test-only catalog |
| mcp/durable_semantics.rs and sqlite_store/semantic_state.rs: complete snapshots and transactional JSON payload storage | Extend the existing snapshot ownership/payload, not a second dependency database |
| compatibility/validator.rs: producer compatibility and supplied dependency currentness | Extend typed validation to all new witnesses; incompatible old semantic snapshots must rederive |
| query/angular_modules.rs: #157 validates known owners independently of discovery completion | Generalize producer obligations/owner validity; do not equate cached discovery with current semantics |
| Angular template catalog and NgRx refinements are specialized consumers | Share resolution facts, preserve framework-specific interpretation |

The critical counterexample is one module importing two distinct declarations
named Leaf from different files under aliases. Both can render as the same
semantic triple. An identity side table populated after shared edge dedup cannot
recover the lost fact. Retention before dedup is therefore mandatory; merely
adding file fields is rejected.

## 3. Backend alternatives

| Criterion | Native in-process Rust (selected) | Compiler/compiler-host-backed |
|---|---|---|
| Correctness | Explicit TS binding/export and project-mode semantics; no inference/typechecker equivalence claimed | Compiler supplies its own module/symbol semantics; host snapshots and Angular meaning still need Clean proof |
| Conformance burden | High: options, conditions, exports, merging, shadowing and negative cases must be tested | Lower TS algorithm burden, but compiler-version matching and host correctness remain |
| TS/Angular versions | Versioned resolution semantics and separate Angular API/version catalog | Compiler version becomes part of every proof; installed version alone cannot decide Angular meaning |
| Package fidelity | Must implement mode-specific conditions, substitution and package boundaries | Strong for the selected compiler/mode; can still select declaration artifacts rather than runtime implementation |
| Compiler options | Parse applicable JSONC/extends/project ownership, no nearest-config shortcut | Compiler config APIs help; ambiguous project ownership and authorization remain host responsibilities |
| Deployment | Existing Rust process; no new executable or project package execution | Adds Node/TS distribution or another approved compiler runtime, startup, protocol and failure lifecycle |
| Performance | Narrow resolution frontier, shared parsed modules, no process transfer; unmeasured | Can reuse a compiler program, but program creation/memory/IPC and multiple project versions are real costs |
| Cache | Source/config/mode-bound immutable evidence | Compiler program/cache must be bound to the same observed host snapshots |
| Determinism | Explicit snapshot/config/options keys and sorted candidate sets | Requires pinned compiler and a deterministic intercepted host, not ambient ts.sys |
| Security/sandbox | All IO through Clean-authorized snapshot host | Custom read-only host plus process restrictions; project plugins/code must never execute |
| Lifecycle | Fine-grained proof invalidation under existing owners | Compiler program updates plus Clean publication/persistence must advance consistently |
| Witnesses | Host directly captures every read/probe; symbol graph retains contributors | Must intercept reads, failed probes, config/module resolution and symbol contributors, including compiler cache hits |
| Product fit | Fits current compilation/artifact/publication lifecycle | New cross-process deployment/version/ownership contract needs explicit approval |

This selection is not based on Rust being easier: native conformance is harder.
It avoids introducing a second executable authority/lifecycle while making every
proof input observable under the existing product owner. The required problem
is module/binding/export identity, not general type inference, overload selection
or arbitrary program evaluation. Do not silently grow that distinction into a
new exclusion for any required #162 import/export/composition form.

Considered but not selected: a native path library plus existing spelling-based
Angular helpers. Oxc's documented FileSystem abstraction and file/missing
dependency collection show that instrumentable native file resolution is
feasible, but its file resolver alone does not establish exported declaration
identity or certify exact TS option conformance. Upstream project/config issues
also preclude treating a library name as proof. A later crate proposal must
review API/MSRV/license, IO/cache observability and mode conformance; it cannot
change this authority contract or bypass the Rust 1.85 manifest requirement.

Official references: [TS module semantics](https://www.typescriptlang.org/docs/handbook/modules/reference),
[compiler API](https://github.com/microsoft/TypeScript/wiki/Using-the-Compiler-API),
[Oxc file resolver](https://docs.rs/oxc_resolver/latest/oxc_resolver/),
[interceptable filesystem](https://docs.rs/oxc_resolver/latest/oxc_resolver/trait.FileSystem.html),
[upstream conformance issues](https://github.com/oxc-project/oxc-resolver/issues).
These support the comparison; no third-party implementation was installed/run.

## 4. Canonical identity representation and owner

The following are conceptual records, not approval to introduce a generic
registry/trait framework or these exact Rust type names:

```text
SourceSnapshot = { physical_source_owner, content_hash, bytes }
DeclarationId  = { physical_source_owner, content_hash,
                   declaration_span, lexical_container, declaration_kind }
SymbolId       = { canonical_declaration_family }
Reference      = { source_snapshot, exact_span, requested_namespace, project }
ExportBinding  = { exporting_module, exported_name, target_symbol, route_evidence }
AngularFact    = { asserting_owner, producer/site, relation,
                   canonical_subject, canonical_object, proof }
```

Declaration spans belong to exact hashed bytes. An edit that shifts a declaration
creates a new identity; stable semantic lineage across edits is not assumed.
Physical owner uses the established canonical path/physical-alias mechanics,
not an alias number, project slug or package name. Persist paths/hash/spans;
revalidate physical alias ownership on reopen instead of persisting OS file IDs.

Legal declaration merging/overload families require binder-established shared
scope and merge compatibility; they are not arbitrary sorted same-name classes.
A symbol can inhabit both value and type spaces. The requested namespace and
type-only import/export restrictions select usable bindings without pretending
a class has two unrelated declarations. Aliases/re-exports identify routes to a
symbol, not new copies of that symbol. Cyclic aliases with no rooted declaration
do not create an identity.

Ownership: the canonical compilation result retains source-local declaration/
reference/export evidence. Completed Angular projection retains immutable facts,
composition/scope proof and dependency witnesses in its existing source-owned
semantic artifact/snapshot. Session state owns the current artifact; durable
snapshots carry the corresponding typed payload. Request-local views own only
temporary resolution/composition memoization, never the sole surviving proof.

Do not key a retained fact by public EntityRef/name/edge ordinal. Its key uses
canonical endpoints plus asserting source and producer evidence site. Persist
this typed evidence in the existing versioned semantic snapshot payload; legacy
snapshots without it cannot be promoted to canonical Angular authority.

## 5. Resolution pipeline

1. Capture the exact written reference and lexical scope in immutable source
   bytes. Resolve local/parameter/namespace/import binding and namespace meaning;
   shadowing is decided here, not after filename search.
2. Establish applicable TS project and options, including JSONC, extends chain,
   option-origin-relative paths, include/exclude/references and explicit project
   ownership. Multiple plausible projects produce an unproven/ambiguous result
   unless the production request/config selects one; do not choose newest index.
3. Resolve the specifier under the effective mode, importing-file format,
   import/require use, TS version, conditions and options. Follow applicable
   relative/extensions/index rules, baseUrl/paths precedence/fallbacks, package
   self-reference/node_modules/type routing, exports/imports maps and conditional
   branches. Spec-invalid targets do not fall back past a package export barrier.
4. Return a physical module plus its resolution evidence. This is **file
   resolution only**. A successful path is not proof that an export exists.
5. Bind the requested export in that module: direct/named/default/namespace and
   type-only forms, local export lists, named re-exports and star barrels. Use
   explicit-export precedence and namespace rules from TS. Resolve aliases to
   canonical declaration families; two routes to one symbol are not ambiguity.
6. Traverse cyclic export graphs by strongly connected components/worklist until
   candidate sets stabilize; an active-stack early empty result must not be
   cached as final. Conflicting rooted symbols remain ambiguous; deterministic
   order is for diagnostics, never authority.
7. Return Resolved/Ambiguous/Unresolved with consumed witnesses. Angular then
   proves package/API role and the appropriate declaration/composition meaning.

Use direct language-owned composition, not a configurable multi-backend runtime
registry. Unsupported/invalid/unresolved semantic inputs remain distinguishable
internally. None permits first-match recovery or silently closes required work.

## 6. Dependency and same-snapshot model

All resolution/interpretation reads use one snapshot host enforcing authorized
roots before access and after physical link resolution. The byte buffer parsed
is the buffer hashed. A later read is allowed only as **validation against**
that hash, never as retroactive certification of different consumed bytes.

| Witness | Binding and invalidation |
|---|---|
| Source/config/package content | Existing SemanticDependency(path, hash) from the consumed bytes; any content mutation invalidates |
| Absent candidate/config/manifest | Explicit absence observation; appearance invalidates. Do not conflate access denied, unsupported path and ENOENT |
| File/directory kind and ordered search | Record metadata facts used to choose a candidate, precedence and any enumerated names; kind/list changes invalidate |
| Canonical/symlink/package path | Record lexical route and physical target evidence, so link retargeting invalidates even if old target bytes stay unchanged |
| Project and option interpretation | Config origins, extends/project ownership, effective mode/conditions, TS semantics version and root authorization fingerprint |
| Ambiguous export/candidate universe | All consulted contenders and relevant negatives/export surface; contender change/removal can resolve or change ambiguity |

Existing SemanticDependency encodes content/absence hashes, but is_current treats
all read errors alike. It cannot alone prove metadata/search/link distinctions.
Extend the existing dependency/snapshot proof vocabulary with typed resolution
witnesses and one validator; do not serialize fake metadata files or build a
parallel generic dependency system. Flatten only witnesses faithfully expressible
as existing path/hash entries; retain the remaining typed proof in the same
snapshot payload. Failure to validate a witness yields unproven/recompute, not
certified absence.

Pure snapshots are lazily captured per path, then validated as a consistent set
before publication. If the filesystem changed during capture, discard/reprepare;
do not claim an atomic filesystem snapshot or immunity to later edits. Mutation,
deletion, restoration, exports/config/project/package changes and alias retargeting
invalidate both positive and negative results. Capture witness sets even for
failed resolution so newly available evidence can restore truth.

## 7. Angular API and composition interpretation

Framework authority is a resolved package instance/export/declaration route plus
the versioned Angular API-role contract, not just package.json.name, file naming,
decorator spelling or a field called ɵcmp. Aliased/re-exported Angular APIs must
resolve back to that role. Project path mappings can shadow @angular/core; that
must resolve to the actual target, never bypass it via an allowlist.

Library declaration artifacts need canonical Angular metadata-type authority
where decorators are compiled away: e.g. a library field's type resolves to the
appropriate Angular declaration API in the proven core package/version. Field
spelling or a lookalike generic type is insufficient. This is Angular-owned
interpretation of type-space evidence, not promotion of type-only imports to
ordinary runtime values.

One Angular composition evaluator consumes exact bound expressions for both
NgModules and standalone Components. Arrays/local consts/nested spreads/imported
arrays/barrels carry their lexical declaration context. An active expression/
symbol identity detects composition cycles; a separate completed memo stores
finished results. Preserve independent proven leaves and unresolved obligations.
Do not confuse this expression recursion with export SCC resolution.

Standalone eligibility and defaults require Angular version evidence; version
absence cannot silently apply today's defaults. NgModule declaration/import/
export visibility and standalone imports derive a closed **visible declaration
set**, not a workspace-wide selector search. Directives/Pipes participate in
eligibility; they do not acquire Component imports metadata by analogy.

Routing, provider/DI, host/query, templates, NgRx, Signals, RxJS, forms, Formly and
testing receive exact symbols/proofs and keep their specialized interpretations.
The shared layer never decides that a declaration is a reducer, a guard or a
provider. Provider token identity is its declaration/API identity or explicit
primitive-token identity within the proved injector boundary; equal terminal
names and equal textual labels are not equivalent tokens.

### Legacy fixture compatibility is explicit

#157 fixtures import @angular/core but do not supply an installed package or
project/package authority witness. The new #162 fixture does. Strict declaration/
package proof cannot honestly certify the missing #157 package from its import
spelling. Neither a native resolver nor a compiler host fixes missing evidence.

Recommendation for implementation: enrich legacy fixture **environment setup**
with explicit synthetic framework package/project evidence while preserving their
behavioral assertions and the historical #157 frozen receipts in Git. Do not
weaken the new authority rule, hide a test-only known-package fallback, or edit
the frozen #162 regression. Include this fixture migration explicitly in the
maintainer's freeze approval; without approval it is not implemented. Old
uninstalled workspaces may retain syntax/lower-bound observations, but cannot
claim fully proved package-backed relationships. Any public behavior/schema
change needed to distinguish such observations must be approved separately.

## 8. Production publication, persistence and MCP consumers

```text
reference/declaration capture -> artifact returns neutral evidence
 -> producer-owned required obligations -> shared authorized acquisition
 -> Angular canonical facts + exact dependency/scope proof
 -> existing owner publication ticket + typed witness validation
 -> current session semantic snapshot + durable semantic payload
 -> final request/batch validation -> canonical consumer join
 -> existing public semantic presentation
```

Publish canonical facts/proof and their presentation as one source-owned semantic
generation. Do not publish the public edges first and retrofit identity later.
An owner reset/delete/recompile replaces or retracts both. Keep source-proven
declaration ownership when memberships become empty, preserving #157 restoration.
Changed contributors make old derived facts ineligible before the next consumer
can observe them, even when target source/discovery is unchanged. Fresh queries
must reconstruct proof from authorized sources; old admitted artifacts cannot
define the candidate universe.

Persistence: extend SemanticCompilationArtifact, semantic snapshot/durable DTO,
compatibility and existing serialized semantic payload coherently. Restoration,
edit/delta acknowledgement and provide/hydration/query publication all consume
the same typed evidence. Reopen validates source/config/witness/producer versions;
missing/incompatible evidence rederives rather than trusting old public edges.
Do not claim schema compatibility without tests, and do not create a second store.

Final proof captures the generation from which facts were derived; it validates
that generation and the recorded witnesses after all relevant preparation. A
later unrelated publication that cannot change those inputs need not invalidate
everything. A relevant supersession forces recomposition/repreparation through
existing mechanisms. Synchronization completion alone proves nothing. No new
global retry limit, timeout, cap or completeness rule is selected here.

### Narrow identity with existing public queries

The retained Angular fact collection has canonical endpoint indexes, so forward/
reverse/scope/provider/selector joins never use EntityRef equality or shared
name-bucket dedup. Existing WorkspaceIndex may nominate declaring/asserting
owners and hold compatible presentation; it cannot be the only authoritative
store for canonical facts. Angular projection must replace its old authoritative
use of lossy storage, not union the old unsafe positives back into answers.

Current MCP domain/type/name requests retain their documented occurrence-bucket
meaning. They can present a union of already-proved occurrences from multiple
same-name declarations; that union is not a certificate that one declaration
was selected. Scope filtering continues to use asserting provenance, not the
object declaration's file. Never reinterpret withinPath as a callee selector.

Rendering sets endpoint file to its declaration source, keeps asserting owner
separate and deduplicates **canonical facts before rendering**, not by shared
name-only helpers. Where two distinct canonical facts project to indistinguishable
public rows (e.g. same-name declarations in one file), retain them internally;
the existing response is explicitly lossy, not a claim of exact physical counts
or uniquely addressable identity. Do not fabricate names/encode spans in names.
If a consumer needs public canonical selection or physical multiplicity, a
separate reviewed API/schema change is required. No broader identity migration
is necessary for internal Angular correctness, and no new public exactness is
claimed by this freeze.

## 9. Adversarial analysis

| Case/consumer | Required outcome under retained identity |
|---|---|
| Two classes named Leaf in different files | Distinct DeclarationIds; exact import picks the referenced symbol |
| Two same-name declarations in one file/scopes | Distinct span/container; lexical binding selects one; public display cannot become an identity key |
| Two directives with the same selector | Distinct identities; evaluate candidates inside the owner's closed visible scope, not globally |
| Named import alias | Local spelling is a reference/display property; joins use original SymbolId |
| Barrel/re-export alias or diamond | Alias routes retain witnesses; equivalent routes to the same rooted symbol collapse canonically |
| Conflicting stars | Distinct surviving symbols remain ambiguous; explicit export precedence applies; no first match |
| Cyclic re-exports | SCC reaches rooted symbol fixed point or remains unresolved; no cached early-empty authority |
| Local shadowing/type-only import | Bind the actual occurrence/namespace; no value role from a different import |
| Same symbol name from two packages/versions | Package instance/physical source separates them; selected conditions and manifests are witnesses |
| Provider tokens/useExisting | Exact token target and injector scope; alias strategy is not useClass |
| Standalone and NgModule | Shared expression evaluator; eligibility and visibility rules remain Angular-specific |
| Routing targets | Resolve actual component/module/config symbol and role; spelling/lazy path alone insufficient |
| Template selectors/defer | Closed visible declaration IDs plus template owner/control evidence; deferability requires additional eligibility proof |
| Host directives/view-content queries | Bind directive/locator/token in declaring scope; exposure/read options stay specialized facts |
| NgRx | Existing qualified action/member/registration authority consumes shared exact symbols; do not replace #152 with name joins |
| MCP forward/reverse | Render proven occurrence union; internal traversal uses canonical endpoints; no implicit single-declaration selection |

## 10. Performance, implementation slices and approval boundary

Expected new work: project discovery/JSONC inheritance, file probes/package
conditions, source parsing and export binding, closure traversal, canonical-fact
retention and witness checks. These are real costs; no measured claim is made.
Snapshot bytes/hash/AST are reused per compatible source; export summaries per
module snapshot; resolutions per reference namespace/project/options/use mode;
composition per owner/expression and canonical dependency generation. Reuse a
negative result only while its negative/config/project witnesses remain current.

Keep source-local canonical evidence in retained artifacts and generation-scoped
indexes; request work memos remove repetition within completed preparation.
Reuse surviving memoized proof only after validation, never by name/cache age.
Shared closure can traverse required imports/config/package sources without
compiling every TypeScript file. Closed-world questions such as project ownership,
selector scope or reference-use eligibility may require inventory beyond one
positive edge; record that universe work rather than pretending a partial walk
closed it. No unconditional whole-workspace recomputation is the default repair.

Smallest convergent implementation slices:

1. Freeze the exact observed #162 test/registration before production edits.
   Establish one authorized snapshot/project host and typed resolution witnesses.
2. Implement TS lexical/module/export declaration authority and mode conformance,
   retaining neutral evidence through the actual compiler artifact. Include
   export SCC/ambiguity/type-only/shadow/config/package negative tests.
3. Attach canonical Angular API/declaration roles and retained fact identity,
   including library declaration evidence and reviewed legacy fixture enrichment.
   Reject foreign decorators and retain correct endpoint provenance by construction.
4. Feed one recursive composition/visibility engine from both legacy and modern
   owners; propagate all contributors/negative obligations. This converges nested,
   barrel/project/package and standalone examples through shared mechanics.
5. Integrate retained facts, typed witnesses, persistence, retraction and final
   query/batch validation across real consumers. Verify collision preservation
   before lossy public projection, not after it.
6. Continue the entire #162 P0/P1 matrix: provider/root/routing, selector scope,
   host/queries/defer and ecosystem preservation. The initial nine failures are
   acceptance examples, never the issue's completion checklist.

Required approval to freeze this proposal: native no-new-runtime foundation,
producer-owned retained canonical facts/typed witness payload, unchanged global
EntityRef equality, and explicit legacy fixture-environment enrichment with
unchanged assertions. The frozen #162 test stays byte-identical.

Not approved/introduced by this design: Node/compiler runtime, resolver crate or
MSRV increase; global shared identity migration; new public canonical-ID selector,
schema/relation vocabulary or completeness promotion. Bring a concrete mapping
for any later feature requiring those changes. No new runtime run is requested
for this design phase. Full native conformance and lifecycle integration remain
implementation/verification obligations, not assumptions.

FOUNDATION DESIGN: READY TO FREEZE
