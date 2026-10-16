# Semantic Evidence Sufficiency and Recoverability Investigation

**Author:** Agent MaxHeadRoom  
**Date:** 2026-10-10  
**Status:** Investigation complete; no production implementation authorized  
**Predecessor:** docs/architecture/SEMANTIC_INTELLIGENCE_ARCHITECTURE_HARDENING_INVESTIGATION.md

## 1. Executive verdict

Clean-CTX's present semantic ceiling is materially self-imposed, but only at a specific boundary.

Tree-sitter and the existing single-parse compiler encounter authoritative invocation structure that the shared call producer intentionally reduces to caller method id, written callee name, written argument count, and spread flag. That reduction discards the invocation occurrence, receiver expression, argument occurrences, generic arguments, chaining/await context, and structural links to imported or injected symbols.

Multiple independent consumers then reconstruct portions of that same structure from raw text: NgRx effects and dispatches, RxJS pipelines, Angular injection, Spring annotations, and EF Core configuration. This is the recurring pattern the investigation was required to prove or falsify.

The finding does not justify semantic synthesis, a universal call graph, compiler-grade type checking, or a durable occurrence mega-graph. Syntax is not declaration identity. The most ambitious cross-domain links still require language resolution, domain authority, protocol normalization, external configuration, or runtime evidence.

The smallest justified correction is a typed, compilation-result-local invocation-occurrence side channel produced during the existing parse. It should preserve only source-proven structure needed by multiple consumers. It should not initially alter CoreOp::Call, canonical wire formats, snapshots, WorkspaceIndex, or public graph identities.

**Decision: DECISION C.** Pause the predecessor's limited resolution extraction long enough to establish this narrow structural evidence boundary. Otherwise its request/state API would be built around artificially weak name-and-arity producers and would immediately require domain extractors to continue reparsing source text.

**Relationship verdict:** **MATERIAL RECONSIDERATION WARRANTED** for evidence-production sequencing only. The predecessor's rejection of generalized synthesis remains authoritative.

## 2. Relationship to the predecessor investigation

The predecessor correctly established that Clean-CTX has earned shared request-scoped resolution requirements, common resolution states, provenance/completeness composition, domain-owned policies, and ephemeral neutral-to-typed refinement. This report does not reopen those conclusions.

It changes one sequencing assumption. The predecessor treated weak producers as the current input boundary and recommended limited extraction over those inputs. Repository evidence now shows that an important producer is weak because Clean-CTX discards reusable syntax it already has. A resolver accepting only written names, arity, and graph candidates would encode that accidental loss into its interface.

Unchanged conclusions:

- Domain authorities decide NgRx, RxJS, Angular HTTP, ASP.NET, Spring, and persistence meaning.
- Neutral evidence precedes typed refinement.
- Ambiguity and incomplete discovery remain explicit.
- Cross-file refinement is ephemeral unless dependency ownership is proven.
- Protocol bridges are not authorized.
- No generalized synthesis layer or capability registry is justified.

Strategic extension:

- Language producers should retain a small set of source-proven invocation occurrences before domain interpretation.
- The limited resolver should consume those occurrences rather than a callee-name-only projection.
- Producer-local raw-text reconstruction should retire only where the new evidence supplies equivalent or stronger authority.

## 3. Repository evidence-flow map

    source
      -> tree-sitter parse
         knows invocation node, function/receiver tree, arguments, nesting, spans
      -> language query captures
         keeps terminal callee and argument nodes/count; receiver omitted
      -> CallProducer
         temporarily keys calls by callee span and knows lexical callable owner
      -> CoreOp::Call
         keeps owner id, callee name, count, spread only
      -> semantic projection
         emits builtin Method --Calls--> written Method with CallEvidence
      -> WorkspaceIndex / durable semantic state
         stores the lossy edge, not occurrence structure
      -> workspace_query / calls_in_file
         returns name/count/spread and presentation ordinals

    raw source
      -> Angular/NgRx/RxJS/Spring/.NET meta-layers
         manually rediscover calls, receivers, arguments, nesting,
         imports, annotations, and ownership

The high-leverage loss is between the parse/capture walk and CoreOp::Call. CallProducer uses (start_byte, end_byte) internally to join captures, then discards that key when emitting the opcode. Language queries bind each argument node but record_argument retains only a count. TypeScript, C#, and Java grammars expose receiver-bearing shapes, but their queries retain only the final callee identifier.

