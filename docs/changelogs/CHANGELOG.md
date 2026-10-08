# Clean-CTX — Changelog

**All notable changes to this project will be documented in this file.**

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
Historical releases are archived per version as `CHANGELOG_<version>.md`; the
version-history registry lives in
[`../CHANGELOG_VERSIONING.md`](../CHANGELOG_VERSIONING.md).

---

## [Unreleased]

### Fixed

* Native operator verification now covers real .NET test success/failure as
  well as both Angular test frameworks. Compressed spy evidence and C# neutral
  base-reference expectations follow their production contracts. The .NET
  warning fixture again contains removable boilerplate; its original assertions
  remain, with an additional exact unchanged-output check.
* Angular signal metadata recognizes `input.required` and `model.required`,
  including spaced and nested generic arguments, while rejecting lookalikes.
  Angular marker generation is now 5 and semantic generation is 4. Existing
  Karma/Jasmine and Vitest/TestBed fixtures exercise required inputs, required
  models, and typed outputs with the real Angular compiler and runtime.
* Angular `inject(Token)` metadata uses the written dependency token instead
  of its receiving field name, in both markers and semantic edges. Empty or
  computed token expressions produce no invented dependency. Angular marker
  generation advances to 4 and semantic generation to 3; older facts require
  regeneration.
* Angular testing metadata now recognizes Jasmine spy factories, method spies,
  and property spies alongside Vitest spies. Comment/string guards, marker
  deduplication, and fidelity gating remain shared. Only the Angular marker
  producer generation advances to 3; semantic generation remains 2.
* Angular signal-based input/output extraction recognizes generic calls and
  retains declared field names instead of whitespace-derived `?` names.
  Comment and string matches no longer produce signal fields or injection
  evidence. Angular marker and semantic producer generations advance so
  pre-fix durable facts must be regenerated.

## [0.9.1] - 2026-10-08

### Queryable neutral C# base types

* **Cross-file C# base-type discovery** — unresolved or cross-file C# base-list
  entries now project as source-true `HasBaseType` relationships from the
  owning `builtin / Class` to the written `builtin / TypeRef`. Forward
  queries can inspect a class's written base types, and reverse queries on the
  neutral `TypeRef` can find classes that name that type without falsely
  classifying the relationship as `Extends` or `Implements`.
* **Scoped durable compatibility** — the changed edge projection is identified
  by a C#-specific semantic-projection producer generation. Pre-change C#
  semantic snapshots are rejected with the existing structured compatibility
  error and must be regenerated; unrelated language snapshots are unaffected.
* **Query guidance** — unsupported reverse lookup on a same-name
  `builtin / Interface` now points callers to the neutral
  `HasBaseType` / `TypeRef` query.

### Verification

* RED was established through tracked production-path forward and reverse
  `workspace_query` regressions, both returning empty edge sets before the
  fix. The unchanged regressions and focused projection, compatibility,
  restore, and coverage suites passed after implementation.
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
