#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct CapabilityCoverage {
    csharp_exact_calls: Option<CSharpExactCallCoverage>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CSharpExactCallCoverage {
    fingerprint: String,
    generation: u64,
}

impl super::WorkspaceIndex {
    pub(crate) fn commit_csharp_exact_call_coverage(
        &mut self,
        fingerprint: String,
        generation: u64,
    ) {
        self.capability_coverage.csharp_exact_calls = Some(CSharpExactCallCoverage {
            fingerprint,
            generation,
        });
    }

    pub(crate) fn has_current_csharp_exact_call_coverage(
        &self,
        fingerprint: &str,
        generation: u64,
    ) -> bool {
        self.capability_coverage
            .csharp_exact_calls
            .as_ref()
            .is_some_and(|coverage| {
                coverage.fingerprint == fingerprint && coverage.generation == generation
            })
    }

    pub(crate) fn dirty_csharp_exact_call_coverage(&mut self) {
        self.capability_coverage.csharp_exact_calls = None;
    }

    #[cfg(feature = "csharp")]
    pub(crate) fn has_csharp_publication_generation(&self, generation: u64) -> bool {
        self.capability_coverage
            .csharp_exact_calls
            .as_ref()
            .is_some_and(|coverage| coverage.generation == generation)
    }
}
