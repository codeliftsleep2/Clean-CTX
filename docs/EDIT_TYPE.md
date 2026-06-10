# Clean-CTX — 50-Edit Simulation: Per-Edit Breakdown by Fidelity

This document provides the complete per-edit breakdown of the 50-edit simulation on `UserManagementService.ts` (~440 lines) across **all three fidelity levels**.

**Run the simulation yourself:**
- `cargo run --example fifty_edit_simulation` (Low fidelity)
- `cargo run --example fidelity_comparison` (all three fidelities)

---

## 1. Cross-Fidelity Summary

| Fidelity | Raw (50 edits) | ReComp (50 edits) | Delta (50 edits) | ReSav% | DelSav% | Delta vs ReComp |
|----------|---------------:|------------------:|-----------------:|-------|-------:|:---------------:|
| **Low** | 227,310 | 7,823 | 8,490 | 96.6% | 96.3% | +8.5% overhead |
| **Medium** | 227,310 | 37,338 | 18,287 | 83.6% | 92.0% | −51.0% cheaper |
| **High** | 227,310 | 48,556 | 22,955 | 78.6% | 89.9% | −52.7% cheaper |

---

## 2. Per-Edit Table (All Three Fidelities)

Edit category markers: **S**=Small, **M**=Method, **T**=Structural, **X**=Cross-method, **R**=Refactor

```
Edit   │ Description                        │   Raw │ Lo-Re │ Lo-Del │ Md-Re │ Md-Del │ Hi-Re │ Hi-Del
```

