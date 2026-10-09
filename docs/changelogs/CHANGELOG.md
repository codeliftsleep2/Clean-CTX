# Clean-CTX — Changelog

**All notable changes to this project will be documented in this file.**

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
Historical releases are archived per version as `CHANGELOG_<version>.md`; the
version-history registry lives in
[`../CHANGELOG_VERSIONING.md`](../CHANGELOG_VERSIONING.md).

---

## [0.9.2] - 2026-10-09

### Structured C# declaration-header authority

* **One parser-owned declaration identity** — C# Class, Interface, Struct, Enum,
  and Record names now come from structured identifier captures shared by
  canonical lowering and builtin registration. Partial declarations, primary
  constructors, and positional records no longer corrupt or truncate the
  public owner identity.
* **Structured Class base-list lowering** — canonical C# Class relationships
  now consume direct parser base-list children. Multiline lists preserve every
  entry, qualified and generic written references remain intact, and comments
  or generic constraints cannot be misread as inheritance. The former
  text-derived Class relationship extractor was removed rather than retained
  as a competing authority.
* **Truthful exact-query capability** — an exact edge response containing a
  derived typed `Extends` or `Implements` edge now establishes that
  query's capability even when the edge is intentionally ephemeral. Coverage
  remains a lower bound when hydration is incomplete, and neutral
  `TypeRef / HasBaseType` alternative-query guidance remains available.
* **Scoped durable compatibility** — existing `CSharpCanonical`,
  `CSharpSemanticInput`, and `CSharpSemanticProjection` producer
  generations were incremented for their changed outputs. No producer key or
  unrelated-language compatibility identity was added or changed.
* **Struct/Record preservation** — C# Struct and Record base entries retain the
  v0.9.1 neutral `HasBaseType` behavior and distinct public owner kinds;
  this release only unifies their declaration-name authority with the
  structured C# identity path.

### Verification

* RED was established with seven tracked production-path regressions covering
  multiline and partial Class headers, constraints and comments, shared
  canonical/builtin identity, exact-query coverage, and producer generations;
  the qualified/generic neutral-reference control passed before implementation.
  The unchanged focused suite passed after implementation.
* The complete release gate and `cargo pkgid -p clean-ctx` remain
  maintainer-run requirements before the `v0.9.2` release tag.

---

## [0.9.1] - 2026-10-09

### Queryable neutral C# class, struct, and record base types

* **Cross-file C# base-type discovery** — unresolved or cross-file C# class, struct, and record base-list
  entries now project as source-true `HasBaseType` relationships from the
  correct `builtin / Class`, `builtin / Struct`, or `builtin / Record` owner to the written `builtin / TypeRef`. Forward
  queries can inspect written base types, and reverse queries on the neutral
  `TypeRef` can find declarations that name that type.
* **Cross-file C# class classification** — exact Class and Interface edge
  queries now refine neutral Class base-list facts into ephemeral `Extends`
  or `Implements` results after completed scoped hydration proves exactly one
  target kind. Cold forward Class queries hydrate each distinct neutral target
  name before classification. The durable `HasBaseType` fact remains
  authoritative and queryable; absent or ambiguous targets remain neutral, and
  Struct/Record facts are never consumed by the Class classifier.
* **Struct and Record identity preservation** — parser-structured C# base-list
  captures now publish every written qualified, generic, multiline, and nested
  base type from the existing `builtin / Struct` or `builtin / Record` owner,
  never through a fabricated Class identity. Republishing or removing an entry
  retracts the old fact without duplicates, and no unsupported `Extends` or
  `Implements` relationship is invented.
* **Scoped durable compatibility** — the changed edge projection is identified
  by a C#-specific semantic-projection producer generation. Pre-change C#
  semantic snapshots are rejected with the existing structured compatibility
  error and must be regenerated; unrelated language snapshots are unaffected.
* **Query guidance** — unsupported reverse lookup on a same-name
  `builtin / Class` or `builtin / Interface` now points callers to the neutral
  `HasBaseType` / `TypeRef` query.

### Verification

* RED was established through tracked production-path forward and reverse
  `workspace_query` regressions, both returning empty edge sets before the
  fix. The unchanged regressions and focused projection, compatibility,
  restore, coverage, Struct/Record publication and lifecycle, and cross-file
  Class classification suites passed after implementation. The final
  all-target/all-feature Clippy gate also passed with zero warnings.
* The complete release gate and live MCP pilot remain maintainer-run
  requirements before the `v0.9.1` release tag.

---

## [0.9.0] - 2026-10-05

`0.9.0` hardens the authority model beneath the correctness-oriented context
infrastructure introduced in `0.8.0`. State no longer becomes current truth
merely because it exists, deserializes, was persisted, or finishes last. It
must cross the structural, compatibility, ownership, source-precondition, and
owner-publication boundaries appropriate to that state.

### Authority and durable-state hardening

