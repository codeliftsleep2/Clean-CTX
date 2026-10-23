# Issue #162: retained declaration evidence for module composition

Agent: Agent MaxHeadRoom
Status: owner APPROVED; implementation authored; focused/consolidated relation and lifecycle regressions GREEN; binary/stdio PASS; remaining consumers and final field verification pending.

## Owner decision

The owner explicitly approved demanded dependency-declaration facets and their
cross-file Angular module-composition consumption. Whole-source/program
retention, independent filesystem/spelling fallback and parallel ownership or
publication mechanisms remain prohibited. The historical proposal below is
preserved as the approved boundary; its pending-decision wording is historical.

## Focused verification checkpoint

Captured operator log `target/issue-162/module-durable-rerun.log` records all
three durable-facet regressions passing, including exact restoration into the
session/index and post-restore mutation retraction. The remaining filesystem
hydration preservation test also passed (1/1). These receipts do not establish
aggregate preservation, binary/stdio acceptance, completeness or live consumer
verification. Historical proposal and RED evidence below remain preserved.

## Demonstrated requirement

AQ02/AQ05 require NgModule membership and eligible declaration roles. AQ25
requires canonical joins and current provenance. The existing consumer is
`angular_meta/module_metadata.rs`, reached by the default Angular semantic
evaluator and exposed by registered `workspace_query` forward edges.

The operator observed all three new public-query failures: an owner decorator
alias loses membership, a target decorator alias loses membership, and a target
with a foreign decorator gains membership despite its own class-role consumer
rejecting it. Initial public-role and unaliased membership controls completed.
Restoration after the foreign assertion remains unclassified.

## Located authority break

The initiating TypeScript capture retains the canonical endpoint for a written
`Leaf` reference. It does not retain the decorator reference proof inside that
dependency's declaration. Package-export evidence presently describes endpoints
of captured initiating-source references, not a dependency declaration's API use.

`module_metadata::project` repeats written-name admission of the owner.
`resolve_leaf` and `declaration_kind` repeat it for targets.
`module_metadata/sources.rs::Contributors` independently reads filesystem bytes
and performs relative exact-file/.ts lookup. Its ordinary content dependencies
are not the shared authorized Host's project/export/physical/negative proof.
Using those reads to recover an Angular role would preserve competing authority.

## Decision requested

Extend the existing retained, source-owned typed evidence with the dependency
declaration facets needed by composition. This is an extension beyond merely
transporting the already-produced initiating-source capture. It affects neutral
acquisition, Angular cross-file consumption and the durable payload carried by
existing session/SQLite compatibility and currentness machinery.

The recommended boundary is:

1. Acquire demanded declaration and composition-reference evidence through the
   existing authorized native resolver and its immutable Host snapshots, under
   the initiating project's actual context. Do not infer authorization from an
   endpoint path, project directory or current index entry.
2. Retain typed declaration IDs, exact decorator callee/argument/reference spans,
   canonical endpoints or non-success outcomes, relevant package/export proof
   and consumed witnesses. Keep language evidence neutral; Angular owns roles,
   membership and metadata interpretation.
3. Bind those facets to the asserting source's existing semantic generation and
   payload. Preserve supplied-source semantics. Do not add another owner,
   registry, cache, lock order, retry policy or public response schema.
4. Migrate module owner and target admission together. Reuse established
   composition mechanics where equivalent, including established array/spread
   behavior. Do not repair direct arrays by silently discarding previously
   supported forms or keeping a spelling-based fallback for failed proof.
5. Old payloads missing required facets preserve only allowed legacy projection
   compatibility. They cannot prove migrated membership; reacquire through the
   normal producer path. Advance existing producer compatibility as needed.
6. Test exact target ownership, aliases, foreign/type-only/shadow refusal,
   dependency mutation and restore through the existing production boundary.
   Keep the observed three-test RED unchanged.

This proposal does not authorize persisting whole dependency source files or
constructing a complete all-program language inventory. If implementation needs
either, report that separately. It also does not grant completeness, reverse
closure or final acceptance from a successful point lookup.

## Alternatives and tradeoffs

| Option | Assessment |
| --- | --- |
| Retained demanded declaration facets through the existing Host/owner | Recommended. More acquisition and payload plumbing, but canonical proof survives publication and can be revalidated. |
| Depend on separately published dependency class-role entries | Could be valid with demonstrated complete, generation-current acquisition and canonical ownership joins. Current point-query history cannot establish that closure; this requires additional lifecycle work. |
| Read dependency files again or normalize decorator spellings | Rejected. It preserves independent authority or disguises alias/name matching as canonical proof. |
| Retain whole dependency sources/program captures | Broader retention/performance policy than needed for the demonstrated claim; not recommended or authorized here. |

## Approval boundary and current state

The repository Architectural Approval Gate requires an explicit decision for
changes to shared persistence/lifecycle or semantic authority boundaries.
Treating this new retained cross-file facet as requiring approval is Agent
MaxHeadRoom's application of that rule; the rule does not name this exact type.
The earlier contract approved transport of existing TypeScriptAuthority and
consumer migration, but did not specify this additional retained dependency
declaration contract.

No production repair was made while preparing this proposal. The three tests
and isolated registration were frozen in stash
`71d8d4cd99b4ba3decee665d909eb27044501bfc` with a temporary index, then restored
byte-identically. SHA-256:
`96354c9a0d2b1227bdd07b9f6c061b366c4c7e229ad59c0dd9e008cfddc19c94`.
Normal index and HEAD were untouched. No runtime command was run by the agent.
