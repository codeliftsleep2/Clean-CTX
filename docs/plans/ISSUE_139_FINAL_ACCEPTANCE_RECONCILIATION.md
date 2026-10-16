# Issue #139 final acceptance reconciliation

**Investigator:** Agent MaxHeadRoom  
**Date:** 2026-10-15  
**Issue:** #139 — Angular/TypeScript modules, routes, class-name parsing, generics, entity typing

## 1. Executive verdict

**#139 VERIFIED — READY FOR CLOSURE**

The approved architecture gate has been implemented in production code with tracked regressions. NgModule metadata now uses bounded value composition and declaration-proven kinds; routes use structured node ownership and a dedicated lazy-children relation; TypeScript heritage excludes generic constraints by syntax role; and find_entities has an explicit declaration-only mode while preserving its occurrence-oriented default.

The user reports the focused #139 regressions, repaired semantic NgModule suite, debug build, and consolidated Issue #139 acceptance harness green. No build, check, Clippy, MCP/live script, benchmark, CBM/provider run, or authoritative final gate was run by Agent MaxHeadRoom. The user subsequently reports the authoritative final gate green, including zero-warning all-target/all-feature Clippy; #139 is verified and ready for closure. No GitHub state, commit, or push changed.

## 2. Approved architecture applied

The approval authorized bounded direct/same-file/one-hop imported NgModule values, independent declaration-kind authority, structured route ownership, Component-only RouteMapsTo, a distinct lazy-children relation, declaration-only entity discovery, generic LGR-003 repair, syntax-role-aware heritage, and a narrow unresolved-spread carrier. The implementation stays within those boundaries.

## 3. Acceptance matrix

| Criterion | Status | Implementation evidence |
|---|---|---|
| Decorated owner identity and qualified edit | SATISFIED — FROZEN | Commit 5ba64a2e; unchanged |
| NgModule plain list | VERIFIED | Bounded metadata projector |
| Same-file/imported spread | VERIFIED | Exact const arrays; one direct-relative hop |
| Unresolved spread/no fake entity | VERIFIED | Rendered non-entity evidence; no semantic endpoint |
| Module relations/RouterModule call | VERIFIED | Slot selects relation; declaration/import proves kind |
| Component/Directive/Pipe/Module/standalone kinds | VERIFIED | Syntax-owned proof; ambiguity fails closed |
| Eager route behavior | SATISFIED — PRESERVED | RouteMapsTo remains Component-only |
| Nested/full-path/exact child ownership | VERIFIED | Recursive route node walk |
| Lazy component/module/route-config | VERIFIED | RouteMapsTo plus LoadsChildren |
| Generic constraints versus heritage | VERIFIED | Parser-owned heritage clauses |
| 3000+ line observation | OWNED ELSEWHERE | #142 |
| Short-file passthrough | SATISFIED — PRESERVED | No policy change |
| Declaration-only entity lookup | VERIFIED | declarations_only query mode |
| LGR-003 Class boundary | VERIFIED | Generic role authority; no blacklist |
| Same-name declaration provenance | SATISFIED — PRESERVED | File-provenanced occurrences |

## 4. NgModule metadata and kinds

src/angular_meta/module_metadata.rs accepts direct slot leaves, exact same-file constant arrays, and exact one-hop named-import constant arrays. Nested imported spreads are not traversed and become unresolved evidence. Only direct-relative files are read.

Slots select DeclaresInModule, ImportsModule, or ExportsFromModule; they never select endpoint kinds. An exact named class declaration with exactly one supported Angular decorator proves the endpoint kind. Missing or ambiguous proof emits no edge. RouterModule calls reduce to the base only when an exact @angular/router import proves RouterModule. The former slot-default projection was removed.

## 5. Routes and compatibility

src/angular_meta/routing/structured.rs replaces the flat path scan with route-array/object ownership. It recursively composes parent and child paths and reads targets, guards, resolvers, and lazy expressions from the exact route object.

loadComponent emits RouteMapsTo only after exact Component proof. loadChildren emits SemanticRelation::LoadsChildren only after exact Module or exported Routes-array proof. Dynamic, external, missing, and ambiguous targets fail closed. The production path now supplies the declaring file. Angular semantic generation advanced 23 to 24. The unresolved-spread evidence also survives the rendered Angular context boundary, so Angular marker generation advanced 5 to 6.

## 6. Generic constraints

The TypeScript layer reads only parser-owned class_heritage clauses. A type-parameter constraint cannot become heritage or corrupt body boundaries. Ordinary class and interface heritage remains represented. TypeScript canonical and semantic-input generations advanced 1 to 2.

## 7. Declaration discovery and LGR-003

find_entities remains occurrence-oriented by default. declarations_only selects WorkspaceIndex declaration authority and permits exact domain/entity_type filtering. Those filters do not alter default occurrence queries. The implementation does not infer from endpoints, filenames, spelling, or a Class blacklist. Genuine declarations named Class remain eligible.

