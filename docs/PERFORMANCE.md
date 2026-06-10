# Clean-CTX — Performance Guide

**Last updated:** 2026-06-07

---

## Compression Savings

### Measured Results (Single Pass)

| File | Raw Tokens | Low (Retained) | Low Savings | Medium (Retained) | Medium Savings | High (Retained) | High Savings |
|------|-----------:|---------------:|------------:|------------------:|---------------:|----------------:|-------------:|
| `sample_service.ts` (32 lines) | 193 | 36 | **81.35%** | 75 | **61.14%** | 75 | **61.14%** |
| `LargeService.ts` (~400 lines) | 2,957 | 119 | **95.98%** | 476 | **83.90%** | 499 | **83.12%** |
| `UserManagementService.ts` (~440 lines) | 3,912 | 155 | **96.04%** | — | — | — | — |

**Key insight:** Larger files compress significantly better because structural overhead (class headers, method signatures, imports) is amortized across more methods. A service with 20+ methods at Low fidelity will consistently exceed 95% savings.

### Savings by Code Structure

| Structure | Low | Medium | High |
|-----------|-----|--------|------|
| Class declaration | `$c Name` → 1 token | `$c Name` → 1 token | `class Name` → 2 tokens |
| Method signature | `name(types):type` → 3-6 tokens | `name(types):type` + `$a` if async | Full keywords preserved |
| Field | `name: type` → 2 tokens (or suppressed) | `name: type` → 2 tokens | `public readonly name: type` → 4+ tokens |
| Import | `$im path` → 2 tokens | `$im path` → 2 tokens | `import { X } from 'path'` → 5+ tokens |

---

## 🧪 Edit Session Benchmarks (Delta Transport)

We simulated a realistic developer editing session on `UserManagementService.ts` (~440 lines) applying 50 sequential edits across 5 categories. The simulation was run at **all three fidelity levels** to compare delta transport performance across compression settings.

### Cross-Fidelity Comparison (50-Edit Cumulative)

| Fidelity | Raw | ReComp | Delta | ReSav% | DelSav% | Delta vs ReComp |
|----------|----:|------:|------:|------:|-------:|:---------------:|
| **Low** | 227,310 | 7,823 | 8,490 | 96.6% | 96.3% | +8.5% overhead |
| **Medium** | 227,310 | 37,338 | 18,287 | 83.6% | **92.0%** | **−51.0%** cheaper |
| **High** | 227,310 | 48,556 | 22,955 | 78.6% | 89.9% | **−52.7%** cheaper |

### Per-Fidelity Details

<details>
<summary><b>Low Fidelity</b> (max compression, daily-use default) — click to expand</summary>

| Pipeline | Total Tokens | Avg per Edit | Savings vs Raw |
|----------|-------------:|:------------:|:--------------:|
| Raw (no compression) | **227,310** | 4,546 | — |
| Clean-CTX full recompression | **7,823** | 156 | **96.6%** |
| Clean-CTX + delta transport | **8,490** | 170 | **96.3%** |

**Per-Edit Category Breakdown:**

| Category | Edits | Raw | ReComp | Delta | ReSav% | DelSav% | Best Delta Edit | Worst Delta Edit |
|----------|-------|----:|------:|------:|------:|-------:|:---------------:|:----------------:|
| Small changes | 1-10 | 39,202 | 1,545 | 988 | 96.1% | 97.5% | #4 (100%) | #3/#8 (91.5%) |
| Method-level | 11-20 | 41,370 | 1,498 | 1,580 | 96.4% | 96.2% | #17 (100%) | #13 (91.3%) |
| Structural | 21-30 | 44,610 | 1,587 | 1,740 | 96.4% | 96.1% | #21 (99.2%) | #29 (91.5%) |
| Cross-method | 31-40 | 49,598 | 1,383 | 2,436 | 97.2% | 95.1% | #33/#39 (100%) | #36 (90.8%) |
| Refactor | 41-50 | 52,530 | 1,810 | 1,746 | 96.6% | 96.7% | #42 (99.4%) | #41 (90.8%) |
</details>

