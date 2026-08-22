// src/mcp/session_stats.rs
//
// Session-level stats accumulator for the Clean-CTX dashboard.
//
// Every `provide_code_context` call (and other compression tools)
// records token savings into `SessionStats`. The `context_stats`
// tool reads from this accumulator to display the dashboard.
//
// Phase 2 (Filter-First Architecture): each compression event is
// tagged with a `SavingsDomain` so the dashboard can show per-domain
// breakdowns without double-counting.

use crate::mcp::cache_hints::CacheMetrics;
use std::collections::HashMap;
use std::time::SystemTime;

/// Per-domain aggregate stats (used in `SessionSummary` for the
/// per-domain breakdown section of the dashboard).
#[derive(Debug, Clone, serde::Serialize)]
pub struct DomainStats {
    /// Domain identifier string.
    pub domain: String,
    /// Total raw tokens for this domain.
    pub total_raw_tokens: usize,
    /// Total compressed tokens for this domain.
    pub total_compressed_tokens: usize,
    /// LLM token savings percentage (0.0 if not a raw→compressed domain).
    pub savings_pct: f64,
    /// Number of unique files in this domain.
    pub file_count: usize,
    /// Additional domain-specific metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens_removed: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_hits: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_misses: Option<usize>,
}

/// Per-file stats entry for the dashboard.
#[derive(Debug, Clone, serde::Serialize)]
pub struct FileStats {
    /// The original file path.
    pub file_path: String,
    /// Raw (uncompressed) token count.
    pub raw_tokens: usize,
    /// Compressed token count.
    pub compressed_tokens: usize,
    /// Savings as a percentage (0.0 to 100.0).
    /// For "delta" strategy files, this is 0.0 (LLM savings don't apply).
    pub savings_pct: f64,
    /// Current version number.
    pub version: u64,
    /// Number of delta operations applied.
    pub delta_count: usize,
    /// Fidelity level used.
    pub fidelity: String,
    /// Whether Angular Meta-Layer was detected.
    pub is_angular: bool,
    /// Strategy used ("full" or "delta").
    pub strategy: String,
    /// Token count from the most recent full compression of this file.
    /// Used to compute delta efficiency. Only meaningful when strategy is "delta".
    pub full_compressed_tokens: Option<usize>,
    /// Delta efficiency: how much CPU is saved vs a full re-compression.
    /// Computed as `1.0 - (delta_tokens / full_compressed_tokens) * 100.0`.
    /// Only meaningful when strategy is "delta".
    pub delta_efficiency_pct: Option<f64>,
    /// Domain this file's stats belong to (Phase 2: Filter-First Architecture).
    pub domain: String,
}

impl FileStats {
    /// True if this file's savings percentage represents real LLM token savings.
    ///
    /// A delta-strategy file that PRESERVED its prior full-compression
    /// savings (see `record_compression`'s `preserve_full_on_delta`) still
    /// has real LLM token savings — the delta is a local CPU event that
    /// doesn't change the file's LLM-visible token profile. So we return
    /// true when `savings_pct > 0.0` even for delta files.
    pub fn has_llm_savings(&self) -> bool {
        self.strategy != "delta" || self.savings_pct > 0.0
    }
}

/// Summary of the entire session.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SessionSummary {
    /// Number of unique files tracked.
    pub total_files: usize,
    /// Sum of raw tokens across all files.
    pub total_raw_tokens: usize,
    /// Sum of compressed tokens across all files.
    pub total_compressed_tokens: usize,
    /// Overall compression savings percentage (LLM token reduction).
    pub total_savings_pct: f64,
    /// Number of full compression operations (producing LLM token savings).
    pub full_compress_count: usize,
    /// Number of delta operations (local CPU-only, no LLM token impact).
    pub delta_count: usize,
    /// Session duration in seconds.
    pub session_duration_secs: u64,
    /// Average compression savings across all files.
    pub avg_savings_pct: f64,
    /// Average delta efficiency across delta-strategy files (CPU savings % vs full re-compress).
    pub avg_delta_efficiency_pct: Option<f64>,
    /// Per-domain breakdown (Phase 2: Filter-First Architecture).
    pub domain_breakdown: HashMap<String, DomainStats>,
}

