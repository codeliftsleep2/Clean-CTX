# B9 schema migration

Agent: **Agent MaxHeadRoom**. Schema implementation: `src/pcg/documents/schema.rs`;
frozen accepted input: `schema_v1.sql`; target: `schema.sql`. This report explains
the change; `PCG_DOCUMENT_AUTHORING.md` remains the controlling contract.

Only an exact recognized v1 or v2 sqlite_master and matching owner scope can open.
The caller begins its existing IMMEDIATE transaction. v1 is copied transactionally
into v2 with deferred FK checks (FK enforcement is never disabled). Exact artifact
`(path,sha,body)` rows replace repeated revision-body ownership. Revisions keep the
same IDs, declaration/evidence JSON, source hashes and predecessor chains. Members,
generations, publications, edges/provenance and legacy FTS rows are copied exactly.
Every historical revision is rehashed and foreign keys checked before continuation.
Only after all copies/checks do old tables disappear within that transaction.
Failure/termination leaves the original committed catalog. Invalid current candidates
roll back migration as well. Existing session completion rechecks sources/profile
before commit. No schema downgrade or old-binary compatibility is fabricated.

New `knowledge_unit` rows own scope and indexed metadata for an immutable document
revision. `knowledge_component` rows own ordinal, role/label, exact artifact FK,
start/end, content hash and source selector. Per-component and composed FTS rows
carry explicit match origins; current membership joins exclude historical revisions.
Historical original source and evidence remain accessible through the legacy history
contract. Old revisions are not silently reinterpreted as proposition records.

No arbitrary SQL, external schema service, new profile, persistent reference cache,
token ceiling or separately maintained knowledge manifest is introduced. The actual
installed profile remains byte-identical to B8. Old binaries explicitly refuse v2.
Deploy the new binary before using the migrated catalog; preserve the original B8
binary and captures for evidence, not for writing back to a migrated database.

Tracked migration tests cover exact history/source/edge preservation, deduplicated
source storage, invalid-candidate rollback and corrupt-historical-body refusal.
Actual-root checks separately compare all 218 identities, 221 relationships, original
body hashes, evidence associations and predecessor reachability against B8.
