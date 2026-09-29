use super::WorkspaceIndex;
use crate::layers::meta::semantic::SemanticEdge;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SemanticFidelity {
    Low,
    Medium,
    High,
}

impl SemanticFidelity {
    pub fn for_request(fidelity: crate::compression::Fidelity) -> Self {
        use crate::compression::Fidelity;

        match fidelity {
            Fidelity::Low => Self::Low,
            Fidelity::Medium => Self::Medium,
            Fidelity::High | Fidelity::Edit | Fidelity::Verbatim => Self::High,
        }
    }

    pub fn from_compilation(fidelity: crate::compression::Fidelity) -> Option<Self> {
        use crate::compression::Fidelity;

        match fidelity {
            Fidelity::Low => Some(Self::Low),
            Fidelity::Medium => Some(Self::Medium),
            Fidelity::High => Some(Self::High),
            Fidelity::Edit | Fidelity::Verbatim => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SemanticCoverage {
    pub(super) fidelity: SemanticFidelity,
    pub(super) source_hash: String,
}

impl WorkspaceIndex {
    pub fn has_current_semantic_projection(
        &self,
        file_path: &str,
        fidelity: SemanticFidelity,
        source_hash: &str,
    ) -> bool {
        self.semantic_coverage
            .get(file_path)
            .is_some_and(|coverage| {
                coverage.source_hash == source_hash && coverage.fidelity >= fidelity
            })
    }

    pub fn replace_semantic_projection(
        &mut self,
        file_path: &str,
        edges: Vec<SemanticEdge>,
        fidelity: SemanticFidelity,
        source_hash: String,
    ) {
        self.remove_file(file_path);
        self.add_edges(file_path, edges);
        self.semantic_coverage.insert(
            file_path.to_string(),
            SemanticCoverage {
                fidelity,
                source_hash,
            },
        );
    }
}
