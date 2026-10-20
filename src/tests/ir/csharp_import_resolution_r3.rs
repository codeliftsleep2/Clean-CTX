use super::*;
use crate::ir::csharp_symbols::{CSharpDeclarationPart, CSharpSourceSymbolIndex, CSharpSymbolKey};

fn part(namespace: &[&str], name: &str, arity: usize, file: &str) -> CSharpDeclarationPart {
    CSharpDeclarationPart {
        key: CSharpSymbolKey {
            namespace_path: namespace.iter().map(|part| (*part).to_string()).collect(),
            containing_type_path: Vec::new(),
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

fn context(namespace: &[&str], imports: &[&[&str]]) -> CSharpOrdinaryImportContext {
    CSharpOrdinaryImportContext {
        namespace_path: namespace.iter().map(|part| (*part).to_string()).collect(),
        containing_type_path: Vec::new(),
        ordinary_imports: imports
            .iter()
            .map(|path| path.iter().map(|part| (*part).to_string()).collect())
            .collect(),
    }
}

#[test]
fn issue_149_ordinary_imports_require_unique_proven_candidates() {
    let empty = CSharpSourceSymbolIndex::default();
    let ef = CSharpTypeIdentity::known_external("Microsoft.EntityFrameworkCore.DbContext");
    let other = CSharpTypeIdentity::known_external("Other.Framework.DbContext");
    let ef_context = context(&["Consumer"], &[&["Microsoft", "EntityFrameworkCore"]]);

    let CSharpResolution::Resolved { identity, .. } = resolve_ordinary_import_type_reference(
        "DbContext",
        &ef_context,
        std::slice::from_ref(&ef),
        &empty,
    ) else {
        panic!("one known import with complete negative source proof must resolve");
    };
    assert_eq!(identity, ef);

    let local = CSharpSourceSymbolIndex::build([part(&["Consumer"], "DbContext", 0, "Local.cs")]);
    let CSharpResolution::Resolved { identity, .. } = resolve_ordinary_import_type_reference(
        "DbContext",
        &ef_context,
        std::slice::from_ref(&ef),
        &local,
    ) else {
        panic!("same-namespace source identity must shadow an imported external identity");
    };
    assert_eq!(identity.qualified_name, "Consumer.DbContext");
    assert_eq!(identity.origin, CSharpTypeIdentityOrigin::AdmittedSource);

    let competing = context(
        &["Consumer"],
        &[
            &["Microsoft", "EntityFrameworkCore"],
            &["Other", "Framework"],
        ],
    );
    assert_eq!(
        resolve_ordinary_import_type_reference(
            "DbContext",
            &competing,
            &[ef.clone(), other],
            &empty
        ),
        CSharpResolution::Ambiguous(CSharpResolutionReason::CompetingCandidates)
    );

    let unknown_competitor = context(
        &["Consumer"],
        &[
            &["Microsoft", "EntityFrameworkCore"],
            &["Unknown", "Package"],
        ],
    );
    let CSharpResolution::Resolved { identity, .. } = resolve_ordinary_import_type_reference(
        "DbContext",
        &unknown_competitor,
        std::slice::from_ref(&ef),
        &empty,
    ) else {
        panic!("an unmodeled import is not evidence of a same-name competitor");
    };
    assert_eq!(identity, ef);

    let unknown_only = context(&["Consumer"], &[&["Unknown", "Package"]]);
    assert_eq!(
        resolve_ordinary_import_type_reference("DbContext", &unknown_only, &[], &empty),
        CSharpResolution::Unresolved(CSharpResolutionReason::UnknownImportContents)
    );

    let imported_source =
        CSharpSourceSymbolIndex::build([part(&["Example", "Models"], "Widget", 1, "Widget.cs")]);
    let source_context = context(&["Consumer"], &[&["Example", "Models"]]);
    let CSharpResolution::Resolved { identity, .. } = resolve_ordinary_import_type_reference(
        "Widget<string>",
        &source_context,
        &[],
        &imported_source,
    ) else {
        panic!("an imported admitted-source generic must resolve by arity");
    };
    assert_eq!(identity.qualified_name, "Example.Models.Widget");
    assert_eq!(identity.generic_arity, 1);
}
