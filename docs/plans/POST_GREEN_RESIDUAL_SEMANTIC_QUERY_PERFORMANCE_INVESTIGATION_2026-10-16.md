# Post-GREEN Residual Semantic Query Performance Investigation — 2026-10-16

**Investigator:** Agent MaxHeadRoom  
**Baseline:** authoritative release gate GREEN  
**Disposition:** **PROCEED NEXT — build-local EF project-root lookup reuse**  
**Implementation status:** prepared; focused GREEN not yet authorized

## Executive Verdict

The historical performance topology is no longer current. Static Tree-sitter
queries, request-local C#/Angular catalogs, request-local EF derivation, and the
three measured EF global-reference scans are amortized. Cross-request catalog
caching remains deferred.

One residual avoidable unit is proven: EF assembly matching repeatedly walks
the filesystem to find the same project root. The scaled GREEN fixture performs
5 project-root lookups for four configurations and 17 for sixteen, although the
fixture contains only two distinct participating C# files. The configuration
comparison itself is required linear work; repeated project discovery for the
same file inside one `EfCoreCatalog::build` is not.

**NEXT TARGET:** memoize `nearest_project(file)` only for the duration of one EF
catalog construction. Do not cache across requests and do not change assembly,
project, declaration, or configuration authority.

## Current Performance Topology

```text
workspace_query request/batch
 -> validate one shared scope
 -> prepare item(s)
 -> hydration discovery (request-key memoized; session completion cache)
 -> acquire/compile only inadequate candidates
 -> publish canonical semantic projection
 -> reconcile C# exact calls once per publishing hydration cycle
 -> invalidate request-local derived state after mutation
 -> build C# catalog once per compatible request evidence interval
    -> parse each admitted C# source once
    -> build reference-candidate index once
 -> build Angular catalog once per compatible completeness interpretation
    -> process each linked template once per catalog
 -> build EF catalog once per compatible request evidence interval
    -> process declarations/configurations/models
 -> mechanical candidate lookup
 -> semantic resolution/refinement
 -> evaluate all prepared items under one index read
 -> serialize/render results
```

| Stage | Current cardinality | Current status |
| --- | --- | --- |
| Request scope | Once per request/batch | Amortized |
| Hydration | Once per request hydration key; durable discovery hits skip acquisition | Required/controlled |
| Candidate compilation/publication | Per inadequate selected file | Required authority work |
| C# reconciliation | Once per publishing hydration cycle | Prior multiplier removed |
| C# catalog | Once per compatible request evidence interval | Amortized |
| C# source parse for catalog | Once per admitted file per catalog build | Required by current catalog evidence |
| Reference-candidate index | Once per C# catalog; one entry per reference | Linear mechanical work |
| Angular catalog | Once per completeness interpretation/evidence interval | Amortized |
| Angular template parse | Once per admitted linked template per catalog | Distinct-input work |
| EF catalog | Once per compatible request evidence interval | Amortized |
| EF reference lookup | Per matched owner bucket | Prior global scans removed |
| EF assembly comparison | Once per configuration mapping for an assembly marker | Required linear membership reasoning |
| EF project-root discovery | Repeated per comparison, including identical files | Proven redundant target |
| Result filtering/rendering | Per item/result | Required output work |

## Historical Findings Reclassified

| Historical finding | Current classification |
| --- | --- |
| Static query compilation per observation | **OBSOLETE / ALREADY AMORTIZED** |
| C#/Angular catalog reconstruction per item/pass | **OBSOLETE / ALREADY AMORTIZED** |
| EF catalog reconstruction per item/pass | **OBSOLETE / ALREADY AMORTIZED** |
| EF global reference scans | **OBSOLETE / ALREADY AMORTIZED** for the three measured consumers |
| Reconciliation once per publication | **OBSOLETE / ALREADY AMORTIZED** |
| Cross-request catalog reconstruction | **DEFER**; no material benefit proof sufficient for durable validity machinery |
| Parser construction | **DEFER**; no material standalone leverage measurement |
| Parse-tree caching | **DEFER**; high lifetime/identity/memory cost without current proof |

