// Debug test to dump tree-sitter-html AST
#[test]
fn dump_html_ast() {
    let html = r#"<div><span>Hello</span></div>"#;
    let language = tree_sitter_html::language();
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(language).unwrap();
    let tree = parser.parse(html.as_bytes(), None).unwrap();
    let root = tree.root_node();
    fn print_node(node: tree_sitter::Node, source: &str, indent: usize) {
        let text = node
            .utf8_text(source.as_bytes())
            .unwrap_or("<error>");
        let truncated = if text.len() > 40 {
            &text[..40]
        } else {
            text
        };
        eprintln!(
            "{}{:?} [{}] \"{}\"",
            " ".repeat(indent),
            node.kind(),
            node.is_named(),
            truncated
        );
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            print_node(child, source, indent + 2);
        }
    }
    print_node(root, html, 0);
}