Imports follow a related path. import.root arrives as raw source text at IRCompiler::emit_import_ir, which searches for " from " and stores one CoreOp::Import(generated_id, module, named_text). Module source survives, but individual bindings, local aliases, namespace/default forms, spans, lexical shadowing, and binding from a use to an import do not.

## 4. Information-retention map

| Evidence | Parser | Captures | Canonical/import IR | Semantic/index | MCP |
|---|---|---|---|---|---|
| Lexical caller | Available | Callable spans | Method id | Written Method | Owner/overload |
| Call occurrence span | Available | Callee span used temporarily | Lost | Lost | Synthetic ordinal |
| Full invocation span | Available | Not captured | Lost | Lost | Lost |
| Terminal callee | Available | Preserved | Preserved | Written Method | Preserved |
| Receiver tree/written form | Available | Deliberately omitted | Lost | Lost | Lost |
| Argument nodes/order | Available | Each node observed | Count only | Count/spread | Count/spread |
| Argument expression/span | Available | Not retained | Lost | Lost | Lost |
| Type arguments | Available | Omitted | Lost | Lost | Lost |
| Await/return/assignment context | AST ancestry | Not joined | Lost | Lost | Lost |
| Chained-call position | AST ancestry | Not retained | Lost | Lost | Lost |
| Optional/conditional access | Available | Some forms unmatched | No fact | No fact | No fact |
| Import module | Available | Whole import | Coarsely preserved | Not use-bound | Weak |
| Individual import/local alias | Available | Not structural | Textualized | Not bound | Lost |
| Field/parameter type | Available | Some declarations | Separate ops | Separate facts | No receiver binding |
| Injected property -> type | Source-proven | Text extractor sees both | Class Injects only | Property lost | Cannot resolve receiver |
| RxJS operator order | Available | Not generic | Lost | Not generic | RxShape only |

This does not claim every parser fact should survive. The loss is consequential where multiple authorities need the same source-proven occurrence structure.

## 5. Evidence classification taxonomy

- **E0:** already captured authoritatively and usable.
- **E1:** source/parser proves it; Clean-CTX loses or flattens it.
- **E2:** recoverable by language-aware binding, scope, imports, types, or member resolution.
- **E3:** recoverable only after framework/domain interpretation.
- **E4:** recoverable through explicit protocol identity and resolution.
- **E5:** statically ambiguous; candidates may be reported but not uniquely asserted.
- **E6:** fundamentally runtime/dynamic.
- **E7:** proof exists outside analyzed source.

For this.userService.save(user), the receiver/member/argument occurrence is E1; binding userService to UserService.save is E2; deciding it is an NgRx effect service dependency is E3.

## 6. Primary end-to-end case study

The strongest production-shaped corpus is src/tests/angular_meta/semantic.rs: loadUsers$ = createEffect(...), actions$.pipe(ofType(loadUsers), exhaustMap(... this.userService.getAll().pipe(map(...loadUsersSuccess...)))). The repository does not contain a coupled frontend HTTP plus backend endpoint/persistence fixture authoritative enough to extend this exact chain, so none is fabricated.

| Hop | Desired fact | Parser | Current state | Class | Loss point |
|---|---|---|---|---|---|
| Effect declaration | loadUsers$ is an effect | Property initializer and call | NgRx text scanner creates Effect | E1/E3 | Initializer ownership absent |
| Effect source | effect reads actions$ | Receiver-bearing pipe call | Text reconstruction | E1/E3 | Receiver/initializer owner lost |
| Action handling | handles loadUsers | Ordered ofType argument | NgRx text parser edge | E1/E3 | Args/order not retained |
| Operator order | ofType before exhaustMap | Pipe argument order | RxShape reparses text | E1/E3 | Expressions/nesting lost |
| Service call | calls userService.getAll | Nested member invocation | Name-like CallsService | E1/E2/E3 | Receiver/span discarded |
| Produced action | invokes loadUsersSuccess | Nested callback call | Name heuristics | E1/E2/E3 | Callback ancestry/binding absent |
| Action declaration | createAction initializes symbol | Variable initializer | Assignment text scan | E1/E3 | Declaration-initializer link absent |
| Service declaration | UserService.getAll target | DI field plus member | Not safely resolved | E1/E2 | Property/type/call bridge absent |
| HTTP request | getAll issues request | Absent from fixture | None | E7 here | Source absent |
| Endpoint/persistence | reaches and writes | Absent from fixture | None | E4/E7 | Separate producers needed |