## Residual EF Work

The post-index scaled GREEN measurements are:

| Work | 4 configurations | 16 configurations | Classification |
| --- | ---: | ---: | --- |
| Configuration declarations examined | 9 | 33 | Required linear declaration classification |
| Configuration/reference comparisons | 5 | 17 | Bounded owner-bucket work |
| Context declarations examined | 9 | 33 | Required current classification pass; materiality unmeasured |
| Context/reference comparisons | 5 | 17 | Bounded owner-bucket work |
| Model occurrences | 1 | 1 | Required |
| Owner/reference comparisons | 1 | 1 | Bounded |
| Synthetic type parses | 5 | 17 | Distinct written types in this fixture |
| Assembly/configuration comparisons | 4 | 16 | Required to admit each configuration |
| Project-root lookups | 5 | 17 | Avoidably repeats identical filesystem discovery |

No nested global-reference loop remains in these EF paths. The dominant proven
redundancy is now below semantic resolution: repeated filesystem project-root
discovery during otherwise required linear assembly membership checks.

## EF Assembly and Project Lookup Analysis

Call path:

```text
workspace_query EF edge preparation
 -> PreparationContext::efcore_catalog
 -> EfCoreCatalog::build
 -> assembly marker resolution
 -> common_project(context.file, marker.file)
    -> nearest_project(context.file)
    -> nearest_project(marker.file)
 -> for each configuration mapping
    -> nearest_project(configuration.file)
    -> compare with marker project
```

`nearest_project` ascends directories and calls `read_dir` at every level until
it finds exactly one `.csproj`, finds ambiguity, or reaches the filesystem root.
In the scaled fixture, context and marker are the same file, and all
configurations share one configuration file. Thus 17 calls cover two unique
file identities. Fifteen large-fixture calls are repeated derivations of the
same build-local filesystem fact.

The `A × K` configuration comparisons remain semantically required: every
configuration mapping must be checked for project membership. Only project-root
discovery is redundant. A build-local map from canonical file string to
`Option<PathBuf>` can retain both success and failure and feed the unchanged
comparison. It is reconstructable, ephemeral, and requires no authority epoch.

## Synthetic Parsing Analysis

Synthetic parsing occurs in `parse_type_shape(written)` by embedding a written
type into `class __CleanCtxProbe : {written} {}` and using the C# grammar. It is
used for each model/entity/configuration/assembly written type before existing
declaration resolution.

The scaled fixture records 5 and 17 parses: one per configuration entity type
plus the assembly marker type. Those inputs are distinct (`Entity0` through
`EntityN` plus `AppDbContext`), so the measurement does not demonstrate repeated
parsing of equivalent input. The outer `BaseReference` stores the outer generic
shape and written text, not a separately authoritative parsed inner generic
argument. Therefore this is **REQUIRED WORK** under current evidence. Parser
pooling would remove setup only; parse-result caching would need an identity and
has no demonstrated duplicate-input payoff.

## Remaining C# Reference and Declaration Work

Global scans still exist outside the three indexed EF consumers:

- `recognizes_edge` and `resolve_edge` scan the reference vector using file,
  owner spelling, and written type;
- `written_references_to` resolves every reference;
- `base_reference` scans references while also checking namespace/nesting;
- `DependencyGraph::build` performs per-edge global reference lookup, then
  declaration resolution and linear target-position lookup;
- declaration resolution scans declarations for candidate path and arity;
- unique-name helpers scan declarations and preserve ambiguity.

These are production-reachable but their current frequency and scaled work are
not instrumented. The existing candidate index could mechanically narrow some
file/owner predicates, but no migration earns itself merely because the index
exists. Classification: **MEASURE FIRST**, below the proven project-root target.

## Angular Residual Work

