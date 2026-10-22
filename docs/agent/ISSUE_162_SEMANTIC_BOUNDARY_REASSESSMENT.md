# Issue #162 — TypeScript Semantic Boundary and Certification Alignment

Agent: Agent MaxHeadRoom  
Date: 2026-10-22  
Status: investigation and recommended scope; no certification or implementation approval implied.

### Owner decision — 2026-10-22

**[RE] The owner accepted Verdict B and the question-driven scope interpretation.**
The governing rule is now:

> No new TypeScript semantic frontier enters implementation merely because it differs from `tsc`. It must have a demonstrated canonical-question → required-fact → authority/dependency → consumer chain. Existing established requirements remain frozen, and publication, persistence, consumer migration, production queryability, completeness, and live verification remain mandatory.

**[ND] Publication and consumption now receive increasing execution priority.**
Correct internal facts alone do not satisfy #162. This acceptance changes the
scheduling rule; it does not certify implementation, remove requirements or
approve a separate global identity/public-contract redesign. Historical pending
decision language below describes the investigation before this acceptance.

Evidence labels used throughout:

- **[RE] PROVEN BY REPOSITORY EVIDENCE** — a located document, source location or recorded receipt establishes the stated observation.
- **[CI] PROVEN BY CURRENT IMPLEMENTATION** — static source inspection establishes a code path; this does not establish runtime correctness.
- **[ND] REQUIRED BY NORMATIVE DOCUMENTATION** — an applicable requirement, not proof it is implemented.
- **[I] INFERENCE** — a reasoned recommendation or conditional dependency; not an established consumer integration.
- **[U] UNRESOLVED** — insufficient requirement, scope, consumer or execution evidence.

## 1. Executive Verdict

**[I] Verdict B — MOSTLY CORRECT, SOME PARITY DRIFT.** The native declaration/reference/export foundation is justified by the accepted Angular question envelope. The execution process nevertheless drifted toward implementing each newly discovered compiler behavior without first proving its particular question/fact/consumer dependency. Repeated generic AQ01/AQ25 linkage did not discharge that obligation. This finding concerns how scope was admitted; it does not prove every resulting implementation unnecessary or incorrect.

**[CI] A second, independent finding is missing required integration.** Production compilation creates `TypeScriptAuthority`, but `SemanticCompilationArtifact::into_parts` explicitly drops it. Angular member/provider consumers still use bounded import binders and written module/export checks. The new neutral authority is not yet their persistent, query-reachable authority. This is the missing-foundation aspect of classification C, not a replacement for the primary B verdict.

**[ND] The stopping target is the approved canonical-question envelope, not complete TypeScript semantics.** Retain necessary identity, required resolution context, witnesses and ambiguity handling. Do not turn full type checking, compiler diagnostic parity or every compiler option into a Phase 2 prerequisite without a required-fact chain. Do not narrow any established Angular P0/P1 question or silently drop its necessary semantics.

**[RE] Document synchronization matters.** `docs/LANGUAGE_META_LAYER_CERTIFICATION.md` is absent from this working tree. It exists in the recorded upstream revision `dbb2df8e96a3f4ce0d1fc2e229d31b01f6d62d54`, which was read directly using Git. The local branch reports two commits behind upstream. No fetch, pull, merge or checkout occurred. This report applies that exact normative standard as requested, while identifying its upstream-only location rather than claiming it is present locally.

**[U] No certification or defensible completion percentage follows.** Earlier conversational 75–85% estimates lacked a closed, weighted requirement inventory and should not guide the remaining implementation budget. Numerous focused operator GREEN reports are implementation evidence, not a certification denominator.

### Governing evidence

