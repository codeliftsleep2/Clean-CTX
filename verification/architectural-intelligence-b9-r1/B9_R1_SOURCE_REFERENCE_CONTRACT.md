# Source-reference implementation record

Agent: **Agent MaxHeadRoom**. Governing definitions remain in document authoring.

Component references pin identity, immutable revision, ordinal, exact role/label,
source path/hash, selector, UTF-8 byte interval and content hash. Returned
acquire_request fields include workspaceRoot and mode=acquire plus a references
array. A single-reference array is individual acquisition; a selected multiple
array is explicit batch acquisition. get_request is structural identity access.
source_request remains an explicit full artifact request, now also revision-pinned.

References are data, not transient handles or a session-local cache. Default
current references refuse if the member revision advances. historical:true must
be explicit and resolves only exact retained associated revisions, with original
qualifications and historical freshness. It never silently adopts current bytes
or certifies current authority. Historical whole-source requires id/revision and
uses retained artifact/evidence bytes; unavailable original bytes explicitly fail.

Each batch input receives a disposition. Exact duplicates retain input positions
and duplicate_of; invalid exact references return no content and make overall
acquisition incomplete. Unknown nested fields/malformed shapes refuse the request.
Security/database corruption remains an operation error. Successful peers are not
described as a complete graph or complete authority acquisition. Required
qualification references appear in structural descriptors; acquisition always
includes every authored qualification of the selected unit/revision.

No historical record, source body or selector is deleted. No guessed line ranges,
manual SQL joins, generated summaries, automatic partitioning or receipt tracking.
