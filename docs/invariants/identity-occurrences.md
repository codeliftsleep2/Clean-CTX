# Invariant family: identity-occurrences

Current normative sections relocated from [the invariant catalog](../ARCHITECTURAL_INVARIANTS.md). Source attributes such as ENFORCED, STRUCTURAL, DOCUMENTED, RESOLVED and qualified enforcement retain their original meanings; GLS status is separate. Original section identities and witnesses remain unchanged except the explicitly documented C13 descriptive correction.

Graph status IMPLEMENTED records the source-described existing mechanism or operative governance contract, not a new successful witness execution. The DOCUMENTED mechanism-reuse rule remains INVESTIGATING in GLS: documentary convention is not promoted to proven machine enforcement. Sections without declarations remain current source contracts awaiting their assigned later migration batch.

### IDENT-001 One Physical File, One Stable Alias

| Property | Value |
|----------|-------|
| **Intent** | All in-session file-keyed state — alias registry, IR context versions, text-delta baselines, LLM text cache, `§PATHMAP` output, AND SessionStats per-file tracking — must attribute every operation to ONE identity per physical file, regardless of which path spelling a caller supplies (Non-CBM audit 2026-08-25 #3: fragmented aliases silently split per-file state so one alias's cache never saw updates recorded under the other; same mechanism fragmented stats into duplicate rows with double-counted totals). |
| **Invariant** | `PathDictionary::get_or_create_alias` maps different caller-supplied spellings of the same on-disk file (absolute path, workspace-root-joined relative form, redundant-segment form) to the SAME alias, and `SessionStats::record_compression`/`file_stats` key on the SAME canonical identity via the shared `dictionary::path::canonical_identity_key`. Canonicalization = `fs::canonicalize` when possible (Windows verbatim `\\?\` prefix stripped so stored keys stay readable); unresolvable paths fall back to the raw argument unchanged so synthetic strings neither collide nor panic. Canonical paths that share one platform physical identity—Windows `(volume serial, file index)`, Unix `(st_dev, st_ino)`—additionally converge through an `McpState`-owned auxiliary registry: canonical paths remain the public/provenance identity, while the first durable spelling remains the SQLite owner and is re-registered at startup. A physical file with multiple hard links is not structurally editable because atomic pathname replacement would split the links. **Deliberate compatibility rule:** ordinary SQLite persistence keys remain caller-shaped as supplied; physical-ID lookup changes only hard-link alias ownership and does not replace durable paths with opaque file IDs. |
| **Enforcement** | `canonical_identity_key` remains the lexical/canonical path authority. `PhysicalIdentityRegistry` adds platform physical alias resolution at session alias, semantic publication, and durable-owner boundaries; persisted owners seed it before requests are served. `apply_edit` rejects Windows `number_of_links > 1` or Unix `st_nlink > 1` before recovery, source reads, staging, or state mutation. Unsupported or unavailable physical metadata falls back to canonical-path identity. The ordinary persistence-key exception remains pinned by `persistence_keys_are_caller_shaped_by_contract`. |
| **Authority** | `src/dictionary/path.rs`; `src/mcp/state/physical_identity.rs`; `McpState::{semantic_owner_path,durable_owner_path}`; `src/mcp/tool_handlers/{core/provide,hydration/publication,edit}.rs`; Windows- and Unix-gated regressions in `src/tests/mcp/state.rs`, `workspace_query_hydration_fidelity.rs`, and `apply_edit_transaction.rs`; `src/tests/mcp/session_stats.rs`; `src/tests/mcp/sqlite_store.rs` |
| **Type** | ENFORCED (test) |
| **Gate** | `cargo test` |

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 304-316, original section `IDENT-001`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `1bdf0f520e1fc7d1b9e3e03c6be33f7d38eae31d67e7c38076f400c06c45dd0e`. Original text remains in Git history; this file is the sole current normative owner of this section.

```architecture-graph
node: IDENT-001
kind: INVARIANT
status: IMPLEMENTED
title: IDENT-001 One Physical File, One Stable Alias
track: FOUNDATION
artifact: docs/invariants/identity-occurrences.md
commit: 655f0a75
note: Source enforcement attributes are retained in prose; admission does not execute witnesses or establish independent validation.
```

### IRWIRE-001 Semantic Binary Round Trip

| Property | Value |
|----------|-------|
| **Intent** | Binary compactness must never discard canonical identity, metadata, ordering, or occurrence multiplicity. |
| **Invariant** | Physical binary `0x04` satisfies `decode(encode(value)) == value` for every valid supported `CompiledIR`. Decoding never fabricates empty or synthetic operands and rejects malformed, trailing, or unsupported-version input structurally. The Phase 7 delta version remains independent. |
| **Enforcement** | `src/tests/ir/binary_wire_v04.rs`, `src/tests/ir/binary_wire.rs`, and `src/tests/ir/round_trip*.rs` enforce all-variant equality, duplicate/order, metadata, malformed-input, wrapper, randomized, and deterministic contracts. `src/tests/mcp/persistence_lifecycle.rs` crosses registered dispatch, buffered persistence, SQLite ownership, reload, corrected-delta replay, and byte-exact editing. |
| **Authority** | `src/ir/binary_wire.rs`, `src/ir/binary_wire/decode.rs`, `src/mcp/sqlite_store/replay.rs`, and `docs/architecture/BINARY_V04_CONTRACT.md` |
| **Type** | STRUCTURAL + ENFORCED (test), including user-verified Phase 8C production persistence |
| **Gate** | Binary-wire and round-trip tracked tests plus the final verification gate |

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 367-378, original section `IRWIRE-001`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `03f0bfa35a2c30f14c3dd9895a938803d8831e8503155f9690c31f6a0e646bf8`. Original text remains in Git history; this file is the sole current normative owner of this section.

```architecture-graph
node: IRWIRE-001
kind: INVARIANT
status: IMPLEMENTED
title: IRWIRE-001 Semantic Binary Round Trip
track: FOUNDATION
artifact: docs/invariants/identity-occurrences.md
commit: 655f0a75
note: Source enforcement attributes are retained in prose; admission does not execute witnesses or establish independent validation.
```

### IRPAT-001 IR Identity Preservation During Consumptive Pattern Transformations

| Property | Value |
|----------|-------|
| **Intent** | Consumptive IR pattern compression must never corrupt the method-identity ownership relationship between a compressed `DefMethod` and the annotations that reference it. |
| **Invariant** | A consumptive IR pattern transformation must not consume a `DefMethod(M)` while leaving surviving IR operations that reference `M` without a valid representation/ownership relationship. If an M-referencing operation cannot be represented within the resulting `PatternOp`, the pattern must decline compression rather than consume the `DefMethod` and orphan the reference. The currently relevant M-referencing operation kinds are `DataFlow`, `SideEffect`, `ExecutionContext`, `ControlFlow`, and `Body` — but the invariant governs ANY future M-referencing op kind. This is an **IR transformation invariant**, not a rule specific to Angular, TypeScript, constructors, or RxJS. |
| **Enforcement** | `CompressingPatternRecognizer` declines CTOR/EMPTY_CTOR compression when an unrepresentable M-reference survives after the pattern's consumed span. Typed `PatternFacts` participate only as annotations; `Body` remains an unrepresentable reference that conservatively blocks compression. The Edit-fidelity regression asserts exact body text, exact absolute span endpoints, and `UnitTable` addressability after both production recognizers. |
| **Authority** | `src/ir/patterns/recognize/guards.rs`, `src/ir/patterns/recognize.rs`, and `src/tests/ir/regression_ctor_pattern_orphan.rs` |
| **Type** | ENFORCED (test) |
| **Gate** | `cargo test` |

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 416-428, original section `IRPAT-001`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `5630c13d0ac04f96625aef95b4b227e1e8498562f94fec2db94005ae8f43ed27`. Original text remains in Git history; this file is the sole current normative owner of this section.

```architecture-graph
node: IRPAT-001
kind: INVARIANT
status: IMPLEMENTED
title: IRPAT-001 IR Identity Preservation During Consumptive Pattern Transformations
track: FOUNDATION
artifact: docs/invariants/identity-occurrences.md
commit: 655f0a75
note: Source enforcement attributes are retained in prose; admission does not execute witnesses or establish independent validation.
```

### PATID-001 Method Identity Survives Pattern Recognition (F2)

| Property | Value |
|----------|-------|
| **Intent** | Pattern recognition classifies or summarizes a method; it must never erase the declaration identity that every downstream consumer depends on. |
| **Invariant** | No IR transformation may delete an identity-bearing method declaration while emitting another op that still semantically refers to that method. The identity-bearing method facts are `DefMethod`, its `Param*`, and its `Return`. A consumptive pattern classification is therefore ADDITIVE: it is emitted AFTER the retained declaration ops and reports them to the caller as `PatternMatch::retained` (`src/ir/patterns/recognize.rs`). Only genuinely redundant, non-identity ops may still be summarized (`Injects` for CTOR, typed `PatternFacts(OVERRIDE)` for OVERRIDE, and adjacent typed pattern-annotation runs the wrapper consumes). `MethodModifiers` remain authoritative and are retained when adjacent to a classified declaration; `Async` is independent execution metadata and never changes a Promise contract into an Observable contract. Reinventing the declaration downstream — for example teaching the hierarchical encoder to rebuild a method from a `PAT` payload — is explicitly NOT a valid alternative: the canonical declaration already exists upstream. This is language-agnostic and pattern-agnostic: it governs every present and future consumptive recognizer. |
| **Enforcement** | All seven consumptive recognizers report a `retained` count and `compress_merged` re-emits those ops unchanged before the `Pattern` op (`src/ir/patterns.rs`, `src/ir/patterns/recognize.rs`). Regressions: `src/tests/ir/pattern_identity.rs` (production-shape CTOR/PROMISE/OBSERVABLE/EMPTY_CTOR identity retention, the complete consumptive surface incl. OVERRIDE/GETTER/SETTER, the no-orphan-classification invariant across Low/Medium/High/Edit, and the structural compression-impact report) and `src/tests/ir/pattern_identity_downstream.rs` (hierarchical `MethodNode`, rendered `M` line, `UnitTable` addressing, semantic registration, and `Calls` caller identity). |
| **Authority** | `src/ir/patterns/recognize.rs` (`PatternMatch`, `MatcherResult`), `src/ir/patterns.rs` (`compress_merged`); regressions `src/tests/ir/pattern_identity.rs`, `src/tests/ir/pattern_identity_downstream.rs` |
| **Type** | ENFORCED (test) |
| **Gate** | `cargo test` |
| **Relationship to IRPAT-001** | IRPAT-001 is the decline rule that protects M-referencing annotations whose payload a `PatternOp` cannot represent. PATID-001 is the stronger, unconditional property that makes the identity survive regardless; IRPAT-001's conservative decline is retained unchanged (weakening it would newly compress shapes that have never been compressed — a separate compression-policy decision). |

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 429-442, original section `PATID-001`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `18a0369b89836ba88e597e835c25889b75f85fbb5f05ee986178daeb7c677dff`. Original text remains in Git history; this file is the sole current normative owner of this section.

```architecture-graph
node: PATID-001
kind: INVARIANT
status: IMPLEMENTED
title: PATID-001 Method Identity Survives Pattern Recognition (F2)
track: FOUNDATION
artifact: docs/invariants/identity-occurrences.md
commit: 655f0a75
note: Source enforcement attributes are retained in prose; admission does not execute witnesses or establish independent validation.
```

### IDX-001 Index-Encoding Invariant

| Property | Value |
|----------|-------|
| **Intent** | Index/search mechanics are implementation detail; entity identity is semantic. The two must not be conflated. |
| **Invariant** | Internal lookup-key encoding MUST NOT leak into semantic entity identity. `WorkspaceIndex` may derive lookup structures, but those encodings must not be reflected in `EntityRef.name`. Selector resolution is a distinct operation from generic name lookup — `resolve_selector()` performs literal lookup against the stored `HasSelector` object name rather than reconstructing an encoded key. |
| **Enforcement** | `src/tests/workspace/index.rs::resolve_selector_literal_forms_are_distinct` (proves literal lookup and that distinct selector forms resolve independently — no cross-form match, no bracket-encoding match). |
| **Authority** | `src/workspace/index.rs` (`resolve_selector`, `name_index`), `src/angular_meta/semantic.rs` (`HasSelector` construction) |
| **Type** | ENFORCED (test) |
| **Gate** | `cargo test` |

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 484-496, original section `IDX-001`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `1e3e0ed2334d7175f675777ff3be99f79c60ec624ab36e992b2d7846bd73d353`. Original text remains in Git history; this file is the sole current normative owner of this section.

```architecture-graph
node: IDX-001
kind: INVARIANT
status: IMPLEMENTED
title: IDX-001 Index-Encoding Invariant
track: FOUNDATION
artifact: docs/invariants/identity-occurrences.md
commit: 655f0a75
note: Source enforcement attributes are retained in prose; admission does not execute witnesses or establish independent validation.
```

### IDX-002 Edge Occurrence Identity Is Source-Occurrence Aware

| Property | Value |
|----------|-------|
| **Intent** | Two files may contain occurrences of the same semantic entity identity while asserting distinct edge evidence. The generic WorkspaceIndex edge layer must retain all of that evidence instead of collapsing equivalent-looking edges. |
| **Invariant** | Semantic entity identity remains `(domain, entity_type, name)` with the file excluded (Model C, unchanged). Edge OCCURRENCE identity is `(asserting source occurrence, relation, subject semantic identity, object semantic identity)`. A repeated extraction of the same triple by the same asserting file produces one indexed occurrence; the same triple asserted by a different file is a distinct occurrence and must never suppress or be suppressed. Consequences: `forward_edges` returns every asserting occurrence's evidence; `reverse_edges` counts every real consumer occurrence; `remove_file` drops exactly the occurrences the removed file asserted and never another file's evidence for the same triple; removal work stays proportional to the affected file (`file_edges`), never to the accumulated workspace. The rule is framework-agnostic — Angular, .NET, Spring, and future semantic layers inherit it, and no semantic layer may compensate for it locally. |
| **Enforcement** | `src/workspace/index/edges.rs` (`EdgeKey` includes the asserting file; occurrence-aware dedup in `add_edges`), `src/workspace/index/remove.rs` (`remove_file` prunes only `file_edges[file]` occurrences), `src/tests/workspace/index_edge_occurrence.rs` (RED-E1…RED-E5: identity, dedup, unique objects, union, forward/reverse evidence), `src/tests/workspace/index_edge_lifecycle.rs` (RED-E6…RED-E11: removal, recompilation, three-project collision, compile-order independence, unrelated names, dotnet-shaped cross-domain case), `src/tests/workspace/index.rs` (`same_edge_inserted_twice_is_indexed_once`, `edge_occurrence_dedup_preserves_cross_file_evidence`, `counters_reflect_insertion_and_dedup`), `src/tests/workspace/index_queries.rs` (`resolve_selector_unique_despite_cross_file_duplicate_selector`), `src/tests/workspace/index_performance.rs` (file-local removal work unchanged). |
| **Authority** | `src/workspace/index.rs`, `src/workspace/index/edges.rs`, `src/workspace/index/remove.rs` |
| **Type** | ENFORCED (test) |
| **Gate** | `cargo test` |

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 497-509, original section `IDX-002`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `83554b937e156dbe6ed74be3cfccba73c32be09f1e66042c1bdef7a8f719f1be`. Original text remains in Git history; this file is the sole current normative owner of this section.

```architecture-graph
node: IDX-002
kind: INVARIANT
status: IMPLEMENTED
title: IDX-002 Edge Occurrence Identity Is Source-Occurrence Aware
track: FOUNDATION
artifact: docs/invariants/identity-occurrences.md
commit: 655f0a75
note: Source enforcement attributes are retained in prose; admission does not execute witnesses or establish independent validation.
```

### WSC-012 Workspace Tuple Identity Is Not Exact Declaration Identity

**Status:** ENFORCED

The (domain, entity_type, name) tuple groups WorkspaceIndex occurrences.
Multiple declarations may share that tuple. Consumers needing exact
declaration/member/overload identity must establish it independently from
current source evidence.

**Enforcement:** occurrence-preserving WorkspaceIndex storage, declaration
authority, and language-specific resolvers.

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 764-775, original section `WSC-012`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `d571577764cff6af48894b87d7052dc5dd564ed26f9750291447b7342389a262`. Original text remains in Git history; this file is the sole current normative owner of this section.

```architecture-graph
node: WSC-012
kind: INVARIANT
status: IMPLEMENTED
title: WSC-012 Workspace Tuple Identity Is Not Exact Declaration Identity
track: FOUNDATION
artifact: docs/invariants/identity-occurrences.md
commit: 655f0a75
note: Source enforcement attributes are retained in prose; admission does not execute witnesses or establish independent validation.
```
