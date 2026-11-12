# B8 exact production query execution traces

Agent: **Agent MaxHeadRoom**. Same execution spine for all seven requested operations.

1. MCP JSON-RPC tools/call reaches registered pcg_documents_v1 descriptor/handle in
   src/mcp/tool_handlers/pcg_documents.rs. Closed Request validates shape, workspace,
   mode, ID/track/text and explicit transport choice. protocol::send_pcg_result emits
   the exact serialized_result through the existing wire seam.
2. documents::Session::query loads session-pinned Trusted profile, rechecks bytes and
   permissions, canonical scope and finite node/edge limits; creates selected exact
   tokenizer. store::open checks schema/scope, uses WAL/FULL/foreign_keys and busy_timeout=0.
3. Begin IMMEDIATE transaction; scan::scan walks configured docs, reads distinct source/
   artifact bytes under file/byte/node/edge limits, parses declarations and hashes
   source/permission witnesses. validate::validate checks complete candidate kinds,
   IDs, endpoints/status consistency. store::missing prevents silent identity deletion.
4. store::publish hashes profile/revisions, verifies exact stored rows, or inserts
   immutable revisions/membership/edges; rebuilds document_search before head/publication.
   Revision SQL:SELECT path, source_sha, declaration, body, evidence, previous FROM
   document_revision WHERE revision=?. It verifies source hash and revision hash.
5. Search SQL:SELECT id FROM document_search WHERE document_search MATCH ? ORDER BY id.
   Bound parameter is literal PassPipeline, replay candidate, or generation78 in the
   requested FTS grammar. Then SELECT id, revision FROM document_member WHERE g=? ORDER
   BY id; Rust selects IDs, materializes full revisions, and applies optional track.
   No graph traversal for search. SQLite filters internally; Rust still loops current
   membership and materializes selected full text. No proposition-specific ranking.
6. Get(PERF-001-REPLAY) selects exact current member only, then full source/evidence.
   Related(IRPAT-001) SQL:SELECT target FROM document_edge WHERE g=?1 AND source=?2
   UNION SELECT source FROM document_edge WHERE g=?1 AND target=?2 ORDER BY 1.
   It materializes endpoint documents, not just edge IDs. Context(PERF-001-REPLAY)
   and context(DOC-157) use traversal::collect BFS:outgoing all kinds plus incoming
   SUPERSEDES, FALSIFIES, VALIDATES, RECLOSES, BANKS. SELECT source, kind, target, evidence
   FROM document_edge WHERE g=?1 AND (source=?2 OR (target=?2 AND kind IN (...)))
   ORDER BY kind, source, target. Rust visited/edge maps construct exact closure and
   one deterministic witness path; node/edge overflow returns no partial bundle.
7. Build version 1 full-source payload. Explicit context source_refs projection interns
   exact(path, hash, text)only for context; metadata/identities/edges/witnesses remain.
   Canonical inner JSON is escaped inside MCP content text and the complete MCP result
   is serialized. Old baseline binary token-overflow clears complete payload to
   blocked diagnostic; B8 source removes only that refusal, not other validation.
8. Capture::completion reobserves directory/source hashes/permissions and profile;
   valid transaction commits before exact bytes return. Invalid candidates retain
   prior historical graph, not stale current authority. Freshness is boundary-observed.

Actual selected/materialized row counts, source identities, formats, diagnostics and
serialized sizes are in B8_QUERY_RECEIPTS.csv and B8_PAYLOAD_COMPOSITION.csv. Refused
production results have zero returned documents, not zero matching stored nodes.
Read-only SQL-selected counts are separately labelled and do not bypass provider
limits. The rebuilt B8 executable completed all seven representative operations ready/current.
`B8_POST_REMOVAL_QUERY_RECEIPTS.csv` records their actual membership, source hashes,
typed relationships and witnesses, separately from the preserved baseline captures.
