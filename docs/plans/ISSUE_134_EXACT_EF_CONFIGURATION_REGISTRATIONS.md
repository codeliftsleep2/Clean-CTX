# Issue #134  Exact EF Configuration Registrations

**Agent:** Agent MaxHeadRoom  
**Status:** Implemented; focused local and native acceptance green  
**Date:** 2026-10-15

## Outcome

EF Core `HasEntity` query results now include declaration-proven entities from:

- existing durable `DbSet<T>` facts;
- structured `modelBuilder.Entity<T>()` calls;
- exact `ApplyConfiguration(new XConfiguration())` calls where the
  configuration implements `IEntityTypeConfiguration<T>`; and
- exact `ApplyConfigurationsFromAssembly(typeof(T).Assembly)` calls where `T`
  resolves to the same uniquely owned `.csproj` as the DbContext.

The result is request-local. No resolved EF edge, project map, or assembly model
is persisted, and no semantic producer generation was changed.

## Approved assembly boundary

The maintainer explicitly approved the following architecture for this slice:

> A uniquely resolved nearest ancestor `.csproj` is a request-local candidate
> assembly boundary. It does not prove an arbitrary assembly expression. EF
> assembly expansion requires both exact referenced-type resolution and matching
> unique project ownership; absent, ambiguous, or unprovable ownership fails
> closed.

This implementation initially recognizes `typeof(ExactType).Assembly`. The type
must resolve through the scoped C# catalog, and its declaration and the DbContext
must share the same unique nearest `.csproj`. Arbitrary variables, external
assemblies, and workspace-wide proximity are not authority.

## Production integration

The query-time path is:

1. normal C# compilation publishes declarations and existing EF facts;
2. the scoped C# catalog reads session-owned source snapshots;
3. structured `OnModelCreating(ModelBuilder name)` invocations are selected;
4. written entity/configuration/marker types resolve to exact declarations;
5. configuration declarations must implement the proven EF interface;
6. assembly expansion applies the approved project-ownership boundary;
7. derived `HasEntity` edges merge with stored edges by semantic identity; and
8. forward and reverse `workspace_query` surfaces expose the current projection.

An assembly-only DbContext can produce request-local results without a durable
`dotnet/DbContext` registration. Its response therefore honestly retains
`identity_indexed: false` and lower-bound coverage rather than changing the
shared coverage contract.

## Tracked regression evidence

`src/tests/mcp/workspace_query_dotnet_efcore_configurations.rs` covers:

- direct Fluent API entities;
- explicit configuration resolution;
- deduplication with an existing `DbSet<T>` fact;
- reverse `Entity -> DbContext` queries;
- ten same-project assembly configurations;
- exclusion of a configuration owned by another `.csproj`;
- rejection of an arbitrary assembly expression; and
- lifecycle retraction after registration removal.

User-run results were reported green for the focused EF module, existing EF
metadata/semantic tests, adjacent `workspace_query_dotnet_` tests, and
`cargo clippy --all-targets --all-features -- -D warnings`.

## Native acceptance evidence

The provider-disabled debug-binary case
`134 / ten-exact-ef-configuration-registrations` passed in:

`target/issue-reconciliation/20261015-112550-0e2d3647/`

`134-ef-configurations.json` records exactly `AuditEntity0` through
`AuditEntity9`, each as a current `AuditDbContext --HasEntity--> Entity` edge
whose subject provenance is `EfContext.cs`.

The consolidated run retained only the previously known, unrelated #139
service-declaration failure.

## Remaining verification

This slice satisfies the EF residual described by the Issue #134 reconciliation
plan. Repository-wide finalization still follows `docs/agent/verification.md`;
the native harness result is operator evidence and does not replace tracked Rust
tests.