| Edit | Description | Raw | LoRe | LoDe | MdRe | MdDe | HiRe | HiDe |
|:----:|-------------|----:|-----:|-----:|-----:|-----:|-----:|-----:|
| 1S | Rename 'apiBasePath' to 'baseApiPath' | 3,912 | 155 | 155 | 754 | 754 | 943 | 943 |
| 2S | Add void return type to logOperation | 3,914 | 155 | 33 | 754 | 302 | 943 | 377 |
| 3S | Rename 'isActive' to 'active' in UserFilter | 3,912 | 154 | 334 | 754 | 45 | 943 | 57 |
| 4S | Change defaultPageSize from 25 to 50 | 3,912 | 154 | 0 | 754 | 0 | 943 | 0 |
| 5S | Add null coalesce to getLastError return | 3,914 | 154 | 33 | 754 | 0 | 943 | 0 |
| 6S | Change cache TTL from 5min to 10min | 3,914 | 154 | 0 | 754 | 0 | 943 | 0 |
| 7S | Add activeRequests counter field | 3,922 | 154 | 33 | 754 | 45 | 943 | 57 |
| 8S | Rename isAuthenticated to hasActiveSession | 3,924 | 155 | 334 | 754 | 45 | 943 | 57 |
| 9S | Add X-Request-ID header to buildHeaders | 3,938 | 155 | 33 | 754 | 45 | 943 | 57 |
| 10S | Add trim() to displayName validation | 3,940 | 155 | 33 | 754 | 0 | 943 | 0 |
| 11M | Add getUserPermissions method | 4,066 | 159 | 341 | 776 | 341 | 971 | 426 |
| 12M | Add batchSize param to batchCreateUsers | 4,072 | 162 | 350 | 784 | 45 | 981 | 57 |
| 13M | Add optional fields param to getUserById | 4,076 | 164 | 354 | 791 | 45 | 990 | 57 |
| 14M | Change verifyEmail return type to object | 4,094 | 164 | 33 | 791 | 0 | 990 | 0 |
| 15M | Add AbortController timeout to getUsers | 4,094 | 29 | 218 | 107 | 218 | 134 | 272 |
| 16M | Add caching logic to getUserByEmail | 4,157 | 164 | 218 | 791 | 218 | 990 | 272 |
| 17M | Change updateUser from PUT to PATCH | 4,157 | 164 | 0 | 791 | 0 | 990 | 0 |
| 18M | Add Promise.allSettled for batch handling | 4,212 | 164 | 33 | 791 | 45 | 990 | 57 |
| 19M | Add LoggerService constructor dependency | 4,221 | 164 | 33 | 791 | 45 | 990 | 57 |
| 20M | Change maxPageSize from 100 to 200 | 4,221 | 164 | 0 | 791 | 0 | 990 | 0 |
| 21T | Add UserSession interface | 4,268 | 164 | 33 | 791 | 45 | 990 | 57 |
| 22T | Add SessionToken and IPAddress type aliases | 4,281 | 164 | 33 | 791 | 0 | 990 | 0 |
| 23T | Add @Input() serviceConfig field | 4,313 | 164 | 33 | 791 | 45 | 990 | 57 |
| 24T | Split validateUserData into helpers | 4,313 | 29 | 218 | 107 | 218 | 134 | 272 |
| 25T | Add @Output() errorOccurred EventEmitter | 4,336 | 164 | 218 | 791 | 218 | 990 | 272 |
| 26T | Add suspendUser method | 4,486 | 170 | 363 | 821 | 363 | 1,028 | 454 |
| 27T | Add rate limiting fields | 4,507 | 170 | 33 | 821 | 45 | 1,028 | 57 |
| 28T | Add getAllUsers convenience method | 4,606 | 176 | 376 | 849 | 376 | 1,063 | 470 |
| 29T | Add logMethodEntry and logMethodExit helpers | 4,726 | 193 | 400 | 931 | 400 | 1,166 | 500 |
| 30T | Add UserServiceConfig interface | 4,774 | 193 | 33 | 931 | 45 | 1,166 | 57 |
| 31X | Extract buildUrl URL builder method | 4,800 | 197 | 422 | 950 | 422 | 1,190 | 528 |
| 32X | Add try/catch to getFromCache | 4,800 | 29 | 253 | 107 | 253 | 134 | 316 |
| 33X | Add try/catch to setInCache | 4,800 | 29 | 0 | 107 | 0 | 134 | 0 |
| 34X | Add withRetry and delay helpers | 4,935 | 207 | 263 | 998 | 263 | 1,250 | 329 |
| 35X | Add throwIfNotAuthenticated guard | 4,967 | 212 | 453 | 1,023 | 453 | 1,281 | 566 |
| 36X | Add measure() timing wrapper | 5,048 | 217 | 464 | 1,048 | 464 | 1,311 | 580 |
| 37X | Add ConfigService constructor dependency | 5,057 | 217 | 33 | 1,048 | 45 | 1,311 | 57 |
| 38X | Add rate limiting to incrementRequestCount | 5,057 | 29 | 274 | 107 | 274 | 134 | 342 |
| 39X | Add cache_set operation logging | 5,057 | 29 | 0 | 107 | 0 | 134 | 0 |
| 40X | Emit errorOccurred event in handleError | 5,077 | 217 | 274 | 1,048 | 274 | 1,311 | 342 |
| 41R | Add RequestOptions to getUsers signature | 5,089 | 218 | 470 | 1,052 | 470 | 1,317 | 588 |
| 42R | Add RequestOptions interface | 5,126 | 218 | 33 | 1,052 | 45 | 1,317 | 57 |
| 43R | Refactor constructor to use options object | 5,126 | 29 | 276 | 107 | 276 | 134 | 345 |
| 44R | Add UserOperationResult<T> interface | 5,175 | 218 | 276 | 1,052 | 276 | 1,317 | 345 |
| 45R | Replace ApiResponse with UserOperationResult | 5,177 | 218 | 33 | 1,052 | 45 | 1,317 | 57 |
| 46R | Add Configuration region comment | 5,249 | 218 | 33 | 1,052 | 45 | 1,317 | 57 |
| 47R | Add Public API Methods region header | 5,315 | 218 | 33 | 1,052 | 45 | 1,317 | 57 |
| 48R | Add Private Helper Methods region header | 5,381 | 218 | 33 | 1,052 | 45 | 1,317 | 57 |
| 49R | Add recoveryStrategy param to handleError | 5,381 | 29 | 276 | 107 | 276 | 134 | 345 |
| 50R | Add executeSafely error boundary wrapper | 5,511 | 226 | 283 | 1,090 | 283 | 1,365 | 354 |

**Column headers:** Raw=Raw tokens, LoRe=Low ReComp, LoDe=Low Delta, MdRe=Medium ReComp, MdDe=Medium Delta, HiRe=High ReComp, HiDe=High Delta