* **Scoped durable compatibility (#116, #117, #119)** — durable canonical and
  semantic state now carry four independent identities:
  `CanonicalConfigIdentity`, `CanonicalProducerIdentity`,
  `SemanticConfigIdentity`, and `SemanticProducerIdentity`. Only dimensions
  capable of affecting the corresponding projection participate. Package,
  crate, Git, SQLite-schema, binary-wire, and whole-configuration fingerprints
  are not treated as semantic compatibility evidence. Missing legacy evidence
  is unproven rather than silently trusted.
* **Compatibility-aware publication (#116)** — restore, replay, recovered edit
  publication, and mutating delta application must produce a
  `CompatibleSemanticProjection` before semantic snapshot, coverage, or
  `WorkspaceIndex` authority changes. Handler-specific remove/add publication
  sequences were consolidated behind the shared typed boundary.
* **Durable semantic structural validation (#121)** — successful JSON decoding
  no longer suffices for adoption. Durable edges enforce intrinsic
  relation/evidence coherence (`Calls` if and only if call evidence exists)
  before entity restoration, interning, or live publication.
* **Coherent checkpoints (#123)** — `save_context` captures canonical IR,
  version, source identity, fidelity, semantic projection, and compatibility
  identities from one owner-authority epoch. Persistence I/O occurs after the
  immutable payload is captured, without imposing a global authority lock.

### Unified owner authority ordering

* **Source-ordered semantic publication (#113)** — same-owner compilation and
  publication use `SemanticPublicationClock` generations established before
  source selection. An older source candidate cannot overwrite a newer
  committed owner merely because it completes later; different owners remain
  independently processable.
* **Delta baseline epochs (#122)** — `delta_code_context` binds baseline
  capture, candidate compilation, delta derivation, pending-transition
  registration, and commit validation to one authority ticket. A transition is
  rejected or retried if its exact `from` baseline or source precondition is
  superseded.
* **Historical adoption ordering (#124)** — restore and replay retain their
  operation generation through final commit. Operation age is distinct from
  snapshot age: a stale in-flight operation cannot win by completion order,
  while a genuinely later operation may intentionally adopt compatible older
  history.
* **Commit-time edit compare-and-swap (#125)** — `apply_edit` re-proves the
  exact physical source bytes under the owner commit boundary before staging,
  durable intent, replacement, or live publication. Atomic replacement alone
  is not considered compare-and-swap.

### Current-source and restore authority

* **WorkspaceIndex source-universe reconciliation (#118)** — an authoritative
  refresh structurally retracts evidence owned by sources proven absent from
  the current semantic source universe. This protects hydrating and
  non-hydrating queries without per-query stale filters, filesystem watching,
  source-independent restore, or CBM authority over semantic truth.
* **Trusted restore admission (#120)** — `restore_context` resolves and admits
  the requested path through `workspaceRoot` and configured additional roots
  before durable identity registration, edit recovery, persistence loading,
  alias/canonical publication, semantic publication, index mutation, or cache
  publication. Nearest-existing-ancestor checks close dangling-symlink escapes.
* **Restored coverage truthfulness (#114)** — compatible restored projections
  reconstruct Low/Medium/High coverage only where certifiable. Edit and
  Verbatim states remain uncertified when equivalent semantic coverage cannot
  be proven.

### Semantic relationship and hydration correctness

* **Truthful inheritance projection (#98)** — `Extends`, `Implements`, and
  `InterfaceExtends` reach the semantic relationship model. Ambiguous C# base
  references remain neutral `BaseTypeRef` facts until resolution supplies
  classification evidence.
* **Fidelity-aware hydration (#99)** — hydration escalates fidelity when
  existing coverage is insufficient, and caches completion only after the
  required semantic projection becomes current.
* **Exact identity/capability reporting (#100)** — workspace queries distinguish
  genuine absence from identity or capability that could not be established;
  an unproven zero is no longer presented as a complete answer.
* **Publication-complete discovery (#112)** — candidate discovery is committed
  only after every selected candidate is already current or publishes the
  required projection successfully. Failed publication remains retryable.
* **Producer and physical-identity corrections (#108, #110)** — declaration
  extraction now handles post-parameter language syntax without corrupting
  identities, while reads may collapse hard-linked aliases and mutations reject
  multiply linked files whose path-based edit ownership would be ambiguous.
* **Verified non-changes (#102, #105)** — isolated declarations were confirmed
  to retain identity through self-`Defines` registration, and constructor-type
  plus inheritance relations remain intentionally outside automatic
  `transitive_dependencies` traversal policy. No speculative repair was added.

### Git change-set correctness

* **Pre-truncation accounting (#126)** — `file_count` records the complete
  changed-file set discovered before `max_files` limits processing. Limit and
  later processing/size skips contribute to `skipped`, preserving
  `added + deleted + modified + renamed + type_changed + skipped == file_count`.
* **File-type changes (#127)** — Git `T` status now maps to
  `FileChange::TypeChanged`, renders as `~ FILE αN: path (type changed)`, and is
  exposed as `_meta.counts.typeChanged`. The legacy four-element count tuple is
  unchanged, and Clean-CTX does not invent specific old/new physical types.

### Documentation

* Living architecture, IR, configuration, developer, diff, README, invariant,
  public MCP schema, roadmap, changelog, and version-registry surfaces now
  describe the 0.9.0 authority model. The September historical audit remains
  unchanged so it stays truthful to its original date.

### Verification

* Tracked regression coverage was added or strengthened for inheritance,
  identity/capability reporting, hydration completion, same-owner publication,
  restored coverage, durable compatibility, semantic snapshot validation,
  source-universe reconciliation, restore admission, delta/save/historical
  races, stale edits, Git truncation accounting, and real Git type transitions.
* Release metadata checks completed by the agent: strict UTF-8, file-size
  policy, whitespace validation, and targeted formatting checks. The complete
  Cargo/Clippy/test/encoding release gate and `cargo pkgid -p clean-ctx` remain
  maintainer-run requirements before the release commit and `v0.9.0` tag.

---

Historical release details:

* [`0.8.0` and `0.8.0-rc`](CHANGELOG_0.8.0.md)
* [`0.7.0`](CHANGELOG_0.7.0.md)
* Older archives are retained in this directory under their shipped versions.
