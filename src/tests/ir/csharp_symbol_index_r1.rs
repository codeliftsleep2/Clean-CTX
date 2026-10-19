use super::*;

fn key(namespace: &str, name: &str, arity: usize) -> CSharpSymbolKey {
    CSharpSymbolKey {
        namespace_path: namespace.split('.').map(str::to_string).collect(),
        containing_type_path: Vec::new(),
        name: name.to_string(),
        generic_arity: arity,
    }
}

fn part(
    key: CSharpSymbolKey,
    file: &str,
    is_partial: bool,
    base_reference: Option<&str>,
) -> CSharpDeclarationPart {
    CSharpDeclarationPart {
        key,
        kind: CSharpDeclarationKind::Class,
        is_partial,
        file: file.to_string(),
        span: CSharpSourceSpan {
            start_byte: 0,
            end_byte: 10,
        },
        base_references: base_reference.into_iter().map(str::to_string).collect(),
    }
}

#[test]
fn r1_partial_parts_compose_while_invalid_duplicates_fail_closed() {
    let partial_key = key("Example", "Widget", 1);
    let duplicate_key = key("Example", "Duplicate", 0);
    let conflicting_key = key("Example", "Conflicting", 0);
    let index = CSharpSourceSymbolIndex::build([
        part(partial_key.clone(), "Widget.Base.cs", true, Some("Base")),
        part(partial_key.clone(), "Widget.Members.cs", true, None),
        part(duplicate_key.clone(), "Duplicate.One.cs", false, None),
        part(duplicate_key.clone(), "Duplicate.Two.cs", false, None),
        part(
            conflicting_key.clone(),
            "Conflicting.One.cs",
            true,
            Some("FirstBase"),
        ),
        part(
            conflicting_key.clone(),
            "Conflicting.Two.cs",
            true,
            Some("SecondBase"),
        ),
    ]);

    assert_eq!(index.len(), 3, "every symbol key must retain a disposition");
    let CSharpSymbolComposition::Resolved(symbol) =
        index.get(&partial_key).expect("composed partial symbol")
    else {
        panic!("valid partial declarations must compose");
    };
    assert_eq!(symbol.parts.len(), 2);
    assert_eq!(symbol.base_reference.as_deref(), Some("Base"));
    assert_eq!(symbol.parts[0].file, "Widget.Base.cs");
    assert_eq!(symbol.parts[1].file, "Widget.Members.cs");

    assert_eq!(
        index.get(&duplicate_key),
        Some(&CSharpSymbolComposition::Ambiguous(
            CSharpSymbolConflict::NonPartialDuplicate
        ))
    );
    assert_eq!(
        index.get(&conflicting_key),
        Some(&CSharpSymbolComposition::Ambiguous(
            CSharpSymbolConflict::ConflictingBaseTypes
        ))
    );
}
