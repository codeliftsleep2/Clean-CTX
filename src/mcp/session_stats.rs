// src/mcp/session_stats.rs
//
// Session-level stats accumulator for the Clean-CTX dashboard.
//
// Every `provide_code_context` call (and other compression tools)
// records token savings into `SessionStats`. The `context_stats`
// tool reads from this accumulator to display the dashboard.
//
// Persistence-ready: `SessionStats` can be serialized to SQLite for
// cross-session history when the persistence layer arrives.

use std::collections::HashMap;
use std::time::SystemTime;

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
    /// Number of full compression operations.
    full_compress_count: usize,
    /// Number of delta operations.
    delta_count: usize,
    /// Session start time.
    started_at: SystemTime,
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
            started_at: SystemTime::now(),
        }
    }

    /// Record a compression event (full compression or delta).
    ///
    /// Updates both the per-file entry and the session totals.
    pub fn record_compression(
        &mut self,
        file_path: &str,
        raw_tokens: usize,
        compressed_tokens: usize,
        fidelity: &str,
        is_angular: bool,
        strategy: &str,
    ) {
        // Deduct previous per-file counters if this file was already tracked
        // (to avoid double-counting across calls)
        if let Some(existing) = self.files.get(file_path) {
            self.total_raw_tokens = self.total_raw_tokens.saturating_sub(existing.raw_tokens);
            self.total_compressed_tokens = self
                .total_compressed_tokens
                .saturating_sub(existing.compressed_tokens);
            // If strategy changed, decrement the old strategy counter
            if strategy != existing.strategy {
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

        // Update session totals
        self.total_raw_tokens += raw_tokens;
        self.total_compressed_tokens += compressed_tokens;
        match strategy {
            "delta" => self.delta_count += 1,
            _ => self.full_compress_count += 1,
        }

        // Compute savings as percentage (use saturating_sub to avoid overflow
        // when compressed_tokens > raw_tokens, e.g. for very small files)
        let savings_pct = if raw_tokens > 0 {
            let saved = raw_tokens.saturating_sub(compressed_tokens);
            (saved as f64 / raw_tokens as f64) * 100.0
        } else {
            0.0
        };

        // Get or create per-file entry
        let entry = self.files.entry(file_path.to_string()).or_insert_with(|| {
            FileStats {
                file_path: file_path.to_string(),
                raw_tokens: 0,
                compressed_tokens: 0,
                savings_pct: 0.0,
                version: 0,
                delta_count: 0,
                fidelity: fidelity.to_string(),
                is_angular,
                strategy: strategy.to_string(),
            }
        });

        entry.raw_tokens = raw_tokens;
        entry.compressed_tokens = compressed_tokens;
        entry.savings_pct = savings_pct;
        entry.version += 1;
        entry.fidelity = fidelity.to_string();
        entry.is_angular = is_angular;
        entry.strategy = strategy.to_string();
        if strategy == "delta" {
            entry.delta_count += 1;
        }
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
        // Recalculate operation counts from the merged file entries
        self.full_compress_count = self.files.values()
            .filter(|f| f.strategy != "delta")
            .count();
        self.delta_count = self.files.values()
            .filter(|f| f.strategy == "delta")
            .count();
    }

    /// Get stats for a specific file, if tracked.
    pub fn file_stats(&self, file_path: &str) -> Option<&FileStats> {
        self.files.get(file_path)
    }

    /// Get stats for all tracked files.
    pub fn all_file_stats(&self) -> &HashMap<String, FileStats> {
        &self.files
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
            self.files
                .values()
                .map(|f| f.savings_pct)
                .sum::<f64>()
                / total_files as f64
        } else {
            0.0
        };

        let session_duration_secs = self
            .started_at
            .elapsed()
            .map(|d| d.as_secs())
            .unwrap_or(0);

        SessionSummary {
            total_files,
            total_raw_tokens: total_raw,
            total_compressed_tokens: total_compressed,
            total_savings_pct: (total_savings_pct * 10.0).round() / 10.0,
            full_compress_count: self.full_compress_count,
            delta_count: self.delta_count,
            session_duration_secs,
            avg_savings_pct: (avg_savings_pct * 10.0).round() / 10.0,
        }
    }

    /// Get the session duration in seconds.
    pub fn session_duration_secs(&self) -> u64 {
        self.started_at
            .elapsed()
            .map(|d| d.as_secs())
            .unwrap_or(0)
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
    output.push_str(&format!("  Total Raw Tokens: {}\n", format_number(summary.total_raw_tokens)));
    output.push_str(&format!("  Total Compressed Tokens: {}\n", format_number(summary.total_compressed_tokens)));
    output.push_str(&format!("  Total LLM Token Savings: {:.1}%\n", summary.total_savings_pct));
    output.push_str(&format!(
        "  Operations: {} full compressions, {} deltas (local CPU only)\n",
        summary.full_compress_count,
        summary.delta_count,
    ));
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
    file_list.sort_by(|a, b| b.savings_pct.partial_cmp(&a.savings_pct).unwrap_or(std::cmp::Ordering::Equal));

    for file in &file_list {
        // Truncate long file paths
        let display_path = if file.file_path.len() > 38 {
            format!("...{}", &file.file_path[file.file_path.len().saturating_sub(37)..])
        } else {
            file.file_path.clone()
        };
        output.push_str(&format!(
            "  {:<40} {:>7} {:>7} {:>6.1}% {:>7}\n",
            display_path,
            format_number(file.raw_tokens),
            format_number(file.compressed_tokens),
            file.savings_pct,
            file.delta_count,
        ));
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
                "savings_pct": (f.savings_pct * 10.0).round() / 10.0,
                "version": f.version,
                "delta_count": f.delta_count,
                "fidelity": f.fidelity,
                "is_angular": f.is_angular,
                "strategy": f.strategy,
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