use super::*;
use crate::ir::csharp_symbols::{CSharpDeclarationPart, CSharpSourceSymbolIndex, CSharpSymbolKey};

fn part(
    namespace: &[&str],
    containing: &[&str],
    name: &str,
    arity: usize,
    file: &str,
) -> CSharpDeclarationPart {
    CSharpDeclarationPart {
        key: CSharpSymbolKey {
            namespace_path: namespace.iter().map(|part| (*part).to_string()).collect(),
            containing_type_path: containing.iter().map(|part| (*part).to_string()).collect(),
            name: name.to_string(),
            generic_arity: arity,
        },
        kind: CSharpDeclarationKind::Class,
        is_partial: false,
        file: file.to_string(),
        span: CSharpSourceSpan {
            start_byte: 0,
            end_byte: 10,
        },
        base_references: Vec::new(),
    }
}

#[test]
fn r2_explicit_resolution_unifies_external_source_generic_and_nested_identity() {
    let widget = part(&["Example", "Models"], &[], "Widget", 0, "Widget.cs");
    let box_type = part(&["Example", "Models"], &[], "Box", 1, "Box.cs");
    let nested = part(&["Example"], &["Outer"], "Inner", 0, "Outer.cs");
    let ambiguous_one = part(&["Example"], &[], "Duplicate", 0, "One.cs");
    let ambiguous_two = part(&["Example"], &[], "Duplicate", 0, "Two.cs");
    let index =
        CSharpSourceSymbolIndex::build([widget, box_type, nested, ambiguous_one, ambiguous_two]);
    let external = [CSharpTypeIdentity::known_external("External.Api.Contract")];

    for written in ["Example.Models.Widget", "global::Example.Models.Widget"] {
        let CSharpResolution::Resolved { identity, .. } =
            resolve_explicit_type_reference(written, &external, &index)
        else {
            panic!("{written} must resolve to the admitted source symbol");
        };
        assert_eq!(identity.qualified_name, "Example.Models.Widget");
        assert_eq!(identity.origin, CSharpTypeIdentityOrigin::AdmittedSource);
        assert_eq!(identity.generic_arity, 0);
    }

    let CSharpResolution::Resolved { identity, .. } =
        resolve_explicit_type_reference("Example.Models.Box<string>", &external, &index)
    else {
        panic!("constructed generic syntax must resolve by definition arity");
    };
    assert_eq!(identity.qualified_name, "Example.Models.Box");
    assert_eq!(identity.generic_arity, 1);

    let CSharpResolution::Resolved { identity, .. } =
        resolve_explicit_type_reference("Example.Outer.Inner", &external, &index)
    else {
        panic!("nested source identity must retain its containing path");
    };
    assert_eq!(identity.containing_type_path, vec!["Outer".to_string()]);

    assert!(matches!(
        resolve_explicit_type_reference("External.Api.Contract", &external, &index),
        CSharpResolution::Resolved {
            identity: CSharpTypeIdentity {
                origin: CSharpTypeIdentityOrigin::KnownExternal,
                ..
            },
            ..
        }
    ));
    assert_eq!(
        resolve_explicit_type_reference("Example.Duplicate", &external, &index),
        CSharpResolution::Ambiguous(CSharpResolutionReason::CompetingCandidates)
    );
    assert_eq!(
        resolve_explicit_type_reference("Missing.Api.Type", &external, &index),
        CSharpResolution::Unresolved(CSharpResolutionReason::MissingAuthority)
    );
}
