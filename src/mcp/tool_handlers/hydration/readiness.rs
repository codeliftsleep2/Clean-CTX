use super::ProjectCoverage;
use crate::cbm::GraphBridge;
use std::path::PathBuf;

pub(super) fn provider_busy(bridge: &GraphBridge, projects: &[(String, PathBuf)]) -> bool {
    let busy = bridge.indexing_state().values().any(|state| {
        matches!(
            state,
            crate::cbm::bridge::IndexingState::InProgress { .. }
                | crate::cbm::bridge::IndexingState::ProviderOwned { .. }
        )
    });
    #[cfg(all(test, feature = "rust"))]
    let busy = busy
        || projects.iter().any(|(project, _)| {
            matches!(
                super::test_project_readiness(project),
                Some(super::TestProjectReadiness::StillIndexing)
            )
        });
    #[cfg(not(all(test, feature = "rust")))]
    let _ = projects;
    busy
}

pub(super) fn skip_busy_provider(
    provider_busy: bool,
    project: String,
    root: PathBuf,
    fallback_roots: &mut Vec<PathBuf>,
    coverage: &mut Vec<ProjectCoverage>,
) -> bool {
    if !provider_busy {
        return false;
    }
    fallback_roots.push(root);
    coverage.push(ProjectCoverage {
        project,
        status: "skipped",
        readiness: None,
        reason: Some("provider_indexing"),
    });
    true
}
