use super::*;
use crate::ir::csharp_resolution::{
    CSharpResolutionBasis, CSharpResolutionEvidence, CSharpResolutionReason, CSharpSourceSpan,
    CSharpTypeIdentityOrigin,
};

fn source(name: &str) -> CSharpTypeIdentity {
    CSharpTypeIdentity {
        qualified_name: format!("Example.{name}"),
        namespace_path: vec!["Example".to_string()],
        containing_type_path: Vec::new(),
        terminal_name: name.to_string(),
        generic_arity: 0,
        origin: CSharpTypeIdentityOrigin::AdmittedSource,
    }
}

fn evidence(file: &str, offset: usize) -> CSharpResolutionEvidence {
    CSharpResolutionEvidence {
        file: file.to_string(),
        span: CSharpSourceSpan {
            start_byte: offset,
            end_byte: offset + 8,
        },
    }
}

fn resolved_arc(
    source: CSharpTypeIdentity,
    target: CSharpTypeIdentity,
    file: &str,
    offset: usize,
) -> CSharpInheritanceArc {
    CSharpInheritanceArc {
        source,
        resolved_base: CSharpResolution::Resolved {
            identity: target,
            basis: CSharpResolutionBasis::ExplicitQualified,
            evidence: vec![evidence(file, offset)],
        },
    }
}

#[test]
fn r5_ancestry_resolves_complete_chains_and_fails_closed_at_every_hop() {
    let leaf = source("Leaf");
    let middle = source("Middle");
    let root = source("Root");
    let terminal = CSharpTypeIdentity::known_external("Framework.Base");
    let graph = CSharpInheritanceGraph {
        arcs: vec![
            resolved_arc(leaf.clone(), middle.clone(), "Leaf.cs", 10),
            resolved_arc(middle.clone(), root.clone(), "Middle.cs", 20),
            resolved_arc(root.clone(), terminal.clone(), "Root.cs", 30),
        ],
    };

    let CSharpResolution::Resolved {
        identity,
        basis,
        evidence: proof,
    } = resolve_type_ancestry(&leaf, &graph)
    else {
        panic!("a complete source chain must resolve to its external terminal");
    };
    assert_eq!(identity, terminal);
    assert_eq!(basis, CSharpResolutionBasis::Inheritance);
    assert_eq!(
        proof,
        vec![
            evidence("Leaf.cs", 10),
            evidence("Middle.cs", 20),
            evidence("Root.cs", 30),
        ]
    );

    let cycle = CSharpInheritanceGraph {
        arcs: vec![
            resolved_arc(leaf.clone(), middle.clone(), "Leaf.cs", 10),
            resolved_arc(middle.clone(), leaf.clone(), "Middle.cs", 20),
        ],
    };
    assert_eq!(
        resolve_type_ancestry(&leaf, &cycle),
        CSharpResolution::Unsupported(CSharpResolutionReason::InheritanceCycle)
    );

    let ambiguous = CSharpInheritanceGraph {
        arcs: vec![CSharpInheritanceArc {
            source: leaf.clone(),
            resolved_base: CSharpResolution::Ambiguous(CSharpResolutionReason::CompetingCandidates),
        }],
    };
    assert_eq!(
        resolve_type_ancestry(&leaf, &ambiguous),
        CSharpResolution::Ambiguous(CSharpResolutionReason::CompetingCandidates)
    );

    let unresolved = CSharpInheritanceGraph {
        arcs: vec![CSharpInheritanceArc {
            source: leaf.clone(),
            resolved_base: CSharpResolution::Unresolved(
                CSharpResolutionReason::UnknownImportContents,
            ),
        }],
    };
    assert_eq!(
        resolve_type_ancestry(&leaf, &unresolved),
        CSharpResolution::Unresolved(CSharpResolutionReason::UnknownImportContents)
    );

    assert_eq!(
        resolve_type_ancestry(&leaf, &CSharpInheritanceGraph::default()),
        CSharpResolution::Unresolved(CSharpResolutionReason::MissingAuthority)
    );
}
