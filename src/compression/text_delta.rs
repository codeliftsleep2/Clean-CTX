// src/compression/text_delta.rs
//
// Phase IV (Idea #12): Delta-Aware Text Compression
//
// Makes the text compression pipeline stateful, enabling delta-based
// transport similar to the IR system. First compression produces full
// output; subsequent compressions emit compact line-level deltas.
//
// Delta Wire Format:
// ```
// §Δfile_alias:from_version:to_version§
// +<added lines>
// -<removed lines>
// ~<modified lines>
// ```
//
// The delta is computed between stored compressed body snapshots —
// the structural output lines before the header is applied.

use std::collections::HashMap;

/// A single text delta between two compression snapshots.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TextDelta {
    /// Target file (path alias)
    pub file: String,
    /// Baseline version this delta applies to
    pub from: u64,
    /// Version after applying this delta
    pub to: u64,
    /// Lines added (not present in baseline)
    pub adds: Vec<String>,
    /// Lines removed (present in baseline but not in current)
    pub dels: Vec<String>,
    /// Lines modified (present in both but changed)
    /// Each tuple is (old_line, new_line)
    pub mods: Vec<(String, String)>,
}

impl TextDelta {
    /// Returns true if the delta has no operations (no changes).
    pub fn is_empty(&self) -> bool {
        self.adds.is_empty() && self.dels.is_empty() && self.mods.is_empty()
    }

    /// Format the delta as the compact §Δ wire format.
    pub fn to_wire_format(&self) -> String {
        let mut out = format!("§Δ{}:{}:{}§", self.file, self.from, self.to);
        for line in &self.adds {
            out.push('\n');
            out.push('+');
            out.push_str(line);
        }
        for line in &self.dels {
            out.push('\n');
            out.push('-');
            out.push_str(line);
        }
        for (old, new) in &self.mods {
            out.push('\n');
            out.push('~');
            out.push_str(old);
            out.push('→');
            out.push_str(new);
        }
        out
    }

    /// Parse a delta from the §Δ wire format.
    pub fn from_wire_format(wire: &str) -> Option<Self> {
        let wire = wire.trim();
        if !wire.starts_with("§Δ") {
            return None;
        }
        // Extract header: §Δfile:from:to§
        let ops_start = 2 + 'Δ'.len_utf8();
        // Find the closing § of the header (the first § after ops_start)
        let header_end = wire[ops_start..].find('§')? + ops_start;
        let inner = &wire[ops_start..header_end];
        let parts: Vec<&str> = inner.split(':').collect();
        if parts.len() != 3 {
            return None;
        }
        let file = parts[0].to_string();
        let from: u64 = parts[1].parse().ok()?;
        let to: u64 = parts[2].parse().ok()?;

        let mut delta = TextDelta {
            file,
            from,
            to,
            adds: Vec::new(),
            dels: Vec::new(),
            mods: Vec::new(),
        };

        // Parse operations after the header
        let ops_str = &wire[header_end + '§'.len_utf8()..];
        for line in ops_str.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Some(rest) = line.strip_prefix('+') {
                delta.adds.push(rest.to_string());
            } else if let Some(rest) = line.strip_prefix('-') {
                delta.dels.push(rest.to_string());
            } else if let Some(rest) = line.strip_prefix('~') {
                // Parse "old→new"
                if let Some(arrow_pos) = rest.find('→') {
                    let old = rest[..arrow_pos].to_string();
                    let new = rest[arrow_pos + '→'.len_utf8()..].to_string();
                    delta.mods.push((old, new));
                }
            }
        }

        Some(delta)
    }
}

/// Computes text-level structural deltas between stored compressed body
/// snapshots and the current compression output.
///
/// Uses a line-level LCS (Longest Common Subsequence) diff to identify
/// added, removed, and modified lines.
#[derive(Debug, Clone, Default)]
pub struct TextDeltaComputer {
    /// Per-file version counter (monotonically increasing)
    versions: HashMap<String, u64>,
    /// Per-file stored compressed body lines (the "snapshot")
    snapshots: HashMap<String, Vec<String>>,
}

impl TextDeltaComputer {
    /// Create a new empty delta computer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if we have a baseline snapshot for the given file.
    pub fn has_baseline(&self, file: &str) -> bool {
        self.snapshots.contains_key(file)
    }

    /// Get the current version for a file (0 if not tracked).
    pub fn file_version(&self, file: &str) -> u64 {
        self.versions.get(file).copied().unwrap_or(0)
    }

    /// Store a full snapshot (compressed body lines) for a file,
    /// incrementing its version. Returns the new version.
    pub fn store_snapshot(&mut self, file: &str, lines: Vec<String>) -> u64 {
        let version = self.versions.entry(file.to_string()).or_insert(0);
        *version += 1;
        let v = *version;
        self.snapshots.insert(file.to_string(), lines);
        v
    }

    /// Compute a delta between the stored snapshot and the new compressed
    /// body lines. Returns `None` if there is no baseline (caller should
    /// store the snapshot and emit full output instead).
    pub fn compute_delta(&self, file: &str, new_lines: &[String]) -> Option<TextDelta> {
        let old_lines = self.snapshots.get(file)?;
        let from_version = self.versions.get(file).copied().unwrap_or(0);

        let (adds, dels, mods) = diff_lines(old_lines, new_lines);

        Some(TextDelta {
            file: file.to_string(),
            from: from_version,
            to: from_version, // caller will increment before storing
            adds,
            dels,
            mods,
        })
    }