### Case-study quantitative summary

- E0 already captured: 0 complete hops; several fragments only.
- E1 source-proven but lost: 7.
- E2 language-resolvable: 3.
- E3 domain-resolvable: 7.
- E4 protocol-resolvable: 1 future bridge.
- E5 statically ambiguous: 0 in this closed fixture; workspace candidates can be ambiguous.
- E6 runtime-only: 0 demonstrated here.
- E7 external/absent: 2 terminal hops.

The dominant limitation inside the available NgRx chain is insufficient Clean-CTX retention.

## 7. TypeScript structural evidence audit

TS_CALL_QUERY structurally recognizes identifier calls, member calls, arguments, spreads, and named bound-arrow declarations. Its comments accurately distinguish model boundaries from recoverable coverage gaps.

Parser-visible but not retained:

- Receiver expression and complete written form.
- Full invocation and callee spans.
- Argument text, kind, order, and span.
- Type arguments such as <User>.
- Parent call/chaining relationships and optional-chain status.
- Await, return, assignment, and argument context.
- Callback ancestry beyond a coarse recognized callable.
- Initializer target for calls outside callable bodies.
- Static/instance classification and declaration binding.

Well retained:

- Innermost recognized callable ownership.
- Bound-arrow owner names and spans.
- Exact written argument-node count.
- Spread qualification.
- Conservative omission for unsupported/unowned forms.

The model is intentionally minimal. It is adequate for a neutral Calls summary but underpowered as source evidence for semantic consumers.

### TypeScript quantitative summary

- E0: caller, terminal callee, count, spread = 4 facts.
- E1: occurrence, receiver, arguments, type args, context, nesting, initializer target = 7 groups.
- E2: import binding, shadowing, receiver type, declaration/member resolution = 4.
- E3/E4: none at language layer.
- E5: computed/dynamic properties and union/polymorphic targets = 2.
- E6: runtime reassignment/proxy/reflection = 1 group.
- E7: unavailable external declarations = 1 group.

## 8. C# comparison

CS_CALL_QUERY has the same normalized contract. Identifier, member-access, and generic-name calls yield final callee and argument count. Conditional access, parenthesized, and cast function shapes are omitted rather than guessed. Receiver structure, generic arguments, named-argument names, and invocation spans exist in the parse but not CoreOp::Call.

A controller calling _service.SaveAsync(dto) and a service calling _db.Users.Add(entity) collapse to SaveAsync(1) and Add(1). Separate type facts exist, but no occurrence receiver bridge connects those names to declarations.

C# may be easier to resolve when explicit types and namespaces exist, but overloads, extensions, conversions, interfaces, partial classes, conditional access, and polymorphism constrain certainty. Roslyn parity is not required for useful candidate/ambiguity output.

### C# quantitative summary

- E0: caller, terminal callee, count = 3.
- E1: receiver, occurrence, argument identity/names, generics, conditional-access coverage, context = 6 groups.
- E2: using/namespace/type/member/overload resolution = 5.
- E3: ASP.NET/EF interpretation = 2 domains.
- E4: 0 within call resolution.
- E5: overload/extension/polymorphic targets = 3.
- E6: reflection/dynamic dispatch = 1 group.
- E7: unavailable assembly metadata = 1 group.

## 9. Generic call evidence audit

1. Tree-sitter holds the function/receiver expression, arguments, type arguments, nesting, and byte ranges.
2. Capture queries retain final callee, argument nodes, spread marker, and callable declarations.
3. CallProducer uses callee span as an internal join key and reduces arguments immediately.
4. CoreOp::Call retains caller id, written callee, count, and spread.
5. CallEvidence retains count/spread and explicitly claims no receiver or resolved target.
6. WorkspaceIndex stores a neutral written-name Calls edge.
7. calls_in_file recompiles current source but returns only callee/count/spread and presentation ordinals.

| Form | Required evidence | Feasibility without full compiler |
|---|---|---|
| Local free function | scope, declarations, arity | Often authoritative if unique |
| Imported function | import binding/module, shadowing | Authoritative in complete scope; external may remain unresolved |
| Static call | receiver syntax plus type/namespace binding | Often resolvable when explicit |
| Instance member | receiver, property/variable type, candidates | Conditional; overload may remain ambiguous |
| Local method | enclosing type, receiver, overloads | Often resolvable |
| Chained call | nesting and intermediate return type | Requires progressively stronger E2 |