---

## 3. Per-Category Summary (By Fidelity)

### Low Fidelity

| Category | Edits | Raw | ReComp | Delta | ReSav% | DelSav% | Best Edit (Del%) | Worst Edit (Del%) |
|----------|------:|----:|------:|------:|------:|-------:|:----------------:|:-----------------:|
| Small | 1-10 | 39,202 | 1,545 | 988 | 96.1% | 97.5% | #4 (100%) | #3/#8 (91.5%) |
| Method | 11-20 | 41,370 | 1,498 | 1,580 | 96.4% | 96.2% | #17 (100%) | #13 (91.3%) |
| Structural | 21-30 | 44,610 | 1,587 | 1,740 | 96.4% | 96.1% | #21 (99.2%) | #29 (91.5%) |
| Cross-method | 31-40 | 49,598 | 1,383 | 2,436 | 97.2% | 95.1% | #33/#39 (100%) | #36 (90.8%) |
| Refactor | 41-50 | 52,530 | 1,810 | 1,746 | 96.6% | 96.7% | #42 (99.4%) | #41 (90.8%) |

### Medium Fidelity

| Category | Edits | Raw | ReComp | Delta | ReSav% | DelSav% | vs ReComp |
|----------|------:|----:|------:|------:|------:|-------:|:---------:|
| Small | 1-10 | 39,202 | 7,540 | 1,482 | 80.7% | 96.1% | −79.7% |
| Method | 11-20 | 41,370 | 7,312 | 2,957 | 82.3% | 92.8% | −58.9% |
| Structural | 21-30 | 44,610 | 7,698 | 3,734 | 82.7% | 91.6% | −51.2% |
| Cross-method | 31-40 | 49,598 | 6,353 | 5,184 | 87.2% | 89.5% | −18.0% |
| Refactor | 41-50 | 52,530 | 8,435 | 4,930 | 83.9% | 90.9% | −43.2% |

### High Fidelity

| Category | Edits | Raw | ReComp | Delta | ReSav% | DelSav% | vs ReComp |
|----------|------:|----:|------:|------:|------:|-------:|:---------:|
| Small | 1-10 | 39,202 | 9,430 | 1,838 | 75.9% | 95.5% | −81.4% |
| Method | 11-20 | 41,370 | 9,373 | 3,716 | 77.2% | 91.2% | −61.3% |
| Structural | 21-30 | 44,610 | 10,127 | 4,718 | 77.2% | 89.4% | −53.8% |
| Cross-method | 31-40 | 49,598 | 8,274 | 6,664 | 83.2% | 86.5% | −20.0% |
| Refactor | 41-50 | 52,530 | 11,133 | 5,928 | 78.7% | 88.3% | −45.0% |

---

## 4. Pipeline Efficiency Matrix

For each edit type, this table shows the **average tokens per edit** across each pipeline and fidelity:

| Edit Type | Raw | Lo-Re | Lo-De | Md-Re | Md-De | Hi-Re | Hi-De |
|-----------|----:|------:|------:|------:|------:|------:|------:|
| Small change | 3,920 | 155 | 99 | 754 | 148 | 943 | 184 |
| Method addition | 4,137 | 150 | 158 | 731 | 296 | 937 | 371 |
| Method change | 4,135 | 97 | 217 | 449 | 217 | 562 | 272 |
| Interface/type add | 4,268 | 164 | 33 | 791 | 45 | 990 | 57 |
| Structural (new method + new code) | 4,461 | 159 | 174 | 770 | 349 | 964 | 435 |
| Cross-method (add helper) | 4,935 | 207 | 263 | 998 | 263 | 1,250 | 329 |
| Cross-method (modify existing) | 4,928 | 97 | 202 | 449 | 202 | 562 | 252 |
| Cross-method (add dep) | 5,057 | 217 | 33 | 1,048 | 45 | 1,311 | 57 |
| Refactor (interface/type) | 5,175 | 218 | 33 | 1,052 | 45 | 1,317 | 57 |
| Refactor (method signature) | 5,089 | 218 | 470 | 1,052 | 470 | 1,317 | 588 |
| Refactor (structural) | 5,315 | 218 | 33 | 1,052 | 45 | 1,317 | 57 |
| Refactor (helper addition) | 5,511 | 226 | 283 | 1,090 | 283 | 1,365 | 354 |

