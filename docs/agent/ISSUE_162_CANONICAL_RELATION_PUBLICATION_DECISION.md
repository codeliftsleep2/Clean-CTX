# Issue #162: canonical relation publication identity

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

The owner explicitly approved this bounded carrier and lifecycle extension.
Global EntityRef equality, legacy edge-key meaning, public JSON and query
selectors remain frozen. The proposal below is historical decision context.

Implementation uses optional internal CanonicalEndpoint annotations beside
EntityRef presentation. Each annotation retains all native declaration identity
fields losslessly; the subject also retains the asserting decorator site. The
neutral carrier imports no IR module. EntityRef equality/hash remain unchanged,
and annotations are skipped by public Serialize. DurableEntityRef explicitly
stores the annotations with default absence for legacy records. No endpoint
identity is recovered from a projected filename or name.

Native module projection constructs the annotations from demanded DeclarationId
values. Artifact/projection/session propagation retains the existing cloned
SemanticEdge values. Durable snapshot conversion stores/restores annotations;
compatibility, session currentness, pending transition and index admission
validate them against the retained native facets/package declarations. Angular
semantic generation advances from 31 to 32. Legacy facts with no annotations
keep their existing key behavior and cannot supply canonical proof.

EdgeKey includes the optional annotations in construction and exact matching.
The existing forward/reverse adjacency retains whole edges; selected_occurrences
returns each admitted occurrence without a second name-based deduplication.
remove_file uses the same EdgeKey matching for both adjacency directions, and
source-generation replacement republishes surviving facts under the same owner.
No second relation index, ownership mechanism or lock order was introduced.

The regression with corrected Windows presentation assertions was frozen in
isolated stash `bdde4dae6a773edec14182c1054b858a6e80fbc4` and restored byte-exactly.
The original observed test/log bytes remain in the earlier RED receipt directory.
Supplemental tracked coverage now checks two pre-publication facts, duplicate
publication, source retirement/replacement, real SQLite endpoint round-trip,
public JSON shape, restart, role mutation and endpoint deletion. These new
assertions are authored and UNRUN. Neither passing behavior nor completeness is
claimed. No Cargo/runtime command was started by the agent.

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

Authorize a bounded extension for migrated canonical TypeScript/Angular relation
facts: retain typed asserting declaration/site and canonical endpoint identities
through the existing semantic result, compatibility, durable snapshot and index
publication lifecycle. Include that evidence in occurrence deduplication,
selection and retirement for these facts. Missing canonical evidence remains
legacy projection evidence and cannot be promoted into canonical proof.

Use existing source ownership, generations, witnesses and compatibility machinery.
Advance producer compatibility if serialized meaning changes. Retain all prior
facts for non-migrated producers under their established keys. Do not change
global EntityRef equality, public response schemas, name-based query selectors,
or add an independent registry/cache/publication lifecycle. Angular interprets
the relation; TypeScript declaration IDs supply endpoint identity.

The exact internal carrier should be chosen after tracing every serialization,
replacement, selection and retirement use. Full snapshots of dependency sources
and whole-program retention remain prohibited. An unavoidable expansion beyond
this scope must be reported before implementation.

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
