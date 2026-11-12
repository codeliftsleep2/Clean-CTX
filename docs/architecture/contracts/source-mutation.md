# Exact source mutation authority

Agent: Agent MaxHeadRoom. Current contract extracted in B2 from the sources below. Original chronology and implementation evidence remain in the linked source records. IMPLEMENTED records the operative source contract, not new validation.

## Mutation authority

apply_edit requires explicit file authority, current tracked source, a selector
that identifies one declaration, a current-source precondition, and compatible
canonical/semantic authority where required. Strong selectors address
declaration-local signature identity; legacy/name-only selectors succeed only
when unique. Ambiguity fails closed. Physical source is revalidated immediately
before commit. Atomic replacement is not authority to overwrite external edits.


Source: [docs/architecture/SEMANTIC_AUTHORITY_ARCHITECTURE.md](../SEMANTIC_AUTHORITY_ARCHITECTURE.md); original section SHA-256 `a0e7afa6dc9bd960459c188eb5ed463ce0775ce39303cf83294c5b3ff41ae133`. Original wording and tense are retained; admission is limited to the implemented scope described here.


```architecture-graph
node: AUTH.SOURCE_MUTATION
kind: AUTHORITY
status: IMPLEMENTED
title: Exact source mutation authority
track: REPOSITORY
artifact: docs/architecture/contracts/source-mutation.md
preserves:
  - EDIT-001
  - WSC-012
  - PUB-001
note: Source-described operative contract; migration does not execute or independently certify its witnesses.
knowledge_scope: authority
knowledge_content:
  - statement self#whole
```