/// Session-level stats accumulator.
///
/// Thread-safe design: used from single-threaded MCP dispatch, so no
/// synchronization primitives are needed.
#[derive(Debug, Clone)]
pub struct SessionStats {
    /// Per-file stats keyed by file path.
    files: HashMap<String, FileStats>,
    /// Total raw tokens across all files.
    total_raw_tokens: usize,
    /// Total compressed tokens across all files.
    total_compressed_tokens: usize,
    /// Number of full compression operations (per-file events).
    full_compress_count: usize,
    /// Number of delta operations.
    delta_count: usize,
    /// Number of CBM pipe-level proxy interception events.
    ///
    /// Tracked SEPARATELY from `full_compress_count` because each CBM call is a
    /// distinct event but only creates ONE per-tool file entry (`cbm://tool`).
    /// `full_compress_count` (derived from unique file entries during `merge()`)
    /// would understate CBM activity after a persistence restore. This counter
    /// survives merges by direct addition.
    cbm_proxy_events: usize,
    /// Session start time.
    started_at: SystemTime,
    /// Per-domain breakdown (Phase 2).
    domain_stats: HashMap<String, DomainStats>,
}

impl Default for SessionStats {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionStats {
    /// Create a new empty stats accumulator.
    /// Session timer starts on creation.
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
            total_raw_tokens: 0,
            total_compressed_tokens: 0,
            full_compress_count: 0,
            delta_count: 0,
            cbm_proxy_events: 0,
            started_at: SystemTime::now(),
            domain_stats: HashMap::new(),
        }
    }

    /// Record a compression event (full compression or delta).
    ///
    /// `full_compressed_tokens`: When `strategy == "delta"`, pass
    /// `Some(full_compress_token_count)` so delta efficiency can be
    /// computed (CPU savings vs re-compressing from scratch).
    /// For non-delta strategies, pass `None`.
    ///
    /// `domain`: identifies which savings domain this event belongs to.
    /// See [`SavingsDomain`] for the complete list. Each file may only
    /// appear in one domain — if a file is re-recorded with a different
    /// domain, the old domain counters are decremented.
    ///
    /// Updates both the per-file entry and the session totals.
    #[allow(clippy::too_many_arguments)]
    pub fn record_compression(
        &mut self,
        file_path: &str,
        raw_tokens: usize,
        compressed_tokens: usize,
        fidelity: &str,
        is_angular: bool,
        strategy: &str,
        full_compressed_tokens: Option<usize>,
        domain: &str,
    ) {
        // Track domain — we need to know which domain this file was in before
        let prev_domain = self.files.get(file_path).map(|f| f.domain.clone());

        // Deduct previous per-file counters if this file was already tracked
        // (to avoid double-counting across calls). Also keep a handle to the
        // old `FileStats` so the domain-transition block below can subtract
        // the exact previously-counted values.
        let prev_file_stats: Option<FileStats> = self.files.get(file_path).cloned();

        // DASHBOARD FIX (R-02 FAANG): When a delta is recorded for a file
        // that previously had a FULL compression, PRESERVE the full
        // compression's token counts and LLM token savings. Delta ops are
        // local CPU events — they do not change the file's LLM-visible token
        // profile. Previously the delta recording subtracted the full
        // compression's real raw/compressed tokens from the session totals
        // and overwrote the per-file entry with 0/0, erasing the savings the
        // file had on its initial hit (dashboard showed N/A).
        let preserve_full_on_delta = strategy == "delta"
            && prev_file_stats
                .as_ref()
                .is_some_and(|f| f.strategy == "full");

        if let Some(ref existing) = prev_file_stats {
            // Only deduct previous counters when NOT preserving full→delta.
            // When preserving, the full compression's tokens/savings remain
            // in the session totals.
            if !preserve_full_on_delta {
                self.total_raw_tokens = self.total_raw_tokens.saturating_sub(existing.raw_tokens);
                self.total_compressed_tokens = self
                    .total_compressed_tokens
                    .saturating_sub(existing.compressed_tokens);
            }
            // If strategy changed, decrement the old strategy counter.
            // When preserving full→delta, the full_compress_count stays
            // (the file still counts as a full compression for LLM savings).
            if strategy != existing.strategy && !preserve_full_on_delta {
                match existing.strategy.as_str() {
                    "delta" => {
                        self.delta_count = self.delta_count.saturating_sub(1);
                    }
                    _ => {
                        self.full_compress_count = self.full_compress_count.saturating_sub(1);
                    }
                }
            }
        }

        // Update domain counters (deduct old domain, add new)
        if let Some(ref pd) = prev_domain {
            if pd != domain {
                if let Some(existing_domain) = self.domain_stats.get_mut(pd) {
                    // Subtract the OLD file values from the old domain, not the new ones.
                    // Using `prev_file_stats` (the previous FileStats) ensures we remove exactly
                    // what was previously counted, regardless of whether the file's
                    // token counts changed between recordings.
                    if let Some(ref prev) = prev_file_stats {
                        existing_domain.total_raw_tokens = existing_domain
                            .total_raw_tokens
                            .saturating_sub(prev.raw_tokens);
                        existing_domain.total_compressed_tokens = existing_domain
                            .total_compressed_tokens
                            .saturating_sub(prev.compressed_tokens);
                    }
                    existing_domain.file_count = existing_domain.file_count.saturating_sub(1);
                    // Recompute savings pct
                    if existing_domain.total_raw_tokens > 0 {
                        let saved = existing_domain
                            .total_raw_tokens
                            .saturating_sub(existing_domain.total_compressed_tokens);
                        existing_domain.savings_pct =
                            (saved as f64 / existing_domain.total_raw_tokens as f64) * 100.0;
                    } else {
                        existing_domain.savings_pct = 0.0;
                    }
                }
            }
        }

        // Update session totals. When preserving full→delta, the delta's
        // raw/compressed tokens are NOT added — the full compression's
        // tokens already represent the file's LLM-visible profile, and the
        // delta is a local CPU event. (The handler passes 0/0 for delta,
        // but this guard makes the invariant explicit and correct even if
        // a caller passes real delta wire tokens.)
        if !preserve_full_on_delta {
            self.total_raw_tokens += raw_tokens;
            self.total_compressed_tokens += compressed_tokens;
        }
        match strategy {
            "delta" => self.delta_count += 1,
            _ => self.full_compress_count += 1,
        }

        // Compute savings as percentage. For delta strategies, savings
        // represent CPU efficiency vs full re-compress, NOT LLM token savings.
        let is_delta = strategy == "delta";
        let (savings_pct, delta_eff_pct) = if is_delta {
            if let Some(full_ct) = full_compressed_tokens {
                if full_ct > 0 && compressed_tokens < full_ct {
                    let eff = ((full_ct - compressed_tokens) as f64 / full_ct as f64) * 100.0;
                    (0.0, Some(eff))
                } else {
                    (0.0, None)
                }
            } else {
                (0.0, None)
            }
        } else {
            let sp = if raw_tokens > 0 {
                let saved = raw_tokens.saturating_sub(compressed_tokens);
                (saved as f64 / raw_tokens as f64) * 100.0
            } else {
                0.0
            };
            (sp, None)
        };

        // Get or create per-file entry
        let entry = self
            .files
            .entry(file_path.to_string())
            .or_insert_with(|| FileStats {
                file_path: file_path.to_string(),
                raw_tokens: 0,
                compressed_tokens: 0,
                savings_pct: 0.0,
                version: 0,
                delta_count: 0,
                fidelity: fidelity.to_string(),
                is_angular,
                strategy: strategy.to_string(),
                full_compressed_tokens: None,
                delta_efficiency_pct: None,
                domain: domain.to_string(),
            });

        // When preserving full→delta, keep the full compression's token
        // counts and savings_pct in the per-file entry (so the dashboard
        // shows the real LLM savings, not N/A). Only the strategy, delta
        // count, and delta efficiency are updated to reflect the delta.
        if !preserve_full_on_delta {
            entry.raw_tokens = raw_tokens;
            entry.compressed_tokens = compressed_tokens;
            entry.savings_pct = savings_pct;
        }
        entry.version += 1;
        entry.fidelity = fidelity.to_string();
        entry.is_angular = is_angular;
        entry.strategy = strategy.to_string();
        entry.domain = domain.to_string();
        if is_delta {
            entry.delta_count += 1;
        }
        entry.full_compressed_tokens = full_compressed_tokens;
        entry.delta_efficiency_pct = delta_eff_pct;

        // Update domain-level stats
        let domain_entry = self
            .domain_stats
            .entry(domain.to_string())
            .or_insert_with(|| DomainStats {
                domain: domain.to_string(),
                total_raw_tokens: 0,
                total_compressed_tokens: 0,
                savings_pct: 0.0,
                file_count: 0,
                tokens_removed: if domain == "cbm_filter" {
                    Some(0)
                } else {
                    None
                },
                cache_hits: if domain == "prompt_cache" {
                    Some(0)
                } else {
                    None
                },
                cache_misses: if domain == "prompt_cache" {
                    Some(0)
                } else {
                    None
                },
            });
        // When preserving full→delta, the domain stats already reflect the
        // full compression's tokens — do not add the delta's (0/0) on top.
        if !preserve_full_on_delta {
            domain_entry.total_raw_tokens += raw_tokens;
            domain_entry.total_compressed_tokens += compressed_tokens;
            if domain_entry.total_raw_tokens > 0 {
                let saved = domain_entry
                    .total_raw_tokens
                    .saturating_sub(domain_entry.total_compressed_tokens);
                domain_entry.savings_pct =
                    (saved as f64 / domain_entry.total_raw_tokens as f64) * 100.0;
            }
        }
        // Increment file count only if this is a new file for this domain
        // (tracked by whether the entry was just created)
        if entry.version == 1 {
            domain_entry.file_count += 1;
        }
    }

    /// Record a CBM pipe-level proxy interception event.
    ///
    /// Unlike `record_compression` (which OVERWRITES a per-file entry — each
    /// new call subtracts the old values and re-adds the new, so repeated
    /// calls to the same tool only reflect the LAST interception), CBM proxy
    /// interceptions are DISTINCT EVENTS that must ACCUMULATE. Repeated
    /// `cbm_proxy` calls previously understated the dashboard because the
    /// per-tool key (`cbm://graph_search`) was overwritten each time.
    ///
    /// This method accumulates raw/compressed tokens in:
    ///   - session totals (never subtracts)
    ///   - the per-tool file entry (sum across calls)
    ///   - the `cbm_filter` domain (sum across calls)
    pub fn record_cbm_proxy(&mut self, tool: &str, raw_tokens: usize, compressed_tokens: usize) {
        // Session totals ACCUMULATE (never subtract)
        self.total_raw_tokens += raw_tokens;
        self.total_compressed_tokens += compressed_tokens;
        // Track as a distinct CBM event (survives merge — see struct doc).
        self.cbm_proxy_events += 1;

        // Per-tool entry accumulates across calls
        let file_path = format!("cbm://{tool}");
        let is_new_tool = !self.files.contains_key(&file_path);
        let entry = self
            .files
            .entry(file_path.clone())
            .or_insert_with(|| FileStats {
                file_path,
                raw_tokens: 0,
                compressed_tokens: 0,
                savings_pct: 0.0,
                version: 0,
                delta_count: 0,
                fidelity: "low".into(),
                is_angular: false,
                strategy: "full".into(),
                full_compressed_tokens: None,
                delta_efficiency_pct: None,
                domain: "cbm_filter".into(),
            });
        entry.raw_tokens += raw_tokens;
        entry.compressed_tokens += compressed_tokens;
        entry.version += 1;
        entry.savings_pct = if entry.raw_tokens > 0 {
            let saved = entry.raw_tokens.saturating_sub(entry.compressed_tokens);
            (saved as f64 / entry.raw_tokens as f64) * 100.0
        } else {
            0.0
        };

        // Domain breakdown accumulates across calls.
        // `file_count` counts UNIQUE tools (first call per tool), consistent
        // with `ir_compression`'s unique-file semantics.
        let domain = self
            .domain_stats
            .entry("cbm_filter".to_string())
            .or_insert_with(|| DomainStats {
                domain: "cbm_filter".into(),
                total_raw_tokens: 0,
                total_compressed_tokens: 0,
                savings_pct: 0.0,
                file_count: 0,
                tokens_removed: Some(0),
                cache_hits: None,
                cache_misses: None,
            });
        domain.total_raw_tokens += raw_tokens;
        domain.total_compressed_tokens += compressed_tokens;
        if is_new_tool {
            domain.file_count += 1;
        }
        let removed = domain
            .total_raw_tokens
            .saturating_sub(domain.total_compressed_tokens);
        domain.savings_pct = if domain.total_raw_tokens > 0 {
            (removed as f64 / domain.total_raw_tokens as f64) * 100.0
        } else {
            0.0
        };
        if let Some(ref mut tr) = domain.tokens_removed {
            *tr = removed;
        }
    }

    /// Record a cache hit event (prompt cache breakpoint dedup).
    /// This is a separate method because cache tokens are saved by NOT
    /// sending them, not by compressing them.
    pub fn record_cache_hit(&mut self, tokens_saved: usize) {
        let domain = "prompt_cache";
        let entry = self
            .domain_stats
            .entry(domain.to_string())
            .or_insert_with(|| DomainStats {
                domain: domain.to_string(),
                total_raw_tokens: 0,
                total_compressed_tokens: 0,
                savings_pct: 0.0,
                file_count: 0,
                tokens_removed: None,
                cache_hits: Some(0),
                cache_misses: Some(0),
            });
        if let Some(ref mut hits) = entry.cache_hits {
            *hits += 1;
        }
        // Treat cache hits as tokens "compressed" to zero
        entry.total_raw_tokens += tokens_saved;
        // compressed_tokens stays 0 — the tokens were never sent
        entry.savings_pct = 100.0; // 100% savings on cached tokens
    }

    /// Record a proxy filter event (tool output filtering savings).
    pub fn record_tool_filter(
        &mut self,
        program: &str,
        original_tokens: usize,
        filtered_tokens: usize,
    ) {
        let domain = "tool_filter";
        let file_path = format!("__proxy__{}", program);
        self.record_compression(
            &file_path,
            original_tokens,
            filtered_tokens,
            "low",
            false,
            "full",
            None,
            domain,
        );
    }

    /// Sync MCP-level `CacheMetrics` into the `prompt_cache` domain entry.
    ///
    /// This bridges the per-breakpoint cache tracking in `McpState.cache_metrics`
    /// into the per-domain dashboard breakdown. After this call, the
    /// `prompt_cache` domain will show hits, misses, and tokens saved.
    ///
    /// ACCUMULATES into `total_raw_tokens` rather than overwriting, so real
    /// proxy cache-read token counts (recorded via `record_cache_hit`) are
    /// preserved alongside the MCP-side dedup savings.
    pub fn sync_cache_metrics(&mut self, metrics: &CacheMetrics) {
        let domain = "prompt_cache";
        let entry = self
            .domain_stats
            .entry(domain.to_string())
            .or_insert_with(|| DomainStats {
                domain: domain.to_string(),
                total_raw_tokens: 0,
                total_compressed_tokens: 0,
                savings_pct: 0.0,
                file_count: 0,
                tokens_removed: None,
                cache_hits: Some(0),
                cache_misses: Some(0),
            });
        // ACCUMULATE hits/misses rather than overwrite. Real proxy cache hits
        // (recorded via `record_cache_hit`) must be preserved alongside the
        // MCP-side dedup hits. Overwriting would erase the proxy's real
        // cache-read token count from the dashboard.
        if let Some(ref mut hits) = entry.cache_hits {
            *hits += metrics.hits;
        }
        if let Some(ref mut misses) = entry.cache_misses {
            *misses += metrics.misses;
        }
        // ACCUMULATE MCP-side dedup savings into total_raw_tokens.
        // Real proxy cache-read tokens (from `record_cache_hit`) are already
        // in total_raw_tokens and must not be overwritten.
        entry.total_raw_tokens += metrics.tokens_saved;
        entry.savings_pct = if entry.total_raw_tokens > 0 {
            100.0
        } else {
            0.0
        };
    }

    /// Merge another `SessionStats` into this one.
    ///
    /// **In-memory data always wins for freshness.**  DB-recovered stats
    /// (from `rebuild_stats()`) may carry placeholder values, so we
    /// treat them as supplementary: only files NOT already tracked
    /// in memory are imported.  For files that appear in both, the
    /// in-memory counters are preserved untouched.
    ///
    /// Session-level counters (`full_compress_count`, `delta_count`)
    /// are still merged so the dashboard header reflects cumulative
    /// work across sessions.
    pub fn merge(&mut self, other: &SessionStats) {
        for (path, other_fs) in &other.files {
            if self.files.contains_key(path) {
                // File already has in-memory stats — in-memory is fresher,
                // so skip overwriting. We still merge version to avoid
                // the dashboard showing lower versions after restart.
                // Deliberately do NOT overwrite tokens/fidelity/strategy.
                if let Some(existing) = self.files.get_mut(path) {
                    existing.version = existing.version.max(other_fs.version);
                    existing.delta_count += other_fs.delta_count;
                }
            } else {
                // DB-only file — import as-is so it appears in the dashboard.
                self.files.insert(path.clone(), other_fs.clone());
            }
        }

        // Recalculate ALL totals from merged files to ensure consistency.
        // This avoids the over-counting bug where session-level counters
        // were blindly added even for files that were skipped during merge.
        self.total_raw_tokens = self.files.values().map(|f| f.raw_tokens).sum();
        self.total_compressed_tokens = self.files.values().map(|f| f.compressed_tokens).sum();
        // Recalculate operation counts from the merged file entries.
        // NOTE: CBM proxy events are added SEPARATELY because they are not
        // derivable from unique file entries — each tool creates one entry
        // regardless of how many times it was called.
        self.full_compress_count = self
            .files
            .values()
            .filter(|f| f.strategy != "delta" && !f.file_path.starts_with("cbm://"))
            .count();
        self.delta_count = self
            .files
            .values()
            .filter(|f| f.strategy == "delta")
            .count();
        // Accumulated CBM proxy events survive merges (in-memory + DB-recovered)
        self.cbm_proxy_events += other.cbm_proxy_events;

        // Rebuild domain stats from merged files
        self.rebuild_domain_stats();
    }

    /// Rebuild domain-level stats from current file entries.
    ///
    /// **Important:** preserves cache-specific metadata (`cache_hits`,
    /// `cache_misses`, `tokens_removed`) for domains that carry it, because
    /// those fields are populated by `sync_cache_metrics()` and `record_tool_filter()`
    /// rather than by file-level compression events.
    fn rebuild_domain_stats(&mut self) {
        // Preserve cache-specific metadata before rebuild
        let preserved_prompt_cache: Option<DomainStats> =
            self.domain_stats.get("prompt_cache").cloned();
        let preserved_cbm_filter: Option<DomainStats> =
            self.domain_stats.get("cbm_filter").cloned();

        let mut domain_raw: HashMap<String, usize> = HashMap::new();
        let mut domain_compressed: HashMap<String, usize> = HashMap::new();
        let mut domain_files: HashMap<String, usize> = HashMap::new();

        for file in self.files.values() {
            let d = file.domain.clone();
            *domain_raw.entry(d.clone()).or_insert(0) += file.raw_tokens;
            *domain_compressed.entry(d.clone()).or_insert(0) += file.compressed_tokens;
            *domain_files.entry(d).or_insert(0) += 1;
        }

        self.domain_stats = domain_raw
            .keys()
            .chain(domain_compressed.keys())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .map(|d| {
                let raw = domain_raw.get(d).copied().unwrap_or(0);
                let compressed = domain_compressed.get(d).copied().unwrap_or(0);
                let file_count = domain_files.get(d).copied().unwrap_or(0);
                let savings_pct = if raw > 0 {
                    let saved = raw.saturating_sub(compressed);
                    (saved as f64 / raw as f64) * 100.0
                } else {
                    0.0
                };
                let mut stats = DomainStats {
                    domain: d.clone(),
                    total_raw_tokens: raw,
                    total_compressed_tokens: compressed,
                    savings_pct: (savings_pct * 10.0).round() / 10.0,
                    file_count,
                    tokens_removed: None,
                    cache_hits: None,
                    cache_misses: None,
                };
                // Restore preserved cache-specific metadata
                if d == "prompt_cache" {
                    if let Some(ref preserved) = preserved_prompt_cache {
                        stats.cache_hits = preserved.cache_hits;
                        stats.cache_misses = preserved.cache_misses;
                        // tokens_removed is not used for prompt_cache, but preserve if set
                        stats.tokens_removed = preserved.tokens_removed;
                        // Preserve real cache-read token counts recorded via
                        // `record_cache_hit` / `sync_cache_metrics`. These are
                        // NOT derived from per-file entries, so they must be
                        // carried across rebuilds.
                        stats.total_raw_tokens = preserved.total_raw_tokens;
                        stats.savings_pct = if stats.total_raw_tokens > 0 {
                            100.0
                        } else {
                            0.0
                        };
                    }
                } else if d == "cbm_filter" {
                    if let Some(ref preserved) = preserved_cbm_filter {
                        stats.tokens_removed = preserved.tokens_removed;
                    }
                }
                (d.clone(), stats)
            })
            .collect();
    }

    /// Get stats for a specific file, if tracked.
    pub fn file_stats(&self, file_path: &str) -> Option<&FileStats> {
        self.files.get(file_path)
    }

    /// Get stats for all tracked files.
    pub fn all_file_stats(&self) -> &HashMap<String, FileStats> {
        &self.files
    }

    /// Get per-domain breakdown stats.
    pub fn domain_breakdown(&self) -> &HashMap<String, DomainStats> {
        &self.domain_stats
    }

    /// Get a summary of the entire session.
    pub fn summary(&self) -> SessionSummary {
        let total_files = self.files.len();
        let total_raw = self.total_raw_tokens;
        let total_compressed = self.total_compressed_tokens;
        let total_savings_pct = if total_raw > 0 {
            ((total_raw.saturating_sub(total_compressed)) as f64 / total_raw as f64) * 100.0
        } else {
            0.0
        };
        let avg_savings_pct = if total_files > 0 {
            self.files.values().map(|f| f.savings_pct).sum::<f64>() / total_files as f64
        } else {
            0.0
        };

        // Compute average delta efficiency across all delta-strategy files
        let delta_files: Vec<&FileStats> = self
            .files
            .values()
            .filter(|f| f.strategy == "delta" && f.delta_efficiency_pct.is_some())
            .collect();
        let avg_delta_eff = if !delta_files.is_empty() {
            let sum: f64 = delta_files
                .iter()
                .filter_map(|f| f.delta_efficiency_pct)
                .sum();
            Some(sum / delta_files.len() as f64)
        } else {
            None
        };

        let session_duration_secs = self.started_at.elapsed().map(|d| d.as_secs()).unwrap_or(0);

        SessionSummary {
            total_files,
            total_raw_tokens: total_raw,
            total_compressed_tokens: total_compressed,
            total_savings_pct: (total_savings_pct * 10.0).round() / 10.0,
            // Full compression ops = per-file events + accumulated CBM proxy events
            full_compress_count: self.full_compress_count + self.cbm_proxy_events,
            delta_count: self.delta_count,
            session_duration_secs,
            avg_savings_pct: (avg_savings_pct * 10.0).round() / 10.0,
            avg_delta_efficiency_pct: avg_delta_eff.map(|v| (v * 10.0).round() / 10.0),
            domain_breakdown: self.domain_stats.clone(),
        }
    }

    /// Get the session duration in seconds.
    pub fn session_duration_secs(&self) -> u64 {
        self.started_at.elapsed().map(|d| d.as_secs()).unwrap_or(0)
    }
}