    /// Store new snapshot lines and return the delta (if baseline existed)
    /// or None (if no baseline — caller should use full output).
    ///
    /// This is the primary entry point for the delta pipeline:
    /// 1. If baseline exists, compute delta
    /// 2. Store new snapshot and increment version
    /// 3. Return the delta with correct from/to versions
    pub fn compute_and_store(&mut self, file: &str, new_lines: Vec<String>) -> Option<TextDelta> {
        let baseline_exists = self.snapshots.contains_key(file);
        if !baseline_exists {
            self.store_snapshot(file, new_lines);
            return None;
        }

        let from_version = self.versions.get(file).copied().unwrap_or(0);
        let old_lines = self.snapshots.get(file).cloned().unwrap_or_default();

        let (adds, dels, mods) = diff_lines(&old_lines, &new_lines);

        let to_version = self.store_snapshot(file, new_lines);

        let delta = TextDelta {
            file: file.to_string(),
            from: from_version,
            to: to_version,
            adds,
            dels,
            mods,
        };

        if delta.is_empty() { None } else { Some(delta) }
    }
}

/// Apply a text delta to a set of baseline lines, producing the new lines.
///
/// Returns an error message if the delta cannot be applied cleanly.
#[allow(dead_code)]
pub fn apply_text_delta(baseline: &[String], delta: &TextDelta) -> Result<Vec<String>, String> {
    let mut result = baseline.to_vec();

    // First: remove deleted lines
    for del in &delta.dels {
        if let Some(pos) = result.iter().position(|l| l == del) {
            result.remove(pos);
        } else {
            return Err(format!(
                "Cannot apply delta: line not found for removal: {}",
                del
            ));
        }
    }

    // Then: apply modifications
    for (old, new) in &delta.mods {
        if let Some(pos) = result.iter().position(|l| l == old) {
            result[pos] = new.clone();
        } else {
            return Err(format!(
                "Cannot apply delta: line not found for modification: {}",
                old
            ));
        }
    }

    // Finally: add new lines at the end
    for add in &delta.adds {
        result.push(add.clone());
    }

    Ok(result)
}

/// Compute line-level diff between old and new line slices.
///
/// Uses a simplified LCS-based approach that identifies:
/// - Lines in `new` but not in `old` → additions
/// - Lines in `old` but not in `new` → deletions
/// - Lines that differ at the same relative position → modifications
///
/// Returns (adds, dels, mods) where mods are (old_line, new_line) tuples.
fn diff_lines(old: &[String], new: &[String]) -> (Vec<String>, Vec<String>, Vec<(String, String)>) {
    let mut adds = Vec::new();
    let mut dels = Vec::new();
    let mut mods = Vec::new();

    // Build frequency maps for matching
    use std::collections::HashMap;
    let mut old_freq: HashMap<&str, usize> = HashMap::new();
    for line in old {
        *old_freq.entry(line.as_str()).or_insert(0) += 1;
    }
    let mut new_freq: HashMap<&str, usize> = HashMap::new();
    for line in new {
        *new_freq.entry(line.as_str()).or_insert(0) += 1;
    }

    // Lines in new but not in old (or more frequent in new) → additions
    for line in new {
        let old_count = old_freq.get(line.as_str()).copied().unwrap_or(0);
        let new_count = new_freq.get(line.as_str()).copied().unwrap_or(0);
        if new_count > old_count {
            adds.push(line.clone());
            // Decrement to avoid counting extras
            if let Some(cnt) = new_freq.get_mut(line.as_str()) {
                *cnt -= 1;
            }
        }
    }

    // Lines in old but not in new (or more frequent in old) → deletions
    for line in old {
        let old_count = old_freq.get(line.as_str()).copied().unwrap_or(0);
        let new_count = new_freq.get(line.as_str()).copied().unwrap_or(0);
        if old_count > new_count {
            dels.push(line.clone());
            if let Some(cnt) = old_freq.get_mut(line.as_str()) {
                *cnt -= 1;
            }
        }
    }

    // Positional modification detection: find lines at similar positions
    // that differ (not already accounted for by add/del)
    let old_remaining: Vec<&str> = old
        .iter()
        .filter(|l| {
            let cnt = old_freq.get(l.as_str()).copied().unwrap_or(0);
            cnt > 0
        })
        .map(|l| l.as_str())
        .collect();
    let new_remaining: Vec<&str> = new
        .iter()
        .filter(|l| {
            let cnt = new_freq.get(l.as_str()).copied().unwrap_or(0);
            cnt > 0
        })
        .map(|l| l.as_str())
        .collect();

    // Simple positional matching for remaining lines
    let len = old_remaining.len().min(new_remaining.len());
    for i in 0..len {
        if old_remaining[i] != new_remaining[i] {
            mods.push((old_remaining[i].to_string(), new_remaining[i].to_string()));
        }
    }

    (adds, dels, mods)
}

#[cfg(test)]
#[path = "../tests/compression/text_delta.rs"]
mod tests;