This evidence benefits ordinary navigation, NgRx, RxJS, Angular HTTP, DI, persistence, and Java/Spring/C# service calls. That breadth earns shared syntax mechanics, not shared semantic policy.

## 10. Receiver, symbol, and import identity

| Expression | E1 evidence | E2 requirement | E3 requirement |
|---|---|---|---|
| this.userService.save(...) | receiver, member, args/span | property -> UserService -> member | domain service meaning |
| this.store.dispatch(...) | receiver, dispatch, action arg | binding to NgRx Store | dispatch/action semantics |
| this.http.post<User>(...) | receiver, member, generic, URL/body | binding to HttpClient | HTTP semantics |
| actions$.pipe(...) | receiver, ordered argument calls | property/import binding | RxJS/NgRx meaning |
| foo.bar.baz(...) | receiver chain/member | types for links | only if domain-recognized |
| importedFunction(...) | identifier/args | import vs local shadow | library meaning if any |
| Namespace.Type.method(...) | qualified receiver/member | namespace/type/static binding | none by default |

CoreOp::Import preserves module and coarse named text, but its alias operand is a generated IR id, not the local source alias. The compiler parses imports with string searches. It cannot reliably bind individual exported/local symbols, namespace/default imports, shadowing, or a use occurrence to an import.

NgRx and RxJS therefore use raw-source package contains gates. This reduces obvious false positives but is not declaration binding. A future import boundary should preserve individual bindings, import kind, exported name, local name, module, and span. Binding uses remains E2 and scope-aware.

## 11. NgRx evidence audit

The NgRx extractors use lexical regions and balanced call-body/split helpers, which are safer than naive splitting. They still reconstruct calls, assignments, receivers, arguments, and callbacks from text:

- Package presence uses textual import gating.
- createAction, createReducer, createEffect, ofType, dispatch, select, and adapters use substring discovery.
- Assignments and callback bodies are manually delimited.
- Action/service/selector names are extracted from text.
- Success/failure production includes naming heuristics.
- Edges carry domain names but no occurrence or declaration binding.

| Fact | Classification |
|---|---|
| Invocation, receiver, arguments, nesting, span | LOWER-LAYER EVIDENCE DEFICIT |
| Package really is NgRx | MIXED: import binding plus domain package authority |
| dispatch means NgRx dispatch | LEGITIMATE DOMAIN INTERPRETATION after E1/E2 |
| ofType argument means handled action | LEGITIMATE DOMAIN INTERPRETATION after E1/E2 |
| Produced action | MIXED; current name heuristics are not authority |
| Reducer on(action, ...) | LEGITIMATE DOMAIN INTERPRETATION after E1/E2 |

The lower layer should eliminate manual structural discovery. NgRx continues to own meaning.

### NgRx quantitative summary

- E0: published domain edges exist, but none carries full occurrence/declaration proof.
- E1: initializer, dispatch receiver/arg, operator order, callbacks, service calls, produced actions, spans = 7 groups.
- E2: imports, receiver type, action/selector/service binding = 4.
- E3: action/effect/reducer/selector/dispatch meaning = 5.
- E4: 0.
- E5: duplicates, conditional outputs, dynamic action expressions = 3.
- E6: runtime factories/configuration = 1.
- E7: absent external declarations/config = 1.

## 12. RxJS evidence audit