/// Render a human-readable dashboard from session stats.
pub fn render_dashboard_text(stats: &SessionStats) -> String {
    let summary = stats.summary();
    let duration = format_duration(summary.session_duration_secs);

    let mut output = String::new();
    output.push_str("═══════════════════════════════════════════════════════════════\n");
    output.push_str("  Clean-CTX Dashboard — Session Stats\n");
    output.push_str("═══════════════════════════════════════════════════════════════\n");
    output.push_str(&format!("  Session Duration: {}\n", duration));
    output.push_str(&format!("  Files Tracked: {}\n", summary.total_files));
    output.push_str(&format!(
        "  Total Raw Tokens: {}\n",
        format_number(summary.total_raw_tokens)
    ));
    output.push_str(&format!(
        "  Total Compressed Tokens: {}\n",
        format_number(summary.total_compressed_tokens)
    ));
    output.push_str(&format!(
        "  Total LLM Token Savings: {:.1}%\n",
        summary.total_savings_pct
    ));
    output.push_str(&format!(
        "  Operations: {} full compressions, {} deltas (local CPU only)\n",
        summary.full_compress_count, summary.delta_count,
    ));
    // Delta efficiency summary line (only shown when there are actual delta ops)
    if let Some(avg_eff) = summary.avg_delta_efficiency_pct {
        output.push_str(&format!(
            "  Δ Efficiency: avg {:.1}% CPU savings vs full re-compress\n",
            avg_eff,
        ));
    }
    output.push_str("───────────────────────────────────────────────────────────────\n");

    // ── Per-Domain Breakdown (Phase 2) ─────────────────────────────
    if !summary.domain_breakdown.is_empty() {
        output.push_str("── Per-Domain LLM Token Savings ──\n");

        // Define display order for domains
        let domain_order = [
            "ir_compression",
            "angular_template",
            "cbm_filter",
            "prompt_cache",
            "tool_filter",
        ];
        let mut has_any = false;

        for domain_key in &domain_order {
            if let Some(ds) = summary.domain_breakdown.get(*domain_key) {
                if ds.total_raw_tokens == 0 && ds.total_compressed_tokens == 0 {
                    continue; // skip empty domains
                }
                has_any = true;
                match *domain_key {
                    "ir_compression" => {
                        output.push_str(&format!(
                            "  IR Compression:      {:>10} → {:>10} ({:>5.1}%↓)\n",
                            format_number(ds.total_raw_tokens),
                            format_number(ds.total_compressed_tokens),
                            ds.savings_pct,
                        ));
                    }
                    "angular_template" => {
                        output.push_str(&format!(
                            "  Angular Templates:   {:>10} → {:>10} ({:>5.1}%↓)\n",
                            format_number(ds.total_raw_tokens),
                            format_number(ds.total_compressed_tokens),
                            ds.savings_pct,
                        ));
                    }
                    "cbm_filter" => {
                        output.push_str(&format!(
                            "  CBM Intercept:       {:>10} → {:>10} ({:>5.1}%↓)\n",
                            format_number(ds.total_raw_tokens),
                            format_number(ds.total_compressed_tokens),
                            ds.savings_pct,
                        ));
                    }
                    "prompt_cache" => {
                        output.push_str(&format!(
                            "  Prompt Cache:                  {:>10} tokens saved (hits)\n",
                            format_number(ds.total_raw_tokens),
                        ));
                    }
                    "tool_filter" => {
                        output.push_str(&format!(
                            "  Tool Filtering:      {:>10} → {:>10} ({:>5.1}%↓)\n",
                            format_number(ds.total_raw_tokens),
                            format_number(ds.total_compressed_tokens),
                            ds.savings_pct,
                        ));
                    }
                    _ => {
                        output.push_str(&format!(
                            "  {}: {:>10} → {:>10}\n",
                            domain_key,
                            format_number(ds.total_raw_tokens),
                            format_number(ds.total_compressed_tokens),
                        ));
                    }
                }
            }
        }

        if has_any {
            output.push_str(&format!(
                "  ─────────────────────────────────────────────\n  Total to LLM:       {:>10} → {:>10} ({:>5.1}%↓)\n",
                format_number(summary.total_raw_tokens),
                format_number(summary.total_compressed_tokens),
                summary.total_savings_pct,
            ));
        }
        output.push('\n');
    }

    output.push_str("───────────────────────────────────────────────────────────────\n");
    output.push_str("  Per-File Breakdown:\n");

    // Table header
    output.push_str(&format!(
        "  {:<40} {:>7} {:>7} {:>7} {:>7}\n",
        "File", "Raw", "Comp", "Save%", "Deltas"
    ));
    output.push_str(&format!(
        "  {:-<40}+{:-<8}+{:-<8}+{:-<8}+{:-<8}\n",
        "", "", "", "", ""
    ));

    // Sort files by savings ascending
    let mut file_list: Vec<&FileStats> = stats.all_file_stats().values().collect();
    file_list.sort_by(|a, b| {
        b.savings_pct
            .partial_cmp(&a.savings_pct)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    for file in &file_list {
        // Truncate long file paths
        let display_path = if file.file_path.len() > 38 {
            format!(
                "...{}",
                &file.file_path[file.file_path.len().saturating_sub(37)..]
            )
        } else {
            file.file_path.clone()
        };

        // Show "N/A" for delta files (LLM token savings don't apply)
        let save_display = if file.has_llm_savings() {
            format!("{:>6.1}%", file.savings_pct)
        } else {
            format!("{:>7}", "N/A")
        };

        output.push_str(&format!(
            "  {:<40} {:>7} {:>7} {} {:>7}",
            display_path,
            format_number(file.raw_tokens),
            format_number(file.compressed_tokens),
            save_display,
            file.delta_count,
        ));

        // Show delta efficiency on a sub-row if applicable
        if file.strategy == "delta" {
            if let Some(eff) = file.delta_efficiency_pct {
                output.push_str(&format!("\n  {:>40} {:>7} Δ eff: {:>5.1}%", "", "", eff));
            } else {
                output.push_str(&format!("\n  {:>40} {:>7}", "", "Δ: CPU only"));
            }
        }

        output.push('\n');
    }

    output.push_str("═══════════════════════════════════════════════════════════════\n");
    output
}

/// Render a JSON dashboard from session stats.
pub fn render_dashboard_json(stats: &SessionStats) -> serde_json::Value {
    let summary = stats.summary();
    let files: Vec<serde_json::Value> = stats
        .all_file_stats()
        .values()
        .map(|f| {
            serde_json::json!({
                "file_path": f.file_path,
                "raw_tokens": f.raw_tokens,
                "compressed_tokens": f.compressed_tokens,
                // null for delta files, numeric for LLM savings
                "savings_pct": if f.has_llm_savings() {
                    serde_json::Value::Number(serde_json::Number::from_f64(
                        (f.savings_pct * 10.0).round() / 10.0
                    ).unwrap_or(serde_json::Number::from(0)))
                } else {
                    serde_json::Value::Null
                },
                "version": f.version,
                "delta_count": f.delta_count,
                "fidelity": f.fidelity,
                "is_angular": f.is_angular,
                "strategy": f.strategy,
                "full_compressed_tokens": f.full_compressed_tokens,
                "delta_efficiency_pct": f.delta_efficiency_pct.map(|v| {
                    (v * 10.0).round() / 10.0
                }),
                "domain": f.domain,
            })
        })
        .collect();

    serde_json::json!({
        "session": {
            "duration_secs": summary.session_duration_secs,
            "total_files": summary.total_files,
            "total_raw_tokens": summary.total_raw_tokens,
            "total_compressed_tokens": summary.total_compressed_tokens,
            "total_savings_pct": summary.total_savings_pct,
            "full_compress_count": summary.full_compress_count,
            "delta_count": summary.delta_count,
            "avg_delta_efficiency_pct": summary.avg_delta_efficiency_pct,
            "domain_breakdown": summary.domain_breakdown,
        },
        "files": files,
    })
}

/// Format a duration in seconds to human-readable string.
fn format_duration(secs: u64) -> String {
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;
    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

/// Format a number with comma separators.
fn format_number(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result
}

#[cfg(test)]
#[path = "../tests/mcp/session_stats.rs"]
mod tests;
