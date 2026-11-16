//! Reconcile confirmed source deletion with the existing owner publication boundary.
//!
//! One query preparation snapshots O(U+A) coverage owners and existing aliases,
//! probes each
//! admitted owner once, then rechecks only missing owners at their commit boundary.
//! No source inventory, mapping evaluation, persistent cache or response filter.
//! Successful owners stay untouched; errors refuse current consumption, not delete.
use super::McpState;
use crate::workspace::scope::WorkspaceScope;
use std::collections::HashSet;
use std::io::{Error, ErrorKind};

fn source_present(owner: &str) -> Result<bool, Error> {
    #[cfg(test)]
    crate::mcp::tool_handlers::query::work_metrics::record_retirement_probe();
    match std::fs::symlink_metadata(owner) {
        Ok(metadata) if metadata.is_file() => std::fs::File::open(owner).map(|_| true),
        // A dangling link or wrong kind is not an established deleted owner.
        Ok(metadata) if metadata.file_type().is_symlink() => {
            std::fs::metadata(owner).and_then(|target| {
                if target.is_file() {
                    std::fs::File::open(owner).map(|_| true)
                } else {
                    Err(Error::other("Semantic source target is not a file"))
                }
            })
        }
        Ok(_) => Err(Error::other("Semantic source owner is not a file")),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

impl McpState {
    pub(crate) fn reconcile_deleted_semantic_sources(
        &self,
        scope: Option<&WorkspaceScope>,
    ) -> Result<(), String> {
        self.reconcile_semantic_sources_with(scope, source_present)
            .map(|_| ())
    }

    fn reconcile_semantic_sources_with(
        &self,
        scope: Option<&WorkspaceScope>,
        mut probe: impl FnMut(&str) -> Result<bool, Error>,
    ) -> Result<usize, String> {
        #[cfg(test)]
        crate::mcp::tool_handlers::query::work_metrics::record_retirement_pass();
        let owners = self.workspace_index_read().semantic_projection_owners();
        let indexed: HashSet<_> = owners.iter().cloned().collect();
        let mut examined = 0;
        for owner in owners {
            if scope.is_some_and(|scope| !scope.admits(&owner)) {
                continue;
            }
            examined += 1;
            let current = probe(&owner).map_err(|error| {
                format!("Cannot establish semantic source currentness for {owner}: {error}")
            })?;
            if current {
                continue;
            }
            let ticket = self.begin_semantic_publication(&owner);
            let retired = ticket.commit(
                || {
                    probe(&owner).map(|present| !present).map_err(|error| {
                        format!("Cannot confirm semantic source deletion for {owner}: {error}")
                    })
                },
                || {
                    self.workspace_index_lock().remove_file(&owner);
                    if let Some(alias) = self.alias_for_path(&owner) {
                        self.ir_context_lock().remove_file(&alias);
                        self.llm_text_cache_lock().remove(&alias);
                        self.forget_semantic_state(&alias);
                    }
                    // Durable rows and path aliases remain historical and retain
                    // their existing restore/replay compatibility requirements.
                    self.forget_context_caches(&owner);
                    crate::mcp::tool_handlers::hydration::invalidate_discovery_for_edited_path(
                        self, &owner,
                    );
                    #[cfg(feature = "csharp")]
                    self.mark_csharp_call_authority_dirty_for_file(&owner)?;
                    Ok::<_, String>(())
                },
            )?;
            if retired.is_none() {
                return Err(format!(
                    "Semantic source changed while retiring {owner}; retry"
                ));
            }
        }
        // Retirement retains dictionary paths as history. A known path that
        // reappears without an indexed projection invalidates completion only;
        // normal source discovery and compilation must establish any new facts.
        // No watcher, tombstone store, cached answer or absence inference.
        let historical: Vec<_> = self.dict_lock().known_paths().map(str::to_owned).collect();
        let mut reappeared = Vec::new();
        for owner in historical {
            if indexed.contains(&owner) || scope.is_some_and(|scope| !scope.admits(&owner)) {
                continue;
            }
            examined += 1;
            if probe(&owner).is_ok_and(|present| present) {
                reappeared.push(owner);
            }
            // An unavailable historical path has no current projection to
            // consume or retire. It cannot supply facts or deletion evidence.
        }
        crate::mcp::tool_handlers::hydration::invalidate_discovery_for_edited_paths(
            self,
            &reappeared,
        );
        Ok(examined)
    }

    #[cfg(test)]
    pub(crate) fn reconcile_semantic_sources_for_test(
        &self,
        probe: impl FnMut(&str) -> Result<bool, Error>,
    ) -> Result<usize, String> {
        self.reconcile_semantic_sources_with(None, probe)
    }
}
