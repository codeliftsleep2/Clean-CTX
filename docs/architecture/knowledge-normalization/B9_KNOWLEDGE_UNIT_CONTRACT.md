# B9 knowledge-unit implementation map

Agent: **Agent MaxHeadRoom**. Governing source: `PCG_DOCUMENT_AUTHORING.md`.

`content.rs::spec` parses existing flat declaration extensions; `Resolver::resolve`
extracts named paired-marker intervals or explicit whole-source content. Marker
examples in fences are inert. Each distinct source's detection/index/hash is reused
operation-locally, including cached invalid-marker results. Shared exact interval
hashing is reused per (source,start,end); no persistent cache or token metric is used.
Capture nodes retain shared Arc source ownership, not sibling source copies.

Scope describes actual ownership: bounded invariant/proposition/decision/finding,
complete authority/contract/evidence, or multi-section content. It is not a new GLS
kind. Roles/labels annotate exact source content, never generated statements.
All units have a statement; optional semantics exist only where authored evidence
supports them. Ordered components preserve exact UTF-8 bytes, including Unicode
normalization distinctions and newline bytes. External content enters the same
source/evidence/currentness and immutable revision identity.

An exact prose section is capability A. Queryable roles/labels and typed graph
predicates are the implemented part of capability B. They do not provide an ontology
for all engineering concepts or a natural-language applicability solver. Lifecycle
status is distinct from decision outcome and executed verification. Structured
historical observations retain their qualifications and original reports.

All 218 source-owned migration dispositions are recorded separately. Complete
historical reports legitimately remain whole; shared invariant families do not.
No fixed chunk size, response-size target, token cap or duplicate normative prose.
