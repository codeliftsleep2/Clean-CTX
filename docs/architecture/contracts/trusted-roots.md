# Trusted workspace roots

Agent: Agent MaxHeadRoom. Current contract extracted in B2 from the sources below. Original chronology and implementation evidence remain in the linked source records. IMPLEMENTED records the operative source contract, not new validation.

### Source and workspace boundaries

| Boundary | Contract |
|----------|----------|
| `workspaceRoot` | Establishes the primary trusted filesystem root for resolution, hydration, and file-local compilation. Pass it explicitly. |
| `additional_roots` | Expands the authorized root set only through configuration; a request cannot invent a new trusted root. |
| `withinPath` | Narrows an already authorized workspace. It never becomes a root and is rejected when outside the authorized root set or when `workspaceRoot` is absent. |
| Hydration | Name-bearing workspace queries may discover and compile candidates only inside authorized roots; `has_cycle` is index-only and never hydrates. |
| `entities_in_file` | Compiles the explicit trusted file when needed and replaces stale semantic facts, including replacing them with an empty projection. |
| Returned paths | Paths returned by Clean-CTX are authoritative; callers must not derive filesystem paths from namespaces or CBM project slugs. |

---


Source: [docs/SECURITY.md](../../SECURITY.md); original section SHA-256 `41dfd57c11dc923f72b335c0c76fa2b5c8beaee8d3aa65cb439e67b0ed6a3931`. Original wording and tense are retained; admission is limited to the implemented scope described here.


```architecture-graph
node: AUTH.TRUSTED_ROOTS
kind: AUTHORITY
status: IMPLEMENTED
title: Trusted workspace roots
track: REPOSITORY
artifact: docs/architecture/contracts/trusted-roots.md
preserves:
  - WSC-004
  - INV.CBM_ID_001
note: Source-described operative contract; migration does not execute or independently certify its witnesses.
```
