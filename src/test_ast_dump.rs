// Quick diagnostic to dump tree-sitter-html AST for modern Angular syntax
fn main() {
    let templates = vec![
        ("basic_html", r#"<div><span>Hello</span></div>"#),
        ("legacy_angular", r#"<div *ngIf="show"><app-card [title]="name"></app-card></div>"#),
        ("modern_at_if", r#"@if (isLoggedIn) { <span>Hello</span> } @else { <span>Bye</span> }"#),
        ("modern_at_for", r#"@for (item of items; track item.id) { <li>{{ item.name }}</li> }"#),
        ("modern_at_defer", r#"@defer (on viewport) { <app-heavy /> } @placeholder { <div>Loading</div> }"#),
        ("modern_at_let", r#"@let greeting = 'Hello';"#),
        ("modern_switch", r#"@switch (mode) { @case ('a') { <app-a /> } }"#),
        ("mixed", r#"@if (cond) { <div><span>{{ name }}</span></div> }"#),
    ];

    let language = tree_sitter_html::language();
    for (name, html) in &templates {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(language).unwrap();
        eprintln!("\n==================== {} ====================", name);
        eprintln!("Input: {}", html);
        let tree = parser.parse(html.as_bytes(), None);
        if let Some(t) = tree {
            let root = t.root_node();
            print_node(root, html, 0);
            eprintln!("Root has_error: {}", root.has_error());
        } else {
            eprintln!("PARSER RETURNED NONE - COMPLETE FAILURE");
        }
    }
}

fn print_node(node: tree_sitter::Node, source: &str, indent: usize) {
    let text = node.utf8_text(source.as_bytes()).unwrap_or("<error>");
    let truncated = if text.len() > 60 { &text[..60] } else { text };
    let error_marker = if node.is_error() || node.has_error() { " *** ERROR ***" } else { "" };
    eprintln!(
        "{}{:?} [named={}, children={}] \"{}\"{}",
        " ".repeat(indent),
        node.kind(),
        node.is_named(),
        node.child_count(),
        truncated.replace('\n', "\\n"),
        error_marker
    );
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        print_node(child, source, indent + 2);
    }
}