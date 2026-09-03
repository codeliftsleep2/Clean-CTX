# Agent Documentation (`docs/agent/`)

The `docs/agent/` directory holds **conditional, procedural** engineering
knowledge that is consulted when the corresponding task type applies. These
files are NOT auto-loaded rules.

## The load model

`.clinerules/` is the live agent-instruction surface:

- `engineering.md` — concise, always-loaded operating policy plus routing.
- `encoding.md` — the authoritative encoding & Unicode rule (always-loaded).

Cline automatically loads every `.md` file under `.clinerules/` for every task.
To keep always-loaded context small, `.clinerules/engineering.md` holds only
universal policy and routes here for detail. It is intentional that
`.clinerules/` is gitignored while this directory (and the rest of `docs/`)
are tracked and versioned.

`docs/ARCHITECTURAL_INVARIANTS.md` remains authoritative for durable
architectural facts and is NOT duplicated here.

## Why this separation exists

- Always-loaded context is paid for on every task, so it must stay small and
  high-signal.
- Procedural knowledge (verification commands, migration checklists, release
  accounting, tooling) matters only for specific task types, so it is loaded on
  demand by routing rather than injected into unrelated work.
- Mechanical requirements (the encoding guard) are already enforced by git hooks
  and CI, so they are referenced, not re-explained at length here.

## File index

| File | Purpose | Read it when ... |
|-----------|------------------------------------------------------|------------------------------|
| `verification.md` | Single authoritative final verification gate | declaring any task complete |
| `architecture.md` | Architectural audit checklist, invariant hierarchy, test-file convention | ending a multi-step architectural task |
| `incremental-migration.md` | Incremental architectural migration procedure | performing a designated migration |
| `releases.md` | Gated release, changelog & versioning accounting | a behavior-affecting build ships |
| `tooling.md` | Comprehensive MCP/code-context tool selection, workflow, and antipatterns guide | choosing how to read, understand, edit, or verify code |
| `DISCOVERY_REGISTRY.md` | Live-discovery ledger: real-workspace findings → root cause → local regression / live scenario | recording or closing a real-world (Claude) discovery |