Angular construction enumerates component identities, deduplicates template
links, reads each inline/external template, parses it once with
`template_ordered::extract`, and resolves each selector through
`WorkspaceIndex::resolve_selector`. Selector resolution already begins with the
workspace `name_index` and reverse `HasSelector` edges; it is not a global
entity scan.

Phase 3 guarantees one catalog per required `discovery_complete` interpretation
and compatible evidence interval. Separate partial and complete catalogs are
semantically distinct because diagnostics and resolution status differ. No
within-catalog duplicate parse or selector comparison amplification is measured.
Classification: **REQUIRED WORK / MEASURE FIRST only if field evidence changes**.

## Reconciliation Residual Work

Reconciliation runs once after a hydration cycle that publishes C# candidates.
It admits the current C# file set, then performs two compilations per admitted
file:

1. compile all files at High fidelity to reconstruct complete declaration
   authority;
2. compile all files again with that complete authority to resolve exact calls,
   preserving each projection's published semantic fidelity.

It then revalidates every source hash before one projection replacement phase.
The passes have different inputs and outputs: the first produces global
declaration authority; the second consumes it. They are not proven equivalent.
No current counter separates per-file compiler sub-work inside these passes,
and no duplicate derivation safe to remove is proven. Classification:
**REQUIRED WORK; MEASURE FIRST before any redesign**.

## Canonical IR Versus Downstream Parsing

Canonical compilation and request-local catalogs both parse source, but they
own different representations and authority:

- publication compilation creates canonical IR and semantic edges at declared
  fidelity;
- C# catalog parsing reconstructs namespace/nesting/import/type-shape facts and
  class source required for query-time ambiguity-preserving resolution;
- Angular template parsing consumes inline/external template source and
  occurrence order not established by the canonical TypeScript projection;
- EF model parsing extracts `OnModelCreating` invocation semantics not stored as
  equivalent canonical authority.

Representation overlap is present, but authority/evidence equivalence is not.
Removing downstream parsing is not justified. A future investigation would
need field-by-field proof that canonical results contain every required fact and
provenance.

## Parser Construction and Parse Trees

Parsers are still created for C# catalog files, EF model bodies and synthetic
types, Angular templates, and several canonical/meta producers. Higher-level
reuse has already removed repeated catalog-level construction. No counter shows
parser allocation/language setup is material relative to distinct parse work.
Parser pooling is **DEFERRED**.

Parse-tree retention would require canonical file/source hash, grammar/producer
identity, bounded ownership, invalidation, concurrency, and memory policy. It
would overlap canonical IR and request-local catalogs without proving that
either can consume the retained tree safely. Parse-tree caching is **DEFERRED**.

## Artifact Cardinality and Memory

| Artifact | Cardinality / size driver | Owner / lifetime | Reconstructable | Persistent |
| --- | --- | --- | --- | --- |
| Registered compiled queries | One per closed registered identity; fixed query complexity | Process-local lazy cells; binary lifetime | Yes | No durable persistence |
| C# catalog | One per compatible request evidence interval; admitted declarations, references, class-source slices copied as strings, candidate index | `PreparationContext`; request lifetime | Yes | No |
| C# candidate index | One key bucket per observed file/owner pair; one integer position per reference | C# catalog; identical lifetime | Yes | No |
| Angular catalogs | Up to two per evidence interval (`partial`, `complete`); components, links, template facts, diagnostics | `PreparationContext`; request lifetime | Yes | No |
| EF catalog | One per compatible evidence interval; derived `HasEntity` edges | `PreparationContext`; request lifetime | Yes | No |

No artifact survives a request except the fixed closed registered-query set.
The candidate index adds linear integer/key overhead. C# declarations copy each
declaration node's source text; nested declarations may overlap source storage,
but no scaled memory measurement demonstrates pathological growth. Memory
optimization is not currently justified.

## Measurement Gaps and Proposed Controls

Existing counters cover catalog construction, C# file parses, Angular template
processing, EF work, and candidate-index cardinality. Missing observations are:

