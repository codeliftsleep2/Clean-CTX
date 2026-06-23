// proxy/src/filter_stats.rs
//
// Per-program filter savings tracking.
// Accumulates token and line savings for the context_stats dashboard.

use std::collections::HashMap;

/// Per-program filter statistics.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct FilterStats {
    /// Per-program stats.
    pub programs: HashMap<String, ProgramFilterStats>,

    /// Total tokens saved across all programs.
    pub total_tokens_saved: u64,

    /// Total lines filtered.
    pub total_lines_filtered: u64,

    /// Total filters applied.
    pub total_applications: u64,
}

/// Statistics for a single program filter.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ProgramFilterStats {
    /// Program name (e.g., "cargo", "npm").
    pub program: String,

    /// Number of times this filter was applied.
    pub applications: u64,

    /// Original tokens before filtering.
    pub original_tokens: u64,

    /// Filtered tokens after filtering.
    pub filtered_tokens: u64,

    /// Tokens saved (original - filtered).
    pub tokens_saved: u64,

    /// Original lines before filtering.
    pub original_lines: u64,

    /// Filtered lines after filtering.
    pub filtered_lines: u64,

    /// Lines removed.
    pub lines_removed: u64,

    /// Total reduction percentage.
    pub reduction_pct: f32,
}

impl ProgramFilterStats {
    fn new(program: &str) -> Self {
        Self {
            program: program.to_string(),
            applications: 0,
            original_tokens: 0,
            filtered_tokens: 0,
            tokens_saved: 0,
            original_lines: 0,
            filtered_lines: 0,
            lines_removed: 0,
            reduction_pct: 0.0,
        }
    }
}

impl FilterStats {
    /// Create new, empty filter stats.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a filter application result.
    pub fn record_application(
        &mut self,
        program: &str,
        original_tokens: usize,
        filtered_tokens: usize,
        original_lines: usize,
        filtered_lines: usize,
    ) {
        let entry = self
            .programs
            .entry(program.to_string())
            .or_insert_with(|| ProgramFilterStats::new(program));

        entry.applications += 1;
        let ot = original_tokens as u64;
        let ft = filtered_tokens as u64;
        entry.original_tokens += ot;
        entry.filtered_tokens += ft;
        entry.tokens_saved += ot.saturating_sub(ft);
        entry.original_lines += original_lines as u64;
        entry.filtered_lines += filtered_lines as u64;
        entry.lines_removed += (original_lines as u64).saturating_sub(filtered_lines as u64);

        if entry.original_tokens > 0 {
            entry.reduction_pct = ((entry.original_tokens as f32 - entry.filtered_tokens as f32)
                / entry.original_tokens as f32)
                * 100.0;
        }

        self.total_tokens_saved += ot.saturating_sub(ft);
        self.total_lines_filtered += (original_lines as u64).saturating_sub(filtered_lines as u64);
        self.total_applications += 1;
    }

    /// Get stats for a specific program.
    pub fn for_program(&self, program: &str) -> Option<&ProgramFilterStats> {
        self.programs.get(program)
    }

    /// Get a summary string for the dashboard.
    pub fn summary(&self) -> String {
        if self.total_applications == 0 {
            return "  No filter applications recorded.".to_string();
        }

        let mut lines = vec![format!(
            "  Total filter applications: {} | Tokens saved: {} | Lines filtered: {}",
            self.total_applications, self.total_tokens_saved, self.total_lines_filtered
        )];

        let mut programs: Vec<&ProgramFilterStats> = self.programs.values().collect();
        programs.sort_by_key(|b| std::cmp::Reverse(b.tokens_saved));

        for p in &programs[..programs.len().min(10)] {
            lines.push(format!(
                "    {:<12} {:>4}x | {:>8} → {:>8} tokens ({:>5.1}%↓) | {:>6} → {:>6} lines",
                p.program,
                p.applications,
                p.original_tokens,
                p.filtered_tokens,
                p.reduction_pct,
                p.original_lines,
                p.filtered_lines,
            ));
        }

        if programs.len() > 10 {
            lines.push(format!("    ... and {} more programs", programs.len() - 10));
        }

        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_stats() {
        let stats = FilterStats::new();
        assert_eq!(stats.total_applications, 0);
        assert!(stats.summary().contains("No filter applications"));
    }

    #[test]
    fn test_record_application() {
        let mut stats = FilterStats::new();

        stats.record_application("cargo", 500, 25, 100, 5);
        assert_eq!(stats.total_applications, 1);
        assert_eq!(stats.total_tokens_saved, 475);

        let cargo_stats = stats.for_program("cargo").unwrap();
        assert_eq!(cargo_stats.applications, 1);
        assert_eq!(cargo_stats.original_tokens, 500);
        assert_eq!(cargo_stats.filtered_tokens, 25);
        assert_eq!(cargo_stats.original_lines, 100);
        assert_eq!(cargo_stats.filtered_lines, 5);
    }

    #[test]
    fn test_multiple_applications() {
        let mut stats = FilterStats::new();

        stats.record_application("cargo", 500, 25, 100, 5);
        stats.record_application("cargo", 300, 10, 60, 2);
        stats.record_application("npm", 2000, 30, 400, 6);

        assert_eq!(stats.total_applications, 3);
        assert_eq!(stats.total_tokens_saved, (500 - 25) + (300 - 10) + (2000 - 30));

        let cargo_stats = stats.for_program("cargo").unwrap();
        assert_eq!(cargo_stats.applications, 2);
        assert_eq!(cargo_stats.original_tokens, 800);
        assert_eq!(cargo_stats.filtered_tokens, 35);
    }

    #[test]
    fn test_summary_format() {
        let mut stats = FilterStats::new();
        stats.record_application("cargo", 500, 25, 100, 5);
        stats.record_application("npm", 2000, 30, 400, 6);

        let summary = stats.summary();
        assert!(summary.contains("filter applications: 2"));
        assert!(summary.contains("cargo"));
        assert!(summary.contains("npm"));
    }
}