//! File-scoped ordering for competing semantic publication candidates.

use super::McpState;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub(super) struct SemanticPublicationClock {
    next_generation: u64,
    committed_generation: u64,
}

pub(super) type SemanticPublicationClocks =
    Mutex<HashMap<String, Arc<Mutex<SemanticPublicationClock>>>>;

pub(crate) struct SemanticPublicationTicket {
    clock: Arc<Mutex<SemanticPublicationClock>>,
    generation: u64,
}

#[cfg(test)]
#[path = "../../tests/mcp/semantic_publication_clock_support.rs"]
mod snapshot_test_support;

impl SemanticPublicationTicket {
    /// Commit only when no newer candidate for this owner has already
    /// committed. The clock remains held through the complete publication so
    /// its durable and live authorities advance as one ordered generation.
    pub(crate) fn commit<T, E>(
        self,
        source_is_current: impl FnOnce() -> Result<bool, E>,
        publish: impl FnOnce() -> Result<T, E>,
    ) -> Result<Option<T>, E> {
        let mut clock = lock_or_recover!(self.clock.lock(), "semantic_publication_clock");
        if self.generation <= clock.committed_generation || !source_is_current()? {
            return Ok(None);
        }
        let published = publish()?;
        clock.committed_generation = self.generation;
        Ok(Some(published))
    }
}

impl McpState {
    fn semantic_publication_clock(&self, file_path: &str) -> Arc<Mutex<SemanticPublicationClock>> {
        let owner = self.semantic_owner_path(file_path);
        lock_or_recover!(
            self.semantic_publication_clocks.lock(),
            "semantic_publication_clocks"
        )
        .entry(owner)
        .or_insert_with(|| Arc::new(Mutex::new(SemanticPublicationClock::default())))
        .clone()
    }

    /// Run a checkpoint capture while publication for this semantic owner is
    /// unable to advance. This observes authority; it does not create a new
    /// publication generation or change the committed generation.
    pub(crate) fn with_semantic_authority_snapshot<T>(
        &self,
        file_path: &str,
        capture: impl FnOnce() -> T,
    ) -> T {
        let clock = self.semantic_publication_clock(file_path);
        let _snapshot = lock_or_recover!(clock.lock(), "semantic_publication_clock");
        capture()
    }

    /// Serialize one already-validated authority mutation with snapshots and
    /// other mutations for the same owner. The callback must not recursively
    /// acquire this owner's publication clock.
    pub(crate) fn with_semantic_authority_update<T>(
        &self,
        file_path: &str,
        update: impl FnOnce() -> T,
    ) -> T {
        self.with_semantic_authority_snapshot(file_path, update)
    }

    /// Assign a monotonic generation before reading the source snapshot that
    /// may later become authoritative.
    pub(crate) fn begin_semantic_publication(&self, file_path: &str) -> SemanticPublicationTicket {
        let owner = self.semantic_owner_path(file_path);
        let clock = self.semantic_publication_clock(&owner);
        #[cfg(test)]
        snapshot_test_support::publication_attempted(&owner);
        let generation = {
            let mut clock_state = lock_or_recover!(clock.lock(), "semantic_publication_clock");
            clock_state.next_generation += 1;
            clock_state.next_generation
        };
        SemanticPublicationTicket { clock, generation }
    }

    #[cfg(test)]
    pub(crate) fn arm_semantic_publication_attempt(&self, file_path: &str) {
        snapshot_test_support::arm(&self.semantic_owner_path(file_path));
    }

    #[cfg(test)]
    pub(crate) fn wait_for_semantic_publication_attempt(&self) {
        snapshot_test_support::wait_until_attempted();
    }
}
