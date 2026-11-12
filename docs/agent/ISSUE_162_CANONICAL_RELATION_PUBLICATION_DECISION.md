# Issue #162: canonical relation publication identity

B4b owner navigation: [AUTH.ANGULAR_CANONICAL_RELATIONS](../architecture/contracts/angular-canonical-relations.md) owns the relocated bounded semantic clauses. Original stage approvals, failures, UNRUN checkpoints and execution counts below remain historical; they do not certify today's executable or close the wider campaign.

Agent: Agent MaxHeadRoom
Status: owner APPROVED; focused and consolidated regressions GREEN; three-session binary/stdio PASS; final field/completeness verification pending.

## Focused execution receipt

Operator logs record module-relations-repair 2 passed, canonical-relations-
deletion-repair 1 passed, and output-barriers-preservation 3 passed. The full
supplemental lifecycle test now reaches its end successfully, including endpoint
deletion preserving the unrelated fact. This required distinguishing proved
relative-module absence from hard acquisition errors while retaining negative
request/witness evidence. Earlier authored/UNRUN statements below describe the
pre-execution checkpoint. Consolidated preservation and binary/live boundaries
remain unverified for the latest changes.

## Owner approval and implementation checkpoint

Current normative owner: [AUTH.ANGULAR_CANONICAL_RELATIONS](../architecture/contracts/angular-canonical-relations.md). Original clause bytes and chronological context remain available at commit `ec7067fd`.

## Demonstrated requirement and RED

AQ02/AQ05 require module declaration/export membership; AQ25 requires exact
canonical joins. Two Directive declarations named Leaf in left.ts and right.ts
are different required endpoints, even when one module references both through
aliases and a shared array. The existing module metadata consumer and registered
workspace_query forward_edges path must preserve both facts.

The operator's module-relations.log records one returned DeclaresInModule edge
where the regression requires two. Later export and mutation assertions have
not executed. The other test stopped at a Windows extended-path formatting
comparison; that assertion now uses existing repository path normalization.
It is not evidence of a production import failure.

Original test and registration bytes, SHA-256 receipt and log hash are preserved
under target/issue-162/module-relations-red/. No assertion about the required
number of endpoints was weakened. No production repair was made.

## Located shared boundary

The canonical module projector collects DeclarationId values in a BTreeSet and
projects endpoint source files onto SemanticEdge objects. Distinct source IDs
remain distinct at this construction boundary by static inspection.

WorkspaceIndex::add_edges constructs EdgeKey in workspace/index/edges.rs.
That key includes the asserting file, relation, subject/object domain/type/name
and optional call argument evidence. Endpoint declaration files are omitted.
The two observed memberships therefore have equal index keys. EdgeKey matching,
retirement and selection depend on this shared representation. This is a static
causal finding combined with a public-query RED, not a runtime trace proving
that no earlier projection loss also exists.

## Decision requested

Current normative owner: [AUTH.ANGULAR_CANONICAL_RELATIONS](../architecture/contracts/angular-canonical-relations.md). Original clause bytes and chronological context remain available at commit `ec7067fd`.

## Alternatives

| Option | Tradeoff |
| --- | --- |
| Typed canonical relation evidence within existing lifecycle | Recommended. Preserves required endpoints and proof with bounded migration, but requires transport and retirement plumbing. |
| Add endpoint file paths to every existing EdgeKey globally | Smaller edit, but changes unrelated producers and treats presentation paths as complete declaration identity. Not recommended. |
| Change global EntityRef equality or encode paths into names | Violates the accepted identity boundary or changes public names. Rejected. |
| Read retained payload in a separate query-only relation overlay | Creates competing publication/retirement behavior. Rejected. |

## Required verification

Preserve the observed same-name RED before repair under the normal isolated
test/registration stash workflow. Prove both canonical facts exist before index
publication, both survive the real public forward query and durable restart,
and mutation/deletion of one endpoint retracts only its facts. Preserve existing
non-TypeScript keys, call evidence, duplicate extraction, replacement and removal
behavior. Run the affected tracked tests and consolidated preservation before
batched binary/stdio and final live acceptance. No completeness or phase closure
follows from this point regression.

## Why approval is needed

AGENTS.md's Architectural Approval Gate requires explicit approval for changes
to shared indexing/persistence, semantic identity boundaries and behavior of
unrelated queries. The approved retained-declaration contract explicitly retains
existing ownership and does not authorize a global identity redesign. Applying
the gate here requires an owner decision on the bounded canonical relation
carrier and lifecycle extension; it does not require approval to preserve RED
evidence or correct the Windows assertion.