<details>
<summary><b>Medium Fidelity</b> (balanced, preserves async/exports) — click to expand</summary>

| Pipeline | Total Tokens | Avg per Edit | Savings vs Raw |
|----------|-------------:|:------------:|:--------------:|
| Raw (no compression) | **227,310** | 4,546 | — |
| Clean-CTX full recompression | **37,338** | 747 | **83.6%** |
| Clean-CTX + delta transport | **18,287** | 366 | **92.0%** |

**Per-Edit Category Breakdown:**

| Category | Edits | ReSav% | DelSav% | Delta vs ReComp |
|----------|-------|------:|-------:|:---------------:|
| Small changes | 1-10 | 80.7% | 96.1% | −79.7% cheaper |
| Method-level | 11-20 | 82.3% | 92.8% | −58.9% cheaper |
| Structural | 21-30 | 82.7% | 91.6% | −51.2% cheaper |
| Cross-method | 31-40 | 87.2% | 89.5% | −18.0% cheaper |
| Refactor | 41-50 | 83.9% | 90.9% | −43.2% cheaper |
</details>

<details>
<summary><b>High Fidelity</b> (full detail, best for code review) — click to expand</summary>

| Pipeline | Total Tokens | Avg per Edit | Savings vs Raw |
|----------|-------------:|:------------:|:--------------:|
| Raw (no compression) | **227,310** | 4,546 | — |
| Clean-CTX full recompression | **48,556** | 971 | **78.6%** |
| Clean-CTX + delta transport | **22,955** | 459 | **89.9%** |

**Per-Edit Category Breakdown:**

| Category | Edits | ReSav% | DelSav% | Delta vs ReComp |
|----------|-------|------:|-------:|:---------------:|
| Small changes | 1-10 | 75.9% | 95.5% | −81.4% cheaper |
| Method-level | 11-20 | 77.2% | 91.2% | −61.3% cheaper |
| Structural | 21-30 | 77.2% | 89.4% | −53.8% cheaper |
| Cross-method | 31-40 | 83.2% | 86.5% | −20.0% cheaper |
| Refactor | 41-50 | 78.7% | 88.3% | −45.0% cheaper |
</details>

### Key Performance Metrics