| Source | Authority and finding |
|---|---|
| [Language/Meta-Layer Certification Standard at the inspected revision](https://github.com/codeliftsleep2/Clean-CTX/blob/dbb2df8e96a3f4ce0d1fc2e229d31b01f6d62d54/docs/LANGUAGE_META_LAYER_CERTIFICATION.md) | [ND] Gates 1–2 declare envelope/questions; Gate 6 limits semantics to those questions; Gates 9–11 require publication, reachability and completeness; Gate 13 makes official implementations development-only oracles; Gate 16 requires dual sign-off. |
| [Canonical Question Matrix Standard](../Clean-CTX_Canonical_Question_Matrix_Standard.md) §§1–5, 6, 11 | [ND] Questions precede facts/implementation; every material capability needs authority, closure, public path and an answer contract. |
| [Accepted foundation](ISSUE_162_FOUNDATION_DESIGN_FREEZE.md) §§4–8 | [RE/ND] Native Rust, canonical endpoints before lossy projection, required resolver forms, retained proof and shared Angular composition are approved. No external compiler runtime or global `EntityRef` change is approved. |
| [Working plan](ISSUE_162_IMPLEMENTATION_PLAN.md) §§1, 6, 6A | [RE/ND] Angular-wide P0/P1 migration and AQ01–AQ26 remain required; no family has been excused here. |
| [Authority scope](ISSUE_162_ANGULAR_AUTHORITY_SCOPE.md), Authority and baseline / family inventory | [ND] Syntax is not framework authority; module paths are not export identity; lower-bound results cannot close required capability gaps. |
| [Architectural invariants](../ARCHITECTURAL_INVARIANTS.md), ARCH-002/003/004, C-22, PERSIST-002, META-LAYER-COMPLETENESS-001 | [ND] Reuse common mechanics, separate canonical facts from presentation, prove production lifecycle, and do not disguise in-envelope omissions as degradation. |
| AGENTS.md and `.clinerules/engineering.md` | [ND] No agent-run runtime gates; tracked regressions, architectural approval and production integration rules apply. |

## 2. Current TypeScript Authority Inventory

**[CI] These are actual implementation responsibilities, not claims of complete conformance.** Paths below are under `src/ir/typescript_resolution/` unless otherwise stated.

| Area | Existing implementation / retained evidence |
|---|---|
| Snapshot and authorization host | `host.rs`, `host_windows.rs`: supplied-source seeding, physical routes, scoped reads, content hashes, negative/error/kind/directory/physical observations. Supplied target hash remains distinct from disk dependency witnesses. |
| Declaration and binding identity | `syntax.rs`, `syntax_imports.rs`: exact physical source/hash/span/kind; locals/exports, type-only bindings and retained source tree. `DeclarationId` includes a presentation name; it is not a package/API-role certificate. |
| Lexical occurrence binding | `references.rs`, `type_references.rs`; shared `typescript_identifier_binding.rs` and `typescript_binding_pattern.rs`: parameters, variable/pattern/loop/catch/function scopes and exact occurrences. Generic/mapped/infer type sites are also captured. |
| Export and alias graph | `resolver.rs`, `namespace.rs`, `declared_scopes.rs`: direct/default/named/star exports, aliases, rooted cycle traversal, scoped namespace members and contender sets. |
| Symbol groups | `declaration_groups.rs`, `global_bindings.rs`, `ambient_groups.rs`: selected overload/interface/class/enum/namespace contributor groups. `Resolution::Merged` retains declaration IDs; it is not a full compatibility checker. |
| Declared/ambient scopes | `syntax_ambient.rs`, `ambient_modules.rs`, `declared_scopes.rs`: declared/dotted namespaces, private/exported body scopes, script ambient modules and patterns, external augmentations. Exact/pattern/file precedence and scoped cycle guards are implemented. |
| Project/configuration | `project.rs`, `project_config.rs`, `project_membership.rs`, `project_options.rs`: JSONC/BOM, inheritance/origins/resets, files/include/exclude/references, explicit ownership and selected option validation. Retained unknown options are not implemented semantics. |
| Module and package resolution | `module_resolution.rs`, `module_references.rs`, `probe.rs`, `relative.rs`, `package*.rs`, `classic.rs`: request kinds, file substitutions, modes/conditions, paths/baseUrl, rootDirs, package/self/private imports and map barriers. |
| Version routing | `project_version.rs`, `version_value.rs`, `version_ranges.rs`, `type_versions.rs`: consumed compiler-manifest evidence, reviewed defaults, semver and versioned package routing. No blanket release-family conformance is established. |
| Program membership | `program.rs`, `program_directives.rs`, `automatic_types.rs`, `compiler_libraries.rs`, `type_packages.rs`: admitted global/program inputs, directives, automatic types, actual compiler declaration libraries/overrides, selected augmentation proof and deferred import acquisition. |
| Logical routes/platform | Host/resolver/probe paths retain logical lookup where applicable while declaration identity remains physical. Windows handle checks and platform-specific authored controls exist. |
| TSX/JSX source parsing | `syntax.rs` selects TSX grammar for `.tsx`/`.jsx`; the capture can follow declarations in these files. This is not a React/JSX framework meta-layer. |
| Import types and queries | `import_attributes.rs`, `import_types.rs`, `type_queries.rs`: statement/expression resolution modes, exact import-type and `typeof` occurrences, erased value-query roles. Qualified erased named aliases remain an explicit unresolved branch. |
| Output/input mappings | `reference_outputs.rs`: explicit referenced declaration-output policy. `local_package_inputs.rs`: local package output-to-input lookup, JS input and shared suffix probing. These are different identity directions, not interchangeable redirection policies. |
| Artifact capture | `capture.rs`: declarations, references, module requests, project/options, declared scopes, program evidence, witnesses and capture disposition/currentness. `SemanticCompilationArtifact::from_compiler` invokes this for TS-family files. |

**[CI] Representation limits:** lexical containers are represented through source spans and separate scoped evidence, not a universal `SymbolId` object. Some occurrence records are tuples; module-file failure plus `ambient_declarations` can represent successful ambient scope resolution. These facts require careful interpretation; they are not a finished meta-layer-facing contract.

**[RE] Evidence limit:** the progress ledger contains preserved behavioral REDs, static checks and operator-reported focused GREEN checkpoints. Many recent GREEN reports have no retained count/log. The newest erased-namespace query batch is authored and UNRUN; this investigation changes none of its assertions or implementation.

## 3. Canonical Question → Semantic Requirement Matrix

**[RE] Actual question IDs come from plan §6A.** No separate repository-wide TypeScript question catalog was located. AQ25 is an Angular canonical-join/completeness question; it must not be stretched into an unlimited standalone TypeScript language-service requirement. “Consumer” below names the intended required owner; existing migration is not implied. Completeness is limited to the stated answer, never all TypeScript programs.

| Semantic responsibility | Question → required fact / authority | Consumer and completeness dependency | Classification; required now? / evidence |
|---|---|---|---|
| Exact declaration/reference identity | AQ01, AQ25 → distinguish actual declaration, owner and reference endpoint from namesakes | TS identity → Angular API/declaration authority; exact endpoint requires same-snapshot binding | REQUIRED-AUTHORITY; YES mechanism [ND]. Integration incomplete [CI]. |
| Lexical shadowing and alias binding | AQ01, AQ08–AQ11, AQ19–AQ23 → written API/token use denotes this declaration, not a local namesake | Angular APIs, providers, members and ecosystem producers; false positives must retract | REQUIRED-AUTHORITY; YES for claimed use sites [ND]. Every possible binding syntax is conditional on the declared forms [I]. |
| Named/default/star exports and barrels | AQ01–AQ03, AQ08, AQ12–AQ14 → exported API/declaration/composition target | API, composition, lazy routing; complete requested export route and contender set | REQUIRED-AUTHORITY; YES [ND, foundation §5]. |
| Namespace qualification | AQ01, AQ08, AQ12, AQ19, AQ25 → every qualifier reaches the canonical target | API/token/route/member consumers; no terminal-name rescue | REQUIRED-AUTHORITY; YES for admitted namespace references [ND]. |
| Group identity and merge ownership | AQ01, AQ10, AQ25 → one legal declaration family versus competing owners | API/declaration/member identity; all relevant contributors retained | REQUIRED-AUTHORITY; YES identity-affecting legality [ND]. Full compiler type compatibility is not thereby required [I]. |
| Runtime/type/type-only eligibility | AQ01, AQ08–AQ11, AQ25 → type evidence cannot authorize runtime API use | API/token/member producers; query-role exceptions must remain non-emitting | REQUIRED-AUTHORITY; YES eligibility [ND]. |
| Qualified erased `typeof` aliases | No specific consuming AQ01/AQ08/AQ10 fact/site has yet been demonstrated | Possible declaration-metadata/type-reference consumer, but no migrated call path | UNRESOLVED; no automatic new repair [U]. AQ25 alone is insufficient. |
| Generic, mapped and conditional-infer identities | Potential AQ10/AQ25 dependency only where a claimed member/declaration relation needs the reference | No independent type-evaluation question in §6A | UNRESOLVED as broad production inventory [U]; CONFORMANCE-ONLY for a required shadow/refusal control [I]. |
| Project ownership / explicit application context | AQ03, AQ05, AQ08, AQ12–AQ14, AQ25 → correct app/project interpretation | Angular producer context → composition/scope; exact answer needs correct context | REQUIRED-DEPENDENCY; YES [ND]. Ordinary callers still lack producer-proved explicit app context [RE]. |
| Config inheritance / origins / JSONC | Same questions → effective lookup policy and its declaring origin | Shared resolver; context changes invalidate dependent facts | REQUIRED-DEPENDENCY; YES for admitted config forms [ND]. Every compiler option is not a requirement [I]. |
| File/mode/import-require selection | AQ01, AQ12, AQ25 → module/export chosen under the actual request conditions | API and lazy target resolver; exactness depends on selecting the right endpoint | REQUIRED-DEPENDENCY; YES for supported project/request forms [ND]. |
| Package maps, self/private imports, path mappings | AQ01–AQ03, AQ08, AQ12, AQ25 → authoritative package/module export route | Shared resolver feeding specialized consumers; barriers and competing routes matter | REQUIRED-DEPENDENCY; YES mechanisms named in foundation §5 [ND]. |
| Version-sensitive routing | AQ01/AQ12/AQ25 → version-qualified package target; separately AQ03/AQ11 framework-version defaults | Language routing versus Angular version interpretation | REQUIRED-DEPENDENCY for consumed routing versions [ND]; testing every semver spelling is not an independent feature [I]. |
| rootDirs, suffixes, preserveSymlinks | AQ01/AQ12/AQ25 → correct endpoint in a project using those policies | Shared resolver; exactness conditional on declared project envelope | REQUIRED-DEPENDENCY when configured in-envelope [ND/I]. Exhaustive unclaimed platform/option combinations remain [U]. |
| Referenced outputs / local input mapping | AQ01/AQ12/AQ25 → source or declaration owner selected by actual build/package policy | API and route identities; cannot silently substitute implementation for declarations or vice versa | REQUIRED-DEPENDENCY for admitted build forms [ND/I]. Default editor policy and every emit mode are not justified by existing tests alone [U]. |
| Ambient modules / script globals / augmentation | AQ01/AQ08/AQ25 where API/token declarations actually come from admitted declarations | Language binding → API/token identity; scope and all required contributors matter | REQUIRED-DEPENDENCY as needed for those declarations [ND/I]; a complete global type universe for every capture is not yet justified [U]. |
| Automatic @types / compiler libraries / overrides | Potential dependency of the previous row, not a queried architectural fact by itself | Required declaration acquisition, if a requested fact depends on those symbols | REQUIRED-DEPENDENCY conditionally [I]; unconditional all-program acquisition is [U], not a universal phase exit requirement. |
| JSX/TSX parsing | AQ01/AQ25 if an admitted API/declaration route traverses such a source | Language declaration identity, not JSX rendering/framework interpretation | REQUIRED-DEPENDENCY for traversed files [I]; JSX framework semantics are OUT-OF-ENVELOPE for this Angular campaign absent a new question [U/RE]. |
| Angular API/package roles and declaration metadata | AQ01, AQ03, AQ10–AQ17, AQ20–AQ23 → this symbol means Angular/router/forms/etc API | Angular and specialized meta-layers; TS endpoint alone is insufficient | META-LAYER-SPECIFIC; YES required later #162 work [ND]. |
| Arrays/spreads/configuration composition and visibility | AQ02–AQ09, AQ13–AQ17, AQ23 → actual membership/provider/visible universe | Shared Angular composition and specialized scopes | META-LAYER-SPECIFIC; YES [ND]. Not TS arbitrary-expression evaluation. |
| NgRx/RxJS/forms meaning | AQ19–AQ22 → actions/registration/stream/control relationships | Specialized meta-layers consuming TS + Angular authority | META-LAYER-SPECIFIC; YES existing envelope [ND]. Equal literals or names cannot establish relationships. |
| Snapshots, physical identity and consumed witnesses | AQ24–AQ26 plus every derived answer → reason path and current dependency inputs | Shared artifact/publication/evaluators; exact/current requires all relevant witnesses | REQUIRED-DEPENDENCY; YES [ND]. Windows/Unix cases support that mechanism, not separate query features [I]. |
| Retention, generation, reverse dependency closure | AQ09, AQ24–AQ26 → surviving current facts and affected consumers | Existing semantic owner/index/durable/query machinery | REQUIRED-DEPENDENCY; YES [ND]. Phase 3/later integration is missing [CI]. |
| Full assignability, inference, overload selection, diagnostics, emit | No corresponding current §6A architectural fact requiring compiler-complete results | No demonstrated consumer or standalone TS question | OUT-OF-ENVELOPE as blanket capabilities [I]. Narrow checks that affect a required identity must be separately justified, not removed wholesale. |
| Differential fixtures | All rows whose required identity semantics overlap TypeScript | Development verification only; agreement supports only equivalent claims | CONFORMANCE-ONLY [ND, certification Gate 13]. No production authority. |

**[ND] All AQ01–AQ26 remain in scope.** AQ04–AQ07 template ownership/visibility/selector/pipe answers, AQ15–AQ18 host/query/defer/control-flow answers and AQ26 presentation are not new TS semantic engines: they consume the particular identities and closure their producers require. Their full question contracts remain in plan §6A; no replacement catalog is invented here.

## 4. Required Authority vs Compiler-Parity Analysis

**[RE] Concrete drift indicators:** recent ledger batches were often introduced by “the next compiler behavior” and closed with “AQ01/AQ25 prerequisites advance,” without a concrete framework assertion, admitted idiom, consumer call site or exactness obligation. The newest erased namespace batch is such an unproven expansion, not an observed framework defect. Its tests alone cannot authorize production semantics.

**[CI/I] A second indicator is acquisition pressure.** Ambient precedence now invokes `program_units` from module-scope lookup; program closure can bring automatic types and installed compiler libraries into an otherwise narrow import lookup. The version-routing fixture subsequently needed explicit library/global isolation. The fixture correction does not prove that unconditional acquisition is the minimum necessary boundary. Reassess the evidence frontier per question; do not bypass inputs that could actually change its answer.

**[ND] Counterexample to indiscriminate reduction:** aliases, private/exported namespace ownership, package conditions, negative probes and physical identity can change the API/declaration a real Angular use denotes. Those are required when the admitted reference uses them, even if expensive. Refusal protects truth during development; it cannot permanently close an in-envelope capability.

**[I] Stopping criterion:** a language behavior is complete for #162 when its declared input forms establish the necessary canonical fact, carry the required proof, survive relevant adversarial/lifecycle controls and feed the designated consumer. Stop there. A compiler feature with no such chain is classified, not automatically scheduled.

## 5. Current Namespace/Merging Frontier Assessment

| Behavior | Assessment and next disposition |
|---|---|
| Namespace import of actual Angular APIs, aliases/barrels, qualified tokens | [ND] REQUIRED-AUTHORITY where the admitted call/token uses that route. Keep required identity and shadow tests; connect to the API/token consumer. |
| Private versus exported namespace members, distinct nested owners | [ND/I] Required to avoid false canonical targets when such declarations occur in consumed source/declaration metadata. Existing exact scope evidence is appropriate; do not flatten names. |
| Cyclic aliases and duplicate declarations | [ND] Required refusal/ambiguity preservation for queried identities. Rooted proofs must survive ordering; no arbitrary cap or terminal-name fallback. |
| Legal namespace/class/function/interface group ownership | [ND] Accepted foundation requires legal declaration families and relevant compatibility. Keep all contributors. [U] The precise necessary compatibility predicates must follow the claimed fact, not automatically all compiler diagnostics. |
| Full enum initializer evaluation / interface assignability / overload applicability | [I] Not required merely to retain declaration families. Separate identity-invalidating collisions from type-checking validity; blanket compiler-complete validation is OUT-OF-ENVELOPE without a new required-fact chain. |
| Ambient module/global merges | [I] Required when admitted declaration packages establish the requested API/token this way. [U] No evidence justifies treating every ambient/library interaction as an independent required question. |
| Pending `typescript_erased_namespace_queries_issue_162.rs` | [CI] Three authored tests; explicit unresolved named-alias branch remains in `type_queries.rs`. [U] No specific Angular fact/consumer that requires these qualified `typeof` forms has been demonstrated. Preserve tests; do not implement merely because TypeScript permits them. Identify a declaration-metadata or real language-question consumer first. |

**[I] Recommendation:** continue namespace work that proves an accepted reference/declaration route, but halt automatic expansion of the latest erased-query frontier until its consumer dependency is established. This is a proposed scheduling boundary, not deletion of existing semantics or an approved reduction of the Angular envelope.

## 6. TypeScript Language → Meta-Layer Contract

**[ND] Minimum reusable contract:** given authorized immutable source, a precisely owned reference span, reference meaning and applicable project context, return the canonical declaration family or a truthful non-success disposition, with the evidence needed to retain and revalidate that result. Angular interprets framework meaning above this boundary.

| Language-owned result | Meta-layer use / necessary limit |
|---|---|
| Physical declaration owner + consumed hash + exact span/kind + relevant scope/family identity | Bind the same semantic declaration across aliases. Identity is snapshot-specific; stable lineage across edits is not implied. |
| Exact reference span and meaning: runtime value, type, or non-emitting query of a value | Prevent erased imports from gaining runtime authority. A type-occurrence container alone must not imply the endpoint is a type declaration. |
| Resolved file-module or declared-module scope, package instance/selected export route | Establish which library declaration is being interpreted. A package-looking name is not Angular API authority. |
| Effective project/configuration/request context and consumed origins | Make producer-selected application/build context explicit; a caller's `None` is not a proved app boundary. |
| Resolved family, ambiguous contenders, unresolved/unavailable/unsupported input obligations | Preserve uncertainty without selecting first matches or erasing independently proved facts. Exact status tags must follow the approved internal/public boundaries. |
| Resolution/provenance dependencies, including necessary negative and physical observations | Reuse and retract based on semantic inputs, not solely target hash or cache presence. |
| Coverage statement for the particular reference/declaration inventory | A successful point lookup is not a complete file/project reference census or reverse-neighborhood proof. |

**[CI] Existing capture supplies pieces, not this complete contract.** `DeclarationId`, `Resolution`, `Witness`, scope and program records are useful. Missing explicit route/meaning/coverage and publication binding must not be guessed from tuples, strings or `capture_current`.

**[ND] Angular ownership:** framework API/decorator roles, compiled Angular declaration metadata, standalone version defaults, NgModule/standalone visibility, DI strategy/injector precedence, template selectors, routing, signal/query/host meanings and ecosystem registrations. TS must not encode these roles or evaluate arbitrary Angular expressions on Angular's behalf.

**[CI] Current integration trace:** authorized compilation request → `compile_source_artifact` → `SemanticCompilationArtifact::from_compiler` → `capture`/`capture_in_project` → artifact field → **`into_parts` drops capture**. No TS canonical payload survives that return through this path. `member_semantics.rs` and `provider_semantics.rs` still use bounded source import evidence and written `@angular/core`/export checks. These are existing preservation paths, not a completed migration to the new resolver.

## 7. Reusable Language-Layer Infrastructure Findings

| Mechanism | Finding / recommendation |
|---|---|
| Canonical payload transport and semantic owner | [ND/CI] Required and missing for TS. Extend the existing artifact/snapshot owner under Phase 3; do not build a parallel TS publication registry. |
| Dependency currentness and publication generations | [CI] Existing C# semantic compilation/dependency/publication machinery is a concrete comparison point. [I] Share ownership/validation mechanics where semantically equivalent, with TS-specific configuration/module witnesses retained. Do not copy C# partition assumptions blindly. |
| Ambiguity and completeness | [ND] Common query meanings should be consistent across layers. [I] Keep language-specific resolution reasons and scope proof; do not generalize all languages into one symbol algorithm or introduce a new global status API in this investigation. |
| Canonical fact keys before projection | [ND] Shared need for asserting owner/site + relation + canonical endpoints. Existing `EntityRef` equality stays unchanged; no global identity redesign is needed to justify retained typed TS evidence. |
| Certification records and oracle runners | [I] Reuse fixture manifests, version receipts, normalization and comparison mechanics across languages when a second real consumer warrants extraction. No generic certification engine is needed now. |
| Parser/scopes/module policy | [ND/I] Remain TypeScript-owned. Windows physical IO mechanics may be reusable after equivalent semantics are demonstrated; project inheritance, namespaces and export precedence are not generic abstractions. |

**[I] Priority:** transport and consume already-produced authority before building another reusable framework. “Share mechanics, specialize semantics” is not approval for a repository-wide refactor.

## 8. Development-Time TypeScript Oracle Strategy

**[ND] Oracle use is development-only and operator-run.** No TypeScript/Angular compiler, Node, ts-morph or language server becomes production authority. This investigation installed or executed none.

**[I] Proposed reusable assets:** a tracked `verification/` fixture manifest associates each case with an existing AQ ID, required fact, admitted compiler/config form and expected normalized result. A development adapter runs the pinned official TypeScript API and a Clean-CTX adapter on equivalent fixture bytes/config/manifests. Generated outputs belong under `target/`. Required regressions remain tracked Rust tests under `src/tests/`.

**[I] Compare equivalent semantics:** normalize physical declaration file, exact original span, namespace/use eligibility, complete contributor set, selected route and resolution disposition. Do not compare raw symbol IDs, AST node-kind spelling, diagnostic strings, type pretty-printing, emitter text or TypeScript's path formatting with Clean-CTX identities. Expose symlink policy and synthetic supplied-source bytes consistently. Merged symbols require all applicable declarations, not a representative chosen by either tool.

| Priority fixture | Required Clean-CTX claim / question |
|---|---|
| Angular decorator/API alias versus local shadow/foreign export | AQ01/AQ25: exact framework endpoint without lookalike admission. |
| NgModule/standalone declaration through a diamond or cyclic barrel | AQ02/AQ03/AQ05: same canonical endpoint, retained contenders and mutation retraction. |
| Lazy route target under import/require/package condition changes | AQ12: selected module/export/declaration under the actual request. |
| Two applications with different path mappings or package instances | AQ14/AQ25: producer-proved project identity, not proximity. |
| Declaration package namespace/ambient merge used by a framework API | AQ01/AQ10/AQ25: all necessary owner/contributor IDs and exported/private separation. |
| DI token alias and type-only/runtime misuse | AQ08/AQ09: token endpoint/use eligibility; Angular strategy remains a separate oracle question. |
| Selected negative candidate appears or dependency/export kind changes | AQ24/AQ25: prior proof invalidates and the selected identity/status changes. |

**[ND] Mismatch procedure:** verify the claim is in-envelope, compiler version and host policy agree, both systems consumed equivalent inputs and the compared facts mean the same thing. Then classify fixture mismatch, Clean-CTX defect, version boundary or missing requirement. Preserve a real in-envelope RED before repair. Agreement is supporting evidence only; it does not prove publication/currentness/public-query behavior.

## 9. Potential Scope Reductions

**[I] Proposed, not enacted:**

1. Replace “full native TypeScript conformance” as an unbounded work generator with the accepted questions' explicit language-input forms and required authority dependencies. Retain every existing P0/P1 Angular question.
2. Stop treating full type inference/assignability, overload applicability, compiler diagnostics and emit behavior as implicit merge-identity prerequisites. Identify the narrower legality conditions actually needed to avoid false identity.
3. Keep extra semver spellings, generic/mapped/infer cases, exotic declaration combinations and platform variations as useful conformance tests where they falsify a required claim; do not make each an independent production capability.
4. Do not grow the newest qualified erased-query work without a concrete consuming fact/site. Classify it UNRESOLVED, not automatically required and not automatically unnecessary.
5. Reassess unconditional program/library expansion against question-specific dependency needs. Preserve all inputs that can change the required answer; no acquisition cap, shortcut or unproved absence is proposed.

**[ND] These are not approved envelope reductions.** The accepted plan prohibits silently excluding required Angular forms. Any actual narrowing needs an explicit question/envelope diff, affected consumers and degradation contract, owner decision and updated certification evidence. This report removes no code or test and rewrites no approved foundation decision.

## 10. Potential Missing Required Semantics

- **[CI/ND] Retained consumption:** neutral TS evidence must survive returned parts and be owned, invalidated, published and consumed through existing production lifecycle boundaries. This is already assigned to Phase 3; its absence is not evidence that Phase 2 must grow compiler semantics indefinitely.
- **[CI/ND] Framework role authority:** exact resolved language identity must reach Angular API/declaration-role evaluators and replace competing admission paths. Written `@angular/core` imports are not the accepted final authority.
- **[RE/ND] Producer-owned context:** applicable project/application context is optional in ordinary paths and not yet demonstrated as derived authoritative producer evidence.
- **[I/U] Package/API provenance:** a file and declaration endpoint alone may not prove package instance, expected library export and supported framework version. Define and retain the minimum route/role evidence per AQ01 and related consumers.
- **[CI/I] Reference meaning and module-scope outcomes:** tuple type sites containing value-query endpoints and file-result errors accompanying ambient success need an explicit consumer interpretation. Propose a typed internal contract before migration; do not redesign public responses here.
- **[U] Inheritance/member authority:** AQ10/AQ15 and relevant declarations may require owner/base/member relations. This resolver inventory does not establish complete inherited-member binding. Trace those concrete consumers before scheduling general type-system work.
- **[U] Broad merge compatibility:** the accepted design requires relevant merge legality, while current groups do not certify full compatibility. Specify identity-changing obligations using consumed API/declaration fixtures; do not infer completeness from `Merged` or demand a whole type checker without that analysis.
- **[ND/U] Query-specific completeness and reverse closure:** point resolutions and forward captures cannot certify AQ09/AQ24/AQ25 reverse universes. The exact per-question proof and response representation remain unfinished.

## 11. Certification Implications

**[ND/CI] Current state is NOT CERTIFIED.** This is a finding that required gates are unsatisfied, not an agent attestation or owner sign-off granting certification.

| Certification gate | Current evidence / missing obligation |
|---|---|
| 1–2 Envelope/questions | [RE] AQ01–AQ26 and accepted architecture exist. [U] Fine-grained TS input-form/version boundaries are not closed; no separate standalone TS language catalog was located. |
| 3–6 Authority/identity/acquisition/semantics | [CI] Substantial native mechanisms and actual capture path exist. [U] Some scope additions lack a specific required-fact chain; conformance and coverage remain incomplete. |
| 7–8 Provenance/lifecycle | [RE] Many focused mutation/identity tests and receipts exist. [CI/U] Capture witnesses are not final-publication proof; complete route and coverage semantics remain to be established. |
| 9 Persistence/publication | [CI] Capture is explicitly dropped at `into_parts`; required lifecycle incomplete. |
| 10–11 Reachability/completeness | [RE] Existing query operations are candidate carriers in §6A, with explicit GAP/PARTIAL rows. No canonical TS/Angular answer completeness follows from internal GREEN. |
| 12 Adversarial regressions | [RE] Extensive authored suites and operator checkpoints; pending erased-query tests remain UNRUN. A question-indexed consolidated receipt is still needed. |
| 13 Differential conformance | [RE/U] Pinned source review informed implementation, but no executed equivalent-fixture compiler differential result was established in this investigation. Source reading is not an oracle run. |
| 14 Performance | [U] No new measured characterization; program-wide acquisition, cloned Units and repeated scope work warrant profiling after required correctness is bounded. |
| 15 Live consumer | [RE/U] No current #162 live canonical-question acceptance established. Batch this after deterministic and binary-path gates, per repository policy. |
| 16 Artifact/dual sign-off | [ND] Neither matching agent attestation nor owner approval of a proved certification envelope exists. This report is an investigation artifact only. |

**[ND] Certification dependencies constrain higher layers.** Angular cannot claim stronger certainty than the TS identity/context proof it consumes; NgRx cannot substitute literal/object shape for required Angular/API/registration authority. Existing useful lower-bound behavior may be preserved without being called certified.

## 12. Recommended #162 Scope

**[I] Retain the accepted architecture and all AQ01–AQ26.** Native TS establishes exact identity and the language-level relationships actually needed by those questions; Angular establishes framework roles/composition; specialized consumers establish their own semantics. Keep the compiler only as a development oracle.

**[I] Replace open-ended frontier expansion with a bounded dependency ledger.** For each remaining candidate, record: actual AQ ID and expected answer; required fact; admitted source/declaration/config form; existing or planned consumer; identity/authority failure it prevents; required retained proof/currentness; smallest falsifying fixture; and a stopping condition. A generic AQ25 citation cannot fill the consumer/fact columns.

**[I] Suggested execution order after alignment:**

1. Resolve specific requirement mappings and semantic-form/version boundaries without changing the Angular question set.
2. Finish only demonstrated Phase 2 identity/resolution blockers; preserve existing tests and truthful unresolved observations.
3. Establish the retained typed TS consumer contract and carry existing evidence through the Phase 3 owner/publication boundary.
4. Migrate one real Angular API/declaration path and shared composition consumers, then all required families, reusing lower-layer identity.
5. Consolidate question-specific adversarial, oracle, lifecycle, preservation and public-query evidence; run live consumer verification last and batch it.

**[ND] Approval boundary:** source/header wording changes that materially narrow the supported envelope, global publication/identity policy or public completeness contracts still require the repository's explicit architectural decision process. This investigation does not perform them.

## 13. Next Smallest Safe Action

**[I] Next action is a requirement decision record, not another compiler-parity patch.** Start with the pending qualified erased namespace queries: trace one real admitted Angular declaration/API/token/member site that needs the `typeof` result. If present, attach it to the actual AQ row, consumer and minimum query-role binding contract and retain the required repair. If absent, keep the tests as authored conformance candidates and do not schedule production expansion from them alone. This does not prove the construct unnecessary; it identifies the unresolved dependency.

**[I] In parallel at the documentation/contract level, specify the smallest existing artifact-to-owner route that preserves the current TS payload.** Do not implement a broad Phase 3 refactor or a new public schema under this reassessment. A concrete retained-payload proposal gives the owner something reviewable and moves toward the already-required consumer boundary.

**[RE] Work performed for this report:** bounded repository/Git source reads, the recorded upstream certification standard, existing query matrix and invariants, a read-only Clean-CTX status/context request, and static document checks. Clean-CTX returned skeleton context for artifact/capture; filesystem body inspection established the quoted ownership path. No tests, compiler, build, Node, server, external runtime, oracle run or live harness was started. No production/test source, frozen regression, normal index, branch or dependency manifest was changed for the reassessment. Only this report and a pointer in the progress ledger were authored.

**[U] Owner reconciliation needed before resuming semantic expansion:** accept or amend the proposed question-driven scope interpretation, and synchronize the upstream certification document through the normal repository workflow when appropriate. No automatic pull or removal is proposed. This is not a request to run another test batch.
