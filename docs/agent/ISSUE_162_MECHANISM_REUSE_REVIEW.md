# Issue #162: shared mechanism and specialized semantics review

Agent: Agent MaxHeadRoom
Date: 2026-10-23
Status: documentation integration and bounded static findings for owner review.

## Scope and normative placement

The owner supplied the Shared Mechanism / Specialized Semantics Doctrine and
requested its application across language and meta-layers. No production refactor
or runtime execution is part of this documentation change. Subsequent operator
member42-*.log receipts confirm 10 focused member and compatibility tests passing,
with no failures/ignored or reported warnings. Member lifecycle and generation-42
binary verification remain pending independently. Pending-runtime statements
below describe the earlier documentation checkpoint.

The single governing definition is
[MECHANISM-REUSE-001](../ARCHITECTURAL_INVARIANTS.md#mechanism-reuse-001-equivalent-mechanics-are-shared-semantics-keep-their-owners),
classified DOCUMENTED, not mechanically enforced. It requires reuse when authority,
lifecycle and correctness contracts are equivalent, prohibits convenient competing
authority, and permits explained specialization when those contracts differ.

Supporting integration uses links rather than competing definitions:

- [Certification standard](../LANGUAGE_META_LAYER_CERTIFICATION.md) §2 links the
  doctrine; §9 explicitly reviews ownership/reuse/specialization and parallel paths;
  owner sign-off includes that review. Existing certification gates and dual
  sign-off remain unchanged.
- [Architecture audit](architecture.md) adds the consumer ownership/reuse review
  and four evidence classifications. Its completed-migration sole-path wording
  now explicitly applies to the contract being replaced.
- [Canonical question matrix](../Clean-CTX_Canonical_Question_Matrix_Standard.md)
  review question 10 links contract-equivalence review to the existing weaker-
  mechanics question. Questions still precede facts, authority and implementation.

The certification file is present in the current working tree and was read here.
The earlier reassessment's upstream-only observation describes its earlier state;
this task performs no Git synchronization or historical evidence rewrite.

## Evidence classification

After the generation-42 focused GREEN, the confirmed exact-span lookup below
was extracted to TypeScriptAuthority::value_reference_at. Member API and field
inject consumers now use it; their Angular role/owner checks remain specialized.
Subsequent member42-reuse-*.log receipts record 20 focused tests passing with
0 failures/ignored and no reported warnings. The corrected member lifecycle batch
subsequently passed 2 tests in member42-lifecycle-rerun.log, with no failures/ignored
or reported warnings. The accessor preserves unique-entry, exact-span and Resolved-only admission,
including refusal of duplicate equal entries. No payload or generation change
is intended. The focused post-extraction receipts supersede its initial UNRUN state.
The table records the pre-extraction finding that justified this bounded reuse.

These findings are static source observations, not a conformance certification or
a claim that all layer contracts have been audited. Passing tests and binary
receipts establish their recorded behavior, not universal contract equivalence.

| Classification | Located path and finding | Consequence |
| --- | --- | --- |
| Confirmed equivalent duplication | member_authority.rs::endpoint and inject_authority.rs::value_endpoint both select the unique exact-span entry from the same retained reference_sites and accept only Resolution::Resolved, returning DeclarationId. Their small lookup contracts match by inspection despite iterator/allocation differences. | A concrete shared retained-reference accessor is a justified extraction candidate during authorized migration. No helper was extracted solely for this doctrine task. This is duplicate lookup mechanics, not evidence of two complete TypeScript resolvers. |
| Likely duplication requiring investigation | member_semantics.rs retains bounded written-import admission in its no-authority compatibility path and separate functional-inject role path; provider_semantics.rs retains bounded identity helpers for nonmigrated registration/strategy forms. | Some questions may be answerable from existing neutral authority, but role, token, strategy and functional-owner contracts must be traced separately. Do not remove compatibility behavior or declare every helper equivalent from its spelling. |
| Likely duplication requiring investigation | module_metadata/sources.rs::Contributors reads dependency bytes independently in the legacy composition path, while migrated canonical composition consumes authorized retained declaration facets. | Independent acquisition is suspicious for the same claimed canonical fact. Establish actual caller/envelope and proof equivalence before removing or reusing it; legacy lower-bound behavior is not automatically identical to canonical composition. |
| Legitimate specialization | Native TS declaration/value resolution; Angular class_role_authority/public_api; DI constructor/string interpretation; module composition eligibility. | Language meaning and framework/API/DI/composition meaning have different owners. Angular consumes identities rather than making TS responsible for Angular roles. Shared lower proof does not make these evaluators interchangeable. |
| Legitimate specialization with shared mechanics | Declaration and string-value CanonicalEndpoint variants require different identity evidence, while both use the existing source-owned artifact/session/durable/index lifecycle. | Preserve the distinct identity contracts and common transport, currentness, generation and retirement. Equal string values and equal object descriptions are different semantic cases. |
| Insufficient evidence | Full forms/routes/RxJS/NgRx and cross-language producer equivalence, full member identity/inheritance/template joins, complete reverse or injector closure. | This bounded source review establishes no cross-family consolidation requirement or certification for those paths. Trace their actual canonical questions and contracts during their migrations. |

The confirmed helper duplication includes code authored in the current member
slice. It is recorded explicitly rather than defended as necessary specialization.
An exact retained-reference accessor belongs at the neutral evidence owner if
extracted; Angular's API/role selection remains above it. This report does not
authorize a generic resolver, registry, trait or new lifecycle around that helper.

## Current member consumer: required ownership answers

The default path is layers/meta/angular_semantic.rs -> member_semantics::
project_with_authority -> member_authority -> existing HasInput/HasOutput
projection -> source-owned publication -> registered workspace_query. The current
repair is authored and its post-repair runtime results are not claimed here.

| Review question | Current located owner and evidence limit |
| --- | --- |
| 1. Where does canonical identity originate? | API endpoints and class declaration IDs originate in native TS retained evidence. HasInput/HasOutput member names remain the established projection; exact canonical member identity is still a gap, not inferred from those names. |
| 2. Where does authority originate? | Native exact occurrence binding and package/export evidence, then shared Angular class/API role interpretation. A written @angular/core module is insufficient. |
| 3. How is evidence acquired? | The normal MCP compilation request supplies authorized roots/project context to the existing immutable Host/resolver capture. Invocation spans come from the existing compilation evidence. |
| 4. How is provenance retained? | TypeScriptAuthority retains IDs, source/hash/spans, reference outcomes, package/project facts and witnesses. The first member slice adds no fabricated per-member declaration annotation; member-specific identity/provenance completion remains open. |
| 5. How is currentness determined? | Existing source hash, consumed witnesses and compatible source-owned generations. Member owner admission reuses class-role currentness checks; no member cache or freshness clock was introduced. |
| 6. How is it persisted/restored? | Existing artifact/projection, RetainedAuthority, SemanticSnapshot, DurableSemanticSnapshot and SQLite path. New member-specific restart verification is still required; string/member certification is not interchangeable. |
| 7. How is it published? | The existing production semantic evaluator and compatible source-owned publication path into WorkspaceIndex. No query-only overlay or separate registry. |
| 8. How are duplicate facts prevented? | Existing EdgeKey occurrence mechanics. These member projections do not yet carry canonical member endpoints; name/owner collisions require their concrete identity work rather than a completeness claim. |
| 9. How are stale facts retired? | Existing source replacement/retirement, dependency currentness and documented external-refresh boundaries. The migrated member family's mutation/lifecycle evidence must still be collected. |
| 10. How is completeness represented? | Existing question/query coverage contracts; no new exact AQ10, reverse-universe, inheritance or template-use claim follows from member relationships. |
| 11. What is shared infrastructure? | Authorized snapshots, language resolution, typed payload, witnesses, compatibility/generations, durable transport, publication, index keys and retirement; Angular public API/class-role admission is shared within Angular. |
| 12. What is specialized semantics? | Angular input/output/model classification, admitted required-member API meaning, owner/member relationships and future public alias/inheritance/template interpretation. Functional DI role semantics remain separately unmigrated. |

## Documentation consistency findings

The inspected normative sections already agree on sharing mechanics, retaining
lower authority, separating projection from identity, and refusing manufactured
authority. No direct normative contradiction was located in that bounded review.

Two apparent tensions are clarified, not replaced:

- The audit's sole-production-path requirement governs replacement of the same
  contract. It does not prohibit legitimate specialized producers with different
  contracts. Their existence is not automatically a competing authority defect.
- Avoid-premature-abstraction guidance remains in force. Equivalent contracts
  justify reuse; similar code or imagined future consumers do not mandate a new
  universal subsystem. New needs remain question-driven and owner-authorized.

The existing standard's share-mechanics principle was retained and linked to the
single invariant rather than replaced by another independent doctrine. No global
identity, persistence, query/completeness, lock/retry policy, supported envelope or
runtime gate was changed. No agent memory files, code, tests or manifests were
edited for this documentation task. No Cargo, Clean-CTX/MCP runtime, live harness,
staging, commit or push was performed. Static UTF-8/link/whitespace checks are
reported separately; documentation review does not attest implementation compliance.
