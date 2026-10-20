use crate::ir::typescript_object_members::project_exported_object_members;

#[test]
fn object_member_identity_is_declaration_scoped() {
    let edges = project_exported_object_members(
        "export const A = { X: '[F] Same' }; export const B = { X: '[F] Same' };",
        "actions.ts",
    );
    let names: Vec<_> = edges
        .iter()
        .map(|edge| edge.subject.name.as_str())
        .collect();
    assert_eq!(names, ["A.X", "B.X"]);
    assert!(
        edges
            .iter()
            .all(|edge| edge.subject.file.as_deref() == Some("actions.ts"))
    );
}

#[test]
fn non_exported_objects_and_non_identifier_members_do_not_publish_identity() {
    let edges = project_exported_object_members(
        "const A = { X: 'x' }; export const B = { ['X']: 'x', ok: 'y' };",
        "actions.ts",
    );
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].subject.name, "B.ok");
}
