# Issue #162: provider owner/context trace and next bounded contract

Agent: Agent MaxHeadRoom
Date: 2026-10-25
Status: bounded static trace; next candidate P1.D1, not an injector-model approval.

The operator subsequently supplied the intended public registration RED after
both prerequisite controls passed. The bounded approval request and batched exit
criteria are recorded in the
[Directive owner decision](ISSUE_162_PROVIDER_DIRECTIVE_OWNER_DECISION.md).
No production repair has been made.

## Tracked RED control authored

angular_provider_directive_owner_issue_162.rs now requires an established
Component registration before testing direct, aliased-barrel and namespace
Directive API routes. Each route requires retained exact native Directive role
proof before the public two-token registration assertion. Same-named tokens,
repeated registration, repeated queries and unchanged consumer bytes are checked.
The control is UNRUN. No production change or architectural approval is claimed.
The operator owns execution; only an intended behavioral assertion failure after
the Component and native-role controls constitutes the applicable RED.

## Finding and proposed slice

The next concrete P1 question is: **which canonical tokens does this proved
Directive directly register through its providers metadata?** This is a local
registration question, not which injector supplies those tokens to a consumer.

Angular 20's official [Directive API](https://v20.angular.dev/api/core/Directive)
includes providers: Provider[]. This establishes the documentary API surface;
it is not an executed Angular oracle or Clean-CTX acceptance result.

The existing evaluator already nominates directive owners. The registration
producer and retained provider-array interpreter then reject them. Extending
the default producer's owner admission, preserving Directive entity identity,
is the candidate bounded contract. No separate directive provider pipeline is
justified by this trace.

## Located production path

| Boundary | Located evidence | Finding |
| --- | --- | --- |
| Native/class role | class_role_authority::api_role and decorator_role | Directive is already a distinct role after canonical core export/package proof. Written decorator spelling alone is insufficient. |
| Default evaluator | layers/meta/angular_semantic::class_semantic_edges, provider call around line 155 | Recognized class roles reach provider_semantics::project_with_authority. The adjacent Component/Module-only block governs module composition, not this provider call. Do not widen that unrelated composition block. |
| Provider producer | provider_semantics::project_with_authority, opening guard | Accepts only Component/Module. It returns before acquiring a directive's array. Its owner-type selection also has only Module/Component outcomes. |
| Retained position admission | string_token_authority::provider_array | Both declaration-role and decorator-role checks accept only Component/Module. This is a second explicit owner boundary, not just a rendering issue. |
| Demanded fields | module_metadata::DECLARATION_FIELDS; native declaration facets | Existing providers/provide/strategy fields and decorated-class facets are already captured through the normal resolver path. Verify actual Directive capture in the regression; do not invent another inventory. |
| Strategy pair authority | provider_pair_authority::pair/admits | Uses the same provider_array owner gate. Directive strategies must reuse it if this owner form is approved. |
| Compilation/ownership | angular_semantic::evaluate -> MetaLayerEvaluation.semantic_edges/dependencies; semantic_compilation::artifact; retained::RetainedAuthority | Existing source-owned production return/retained paths carry the facts. No new owner lifecycle is needed for a directive. |
| Persistence/currentness | existing SemanticSnapshot/DurableSemanticSnapshot and retained admission | Identity/occurrence proof must survive existing SQLite/session/index restoration and currentness, as in prior slices. This trace does not establish new Directive lifecycle acceptance. |
| Public exposure | existing registered workspace_query forward/reverse operations and WorkspaceIndex | Existing Directive and RegistersProvider concepts can express the bounded question. Public-path proof is still required; no new enum or schema field is proposed. |

## Owner/context inventory limits

| Context | What this trace establishes | Remaining contract |
| --- | --- | --- |
| Component/Module direct providers | Migrated producer and existing receipts cover bounded local registrations. | Indirect composition, all forms and effective scope remain open. |
| Directive direct providers | API surface and concrete producer/admission guards located. | Demonstrate RED, then owner-authorize/verify the bounded extension. |
| Component viewProviders | Not among captured provider fields in the inspected field list. | Separate visibility/owner-position contract; do not treat it as providers by spelling. |
| Injectable providedIn | decorator_args parses selected string metadata. | This is not proof of an application/environment injector registration or satisfaction path. |
| Route providers | Routing shape is projected separately from provider_semantics. | Exact route/config owner and its provider/injector context need their own trace; functional DI role proof does not supply it. |
| Bootstrap/application/environment providers | No integrated injector model was established by this bounded inspection. | Trace actual APIs/config merge/ownership before designing effective-provider semantics. Absence from this trace is not proof no relevant code exists elsewhere. |
| TestBed providers | testing extraction is separately called by the default evaluator; inspected configuration code renders selected provider names. | Canonical test owner/config/provider authority and test-versus-production scope remain W10/W04 obligations. |

## P1.D1 exit contract and batched verification plan

Proposed production behavior: an exact native/API-proved Directive with an
admitted direct providers array publishes canonical RegistersProvider edges
whose subject is that Directive and whose objects retain the provided-token
identities. Existing approved strategy policies are reused; neither Directive
nor Component admission is inferred from terminal names. Pipe/Service and
foreign/shadow/type-only decorators do not gain provider-owner authority.

This is an extension of Angular provider-owner admission. If behavioral RED
confirms it, record the bounded owner decision before changing production;
existing strategy approvals are not an automatic all-owner approval.

Plan the whole slice before repair:

1. Real public-query control with a nonempty existing Component registration,
   then native Directive-role proof and expected Directive registration through
   alias/barrel/namespace API routes. Observe the earliest valid RED.
2. Under approval, reuse existing owner/field/token/strategy mechanics and
   preserve exact Directive subject identity. Batch foreign/shadow/type-only,
   competing owner/field and existing-owner preservation controls.
3. Batch complete retained/durable restoration before queries, unchanged-owner
   dependency/API mutation/repair and independent same-name owner retirement.
4. One stable lint/rebuilt-binary acceptance batch in the existing driver where
   practical; update P1's subrow without declaring P1, P3 or W04 closed.

No element-injector instance, provider precedence, winning registration, view
visibility, alias-chain resolution, consumer satisfaction or reverse completeness
is established by D1. Those remain P3–P6 questions with their own scope proofs.

## Operational and evidence limits

Clean-CTX graph discovery failed because its CBM connection exited. Both requested
code-context items returned skeletons successfully; targeted source reads supplied
the bodies used here. This tooling event is not a semantic defect diagnosis.
No Cargo/test/build/lint/binary/server or field harness was started by the agent.
No production code was changed and no approval to a new injector model is inferred.