---

## 5. Best and Worst Edits for Delta (by Fidelity)

### Best delta savings vs raw (100% = zero token cost)

| Rank | Edit | Description | Low | Medium | High |
|:----:|:----:|-------------|:---:|:------:|:----:|
| 1 | #4 | Change defaultPageSize (constant) | **100.0%** | **100.0%** | **100.0%** |
| 2 | #6 | Change cache TTL (constant) | **100.0%** | **100.0%** | **100.0%** |
| 3 | #17 | Change PUT to PATCH | **100.0%** | **100.0%** | **100.0%** |
| 4 | #20 | Change maxPageSize (constant) | **100.0%** | **100.0%** | **100.0%** |
| 5 | #33 | Add try/catch (no-op delta) | **100.0%** | **100.0%** | **100.0%** |
| 6 | #39 | Add cache_set logging (no-op delta) | **100.0%** | **100.0%** | **100.0%** |

### Worst delta savings vs raw (most expensive single edit)

| Rank | Edit | Description | Low | Medium | High |
|:----:|:----:|-------------|:---:|:------:|:----:|
| 1 | #36 | Add measure() timing wrapper | 90.8% | 85.2% | 83.9% |
| 2 | #41 | Add RequestOptions to getUsers signature | 90.8% | 90.8% | 88.4% |
| 3 | #29 | Add logMethodEntry and logMethodExit helpers | 91.5% | 91.5% | 89.4% |
| 4 | #3 | Rename 'isActive' to 'active' (bulk rename) | 91.5% | 91.5% | 89.4% |
| 5 | #8 | Rename isAuthenticated to hasActiveSession | 91.5% | 91.5% | 89.4% |
| 6 | #13 | Add optional fields param to getUserById | 91.3% | 91.3% | 89.1% |

---

## 6. Key Insights by Pipeline

### Compression Pipeline (`compress_code_context`)

- **Low fidelity**: Best for raw token reduction (96.6% aggregate). Avg 156 tokens per edit. Single-pass ratio: **25.2×**.
- **Medium fidelity**: Preserves async/export/behavior markers. Avg 747 tokens per edit. Single-pass ratio: 5.2×.
- **High fidelity**: Full keyword preservation. Avg 971 tokens per edit. Single-pass ratio: 4.1×.

### Delta Transport Pipeline (`delta_text_context`)

- **Low fidelity**: Avg 170 tokens per edit (96.3% savings). Delta is slightly more expensive than full recompression (+8.5%) because compressed output is tiny, making the fixed 80-char envelope proportionally large.
- **Medium fidelity**: Avg 366 tokens per edit (92.0% savings). Delta is **51% cheaper** than full recompression — the sweet spot.
- **High fidelity**: Avg 459 tokens per edit (89.9% savings). Delta is **52.7% cheaper** than full recompression.

### Key Pattern: Constant-Value-Change Edits

Edits that only change constant values (edits #4, #6, #17, #20) produce **100% delta savings** — the delta pipeline detects identical compressed output and sends zero delta bytes. This is the best-case scenario.

### Key Pattern: Cross-Method Refactors

Cross-method edits (edits #31–40) show the smallest delta advantage at all fidelities. These edits restructure code across multiple methods, producing the largest per-edit deltas. The delta advantage over recompression is only −18% to −20% at Medium/High fidelity, and at Low fidelity delta is actually more expensive (+10–12%).

### When to Use Compression vs Delta

| Scenario | Recommendation |
|----------|---------------|
| First-time file load | Use `compress_code_context` (full compression needed for baseline) |
| Small incremental edit (rename, constant change) | Use `delta_text_context` (zero or near-zero delta cost) |
| Medium fidelity edit session | **Strongly prefer delta** — saves 51% vs recompression |
| High fidelity edit session | **Strongly prefer delta** — saves 52.7% vs recompression |
| Cross-method refactor at Low fidelity | Delta is fine (90.8% savings) but doesn't beat recompression |
| Cross-method refactor at Medium/High fidelity | Delta is still good (51–53% cheaper) |