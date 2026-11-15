# B11 disposition 02: exact bootstrap and remaining decisions

Agent: **Agent MaxHeadRoom**. Proposals only: no registry, protected set or
authorization submission mechanism is controlling or implemented.

## Membership representation and reconciliation

Proposed eventual location: `docs/architecture/knowledge-governance/corpus.csv`.
The exact proposed contents are `B11_PROPOSED_GOVERNED_CORPUS.csv`: 153 rows,
one canonical repository-relative source_path and semicolon-delimited expected_ids
per row, covering 218 distinct accepted identities. It duplicates no title,
status, components, relationships, evidence or source bytes. CSV follows the
repository's existing evidence-inventory convention; no sufficient existing
governed-membership registry was found in the inspected document implementation.

The accepted input is the successful B9-R1 production refresh:
`target/pcg-b9-r1/delivery-refresh-20261113-134605-318/production/graph/`.
It pins generation
`684f1ca847aaec8398506ca3e8d0ebd3bf8f5bbaacb26ba88ede062fb17c6d70`.
`B11_BOOTSTRAP_ACCEPTED_OWNERSHIP.csv` records each accepted ID, path and revision.
All source paths are inside the existing `docs` discovery root. This is accepted
publication evidence, not new owner authorization. The current B11 policy amendment
changes the shared source of AUTH.PCG_DOCUMENTS/AUTH.PCG_GLS; those accepted pins
are intentionally historical until actual reconciliation is observed. No other
accepted source artifact hash drift was found in the bounded static comparison.

`B11_BOOTSTRAP_UNREGISTERED_SOURCES.csv` accounts for 322 other current Markdown
sources under docs. They remain outside this proposed accepted-source bootstrap;
the inventory makes no semantic assertion that they should all remain ungoverned.
It preserves historical/supporting/research/ordinary files rather than forcing
declarations on them. A declaration-free unregistered document's intended governance
cannot be determined mechanically; owner/author registration remains necessary.

Proposed validation: registered paths must yield their exact expected stable ID
sets; declaration-bearing unregistered sources reject with a registration diagnostic;
ordinary unregistered Markdown is not graph admission. Compare candidate membership
against accepted membership before considering candidate removals or moves.
Same-ID relocation requires an explicit authorized registry change and preserves
revision/history paths. Registry removal cannot erase accepted identities through
a new candidate list. Bootstrap and update trust must be accepted first.

## Protected set proposal

Proposed eventual location: `docs/architecture/knowledge-governance/protected.csv`.
`B11_PROPOSED_PROTECTED_COMPONENTS.csv` records the exact proposed 221 existing
components belonging to 137 identities with accepted revisions, selectors, roles,
labels and hashes. Those baseline fields are review evidence; the eventual protected
set needs only stable identity/component selectors plus explicitly approved
association-protection policy, not duplicated lifecycle or content.

Selection basis is source-declared authority scope and invariant scope in the
normative invariant owners indexed by `docs/ARCHITECTURAL_INVARIANTS.md`, plus
PERF-005-BOUNDED, whose `performance-doctrine.md` standing explicitly identifies
the mandatory AF-PERF-001 implementation constraint and verification obligation.
It is not based on word matching or kind AUTHORITY alone: 73 invariant identities
are included alongside 64 authority identities. Research/evidence/proposal units
are not indiscriminately protected because they mention an authority.

This is a conservative review candidate, not a completed determination that all
221 components are controlling. Some accepted selectors are `whole`, and some
authority artifacts retain historical context. Owner review must narrow or reject
any overly broad component before activation; the runtime may not secretly protect
unrelated artifact content. If a whole selector mixes controlling and unrelated
prose, existing source-owned exact markers can identify a narrower authorized
boundary. Do not infer new sub-boundaries or silently redesign B9 representation.

## Exact change detection and descriptor

Proposed protected-state comparison uses exact selected UTF-8 content hashes,
stable identities, roles/labels, and required qualification content/membership.
Include declared typed relationships affecting the protected identity and applicable
evidence associations according to their source-owned contract. Removing a required
qualification or changing a governing endpoint is a change even if the statement
bytes are unchanged. Compare protection-set and corpus-registry changes against
their previously accepted states, never only the candidate versions.

