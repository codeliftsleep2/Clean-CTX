# B11 owner disposition 01: bounded mechanism findings

Agent: **Agent MaxHeadRoom**. Proposals for owner disposition; not implemented.

Supersession: owner disposition 02 accepts the findings but rejects embedding
membership in operational profile configuration without demonstrated necessity.
The historical proposal below is retained; the current concrete proposal is
`B11_DISPOSITION_02_HANDOFF.md`, with separate review-only corpus/protected CSVs.
Exact protected-content comparison is now owner policy; bootstrap sets and the
authorization submission mechanism remain held. No proposed mechanism here is
controlling or implemented.

## Authority and current mechanisms

The accepted AUTH.PCG_DOCUMENTS component acquired in
`target/pcg-b11/authority-20261113-152531-723/acquisition/2.payload.json`,
acquisition index 3, pins revision
`4a91082a514377408cf126819c387263d1d76dd1a90139bbc6791f18d235779a`.
Its admission/history clauses block source removal, permit same-ID relocation,
and explicitly report revision authorization not established. The acquired GLS
authority at index 0 expressly supplies no pairwise transition/permission matrix.
The owner's B11 disposition 01 establishes new policy prospectively. The narrow
policy amendment is in the actual governing PCG_DOCUMENT_AUTHORING.md, outside
the preserved historical GLS source section. The amendment does not authenticate
itself or claim an implemented runtime check.

Production evidence: `profile.rs::Profile` has document roots and operational
limits, not governed-document membership or authorization entries. `scan.rs`
enumerates Markdown under those roots and skips files without declarations.
`store.rs::missing` compares accepted head identities and refuses disappearance.
`store.rs::reconcile` derives exact candidate revisions and records predecessor
links; it does not verify owner approval. `validate.rs` checks GLS candidate
consistency, not permission. The source authoring workflow is procedural guidance,
not a session receipt or authenticated authorization channel.

## Governed membership proposal

Reuse accepted identity membership to prevent escape for already-admitted nodes.
Do not label every file under document_roots governed: docs contains ordinary
documentation, historical reports and supporting material. No classifier is proposed.

Smallest extension proposed: explicit governed source membership in the existing
owner-controlled profile, as canonical repository-relative paths with the stable
identities expected from each source. Treat directory discovery as candidate
discovery only. Initial membership must be owner-reviewed; bootstrap must not
automatically equate every current declaration with authorized governance scope.
An explicit governed path with no required declaration rejects. Files outside
explicit membership may remain ordinary documentation; a declaration-bearing new
source requires a resolved membership disposition before accepted publication.

Retain membership obligations through the accepted head and exact identity set.
Relocation requires the replacement path and same identities in the owner's
membership update; no content similarity or filename heuristic establishes a move.
An explicit owner-resolved membership update must accompany removal/reclassification.
Existing profile pinning means changed membership takes effect in a fresh session.
Compatibility decision required: distinguish membership changes from physical
document-root changes so a legitimate membership update can retain the same DB and
history rather than demand a new database. Never silently rewrite prior scope.

This is a proposal, not authorization to make profile membership authoritative.
Owner must decide representation, bootstrap set, behavior of newly declared but
unlisted sources, membership-update trust and legitimate relocation procedure.
Current deletion refusal remains operative in the meantime. New declaration-free
governed admission cannot be claimed mechanically enforced yet.

Positive witnesses: listed new document with correct identities; ordinary unlisted
Markdown; owner-resolved same-ID move; permitted membership reclassification.
Negative witnesses: listed source without declaration; stripped known source;
unlisted attempted new controlling identity; renamed source without resolved
membership update; candidate editing its own membership; foreign-root path.

## Revision authorization proposal and trust gap

Use the accepted immutable predecessor and existing candidate revision digest.
A bounded authorization binding would need: workspace/catalog scope identity,
affected controlling identities, accepted predecessor revision for each identity,
and the exact proposed revision/change digest including declared components,
associated evidence and typed relationships. No title, branch, commit or free-form
approval field substitutes for that tuple. It must reject substitutions and reuse
against another predecessor. Preserve the authorization's historical applicability;
no general event ledger or signing system is proposed.

Candidate source, candidate graph and the publication transaction cannot supply
their own approval. A separate owner-controlled input to the existing trusted
profile is the smallest plausible local boundary, but its owner-authenticity is
NOT established merely by using another file. Agents and owner may share the same
OS account and filesystem permissions. The existing profile pin and permission
checks establish immutability during a session, not who approved its contents.
Therefore an agent-editable JSON authorization list cannot implement this policy
honestly without an explicit owner-approved trust and submission contract.

An owner-only operator submission step through existing host infrastructure could
provide the missing input, but no such authenticated mechanism was found in the
inspected document entry points. Designing an approval service, capability system,
cryptographic identity or M4b ledger is outside B11. Hold this mechanism for owner
disposition; report revision authorization unresolved rather than claiming success.

Identifying semantic authority changes is a separate limitation. Kind AUTHORITY
alone is insufficient: invariants and mandatory constraints/obligations may be
components of other kinds. Hash changes are not semantic proof. A bounded proposal
is an owner-established set of controlling identities/components; changes affecting
that set remain unresolved until exact authorization or owner-established evidence
of an ordinary non-authority revision exists. The initial set, treatment of source
relocation and qualification changes, and nonsemantic-change determination require
owner disposition. No keyword or LLM inference gate is proposed.

Positive witnesses: exact predecessor/candidate owner authorization; ordinary
evidence/proposal revision outside controlling scope; previously authorized source
move with explicit membership resolution. Negative witnesses: candidate approval
text; same approval against another digest; changed predecessor; another workspace;
removed mandatory qualification; policy weakening its own authorization requirement.
Tests must disclose unresolved authorization when trusted input is absent, rather
than treating structural admission as permission or indiscriminately blocking all
ordinary source revisions.

## Next decision and independent work

Owner disposition is required before either proposed mechanism is implemented.
No new profile fields, membership tables, admission restrictions or authorization
state were added. Existing behavior is not represented as enforcing the new policy.
The diagnostic-reference correction and existing-rule rejection/rollback/currentness
verification continue independently. The next tracked authority-diagnostic RED
capture is still pending; do not skip RED because policy design is held.
