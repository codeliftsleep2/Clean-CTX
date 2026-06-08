// src/ir/symbol_table.rs
//
// Phase B: Global Symbol Table
// Unified symbol registry that subsumes SymbolDictionary and PathDictionary
// into a cross-stage, version-tracked registry.
//
// This module provides:
//   - GlobalSymbolTable: cross-stage symbol registry with version tracking
//   - SymbolEntry: per-symbol metadata (alias, original name, kind, file, versions)
//   - SymbolKind: classification of symbol types (class, method, field, etc.)

use std::collections::HashMap;

/// What kind of symbol this is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolKind {
    Class,
    Method,
    Field,
    Interface,
    Param,
    Import,
    Type,
}

impl SymbolKind {
    /// Return the alias prefix for this kind.
    pub fn prefix(&self) -> &'static str {
        match self {
            SymbolKind::Class => "C",
            SymbolKind::Method => "M",
            SymbolKind::Field => "F",
            SymbolKind::Interface => "I",
            SymbolKind::Param => "P",
            SymbolKind::Import => "IM",
            SymbolKind::Type => "T",
        }
    }
}

/// A single symbol entry in the global registry.
#[derive(Debug, Clone)]
pub struct SymbolEntry {
    /// Machine alias (e.g., "C1", "M3", "P2")
    pub alias: String,
    /// Original name (e.g., "SampleService", "processComplexData")
    pub original: String,
    /// Symbol kind
    pub kind: SymbolKind,
    /// Which file defines this symbol
    pub file_id: String,
    /// Version when first registered
    pub version_first: u64,
    /// Version when last modified
    pub version_last: u64,
}

/// Cross-stage global symbol table.
///
/// Tracks all symbols across all files, with version-based change tracking.
/// Provides efficient lookup by alias, original name, file membership, and
/// version range.
#[derive(Debug, Clone)]
pub struct GlobalSymbolTable {
    /// Monotonically increasing version counter
    version: u64,

    /// alias → SymbolEntry
    symbols: HashMap<String, SymbolEntry>,

    /// original_name → alias (reverse index)
    reverse: HashMap<String, String>,

    /// file_id → set of symbol aliases defined in that file
    file_members: HashMap<String, Vec<String>>,

    /// Next alias counter per kind (C1, C2, ... M1, M2, ... F1, F2, ...)
    counters: HashMap<SymbolKind, u32>,
}

impl GlobalSymbolTable {
    /// Create a new empty symbol table.
    pub fn new() -> Self {
        Self {
            version: 0,
            symbols: HashMap::new(),
            reverse: HashMap::new(),
            file_members: HashMap::new(),
            counters: HashMap::new(),
        }
    }

    /// Get current version.
    pub fn version(&self) -> u64 {
        self.version
    }

    /// Bump version (called after each delta application).
    pub fn bump_version(&mut self) -> u64 {
        self.version += 1;
        self.version
    }

    /// Set the version explicitly (used during catch-up / bootstrap).
    pub fn set_version(&mut self, version: u64) {
        self.version = version;
    }

    /// Generate the next alias for a given kind.
    ///
    /// Aliases follow the pattern: <prefix><counter>
    /// For example: C1, C2, M1, M2, F1, F2, etc.
    pub fn next_alias(&mut self, kind: SymbolKind) -> String {
        let counter = self.counters.entry(kind).or_insert(0);
        *counter += 1;
        format!("{}{}", kind.prefix(), counter)
    }

