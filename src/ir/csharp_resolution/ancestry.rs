//! Framework-neutral traversal of already-resolved C# class-base arcs.

use super::{
    CSharpResolution, CSharpResolutionBasis, CSharpResolutionReason, CSharpTypeIdentity,
    CSharpTypeIdentityOrigin,
};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CSharpInheritanceArc {
    pub source: CSharpTypeIdentity,
    pub resolved_base: CSharpResolution<CSharpTypeIdentity>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CSharpInheritanceGraph {
    pub arcs: Vec<CSharpInheritanceArc>,
}

/// Resolve a source type's class ancestry to an admitted external terminal.
pub fn resolve_type_ancestry(
    start: &CSharpTypeIdentity,
    graph: &CSharpInheritanceGraph,
) -> CSharpResolution<CSharpTypeIdentity> {
    let mut current = start.clone();
    let mut visited = HashSet::new();
    let mut proof = Vec::new();

    loop {
        if current.origin == CSharpTypeIdentityOrigin::KnownExternal {
            return CSharpResolution::Resolved {
                identity: current,
                basis: CSharpResolutionBasis::Inheritance,
                evidence: proof,
            };
        }
        if !visited.insert(current.clone()) {
            return CSharpResolution::Unsupported(CSharpResolutionReason::InheritanceCycle);
        }

        let mut matching = graph.arcs.iter().filter(|arc| arc.source == current);
        let Some(arc) = matching.next() else {
            return CSharpResolution::Unresolved(CSharpResolutionReason::MissingAuthority);
        };
        if matching.next().is_some() {
            return CSharpResolution::Ambiguous(CSharpResolutionReason::CompetingCandidates);
        }

        match &arc.resolved_base {
            CSharpResolution::Resolved {
                identity, evidence, ..
            } => {
                proof.extend(evidence.iter().cloned());
                current = identity.clone();
            }
            CSharpResolution::Ambiguous(reason) => {
                return CSharpResolution::Ambiguous(*reason);
            }
            CSharpResolution::Unsupported(reason) => {
                return CSharpResolution::Unsupported(*reason);
            }
            CSharpResolution::Unresolved(reason) => {
                return CSharpResolution::Unresolved(*reason);
            }
        }
    }
}

#[cfg(test)]
#[path = "../../tests/ir/csharp_ancestry_resolution_r5.rs"]
mod tests;
