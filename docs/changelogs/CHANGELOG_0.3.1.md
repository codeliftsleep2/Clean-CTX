---

## [0.3.1] — 2026-08-23 — C-22 Canonical Source-Span Contract & Multi-Class Fix

### Added

#### C-22: Canonical Decorator/Annotation/Attribute-Inclusive Class Source Contract

- **Architectural invariant established:** `MetaLayerPass` derives class source spans from `PassContext.captures` (the canonical `CapEntry` capture identity) — NOT from `CoreOp::DefClass.name`. This eliminates the semantic corruption where `DefClass.name` carried full source text instead of a class name.
- **`class_source_from_capture()`** in `src/meta_util.rs` — trilingual backward scan for TS `@Name(...)`, Java `@Name`, C# `[Name]`. Non-decorated classes use the declaration-keyword byte as fallback (backward compatible).
- **`MetaLayer::enrich()`** now receives `class_captures: &[String]` directly, eliminating the `DefClass.name` round-trip that previously corrupted framework marker detection for Java and C# meta-layers.
- **Architectural debt resolved:** P0 design document (`docs/P0_SOURCE_SPAN_CONTRACT_DESIGN_2026-08-22.md`) marked as IMPLEMENTED with all 5 recommended steps complete.

#### R-43b: Multi-Class-Per-File Support & Per-Class Isolation

- **Cross-contamination fix:** Three architectural defects allowed markers to leak between classes in multi-class files:
  1. `find_class_source_start` backward scan could walk past preceding class's closing `}`
  2. `find_class_body_open` only scanned for `class ` keyword (missing `interface`/`enum`/`record`/`struct`)
  3. `MetaLayer::enrich` extracted `class_captures` from `DefClass.name` (a semantic round-trip)
- **Class boundary guard:** `find_class_source_start` now stops at a closing `}` — `}` is never a valid annotation prefix.
- **Type-aware body scanning:** `find_class_body_open` handles all type keywords (`class`, `interface`, `enum`, `record`, `struct`) for Spring Boot and Angular.
- **Per-class metadata invariant:** A meta-layer may inspect only the exact source span belonging to the type it is enriching. It must never infer ownership from neighboring or whole-file text.

#### Multi-Class Verification Tests (9 new)

- **Java/Spring Boot (3 tests):** `MultiClassFixture.java` (6 classes) — verifies `@RestController`/`@Service` markers don't leak to `AppConfig` or `HealthController` at Low/Medium/High.
- **TypeScript/Angular (3 tests):** `MultiClassFixture.ts` (5 classes) — verifies `@Component`/`@Injectable` markers don't cross-contaminate at Low/Medium/High.
- **C#/.NET (3 tests):** `MultiClassFixture.cs` (5 classes) — verifies `[ApiController]` markers don't leak to `NotificationHub` or `InventoryDbContext` at Low/Medium/High.
- Each test asserts document-order preservation and per-class marker isolation through the production `compress_text` pipeline.

### Fixed

- **UTF-8 encoding corruption** in 4 test assertions (`src/tests/compression/pipeline.rs`): mojibake (corrupt `0xCE+0xA6`→`Φ`, `0xCE+0xB1`→`α`, `0xC2+0xA7`→`§`) caused false-negative failures in `compress_text_with_aliases_medium_fidelity`, `compress_text_with_aliases_high_fidelity`, `compress_text_emits_angular_component_markers`, and `compress_text_emits_angular_injectable_markers`. Restored via `git checkout` and re-appended with explicit UTF-8 encoding.

### Changed

- `docs/ARCHITECTURAL_INVARIANTS.md` — C-22 added as ENFORCED invariant.
- `docs/ARCHITECTURE_OVERVIEW.md` — System diagram now shows Spring Boot and .NET meta-layer boxes plus the `LayerRegistry` dispatch pattern.
- `docs/COMPILER_IR.md` — §3.1 pipeline description updated to include `DotNetMetaLayer`, C-22 class-captures derivation, and multi-class invariant. §9 meta-layer table expanded with .NET and full Angular ecosystem markers.
- `docs/P0_SOURCE_SPAN_CONTRACT_DESIGN_2026-08-22.md` — Status changed from DESIGN ONLY to IMPLEMENTED; Executive Summary table updated to reflect all three meta-layers producing markers on both IR and text paths.
- `extradocs/FAANG_AUDIT_FINDINGS.md` — P0-4 (dual meta-layer systems) marked as resolved.
- All 406 existing tests remain green.

### Version history

| Version | Date | Highlights |
|---------|------|------------|
| 0.3.1 | 2026-08-23 | **Architectural fixes.** C-22 canonical source-span contract, R-43b multi-class per-class isolation, class boundary guards, type-aware body scanning, 9 multi-class verification tests |

---

