use super::*;
use crate::mcp::tool_handlers::query::csharp_resolution::{TypeKind, TypeShape};

fn declaration(file: &str, path: &[&str], arity: usize) -> Declaration {
    Declaration {
        kind: TypeKind::Class,
        file: file.to_owned(),
        path: path.iter().map(|part| (*part).to_owned()).collect(),
        name: path.last().expect("declaration name").to_string(),
        arity,
        source: format!("class {} {{}}", path.last().expect("declaration name")),
    }
}

fn reference(
    namespace: &[&str],
    nesting: &[&str],
    name: &str,
    arity: usize,
    imports: &[&[&str]],
) -> BaseReference {
    BaseReference {
        file: "Owner.cs".to_owned(),
        owner: "Owner".to_owned(),
        namespace: namespace.iter().map(|part| (*part).to_owned()).collect(),
        nesting: nesting.iter().map(|part| (*part).to_owned()).collect(),
        written: name.to_owned(),
        shape: TypeShape {
            qualifier: Vec::new(),
            name: name.to_owned(),
            arity,
        },
        imports: imports
            .iter()
            .map(|path| path.iter().map(|part| (*part).to_owned()).collect())
            .collect(),
    }
}

fn catalog(declarations: Vec<Declaration>) -> Catalog {
    let declaration_candidates = DeclarationCandidateIndex::build(&declarations);
    Catalog {
        declarations,
        declaration_candidates,
        ..Catalog::default()
    }
}

#[test]
fn declaration_candidates_preserve_resolution_and_ambiguity_contracts() {
    let unique = catalog(vec![declaration("Unique.cs", &["Example", "Widget"], 1)]);
    let target = unique
        .resolve_declaration(&reference(&["Example"], &[], "Widget", 1, &[]))
        .expect("unique declaration");
    assert_eq!(target.file, "Unique.cs");

    let duplicate = catalog(vec![
        declaration("First.cs", &["Example", "Widget"], 1),
        declaration("Second.cs", &["Example", "Widget"], 1),
    ]);
    assert!(
        duplicate
            .resolve_declaration(&reference(&["Example"], &[], "Widget", 1, &[]))
            .is_none(),
        "same path and arity across files must remain ambiguous"
    );

    let arities = catalog(vec![
        declaration("One.cs", &["Example", "Widget"], 1),
        declaration("Two.cs", &["Example", "Widget"], 2),
    ]);
    let target = arities
        .resolve_declaration(&reference(&["Example"], &[], "Widget", 2, &[]))
        .expect("arity-specific declaration");
    assert_eq!(target.file, "Two.cs");

    let imported = catalog(vec![declaration("Imported.cs", &["Imported", "Widget"], 0)]);
    let target = imported
        .resolve_declaration(&reference(&["Current"], &[], "Widget", 0, &[&["Imported"]]))
        .expect("later imported candidate path");
    assert_eq!(target.file, "Imported.cs");

    let nested = catalog(vec![declaration(
        "Nested.cs",
        &["Example", "Outer", "Widget"],
        0,
    )]);
    let target = nested
        .resolve_declaration(&reference(&["Example"], &["Outer"], "Widget", 0, &[]))
        .expect("namespace/nesting candidate path");
    assert_eq!(target.file, "Nested.cs");
}
