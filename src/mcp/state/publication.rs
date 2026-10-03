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
    /// Assign a monotonic generation before reading the source snapshot that
    /// may later become authoritative.
    pub(crate) fn begin_semantic_publication(&self, file_path: &str) -> SemanticPublicationTicket {
        let owner = self.semantic_owner_path(file_path);
        let clock = lock_or_recover!(
            self.semantic_publication_clocks.lock(),
            "semantic_publication_clocks"
        )
        .entry(owner)
        .or_insert_with(|| Arc::new(Mutex::new(SemanticPublicationClock::default())))
        .clone();
        let generation = {
            let mut clock_state = lock_or_recover!(clock.lock(), "semantic_publication_clock");
            clock_state.next_generation += 1;
            clock_state.next_generation
        };
        SemanticPublicationTicket { clock, generation }
    }
}