- unique versus repeated project-root discovery inputs;
- remaining non-EF C# reference/declaration comparisons by consumer;
- reconciliation files and compiler passes/sub-work;
- canonical producer executions relative to downstream parses;
- Angular selector occurrences/candidate comparisons;
- parser construction separately from parse execution;
- artifact byte estimates/high-water marks.

Only the first gap is needed for the selected next target, and the existing
fixture already fixes the unique input set at two files. A future regression
should additionally count cache misses or underlying filesystem traversals.

## Candidate Classification and Ranking

| Rank | Candidate | Classification | Proof / scaling / frequency | Reduction | Risk / verification |
| --- | --- | --- | --- | --- | --- |
| 1 | Build-local EF project-root lookup reuse | **PROCEED NEXT** | 5→17 lookups over 2 unique files as configurations grow 4→16 | Large fixture: 17 traversals to 2 | Low; mechanical map, deterministic counters |
| 2 | Remaining C# reference/declaration lookup indexing | **MEASURE FIRST** | Static scans remain; current scaled frequency absent | Unknown | Low-to-moderate identity/ambiguity risk |
| 3 | Reconciliation internal reuse | **MEASURE FIRST / REQUIRED WORK** | Two full passes, but semantically distinct | Unknown | High authority risk |
| 4 | Angular selector/template refinement | **REQUIRED WORK** | Once per distinct template/occurrence per interpretation | No redundancy proven | Medium semantic risk |
| 5 | Synthetic type parsing | **REQUIRED WORK** | 5→17 for distinct inputs | No equivalent-input duplication proven | Medium resolution risk |
| 6 | Parser pooling | **DEFER** | Allocation cost unmeasured | Likely small | Concurrency/lifetime complexity |
| 7 | Parse-tree caching | **DEFER** | Reparse exists; material duplicate-tree payoff unproven | Unknown | High memory/validity complexity |
| 8 | Cross-request catalogs | **DEFER / FROZEN** | Residual benefit remains unmeasured | Unknown | Highest validity risk |

## Selected Next Phase

**PROCEED NEXT — build-local EF project-root lookup reuse.**

Smallest safe boundary:

1. introduce an ephemeral lookup owned by one `EfCoreCatalog::build` call;
2. key by the exact existing file string passed to `nearest_project`;
3. retain `Option<PathBuf>` so failures and ambiguous projects are reused too;
4. route only `common_project` and assembly configuration membership through
   the build-local lookup;
5. leave every configuration comparison and entity insertion unchanged;
6. add counters for underlying filesystem resolutions versus cache hits;
7. prove the scaled fixture changes project-root traversals from `5/17` to
   `2/2`, while assembly comparisons remain `4/16`, synthetic parses remain
   `5/17`, and semantic results remain identical;
8. discard the lookup with the EF build. No request/session cache.

### Invariants

- Project lookup reuse is execution machinery, never project authority.
- Cache key equality is exact existing file-string equality.
- Cache `None` as well as `Some`; do not turn ambiguity/failure into certainty.
- Preserve the current nearest-project algorithm and every comparison.
- Do not change EF entity identity, configuration resolution, provenance,
  ordering, fidelity, discovery, reconciliation, or completeness.
- No state crosses an EF catalog construction, request, or mutation boundary.

### Commands Requiring Separate Authorization

No new test is prepared and no Cargo command was run. After the next phase is
explicitly authorized and its structural regression exists, its focused command
should remain within the existing module:

```text
cargo test --all-features "mcp::tool_handlers::query::tests_performance_work_counts::ef_reference_scans_scale_multiplicatively_after_catalog_reuse" -- --exact --nocapture
```

The candidate-index and EF configuration suites should then be rerun as
affected verification. Builds, Clippy, benchmarks, MCP/CBM, live verification,
and the authoritative release gate require their own authorization.

## Release Baseline

The repository entered this investigation GREEN across correctness, Phases
1–3, request-local EF reuse, candidate indexing, affected regressions, and the
authoritative release gate. This investigation changes documentation only and
does not alter that baseline.
