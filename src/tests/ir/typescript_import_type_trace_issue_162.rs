use super::declared_scopes::NamespaceScope;
use super::{Namespace, Resolution, Resolver};

#[test]
fn issue_162_import_type_trace_qualified_export_boundary() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path();
    std::fs::write(path.join("tsconfig.json"), r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler","moduleDetection":"legacy","noLib":true,"types":[]}}"#).unwrap();
    let leaf = "export namespace Core { export interface Shape {} }";
    std::fs::write(path.join("leaf.ts"), leaf).unwrap();
    let expression = "import('./leaf').Core.Shape";
    let source = format!("type A = {expression};");
    let file = path.join("app.ts");
    std::fs::write(&file, &source).unwrap();
    let unit = super::syntax::Unit::parse(&file, &source).unwrap();
    let tree = unit.tree.as_ref().unwrap();
    let start = source.find(expression).unwrap();
    let node = tree
        .root_node()
        .named_descendant_for_byte_range(start, start + expression.len())
        .unwrap();
    let (call, members) = super::import_types::call_and_members(node, &source).unwrap();
    assert_eq!(members, ["Core", "Shape"], "AST: {}", node.to_sexp());
    assert!(super::import_types::in_type_position(call));
    let mut resolver = Resolver::new(&[path.to_path_buf()]).unwrap();
    let module = resolver.module(&file, "./leaf").unwrap();
    assert_eq!(
        std::fs::canonicalize(&module).unwrap(),
        std::fs::canonicalize(path.join("leaf.ts")).unwrap()
    );
    let local = resolver.namespace_member(&module, "Core", "Shape", Namespace::Type);
    assert!(
        matches!(local, Resolution::Resolved(_)),
        "local scope: {local:?}"
    );
    let exported = resolver.namespace_chain_from(
        &NamespaceScope::module(&module),
        "Core",
        &["Shape"],
        Namespace::Type,
        true,
    );
    assert_eq!(exported, local, "export scope differs from local scope");
    let imported = resolver.import_type_reference(&file, &source, node);
    assert_eq!(
        imported,
        Some(exported),
        "AST: {}; members: {members:?}",
        node.to_sexp()
    );
}
