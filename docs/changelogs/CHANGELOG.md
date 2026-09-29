# Clean-CTX — Changelog

**All notable changes to this project will be documented in this file.**

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
Historical releases are archived per version as `CHANGELOG_<version>.md` (`_a`/`_b` suffixes mark split sub-sections of one release); the version-history registry lives in [`CHANGELOG_VERSIONING.md`](CHANGELOG_VERSIONING.md).


---

## [0.8.0-rc] - 2026-09-21

### Architecture lockdown

* **Typed canonical ownership and checked projection** — class, interface,
  method, field, and parameter identities are compiler-distinct; all current
  operations have exhaustive validation, declared cardinality/order semantics,
  and stable-ID projection independent of stream position. TypeScript, Java,
  C#, and Rust declaration modifiers use declaration-owned structure.
* **Semantic families remain distinct** — declaration modifiers, control
  summaries, pattern facts, side effects, execution contexts, bodies/spans,
  relationships, calls, and explicit interface members have dedicated typed
  contracts. The compact LLM view remains independently minimized.
* **Exact transport and replay** — physical binary `0x04` round-trips complete
  canonical streams, while production `dv:2` deltas preserve position,
  occurrence identity, duplicates, and expected-tuple conflict detection.
* **Transactional production lifecycle** — canonical IR and complete semantic
  edges persist as one file-scoped logical state. Save, restore, replay,
  accepted deltas, replacement, deletion, byte-exact edits, and crash recovery
  commit durable state before live publication. Observational tools never
  flush or create ownership.
* **Phase 9 production certification** — every registered IR/MCP operation and
  semantic family was traced through dispatch, production compilation/input,
  validation, session/durable ownership, lifecycle transitions, consumers,
  and registered responses. Findings P9-01 through P9-27 were repaired and
  user-verified. Incomplete legacy fallback artifacts are registered,
  read-only quarantine evidence through `inspect_legacy_fallbacks`.
* **Release-candidate boundary** — repository gates and operator production
  scenarios were reported green by the maintainer. This is `0.8.0-rc`; final
  `0.8.0` remains withheld until live field testing is complete.

### Fixed

* **IR pattern classification no longer deletes the method it classifies (F2 — new invariant `PATID-001`)** — the consumptive `CompressingPatternRecognizer` used to consume every recognized method's identity — `DefMethod`, its `Param*`, and its `Return` — and replace the whole span with a single `PAT` op. A classified method therefore vanished from every representation that reads the declaration while the pattern op still referred to it: no hierarchical `MethodNode` (so no rendered `M <name>` line at all), no `UnitTable` record (so `apply_edit` reported "unit not found" for the method whose body had survived), no semantic registering occurrence, and a caller-side `Calls` id that resolved to no name. Pattern recognition is now a purely ADDITIVE classification at the producer boundary: each matcher reports the identity-bearing prefix it matched (`PatternMatch { retained_start, retained, consumed }`) and `compress_merged` re-emits those ops unchanged, in their original order, immediately before the `PAT` op. Only genuinely redundant non-identity ops are still summarized (`Injects` for CTOR, `Flags(ASYNC)` for OBSERVABLE, `Flags(OVERRIDE)` for OVERRIDE, and the additive/trailing `Flags(M)` runs the centralized wrapper already consumed), so the PROMISE, EMPTY_CTOR, GETTER and SETTER classifications are now purely additive and the CTOR/OVERRIDE shapes lose exactly one summarized op. Nothing about the representation changed: `CoreOp::Pattern`, its vocabulary, its named/binary wire tuples, the delta key formula and the hierarchical `PatternEntry` payload are untouched — no new opcode, no schema version, no payload carrying reconstruction data, and no downstream consumer was modified (the projection, renderer, `UnitTable`, semantic projection and `Calls` attribution all receive the declaration they already know how to consume). (`src/ir/patterns/recognize.rs`, `src/ir/patterns.rs`)

* **Method identity survives even where no classification fires** — preservation is unconditional and does not depend on a match: a region that compresses retains its declaration, and a region the IRPAT-001 guard declines is byte-identical to before.

