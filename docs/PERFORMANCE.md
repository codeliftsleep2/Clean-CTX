# Clean-CTX — Performance Guide

**Last updated:** 2026-06-07

---

## Compression Savings

### Measured Results

| File | Raw Tokens | Low (Retained) | Low Savings | Medium (Retained) | Medium Savings | High (Retained) | High Savings |
|------|-----------:|---------------:|------------:|------------------:|---------------:|----------------:|-------------:|
| `sample_service.ts` (32 lines) | 193 | 36 | **81.35%** | 75 | **61.14%** | 75 | **61.14%** |
| `LargeService.ts` (~400 lines) | 2,957 | 119 | **95.98%** | 476 | **83.90%** | 499 | **83.12%** |

**Key insight:** Larger files compress significantly better because structural overhead (class headers, method signatures, imports) is amortized across more methods. A service with 20+ methods at Low fidelity will consistently exceed 95% savings.

### Savings by Code Structure

| Structure | Low | Medium | High |
|-----------|-----|--------|------|
| Class declaration | `$c Name` → 1 token | `$c Name` → 1 token | `class Name` → 2 tokens |
| Method signature | `name(types):type` → 3-6 tokens | `name(types):type` + `$a` if async | Full keywords preserved |
| Field | `name: type` → 2 tokens (or suppressed) | `name: type` → 2 tokens | `public readonly name: type` → 4+ tokens |
| Import | `$im path` → 2 tokens | `$im path` → 2 tokens | `import { X } from 'path'` → 5+ tokens |

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
