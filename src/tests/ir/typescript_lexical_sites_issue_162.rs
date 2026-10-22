use super::{Resolution, TypeScriptAuthority};

#[test]
fn issue_162_catch_occurrence_has_exact_owner_and_cannot_escape_to_following_use() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("app.ts");
    let source = "try {} catch (caught) { use(caught); } use(caught);";
    std::fs::write(&file, source).unwrap();
    let capture = TypeScriptAuthority::capture(&file, source, &[root.path().to_path_buf()]);
    let inside = source.find("use(caught)").unwrap() + 4;
    let outside = source.rfind("use(caught)").unwrap() + 4;
    let (_, _, Resolution::Resolved(id)) = capture
        .reference_sites
        .iter()
        .find(|site| site.0 == inside)
        .unwrap()
    else {
        panic!("catch binding must be retained");
    };
    assert_eq!(id.kind, "catch_clause");
    assert!(capture.declarations.contains(id));
    assert!(
        capture
            .reference_sites
            .iter()
            .any(|site| site.0 == outside && matches!(site.2, Resolution::Unresolved(_)))
    );
    let declaration = source.find("caught").unwrap();
    assert!(
        !capture
            .reference_sites
            .iter()
            .any(|site| site.0 == declaration)
    );
}

#[test]
fn issue_162_function_var_site_retains_declaration_without_escaping_function() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("app.ts");
    let source = "function f() { if (flag) { var item = 1; } use(item); } use(item);";
    std::fs::write(&file, source).unwrap();
    let capture = TypeScriptAuthority::capture(&file, source, &[root.path().to_path_buf()]);
    let inside = source.find("use(item)").unwrap() + 4;
    let outside = source.rfind("use(item)").unwrap() + 4;
    let (_, _, Resolution::Resolved(id)) = capture
        .reference_sites
        .iter()
        .find(|site| site.0 == inside)
        .unwrap()
    else {
        panic!("function var must bind across its nested block");
    };
    assert_eq!(id.kind, "variable_declarator");
    assert_eq!(&source[id.start..id.end], "item = 1");
    assert!(capture.declarations.contains(id));
    assert!(
        capture
            .reference_sites
            .iter()
            .any(|site| site.0 == outside && matches!(site.2, Resolution::Unresolved(_)))
    );
}

#[test]
fn issue_162_top_level_occurrences_bind_classes_without_treating_variable_names_as_uses() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("app.ts");
    let source = "class Leaf {} const selected = Leaf; use(Missing);";
    std::fs::write(&file, source).unwrap();
    let capture = TypeScriptAuthority::capture(&file, source, &[root.path().to_path_buf()]);
    let reference = source.rfind("Leaf").unwrap();
    assert!(
        capture
            .reference_sites
            .iter()
            .any(|site| site.0 == reference
                && matches!(&site.2, Resolution::Resolved(id) if id.kind == "class_declaration"))
    );
    let declaration = source.find("selected").unwrap();
    assert!(
        !capture
            .reference_sites
            .iter()
            .any(|site| site.0 == declaration)
    );
    let missing = source.find("Missing").unwrap();
    assert!(
        capture
            .reference_sites
            .iter()
            .any(|site| site.0 == missing && matches!(site.2, Resolution::Unresolved(_)))
    );
}

#[test]
fn issue_162_outer_lexical_identity_survives_an_unrelated_loop_shadow() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("app.ts");
    let source = "const item = 1; for (let item = 0; item < 1; item++) {} use(item);";
    std::fs::write(&file, source).unwrap();
    let capture = TypeScriptAuthority::capture(&file, source, &[root.path().to_path_buf()]);
    let reference = source.rfind("item").unwrap();
    let (_, _, Resolution::Resolved(id)) = capture
        .reference_sites
        .iter()
        .find(|site| site.0 == reference)
        .unwrap()
    else {
        panic!("outer lexical declaration required");
    };
    assert_eq!(&source[id.start..id.end], "item = 1");
}

#[test]
fn issue_162_lexical_shadow_sites_keep_distinct_spans_and_retained_owners() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("app.ts");
    let source = "const Selected = 1; use(Selected); { const Selected = 2; use(Selected); }";
    std::fs::write(&file, source).unwrap();
    let capture = TypeScriptAuthority::capture(&file, source, &[root.path().to_path_buf()]);
    let outer = source.find("use(Selected)").unwrap() + 4;
    let inner = source.rfind("use(Selected)").unwrap() + 4;
    let identity = |start| match &capture
        .reference_sites
        .iter()
        .find(|site| site.0 == start)
        .unwrap()
        .2
    {
        Resolution::Resolved(id) => id,
        other => panic!("exact local binding required: {other:?}"),
    };
    let outer_id = identity(outer);
    let inner_id = identity(inner);
    assert_ne!(outer_id.start, inner_id.start);
    assert_eq!(outer_id.source_hash, inner_id.source_hash);
    assert!(capture.declarations.contains(outer_id));
    assert!(capture.declarations.contains(inner_id));
    assert_eq!(&source[inner_id.start..inner_id.end], "Selected = 2");
}

#[test]
fn issue_162_loop_scoped_lexical_candidate_cannot_grant_outside_identity() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("app.ts");
    let source = "for (let Scoped = 0; Scoped < 1; Scoped++) {} use(Scoped);";
    std::fs::write(&file, source).unwrap();
    let capture = TypeScriptAuthority::capture(&file, source, &[root.path().to_path_buf()]);
    let outside = source.rfind("Scoped").unwrap();
    assert!(
        capture
            .reference_sites
            .iter()
            .any(|site| site.0 == outside && matches!(site.2, Resolution::Unresolved(_)))
    );
}