LGR-003 is **verified repaired**. The user reports the declaration-query regressions, consolidated stdio harness, and authoritative final gate green.

## 8. Tracked regressions

Added and registered:

- src/tests/angular_meta/module_metadata_issue_139.rs
- src/tests/angular_meta/routing_issue_139.rs
- src/tests/ir/layers/typescript_issue_139.rs
- src/tests/mcp/workspace_query_issue_139.rs

Coverage includes bounded composition, unresolved spreads, independent kinds, RouterModule calls, nested/full-path routes, lazy targets, dynamic failure, generic constraints, ordinary heritage, declaration-only LGR-003 behavior, genuine Class declarations, provenance, and unchanged default occurrences.

## 9. Verification evidence

Agent MaxHeadRoom ran only fast non-compiling inspection: targeted rg/Get-Content, PowerShell parser validation, git status/diff, git diff --check, encoding checks, and line counts. git diff --check reported no whitespace errors; modified code/test files are below 615 lines by static count. The user reports module_metadata_issue_139, routing_issue_139, typescript_issue_139, workspace_query_issue_139, semantic_dependency_lifecycle_issue_139, the repaired semantic_ngrx suite, the debug build, and consolidated live acceptance green. No build, check, Clippy, binary/live MCP, benchmark, provider, or final-gate command was run by the agent.

Run:

    cargo test --all-features module_metadata_issue_139
    cargo test --all-features routing_issue_139
    cargo test --all-features typescript_issue_139
    cargo test --all-features workspace_query_issue_139
    cargo test --all-features decorated_owner_identity_
    cargo test --all-features framework_declaration_publication_

The focused commands and the debug `verification/issue-reconciliation/Invoke-Issue139Acceptance.ps1` run are user-reported green. Run the authoritative Final Verification Gate from docs/agent/verification.md; its command list is not duplicated here.

## 10. Boundaries and remaining work

#133–#138 and #140 were not re-scoped. #141/LGR-002 was untouched. #142 retains completeness, admission, refresh, history, truncation, and global diagnostics. The only diagnostic-like addition is the authorized narrow unresolved-spread carrier.

Implementation, focused regression verification, debug production-boundary acceptance, LGR-003 reconciliation, zero-warning Clippy, and the authoritative final gate are user-reported green. No technical acceptance residual remains in #139. GitHub closure, commit, and push remain separate maintainer actions and were not performed.

**Final verdict: #139 VERIFIED — READY FOR CLOSURE.**
## 11. Approved lifecycle repair addendum

The architecture gate was approved on 2026-10-15 and implementation resumed. The structured-route ownership warning was traced to the operator harness, not production: decoded MCP text contains logical LF line endings, while the PowerShell assertion split only on Windows `[Environment]::NewLine`. `Invoke-Issue139Acceptance.ps1` now splits on `\r?\n`. No production route-tree ownership code was changed in response to that false negative.

The lazy-target refresh warning exposed a separate production lifecycle defect. `refresh_workspace` correctly retracted WorkspaceIndex owners, but `provide_code_context` could reuse unchanged canonical IR and an in-memory semantic-edge snapshot keyed only by the unchanged asserting source. Because the snapshot did not retain the external lazy-target source consulted during derivation, republishing unchanged `app.routes.ts` resurrected `RouteMapsTo -> LazyComponent` after `lazy.component.ts` was deleted.

The bounded repair separates canonical IR validity from derived-semantic validity. Angular lazy-route resolution records a source identity/hash fingerprint for each direct-relative target it actually consults, including an explicit missing state. That provenance travels through the meta-layer evaluation, compiler side output, in-memory semantic snapshot, durable semantic snapshot, compatibility validation, explicit saves, delta acknowledgement, and crash-recoverable edit intent. Cached semantic reuse now requires every recorded dependency to match current source state. Missing or changed dependencies invalidate only the derived semantic snapshot and force semantic re-derivation; the edge remains owned by `app.routes.ts`, and failed target resolution remains closed.

Tracked coverage in `src/tests/mcp/semantic_dependency_lifecycle_issue_139.rs` covers unchanged, changed, deleted, restored, purely local, and refresh-retract-republish behavior. The reusable live harness remains separate operator evidence. General auditing of dependency-capture completeness across other cross-file producers remains #142; this repair adds no second dependency graph, semantic index, watcher, or global completeness policy.

Agent MaxHeadRoom did not run Cargo, binaries, MCP/live harnesses, benchmarks, providers, or the final gate. The user reports `cargo test --all-features semantic_dependency_lifecycle_issue_139`, `cargo test --all-features issue_139`, the debug build, and the corrected Issue #139 acceptance harness green. This establishes tracked focused regression and operator/live production-boundary evidence; the authoritative final gate is also user-reported green.