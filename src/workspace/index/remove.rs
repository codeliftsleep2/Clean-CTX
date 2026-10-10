// src/workspace/index/remove.rs
//
// File removal / recompilation cleanup for WorkspaceIndex.
//
// Removal is file-local and occurrence-exact. `file_edges[file]` holds the edge
// occurrences that file asserted, so the work performed is proportional to the
// affected file and never scans the accumulated workspace.
//
// Presence of a same-named entity (or of an identical semantic triple) in
// another file must not affect removal: only the occurrences asserted by the
// removed file are dropped.

use super::WorkspaceIndex;
use super::edges::EdgeKey;
use std::collections::HashSet;

impl WorkspaceIndex {
    /// Retract every semantic projection owned by a file below `root`.
    ///
    /// This is the current-source authority transition used after a successful
    /// explicit external refresh. The refresh proves that every projection
    /// from the previous root generation must be revalidated before it can be
    /// current again; later hydration republishes eligible owners from fresh
    /// canonical source. Retraction is deliberately root-scoped and does not
    /// treat any target-specific discovery result as a repository inventory.
    pub fn remove_files_in_root(&mut self, root: &str) -> usize {
        let mut owners: HashSet<String> = self
            .file_map
            .keys()
            .chain(self.file_edges.keys())
            .chain(self.semantic_coverage.keys())
            .filter(|owner| crate::workspace::path_identity::is_within_root(owner, root))
            .cloned()
            .collect();
        let removed = owners.len();
        for owner in owners.drain() {
            self.remove_file(&owner);
        }
        removed
    }

    /// Remove all edge occurrences and entity occurrences originating from the
    /// given file.
    ///
    /// Called when a file is recompiled (stale edges removed before fresh ones
    /// are inserted) or when a file is deleted from the workspace. Preserves all
    /// other files' edges and entities.
    pub fn remove_file(&mut self, file_path: &str) {
        self.semantic_coverage.remove(file_path);
        #[cfg(test)]
        self.name_cleanup_buckets_examined.clear();

        // Phase 1: Remove exactly the edge occurrences this file asserted.
        // `file_edges[file]` contains only this file's occurrence keys, so this
        // can neither drop another file's evidence for the same semantic triple
        // nor scan the accumulated workspace.
        let removed_keys = self.file_edges.remove(file_path).unwrap_or_default();
        self.edge_count = self.edge_count.saturating_sub(removed_keys.len());
        for key in &removed_keys {
            self.edge_set.remove(key);
            self.drop_occurrence(key);
        }

        // Phase 2: Remove entity keys tracked to this file.
        if let Some(keys) = self.file_map.remove(file_path) {
            let affected_names: HashSet<String> =
                keys.iter().map(|(_, _, name)| name.clone()).collect();

            // Remove entities that only existed in this file.
            for key in &keys {
                if let Some(occurrences) = self.entities.get_mut(key) {
                    occurrences.retain(|e| e.file.as_deref() != Some(file_path));
                    if occurrences.is_empty() {
                        self.entities.remove(key);
                    }
                }
            }

            // Drop adjacency buckets whose identity no longer exists in the
            // index: Phase 1 already removed this file's occurrences, and no
            // surviving edge can reference a vanished identity, so such a
            // bucket holds no evidence.
            for key in &keys {
                if !self.entities.contains_key(key) {
                    self.forward.remove(key);
                    self.reverse.remove(key);
                }
            }

            // Phase 3: Clean only names represented by the removed file.
            // Unrelated name buckets cannot contain any of `keys`, so visiting
            // them would make recompilation scale with total workspace size.
            for name in affected_names {
                #[cfg(test)]
                self.name_cleanup_buckets_examined.push(name.clone());

                let remove_bucket = if let Some(name_keys) = self.name_index.get_mut(&name) {
                    name_keys.retain(|key| self.entities.contains_key(key));
                    name_keys.is_empty()
                } else {
                    false
                };
                if remove_bucket {
                    self.name_index.remove(&name);
                }
            }
        }
    }

    /// Drop the stored occurrence identified by `key` from both adjacency
    /// indexes.
    ///
    /// Exact and file-local: only the occurrence asserted by the asserting
    /// file of `key` with the triple of `key` is removed. Entries asserted by
    /// other files, including entries carrying an identical triple, are
    /// preserved.
    fn drop_occurrence(&mut self, key: &EdgeKey) {
        let subject_key = key.subject_key();
        if let Some(entries) = self.forward.get_mut(&subject_key) {
            entries.retain(|stored| !key.matches(&stored.asserting_file, &stored.edge));
        }
        let object_key = key.object_key();
        if let Some(entries) = self.reverse.get_mut(&object_key) {
            entries.retain(|stored| !key.matches(&stored.asserting_file, &stored.edge));
        }
    }

    #[cfg(test)]
    pub(super) fn name_cleanup_buckets_examined(&self) -> &[String] {
        &self.name_cleanup_buckets_examined
    }
}
