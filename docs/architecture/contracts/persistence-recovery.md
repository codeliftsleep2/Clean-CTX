# Persistence ownership and recovery

Agent: Agent MaxHeadRoom. Current contract extracted in B2 from the sources below. Original chronology and implementation evidence remain in the linked source records. IMPLEMENTED records the operative source contract, not new validation.

### Design Decisions

- **Load is not authority**: durable rows may decode successfully yet remain
  ineligible for live adoption. Current source/fidelity plus separate canonical
  configuration, canonical producer, semantic configuration, and semantic
  producer identities must validate before their respective live publication.
- **Trusted restore admission**: `workspaceRoot` plus configured additional
  roots define the restore admission boundary. Admission precedes recovery,
  durable owner registration/loading, and every live or cached side effect.
- **Coherent checkpoint capture**: canonical IR, source identity, fidelity,
  semantic projection, and compatibility identities are captured from one
  owner authority epoch under that owner's guard; persistence I/O follows
  after capture and does not impose a global authority lock.
- **Owner-scoped ordering**: authority-changing operations for one semantic
  owner share one commit domain, so an older in-flight operation cannot win by
  finishing later. Different owners remain independent. The ordering tracks
  operation generation, not the historical age of a selected snapshot.
- **Physical source preconditions**: an edit re-proves its exact source-byte
  precondition at the owner commit boundary before intent staging or atomic
  replacement; atomic replacement alone is not compare-and-swap.
- **Durable publication boundary**: persistence-enabled semantic operations commit the requested file's canonical IR and complete edge snapshot before publishing live state; failure is structural and mutation-free.
- **File-scoped authority**: reads never flush pending work, and one file's save, restore, replay, delta, edit, or deletion cannot commit another file's lifecycle state.
- **Shared edit recovery**: pending byte-exact edit intents resolve before any durable semantic load, compilation, checkpoint, or index hydration becomes authoritative.
- **Content-hash deterministic IDs**: `ctx-{sha256_hex}` ensures idempotent saves (same content → same ID → UPSERT).
- **Thread-safe state**: `McpState` is wrapped in `RwLock` for the A-09 multi-threaded dispatcher — parallel reads, serial writes.
- **Lazy initialization**: DB only opens when persistence is enabled — zero overhead for users who don't need persistence.
- **Physical `0x04` authority**: canonical identity, version, operations, order, duplicates, bodies, and spans round-trip in the BLOB itself; DB metadata must agree rather than repair it.


Source: [docs/ARCHITECTURE_OVERVIEW.md](../../ARCHITECTURE_OVERVIEW.md); original section SHA-256 `bd8f8919daa2bda8cc6f5bd52ce42ca297fa35202b30dc65ebfeca52c68b67b6`. Original wording and tense are retained; admission is limited to the implemented scope described here.


The thread-safe-state and lazy-initialization bullets are retained descriptive implementation notes, not a global locking contract. The scoped invariant owners govern adoption and ordering.

```architecture-graph
node: AUTH.PERSISTENCE_RECOVERY
kind: AUTHORITY
status: IMPLEMENTED
title: Persistence ownership and recovery
track: REPOSITORY
artifact: docs/architecture/contracts/persistence-recovery.md
preserves:
  - PERSIST-001
  - PERSIST-002
  - PUB-001
  - AUTH.BINARY_V04
note: Source-described operative contract; migration does not execute or independently certify its witnesses.
knowledge_scope: authority
knowledge_content:
  - statement self#whole
```