| Metric | Low | Medium | High |
|--------|:---:|:------:|:----:|
| Break-even edit | #1 | #1 | #1 |
| Single-pass compression ratio | **25.2×** | 5.2× | 4.1× |
| Best delta savings vs raw | 100.0% (edit #4) | 100.0% (edit #4) | 100.0% (edit #4) |
| Worst delta savings vs raw | 90.8% (edit #41) | 86.5% (edit #36) | 82.0% (edit #36) |
| Delta vs ReComp advantage | +8.5% overhead | **−51.0% cheaper** | **−52.7% cheaper** |
| Simulation runtime | 5.65s | 6.87s | 6.27s |

### Key Findings

1. **Low fidelity** (daily default): Delta delivers 96.3% savings vs raw, within 0.3 pp of recompression. The fixed delta envelope cost (~80 chars) adds +8.5% overhead because compressed output is so tiny (avg 156 tokens).
2. **Medium and High fidelities**: Delta transport is **actually cheaper** than full recompression — by 51% and 52.7% respectively. This is because larger compressed outputs (5–6× bigger than Low) make the line-level delta significantly smaller than re-running the full compression pipeline.
3. **Delta breaks even immediately** at all fidelities — cumulative delta cost ≤ full recompression from Edit #1.
4. **Higher delta overhead on cross-method edits**: The "Cross-method" category shows the smallest delta advantage (−18% to −20%) because these edits restructure code across many methods, producing large deltas relative to the compressed baseline.

### Run It Yourself

```bash
# Low fidelity (the fifty_edit_simulation example)
cargo run --example fifty_edit_simulation

# All three fidelities (cross-fidelity comparison)
cargo run --example fidelity_comparison
```
Both examples generate full per-edit tables with raw/recompression/delta costs and cumulative totals.

---

## Delta Pipeline Performance

The delta pipeline's performance depends on the **edit size ratio** — how many lines changed vs. how many stayed the same:

| Edit Size Ratio | Full Recompression | Delta Transport | Delta Advantage |
|:---------------:|:------------------:|:---------------:|:---------------:|
| 0% (no change) | ~80 ms (cached: ~50 µs) | ~50 µs | **Match** |
| 1-5% (small edit) | ~80 ms | ~40 ms | ~2× faster |
| 5-20% (medium edit) | ~80 ms | ~45 ms | ~1.8× faster |
| >20% (large edit) | ~80 ms | ~55 ms | ~1.5× faster |

Delta transport avoids the full parse-compress-BPE pipeline by computing line-level diffs between compressed body snapshots. For small edits, this is significantly faster. For large restructures, full recompression may be preferred (the delta pipeline can issue a full snapshot as a fallback).

### Fidelity-Dependent Delta Trade-Off

The delta overhead vs full recompression is fidelity-dependent:

| Fidelity | Compressed Size | Delta Overhead | Why? |
|----------|:---------------:|:--------------:|------|
| **Low** | ~155 tokens (tiny) | +8.5% | Fixed delta envelope (~80 chars) is proportionally large |
| **Medium** | ~747 tokens | **−51% cheaper** | Delta lines < re-compressed lines |
| **High** | ~971 tokens | **−52.7% cheaper** | Delta lines < re-compressed lines |

**Practical guidance:**
- If you use **Low fidelity** and the compressed output is already tiny, full recompression's overhead is negligible — delta doesn't hurt but doesn't help much either
- If you use **Medium or High fidelity**, delta transport provides a significant additional savings on top of the base compression
- For maximum edit-session efficiency, the pipeline could auto-detect fidelity and choose the optimal transport strategy

---

## Caching Performance

### Cache Hit Ratio

The `LocalStateCache` provides two tiers of caching:

1. **Content-hash registry** — SHA-256 of file bytes
   - Cache miss: ~80 ms for a 1 MB TypeScript file (parse + compress + BPE)
   - Cache hit: ~50 µs (hash comparison only)
   - **Speedup: ~1,600x**

2. **Raw-token count side-table** — skip BPE re-encode on cache hit
   - Without side-table: ~80 ms (includes BPE encode of full source)
   - With side-table: ~5 ms (retrieve stored count + format output)
   - **Speedup: ~16x** on top of content-hash cache

### Baseline Snapshot Cache (diff_code_context)

| Scenario | First Call | Subsequent Call (no change) | Subsequent Call (with change) |
|----------|-----------:|----------------------------:|------------------------------:|
| Time | ~80 ms (full parse + snapshot) | ~50 µs (hash match → skip) | ~40 ms (parse + diff + rotate) |

---

## File Size Limits

| Guard | Limit | Behavior |
|-------|------|----------|
| `MAX_LINE_BYTES` | 16 MB | Rejects oversize JSON-RPC request with `-32600` |
| `MAX_FILE_BYTES` | 10 MB | Rejects oversize source file with `FileTooLarge` error |
| `MAX_WALK_DEPTH` | 32 levels | Stops directory recursion at depth 32 |

---

## Workspace Compression Scalability

### Current (single-threaded)

Measured on an AMD Ryzen 7 7840U (8 cores, 16 threads):

| Files | Time | Memory (RSS) |
|------:|-----:|-------------:|
| 10 | ~0.8 s | ~35 MB |
| 100 | ~8 s | ~50 MB |
| 1,000 | ~80 s | ~120 MB |
| 10,000 | ~N/A (deferred) | ~N/A (deferred) |

### Future (with F-19 streaming + F-20 rayon)

Estimated targets for the same hardware:

| Files | Time (estimated) | Memory (estimated) |
|------:|-----------------:|-------------------:|
| 10 | ~0.3 s | ~35 MB |
| 100 | ~2 s | ~40 MB |
| 1,000 | ~15 s | ~60 MB |
| 10,000 | ~120 s | ~150 MB |

---

## BPE Token Counting Performance

The cl100k BPE engine (via `tiktoken-rs`) is loaded once at server startup via `OnceLock`:

| Operation | Time |
|-----------|-----:|
| First load (BPE data init) | ~200 ms |
| `encode_with_special_tokens("")` | ~2 µs |
| `encode_with_special_tokens(1 KB source)` | ~50 µs |
| `encode_with_special_tokens(100 KB source)` | ~3 ms |
| `encode_with_special_tokens(1 MB source)` | ~25 ms |

---

## Decompressor Performance

| Operation | Before F-15 | After F-15 | Speedup |
|-----------|------------:|-----------:|--------:|
| Decompress 1,000 lines | ~12 ms | ~0.8 ms | **15x** |

**Why:** F-15 precomputes the sorted opcode list in `parse()` instead of re-sorting inside the per-line loop of `decompress()`. This changes `O(L × N log N)` to `O(L × N)` where L = line count and N = opcode count (34 primitives + custom).

---

## Memory Profile

| Component | Memory |
|-----------|--------|
| BPE engine (cl100k) | ~4 MB (shared, loaded once) |
| Tree-sitter parser (TS) | ~1.5 MB per instance |
| Tree-sitter parser (C#) | ~1.5 MB per instance |
| Tree-sitter parser (HTML) | ~1.5 MB per instance (Phase 2) |
| Typical compressed output (100 KB file) | ~2-5 KB |
| `PathDictionary` (1,000 entries) | ~150 KB |
| `SymbolDictionary` (100 entries) | ~8 KB |
| `LocalStateCache` (1,000 entries) | ~200 KB |

---

## Performance Optimization Checklist

If compression is slower than expected:

1. **Check cache hits** — identical files should compress in <1 ms on repeat calls
2. **Use exclusions** — add `node_modules`, `dist`, `build/` to `.clean-ctx.json` `exclude_patterns`
3. **Prefer `compress_code_context` over `compress_workspace`** for single files
4. **Use the lowest acceptable fidelity** — Low fidelity strips more content → faster compression
5. **Avoid very large files** — files >10 MB are rejected; files >1 MB are slow to BPE-encode
6. **Restart the server periodically** — the cache grows unboundedly within a session

---

## Microbenchmarks

```bash
# Run all tests (includes performance-sensitive tests)
cargo test

# Specific performance tests
cargo test bpe_returns_same_pointer_repeatedly
cargo test compress_file_cache_hit_returns_notice
cargo test decompress_with_precomputed_opcodes_matches_expected
cargo test workspace_shares_aliases_with_per_file_tool
```

---

## Deferred Performance Improvements

| Issue | Description | Planned For |
|-------|-------------|-------------|
| F-19 | Streaming workspace walk (replace collect-then-sort) | Future release |
| F-20 | Rayon parallelization for `compress_workspace` | Future release (blocked by tree-sitter `!Send`) |
| — | IR-level delta pipeline integration with MCP `delta_code_context` tool | Current release |
| — | Text-level delta auto-detection: switch between full recompression and delta based on edit size ratio | Future release |

---

## Angular Meta-Layer Bundling (Phase 2)

The Phase 2 bundling pass adds **zero overhead** to non-Angular workspaces. Angular workspaces pay a small cost only during `compress_workspace`:

| Operation | Time (estimated) | Notes |
|-----------|-----------------|-------|
| Triplet resolution | ~0.01 ms per component | Filesystem `is_file()` check |
| Template shape extraction | ~0.1 ms per template | tree-sitter-html parse + walk |
| Style shape extraction | ~0.05 ms per stylesheet | Byte-level scanner |
| `§ΦMAP` footer formatting | ~0.01 ms | BTreeMap iteration |

**Token savings example:** A workspace with 5 Angular components (each with `.html` + `.scss`) would have raw HTML/SCSS files totaling ~10,000 tokens. The bundled output replaces this with 5 one-line shape summaries (~50 tokens) — a **~99.5% reduction** for the template/style content.

### Bundle output format

```text
// ===== Φ1: user-card.component =====
// files: α1, α2, α3
// Φtpl:div,h2,p,button,app-avatar [ngIf] [ngFor] [(ngModel)] {{}}x4 (click)
// Φsty:.card,.card-text,.btn-primary $primary-color,$card-padding @include
```
