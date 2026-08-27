# Architecture (`docs/agent/architecture.md`)

This document holds the detailed architectural audit material, the
invariant-enforcement hierarchy, and the test-file convention. It is NOT
always-loaded; consult it when ending a multi-step architectural task.

## Authoritative reference

Do not duplicate the durable architectural facts. `docs/ARCHITECTURAL_INVARIANTS.md`
is authoritative for:

- the invariant catalog (e.g. `PIPELINE-001` compilation-pipeline ordering),
- the resolved/migration debt record (e.g. `ARCH-DEBT-001` — PassPipeline
  migration, RESOLVED),
- the classification model (ENFORCED / STRUCTURAL / DOCUMENTED / DEFERRED /
  PROPOSED / RESOLVED).

That document also defines "How to Add a New Invariant" and states that no
separate fitness-function framework, registry, or gate abstraction exists. The
architectural gate is the existing CI pipeline: `cargo test` + `cargo clippy
--all-targets -- -D warnings`.

## Invariant-enforcement hierarchy

When introducing or formalizing an invariant:

1. **First:** determine whether the Rust type system can enforce it. If yes,
   prefer structural enforcement.
2. **Second:** determine whether an existing test already enforces it. If yes,
   document the invariant and identify the test authority.
3. **Third:** determine whether an existing compiler or Clippy mechanism can
   enforce it.
4. **Only then:** add a dedicated architectural test if necessary.

Do not create a generic fitness-function framework, invariant registry,
architectural-gate framework, dependency analyzer, or similar abstraction
merely to formalize architectural rules unless explicitly approved. Prefer the
simplest enforcement mechanism that provides reliable protection.

## Test discipline

- Tests are architectural assets, not merely regression checks. When a bug or
  architectural failure is discovered, add a regression test when appropriate.
- Prefer tests that capture the invariant or behavior that must remain true.
- Add integration tests when behavior crosses architectural boundaries; add
  end-to-end tests when the behavior cannot be meaningfully verified at a lower
  level.
- Do not remove existing regression coverage merely because the implementation
  has moved. When relocating functionality, preserve its existing test
  coverage.

## Test-file convention

Clean-CTX uses Rust's `#[path = "..."]` attribute to keep test files in the
dedicated `src/tests/` directory rather than inline in source files. Each
source module declares its tests with a pattern such as:

```rust
#[cfg(test)]
#[path = "../tests/mcp/heuristics.rs"]
mod tests;
```

- Test files live in `src/tests/`, mirroring the source module structure.
- Source files remain focused on production logic.
- Tests are compiled only in `#[cfg(test)]` builds.
- The `#[path]` reference must match the actual file location; broken paths
  cause compilation failures.
- Do not move tests inline into production source files unless explicitly
  requested.

## Post-Task FAANG-Level Architectural Audit

Before declaring any multi-step architectural task complete, perform a
comprehensive architectural audit. For incremental migrations, this audit
occurs **during finalization**, not after every individual migration phase.

### Improper implementations

- Logic errors
- Incorrect architectural patterns
- Incomplete feature coverage
- Behavior accidentally changed during migration

### Architectural gaps

- Missing stages
- Missing integration points
- Unhandled edge cases
- Incomplete migration
- Old and new paths accidentally coexisting

### Guard and validation issues

- Missing validation
- Incorrect error propagation
- Missing invariants
- Incorrect ordering
- Unsafe patterns
- Incorrect state ownership

### SOLID / Separation of Concerns

Check for violations of: Single Responsibility, Open/Closed, Liskov
Substitution, Interface Segregation, Dependency Inversion, and Separation of
Concerns.

### Red flags

Look for: code smells, anti-patterns, unnecessary complexity, performance
bottlenecks, security vulnerabilities, unnecessary synchronization, unnecessary
cloning, ownership workarounds, duplicate logic, dead architectural paths.

### Rust best practices

Check for: idiomatic ownership and borrowing, appropriate error handling,
unnecessary allocations, unnecessary `clone()`, unnecessary interior
mutability, appropriate visibility, correct module boundaries, correct
feature-gating, Clippy compliance.

### Architectural documentation

Verify that:

- Documentation describes the actual production architecture.
- Architectural debt records are accurate.
- Deferred decisions are actually deferred.
- Completed migrations are no longer documented as deferred.
- Comments do not contradict authoritative architectural documentation.
- No stale references to removed architecture remain.

### Audit completion requirement

Do not declare the architectural task complete if the audit identifies a
critical or high-severity issue. Fix identified issues before finalization.

## Completed-migration documentation expectations

When a migration is complete:

- The new architecture must be the sole production path.
- The old implementation must be removed or reduced to legitimate
  orchestration.
- No duplicate execution paths may remain; no old implementation may
  accidentally execute alongside the new implementation.
- Obsolete state ownership, imports, comments, and documentation must be
  removed/corrected.
- Architectural debt records and invariant documentation must accurately
  describe the new architecture.
- The final architectural audit must pass and the complete verification gate
  must pass.

These expectations are part of the migration finalization procedure; see
`incremental-migration.md` for the workflow and `verification.md` for the
gate.