    /// Register a new symbol.
    ///
    /// If an entry with the same alias already exists, it will be overwritten
    /// (for delta replay scenarios). The reverse index and file membership
    /// are updated accordingly. When overwriting, `version_last` is bumped
    /// to the current version via `touch()` (F-24).
    pub fn register(
        &mut self,
        alias: String,
        original: String,
        kind: SymbolKind,
        file_id: &str,
    ) {
        // If alias already exists, clean up old reverse/file members first
        if let Some(existing) = self.symbols.get(&alias) {
            self.reverse.remove(&existing.original);
            if let Some(members) = self.file_members.get_mut(&existing.file_id) {
                members.retain(|a| a != &alias);
            }
        }

        let entry = SymbolEntry {
            alias: alias.clone(),
            original: original.clone(),
            kind,
            file_id: file_id.to_string(),
            version_first: self.version,
            version_last: self.version,
        };

        self.reverse.insert(original, alias.clone());
        self.symbols.insert(alias.clone(), entry);
        self.file_members
            .entry(file_id.to_string())
            .or_default()
            .push(alias.clone());

        // F-24: Bump version_last after overwrite so `get_changed_since`
        // correctly reports the re-registered symbol as changed.
        self.touch(&alias);
    }

    /// Unregister a symbol (for delta deletions).
    ///
    /// Removes the symbol from all indexes and returns the entry if found.
    pub fn unregister(&mut self, alias: &str) -> Option<SymbolEntry> {
        if let Some(entry) = self.symbols.remove(alias) {
            self.reverse.remove(&entry.original);
            if let Some(members) = self.file_members.get_mut(&entry.file_id) {
                members.retain(|a| a != alias);
            }
            Some(entry)
        } else {
            None
        }
    }

    /// Update a symbol's version (for delta modifications).
    ///
    /// Sets `version_last` to the current table version.
    pub fn touch(&mut self, alias: &str) {
        if let Some(entry) = self.symbols.get_mut(alias) {
            entry.version_last = self.version;
        }
    }

    /// Look up a symbol by alias.
    pub fn get(&self, alias: &str) -> Option<&SymbolEntry> {
        self.symbols.get(alias)
    }

    /// Look up a symbol by original name.
    pub fn get_by_original(&self, original: &str) -> Option<&SymbolEntry> {
        self.reverse
            .get(original)
            .and_then(|a| self.symbols.get(a))
    }

    /// Get the alias for an original name (convenience method).
    pub fn alias_for(&self, original: &str) -> Option<&str> {
        self.reverse.get(original).map(|s| s.as_str())
    }

    /// Get all symbols for a file.
    pub fn get_file_symbols(&self, file_id: &str) -> Vec<&SymbolEntry> {
        self.file_members
            .get(file_id)
            .map(|aliases| {
                aliases
                    .iter()
                    .filter_map(|a| self.symbols.get(a))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all symbols modified in a version range.
    ///
    /// Returns all symbols whose `version_last` is greater than `since_version`.
    /// This is useful for determining which symbols changed between deltas.
    pub fn get_changed_since(&self, since_version: u64) -> Vec<&SymbolEntry> {
        self.symbols
            .values()
            .filter(|e| e.version_last > since_version)
            .collect()
    }

    /// Get all symbols modified at or after a given version.
    pub fn get_changed_at_or_after(&self, version: u64) -> Vec<&SymbolEntry> {
        self.symbols
            .values()
            .filter(|e| e.version_last >= version)
            .collect()
    }

    /// Get all symbols in the table.
    pub fn all_symbols(&self) -> Vec<&SymbolEntry> {
        self.symbols.values().collect()
    }

    /// Get the total number of registered symbols.
    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    /// Check if the table is empty.
    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }

    /// Get the number of files tracked.
    pub fn file_count(&self) -> usize {
        self.file_members.len()
    }

    /// Get the list of file IDs tracked.
    pub fn file_ids(&self) -> Vec<&str> {
        self.file_members.keys().map(|s| s.as_str()).collect()
    }

    /// Clear all symbols and reset counters (for full rebuild scenarios).
    pub fn clear(&mut self) {
        self.symbols.clear();
        self.reverse.clear();
        self.file_members.clear();
        self.counters.clear();
        self.version = 0;
    }

    /// Check if a symbol exists by alias.
    pub fn contains(&self, alias: &str) -> bool {
        self.symbols.contains_key(alias)
    }

    /// Check if a symbol exists by original name.
    pub fn contains_original(&self, original: &str) -> bool {
        self.reverse.contains_key(original)
    }
}

impl Default for GlobalSymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "../tests/ir/symbol_table.rs"]
mod tests;