# B9 query contracts and deterministic responsibilities

Agent: **Agent MaxHeadRoom**. Summary of the controlling source contract at
`docs/architecture/PCG_DOCUMENT_AUTHORING.md`, not a second normative authority.

| Mode | SQLite/Rust execution | Result responsibility |
|---|---|---|
| search | Current-member join to knowledge_unit and component/composed FTS; SQL metadata and role/label predicates; stable identity ordering | Metadata and exact match-origin interval/hash references, acquisition requests; no source bodies or closure claim |
| get | Exact current identity/revision; ordered component interval reads with content-hash validation | Exact source-owned content, required qualifications, evidence references and outgoing declared typed edges; filtered request states unit_complete=false |
| related | SQL incident-edge predicate by kind/direction and endpoint identity lookup | Actual directed edges/provenance/meaning and endpoint metadata; no endpoint source materialization |
| context | Existing Rust BFS over SQL adjacency with unchanged traversal kinds and finite structural limits | All selected exact units, required evidence associations, typed edges and one deterministic witness per node; no authoritative partial result |
| source | Identity-scoped association check, observed original bytes and optional pinned hash check | Exact complete UTF-8 or lossless binary original artifact; changed pin explicitly refuses |

FTS indexes only admitted knowledge components and explicit identity/title fields.
Composed matching may combine source-declared components; it does not assert a
contiguous single-artifact quotation. Per-role searches match only those roles.
Title/identity, direct statement, rationale, evidence and other component origins
remain distinguishable. No word match establishes engineering applicability.

Source observations, permissions, profile scope and completion are rechecked on
every production query. Source-bound revisions legitimately advance across shared
source edits while stable node identities persist. No global cache policy changes.
Missing representation, invalid boundaries, structural overflow, security/database
errors and source drift remain explicit. Complete means complete for the named
operation; all semantically relevant undeclared relations cannot be inferred.