Do not classify semantic equivalence. A byte change in selected protected content,
including punctuation, requires authorization. A source hash or interval-offset
change outside the selected content is not by itself a protected content change.
Ownership/selector relocation and association changes remain separately represented
and authorized as applicable. Historical revision records retain complete original
source bytes and source hashes even when the protection digest is unchanged.

Proposed Rust-computed descriptor, not a new approval ledger: version; canonical
workspace/catalog identity; accepted membership/protection state digests; affected
identity/component selectors; exact accepted predecessor revisions; exact proposed
selected-content/qualification hashes; and applicable sorted typed relationship,
evidence, membership and protection deltas. Hash the existing deterministic canonical
serialization of that descriptor. Authorization binds the exact descriptor digest
and predecessor tuple, not an unrestricted source path or prospective future change.
Do not include an authorization field inside its own digest. Replay after acceptance
is idempotent only for that already-accepted state, not a grant for a later revision.

## Owner-operated submission alternatives

Inspected `scripts/agent/state.mjs::authorPermission/requireAuthorized` checks
GitHub collaborator permissions through gh api; `agent.mjs::cmdApprove` approves an
issue plan then starts implementation. This is an authenticated collaborator event
boundary, not an owner-only protected-revision authorization contract. It binds no
exact predecessor/candidate descriptor and cannot currently authorize B11 revisions.

1. Extend that existing hosted boundary for an explicit owner-established actor
   allowlist and exact descriptor approval. The host must validate actor identity,
   descriptor and predecessor against trusted accepted state, and deliver a bounded
   approval result to reconciliation. This can distinguish an authenticated human
   event from candidate text, provided the agent lacks credentials capable of
   impersonating the owner and cannot replace trusted workflow inputs. It requires
   new host integration and an approved trust contract; none is implemented.
2. Local owner review plus a manually submitted exact descriptor approval using
   existing trusted local configuration. This is the smallest operational workflow,
   but only procedural separation if agent and owner share the same account/credentials.
   A CLI or separate file cannot authenticate who invoked/wrote it. It must never
   be reported as authenticated authorization. Acceptable only if the owner explicitly
   accepts that trust guarantee for normal cooperative development.

No signing infrastructure, enterprise identity, approval service or M4b is proposed.
Neither option protects against a malicious actor with unrestricted code/OS/DB access.
Choice of boundary and its exact submission contract remain held for owner acceptance.

## Compatibility, lifecycle and verification

Do not add membership to operational limit configuration. Reuse workspace identity,
existing discovery roots and profile pinning, while reading the separately accepted
registry as governed source input and observing it at both existing boundaries.
Registry changes should advance publication generation under the same database;
no physical scope change or new DB is implied. Accepted registry/protection snapshots
must persist alongside publication history so rollback and self-modification checks
use accepted state. This is a required compatibility extension to design/approve,
not a schema already implemented. Existing schema migrations must remain atomic.
Historical acquisition must continue resolving old source revisions and qualifications.

Positive witnesses: ordinary unregistered Markdown; listed new source; exact same-ID
authorized relocation; unchanged protected components amid ordinary outside edits;
exact approved content/association change; corrected candidate after rejection.
Negative witnesses: absent expected declaration; foreign/duplicate registry ownership;
unregistered declared source; candidate simultaneous registry/protection removal;
unauthorized punctuation/qualification/relationship/evidence change; approval reuse
against another predecessor, workspace or digest; source drift at completion.
Every rejection must preserve accepted graph/history and disclose currentness honestly.

## Independent B11 work and requested disposition

Initial diagnostic RED is preserved and GREEN passed two unchanged tests and all
41 document tests. Exact governing-reference tests are prepared but their operator
RED receipt is still pending (`invoke-admission-red.ps1 -Authority`). No corresponding
governing-reference production fix has been made yet. That correction continues
independently of held registry/authorization design.

Owner decisions requested: exact membership bootstrap; protected-set selection and
any narrower source-owned boundaries; accepted registry/protection snapshot lifecycle;
association-change coverage; and hosted-authenticated versus explicitly procedural
authorization submission. Until those decisions, do not activate the proposed CSVs,
claim either new policy enforced, or let structural admission imply authorization.
