# Incremental Architectural Migration (`docs/agent/incremental-migration.md`)

This is the detailed procedure for an explicitly designated **incremental
architectural migration**. It is NOT always-loaded; consult it only when
performing a designated migration.

## Established-green-baseline concept

The branch has an established green baseline before a migration begins.
Re-running expensive repository-wide checks after every small structural
change provides little additional information and wastes time.

Do **not** repeatedly run, after every small implementation phase:

- the full test suite,
- full Clippy,
- full formatting checks,
- other repository-wide verification.

Instead:

- Use the existing code as the behavioral reference.
- Inspect the affected code carefully.
- Preserve established behavior.
- Continue the migration incrementally.
- Use targeted inspection or reasoning when necessary to verify correctness.
- Do not treat the absence of a repeated test run as permission to ignore
  correctness.

The goal is to separate **implementation velocity** from **final
verification**.

## Behavioral preservation during migration

When migrating existing production behavior into a new architectural
structure:

- Preserve existing behavior unless a deliberate behavioral change has been
  explicitly approved.
- Treat the existing production implementation as the behavioral reference
  until the replacement is fully integrated.
- Prefer mechanical extraction, delegation, and relocation over rewriting
  working logic.

Preserve, specifically: instruction ordering, ID allocation order, mutation
semantics, error propagation and error classification, feature-gated behavior,
execution ordering, the ordering of configured layers/recognizers/handlers and
other extensible components, existing edge-case behavior, existing
serialization and canonical-representation behavior, and existing test
expectations unless the expectation itself is intentionally being changed.

Do not combine an architectural migration with an unrelated behavioral
redesign. If cleanup or simplification is discovered during the migration: do
not silently incorporate it — determine whether it is necessary for the
migration; if not, defer it to a separate change; if necessary, document the
reason.

The preferred migration pattern is:

```
existing behavior → structurally relocate it → establish new architectural
boundary → remove old implementation → verify equivalence
```

not:

```
existing behavior → rewrite while migrating → hope tests catch everything
```

## Incremental implementation workflow

For a designated incremental architectural migration:

1. **Understand the existing architecture first.**
2. **Identify the actual production behavior.**
3. **Identify the intended architectural boundary.**
4. **Document discrepancies between the two.**
5. **Migrate incrementally.**
6. **Preserve behavior during the migration.**
7. **Remove obsolete paths only after the replacement is ready.**
8. **Update architectural documentation to reflect reality.**
9. **Perform a final architectural audit.**
10. **Run the complete verification gate.**

Do not allow documentation, intended architecture, or comments to be treated
as proof of how production actually behaves. When documentation and
implementation disagree: **implementation determines current behavior;
documentation determines intended architecture only when explicitly
established as authoritative.**

## Migration cleanup

Before declaring the migration complete:

- The new architecture must be the sole production path.
- The old implementation must be removed or reduced to legitimate
  orchestration.
- No duplicate execution paths may remain.
- No old implementation may accidentally execute alongside the new
  implementation.
- Obsolete state ownership must be removed.
- Obsolete imports must be removed.
- Obsolete comments and documentation must be corrected.
- Architectural debt records must be updated.
- The architectural invariant documentation must accurately describe the new
  architecture.

## Migration finalization checklist

An incremental migration is not complete merely because the new architecture
compiles or has been wired into production. The complete Final Verification
Gate is mandatory during migration finalization (see `verification.md`):

1. `cargo fmt --all -- --check`
2. `cargo clippy --all-targets -- -D warnings`
3. `cargo test --workspace --all-targets --all-features`
4. Confirm all checks pass.
5. If anything fails: investigate; determine whether it was caused by the
   migration or was genuinely pre-existing; fix migration-caused failures; do
   not dismiss failures simply because they were unexpected; re-run the
   appropriate verification until the final state is understood and green.
6. Perform the required architectural audit before declaring the migration
   complete (see `docs/agent/architecture.md` — do not duplicate it here).

**An architectural migration is NOT complete until the final verification gate
has been performed.**

The intended workflow is:

```
incremental implementation → architectural cleanup → final audit →
complete verification → green repository
```

## Completed-migration documentation

Documentation and tests follow the migration. See `architecture.md` →
"Completed-migration documentation expectations". Durable architectural facts
live in `docs/ARCHITECTURAL_INVARIANTS.md`; once a migration is complete, its
debt record is marked RESOLVED there rather than being carried as migration
procedure.