RxShape retains observable/subject summaries, pipe owners, ordered operators, and combinator arguments. Its producer searches for .pipe(, scans backward for an owner, balances call bodies, splits top-level arguments, parses names, and truncates argument summaries.

For actions$.pipe(ofType(saveUser), switchMap(...), map(...), catchError(...)), the AST proves occurrence, receiver, ordered expressions, nested callbacks/calls, and spans. Imports/binding are E2. Operator behavior is E3. Runtime emitted values remain E5/E6.

RxJS needs both shared invocation evidence and RxJS-local ordered interpretation. Operator order must not become an unordered edge set.

### RxJS quantitative summary

- E0: RxShape reconstructs and retains order = 1 capability.
- E1: occurrence, receiver, args, callbacks, nested calls, spans = 6.
- E2: import and declaration binding = 2.
- E3: operator/library meaning = 1 group.
- E4: 0.
- E5: callback result/higher-order target ambiguity = 2.
- E6: scheduler/runtime stream behavior = 1.
- E7: absent external declarations = 1.

## 13. Angular DI and service resolution

Constructor parameters prove a parameter/property name and type. The DI extractor publishes injection identity as the type and the graph emits Component --Injects--> Service. That valid summary discards the property name required to bind this.userService to UserService.

For constructor injection, property-to-type is E1, Angular participation E3, and type/import/member binding E2.

For private userService = inject(UserService), the AST can prove field, inject call, and token argument. Current decorator extraction scans signal-like fields textually and retains no occurrence/property bridge. This path needs a focused correctness audit before becoming resolution authority.

Minimum chain:

    receiver this.userService
      -> same-class property userService
      -> constructor type or inject token UserService
      -> import/namespace and declaration candidates
      -> member candidates named save

Every arrow needs evidence and ambiguity handling. Name similarity is never proof.

## 14. Frontend HTTP evidence

For this.http.post<User>('/api/users', user), source structure can provide receiver, post member, User type argument, URL/body expressions, occurrence span, containing method, and whether the URL is literal, template, concatenation, or dynamic.

Language resolution binds http to a field and HttpClient import (E2). Angular HTTP authority classifies POST (E3), distinguishing FakeHttp.post. Literal paths are authoritative written evidence; partial expressions are conditional; arbitrary runtime expressions are E5/E6. Environment base URLs/interceptors may require E7.

No current generic call fact is sufficient. No backend bridge should be considered until a frontend producer emits trustworthy request occurrences with exact/partial/dynamic path status.

### Frontend HTTP quantitative summary

- E0: containing callable and terminal post name = 2 fragments.
- E1: receiver, occurrence, type arg, URL, body, expression form = 6.
- E2: property/type/import binding = 3.
- E3: HttpClient operation classification = 1.
- E4: request-to-endpoint resolution = 1.
- E5: partial URL/candidate ambiguity = 2.
- E6: runtime URL/interceptor behavior = 1.
- E7: environment/base/proxy/deployment routing = 1.

## 15. Backend endpoint evidence

### ASP.NET

The .NET layer textually extracts controllers, attributes, routes, verbs, parameters, and return types. Local action facts contain more detail than public edges, but graph projection separates controller/action and controller/base-route summaries instead of one occurrence-bearing endpoint identity with combined route, verb, handler, path parameters, and types.

Attributes and methods are structurally present (E1). Combining routes and ASP.NET conventions is E3. Conventional routing, filters, generated endpoints, and deployment configuration can be E6/E7.

### Spring

spring_meta::semantic reuses string-scanning annotation extractors. Method annotations yield method name/kind/argument, but mappings enter a class-wide list and produce Controller --EndpointMapsTo--> Endpoint("METHOD path"). The handler relationship and composed controller/method route are lost.

Java annotations/methods are parser-visible (E1). Aliases, composed annotations, route combination, and Spring conventions are E3. Config routing can be E7; runtime registration E6.

### Backend endpoint quantitative summary

- E0: controller and partial route/method summaries = 2 groups.
- E1: annotation occurrence, handler, route components, parameters/types/spans = 5.
- E2: annotation/import/type/declaration binding = 3.
- E3: ASP.NET/Spring rules = 2.
- E4: normalized request-to-endpoint identity = 1.
- E5: overlaps/conventions/version candidates = 1.
- E6: runtime registration = 1.
- E7: routing/version/proxy configuration = 1.

## 16. Persistence evidence

EF Core currently establishes DbContext --HasEntity--> Entity from textual DbSet discovery. It does not prove reads, tracking, writes, or commits.

| Observation | Proven meaning |
|---|---|
| DbSet<User> Users | Entity set exists; not a read/write |
| _db.Users reference | Set access; operation unknown |
| Where/query enumeration | Potential query/read; execution may be deferred |
| Add/Update/Remove on bound EF receiver | Tracking/mutation intent |
| SaveChanges on bound DbContext | Commit attempt, not successful runtime write |
| arbitrary Save() | Only a method name |

Receiver/invocation evidence is E1; type/member binding E2; EF/JPA meaning E3. Repository abstractions may remain E5. Schema/triggers are E7; success and affected rows E6.

### Persistence quantitative summary

- E0: DbContext-to-entity summary = 1 group.
- E1: receiver, chained set access, args/spans, commit context = 4.
- E2: receiver/property/type/member binding = 3.
- E3: read/track/write/commit distinctions = 4.
- E4: 0.
- E5: repository abstraction/deferred execution = 2.
- E6: successful commit/runtime transaction = 1.
- E7: schema/triggers/config = 1.

## 17. Meta-layer duplication audit

| Area | Reconstructed mechanics | Legitimate semantics | Verdict |
|---|---|---|---|
| NgRx | calls, assignments, receivers, args, callbacks, imports | NgRx relationships | MIXED; large lower-layer deficit |
| RxJS | pipe discovery/owner/args/order | stream/operator meaning | MIXED |
| Angular DI | constructor/inject names/types | Angular token rules | MIXED |
| Angular templates | ordered template grammar/control ancestry | template semantics | LEGITIMATE local structure |
| Spring | annotations, methods, fields | mapping/autowiring | MIXED |
| ASP.NET | attributes/actions/routes | routing | MIXED |
| EF Core | classes/properties/calls/chains | persistence meaning | MIXED |

Text extraction is not categorically wrong. It is debt when a language parser already has the same structure and several consumers rebuild it.

## 18. Information-loss ledger

| Evidence | Available | Lost at | Consequence | Recoverable | Beneficiaries | Leverage |
|---|---|---|---|---|---|---|
| Invocation/full span | AST | query/opcode | no stable occurrence | Yes E1 | all call domains | HIGH |
| Receiver tree | AST | query | fake receivers indistinguishable | Yes E1 | all call domains | HIGH |
| Ordered arguments | AST/captures | record_argument | source rescanning | Yes E1 | NgRx/RxJS/HTTP/persistence | HIGH |
| Nesting/parent | AST | capture/model | chains/callbacks rebuilt | Yes E1 | generic/RxJS/NgRx | HIGH |
| Await/return/assignment | ancestry | not captured | weak behavioral ownership | Yes E1 | effects/HTTP/persistence | HIGH |
| Import bindings/aliases | AST | textual import IR | package gates without binding | Yes E1/E2 | all TS/Java domains | HIGH |
| Injected property -> type | TS/DI parser | class Injects edge | receiver cannot bind | Yes E1/E2/E3 | DI/service/HTTP/NgRx | HIGH |
| Generic args | AST | call query | response/API type lost | Yes E1 | HTTP/repositories | MEDIUM |
| Callback ancestry | AST | callable normalization | produced values lose origin | Yes E1 | NgRx/RxJS | HIGH |
| Endpoint handler/composed route | annotation/method AST | domain projection | weak endpoint identity | Yes E1/E3 | HTTP impact | HIGH |
| Named argument names | C# AST | count-only call | weaker overload evidence | Yes E1 | C#/ASP.NET | MEDIUM |
| Runtime implementation | runtime | unavailable | unique target impossible | No E5/E6 | all | LOW retention value |

## 19. Static provability matrices

### Save-button-to-persistence path

| Hop | Source proof | Current strength | Gap | Recoverability |
|---|---|---|---|---|
| Template click -> handler | AUTHORITATIVE with ownership | event linkage not established | E1/E2/E3 | High |
| Handler -> dispatch | AUTHORITATIVE with bound Store/action | text edge | E1/E2/E3 | High |
| Action -> effect | AUTHORITATIVE under NgRx | text edge | E1/E2/E3 | High |
| Effect -> service | CONDITIONAL/AUTHORITATIVE | name-like edge | E1/E2/E3 | High/conditional |
| Service -> HTTP | AUTHORITATIVE for bound HttpClient/exact path | absent | E1/E2/E3 | High for constrained forms |
| Request -> endpoint | CONDITIONAL | absent | E4/completeness | Medium |
| Endpoint -> service | CONDITIONAL/AUTHORITATIVE | partial | E1/E2/E3 | Medium/high |
| Service -> persistence | CONDITIONAL under bound EF/JPA | entity only | E1/E2/E3/E5 | Medium |
| Commit succeeded | UNAVAILABLE | unavailable | E6 | Not static |

Nine hops: five have strong source authority for constrained forms but are weakened; three require conditional resolution; one is runtime-only.

Reverse backend impact requires forward occurrences with identity/provenance. Current name-only edges cannot safely invert several hops. Ordinary calls range from authoritative within closed local scope to ambiguous across overloads, interfaces, chains, and external packages.

Across cases E1 is the largest actionable class. E4-E7 dominate later cross-domain/runtime boundaries, so richer evidence helps substantially without authorizing unconditional synthesis.

## 20. Cross-ecosystem falsification

**React/TypeScript:** JSX events, functions, Redux dispatch, fetch/axios, and hooks need the same occurrences, receivers, arguments, imports, callback ancestry, and spans. React meaning stays local.

**Java/Spring:** Java method_invocation exposes receiver-bearing structure, but JAVA_CALL_QUERY retains only name and argument count. Controller -> service -> repository benefits from the same evidence; Spring meaning stays local.

**C#:** explicit types help resolution, but the shared call producer makes the same reduction. Controller -> service -> DbContext is not already solved.

**Rust:** method calls expose receivers, but traits, deref, generics, macros, and associated functions make resolution language-specific. A shared envelope may preserve written structure, never normalize Rust resolution semantics.

The result supports shared syntax mechanics and rejects shared semantic authority.

## 21. Structural-evidence-substrate assessment

The invocation-occurrence hypothesis is confirmed:

1. Receiver, ordered args, span, nesting/context, and binding inputs recur.
2. Generic calls, NgRx, RxJS, HTTP/DI, EF, Spring, and React need them.
3. Tree-sitter already exposes syntax and spans, not resolved identity.
4. Preservation would substantially reduce text reparsing.
5. Generic call navigation also improves.
6. Multiple meta-layers benefit.
7. Typed evidence beside compilation result is the best initial placement.
8. Request-local size is manageable; durable indexing may be large.
9. Initial durability is unnecessary.
10. WorkspaceIndex representation is unnecessary initially.

Smallest earned contract:

- source identity/hash;
- proven containing callable/declaration;
- zero-based half-open UTF-8 invocation span;
- written function/callee form;
- optional receiver form/span;
- terminal name/span;
- ordered argument occurrences with spans and source slices or kinds;
- spread/named qualifiers where supported;
- type-argument occurrences;
- immediate context: nesting, argument index, initializer, await/return/assignment;
- explicit omission/status for unsupported shapes.

It contains no resolved declaration, framework kind, HTTP verb, persistence meaning, or confidence score.

## 22. Canonical IR versus alternative placement

### A. Enrich canonical IR

One serialized representation, but it changes wire/binary/hierarchy, validation, hashes, deltas, compatibility, snapshots, rendering, and storage. It overloads a compact semantic instruction set with occurrence-heavy syntax. Reject initially.

### B. Typed structural evidence beside canonical IR — recommended

Produced during the same parse, keeps canonical IR simple, supports occurrences, serves multiple consumers, and can remain ephemeral. It adds an explicit compilation-result lifecycle. This is the smallest correct boundary.

### C. Producer-local capture access

Minimal central change, but preserves repeated extraction and cannot improve generic calls. Keep only for domain-specific syntax.

### D. Query-time source reinspection

Current and non-durable, but repeats parsing and diverges across consumers. Acceptable for rare features, not this recurring requirement.

### E. Current architecture

Truthful for neutral name/arity calls, insufficient as the next resolution foundation.

## 23. MCP and user-verification implications

No harness is implemented here. Future capabilities must be verifiable from fresh state.

| Capability | Positive | Unresolved/ambiguous | Incomplete | Adversarial | Lifecycle |
|---|---|---|---|---|---|
| Call occurrence | receiver/args/span | dynamic form neutral | parse failure explicit | nested commas safe | edits change spans; delete retracts |
| NgRx dispatch | bound Store/action | missing/duplicate action neutral | partial discovery forbids typing | FakeStore.dispatch not NgRx | refresh removes stale result |
| RxJS pipe | bound RxJS/order | unknown operator written | missing import explicit | Thing.pipe not RxJS | order/currentness retained |
| Angular HTTP | bound HttpClient/path | dynamic URL classified | dependency partial | fake post not HTTP | changes retract |
| Persistence | bound DbContext operation | repository conditional | candidates partial | Save() not persistence | edit/delete retract |
| Endpoint | handler/composed route | overlaps candidates | controllers partial | foreign annotation rejected | replacement retracts |

calls_in_file is a natural first proving surface because it recompiles current source and avoids durable occurrence semantics. Public schema changes need separate approval and RED tests.

## 24. Performance, storage, and lifecycle

Collect evidence in the existing combined query/capture walk; never add a second parse. Eager text allocation may be expensive, so prefer spans and lazily sliced current source while ownership is valid.

Occurrences scale with call sites and can dwarf declarations. Persisting them in WorkspaceIndex/SQLite would expand snapshots, compatibility, deduplication, and invalidation. No durability is justified initially.

Ephemeral evidence belongs to one source identity/hash. It must not survive replacement, deletion, refresh invalidation, or hash mismatch. The side channel alone changes no durable producer output and requires no generation bump.

## 25. Recommended roadmap

1. **Freeze the narrow evidence contract.** TypeScript/C# first; written evidence only; explicit omissions.
2. **Add RED retention tests under src/tests.** Receivers, nesting/chains, callbacks, generics, spread/named args, context, adversarial text; preserve existing Calls behavior.
3. **Implement the ephemeral side channel in the same parse.** Return it beside canonical/semantic output; no serialization/index/public API.
4. **Prove two different consumers.** Generic call occurrence detail and one NgRx/RxJS reconstruction replacement. Revise or abandon sharing if language/domain leakage appears.
5. **Resume limited extraction.** Consume occurrence evidence with explicit ambiguity/completeness; begin with bounded imports/local receiver binding.
6. **Pilot domains separately.** NgRx, Angular HttpClient, and C#/EF. Investigate HTTP bridging only afterward.

## 26. Explicit decision A/B/C/D

**DECISION C — pause limited extraction and first strengthen a specific structural evidence substrate.**

The threshold is met because the resolver input shape is materially affected. Name + count + spread cannot distinguish FakeStore.dispatch from NgRx Store dispatch, bind this.userService.save, preserve repeated same-name occurrences, or explain ordered/nested RxJS calls. Adding these later would break the resolver contract or retain parallel text-resolution paths.

The pause covers only the occurrence contract and a two-consumer proof. It does not authorize reference/annotation registries, broad symbol solving, domain migrations, persistence, or public API expansion.

## 27. Open questions

Blocking:

1. Retain argument spans/kinds only, or eagerly owned text? Prefer spans plus current source where safe.
2. Which v1 contexts are required: argument nesting, initializer, await, return, assignment?
3. How do initializer calls receive a proven declaration owner without fabricating a callable?
4. Can full invocation roots join the combined query without duplicate-match explosion?
5. Where does the collection live so all consumers share source hash/lifecycle?

Non-blocking:

- Java/Rust adapters after TypeScript/C# proof.
- Individual import-binding evidence may be next, but should not be bundled without need.
- Annotation occurrences may later earn similar treatment; not approved here.

## 28. Rejected abstractions and strongest falsification

Rejected:

- StructuralEvidenceEngine, EvidenceBus, registries, universal occurrence framework.
- Universal invocation graph or durable storage of every occurrence.
- Full AST retention or compiler emulation.
- One cross-language receiver resolver.
- General synthesis or protocol bridge.
- Replacing domain authority with language guesses.

Strongest argument against richer evidence: syntax is not identity; receivers are dynamic, imports shadowed, callbacks polymorphic, routes external, and storage runtime-dependent. Rich occurrences cost memory while many answers remain conditional. This defeats canonical-IR enrichment and broad durability.

It does not defeat the narrow side channel: multiple consumers reconstruct the same parser-known structure; spans/receivers/args are facts, not guesses; ephemeral ownership limits cost; ambiguity remains explicit; and a two-consumer proof gates expansion.

Strongest confirmation:

    tree-sitter knows receiver + arguments + nesting + spans
      -> capture retains final name and counts arguments
      -> CoreOp discards occurrence structure
      -> NgRx/RxJS/Angular/Spring/.NET recreate portions from text

This occurs across unrelated languages and domains.

## 29. Final verdict

The hypothesis is **strongly confirmed for invocation-centric capabilities and partially confirmed for end-to-end system understanding**.

Clean-CTX is leaving trustworthy intelligence on the floor: occurrence spans, receivers, ordered arguments, nesting/context, type arguments, and property/import binding inputs. Preserving them improves ordinary calls and several meta-layers without weakening correctness.

Source still cannot prove everything. Binding is E2; framework meaning E3; HTTP matching E4; polymorphism/dynamics E5/E6; deployment/schema configuration E7. Richer evidence must clarify those boundaries, not erase them.

The predecessor's limited-extraction model remains correct, but should not be implemented over an artificially impoverished producer. First preserve a narrow ephemeral invocation occurrence through the existing parse, prove generic and NgRx/RxJS consumers, then resume limited extraction.

Repository evidence supports this future invariant with an exception:

> Do not discard structured source evidence required by multiple semantic authorities when those authorities would otherwise reconstruct it heuristically from text—unless retention imposes disproportionate lifecycle, storage, or compatibility cost, in which case keep reconstruction local and document the boundary.

**Final decision: DECISION C. MATERIAL RECONSIDERATION WARRANTED for evidence sequencing; generalized synthesis remains rejected.**