* **The conservative IRPAT-001 decline is retained deliberately, not weakened** — retaining identity removes the *orphanhood* basis for declining compression when a surviving `DataFlow`/`SideEffect`/`ExecutionContext`/`ControlFlow`/`Body`/`Call` still references the method, but the guard stays as it is: weakening it would newly compress shapes that have never been compressed, which is a compression-policy change carrying its own stream, wire and delta consequences, and the retained guard cannot orphan anything.

### Changed

* **Flat-stream content changes wherever a pattern matches** — the retained declaration ops are re-emitted and the `PAT` op now follows them, so structural op counts move for exactly the matched spans: CTOR (`DEF_M` + `SIG` + `RET` + `INJECTS`) goes from 1 output op to 5 (only `INJECTS` is summarized into the classification) and PROMISE (`DEF_M` + `RET`) goes from 1 to 3 (nothing but identity matched, so the classification is additive). No opcode, operand arity, named-tuple shape, binary-wire layout, delta key formula or hierarchical field changed, so this is a content-sequence change and not a format change; a stored pre-change flat baseline can therefore report a one-time spurious mod set for those regions (per-item multiplicity in delta transport, F8, remains unfixed and out of scope).

* **Migration lineage** — method-identity retention was the first bounded
  slice. Later slices in this same release completed attribution, semantic
  families, occurrence-aware delta, exact binary persistence, and the
  registered production lifecycle without weakening the conservative body
  fallback.

### Not yet wired / observed but deliberately not fixed

* **`Fidelity::Low` cannot produce the Promise/Observable classifications** — live verification through the real pipeline showed that at `Low` the TypeScript method label is compacted to `load()` (the declared return type is not part of the compacted declaration), so the emitted `Return` is the void alias and the promise-like return the PROMISE/OBSERVABLE recognizers require simply does not exist. Measured behaviour over the three probe fixtures: CTOR is active at Low/Medium/High and declined at Edit (the Edit-only `Body` op detaches the trailing `Flags` run the wrapper consumes), PROMISE is active at Medium/High/Edit, and OBSERVABLE is active at Medium/High while degrading to PROMISE at Edit. Identity is preserved at every fidelity either way. This is a pre-existing fidelity/compaction property, it is not one of F1–F14, and it is recorded here rather than repaired.

### Tests

* `src/tests/ir/pattern_identity.rs` — the producer-boundary contract: production-shape CTOR / PROMISE / OBSERVABLE / EMPTY_CTOR identity retention plus the runtime-derived method ids (`RED-F2-1`…`RED-F2-5`), the complete consumptive surface including the recognizer-boundary OVERRIDE / GETTER / SETTER shapes with their non-production-reachability documented (`RED-F2-6`), the all-fidelity no-orphaned-classification invariant ( `RED-F2-12`), the exact retained-declaration output and op-count contract for CTOR and PROMISE, and a non-RED invariant guard (`identity_survives_where_a_classification_cannot_fire`) pinning that identity never becomes conditional on a match.
* `src/tests/ir/pattern_identity_downstream.rs` — the downstream contracts, each read from a real production compilation: hierarchical `MethodNode` retention with the mandatory production-shape regression (`RED-F2-7`), the rendered `M` line alongside `P PROMISE` on the compressed path (`RED-F2-8`), `UnitTable` addressing at Edit fidelity (`RED-F2-9`), semantic declaration projection (`RED-F2-10`), and `Calls` caller identity (`RED-F2-11`).
* `src/tests/ir/call_pattern_orphan.rs` — the two ctor/promise "control" expectations updated to the retained-declaration stream (implementation-shape tests; their semantic contract is preserved — the region is still classified, nothing is orphaned by that compression, and an unrelated caller is still reported as `E011`).
* `docs/ARCHITECTURAL_INVARIANTS.md` — new invariant `PATID-001` ("method identity survives pattern recognition") with its enforcement, authority, and its relationship to `IRPAT-001`; `docs/COMPILER_IR.md` — the pass sequence and the Layer-4 description now state identity-preserving classification semantics.

### Verification

* The maintainer reported the complete repository gates green after the final
  P9-27 repair, including zero-warning Clippy and tracked tests.
* The maintainer also reported the Phase 9 registered-production operator
  scenarios green. Those ignored `target/phase9-verification/` artifacts are
  live reachability evidence, not substitutes for tracked tests or CI.
* Phase 10 documentation/version edits require one final user-run gate before
  the release-candidate commit; no agent-run Cargo result is claimed.
