// src/tests/angular_meta/template.rs
//
// Tests for the Angular-syntax template extractor.
// Includes both legacy (Tier 2) and modern (Phase 2.5) syntax.

use crate::angular_meta::template::{
    TemplateShape, extract_template_shape, extract_template_shape_with_depth,
};

// NOISE REDUCTION (2026-08-25): AST-dump debug tests commented out — their
// sole purpose was printing tree-sitter parse trees on every run. Uncomment
// locally when debugging template parsing.
//
/// Debug: dump tree-sitter-html AST to understand node types.
// #[test]
// fn dump_html_ast() {
//     let html = r#"<div><span>Hello</span></div>"#;
//     let language = tree_sitter_html::LANGUAGE.into();
//     let mut parser = tree_sitter::Parser::new();
//     parser.set_language(&language).unwrap();
//     let tree = parser.parse(html.as_bytes(), None).unwrap();
//     let root = tree.root_node();
//     fn print_node(node: tree_sitter::Node, source: &str, indent: usize) {
//         let text = node.utf8_text(source.as_bytes()).unwrap_or("<error>");
//         let truncated = if text.len() > 40 { &text[..40] } else { text };
//         eprintln!(
//             "{}{:?} [named={}] \"{}\"",
//             " ".repeat(indent),
//             node.kind(),
//             node.is_named(),
//             truncated
//         );
//         let mut cursor = node.walk();
//         for child in node.children(&mut cursor) {
//             print_node(child, source, indent + 2);
//         }
//     }
//     print_node(root, html, 0);
// }
//
/// Debug: dump AST for Angular template with bindings.
// #[test]
// fn dump_angular_template_ast() {
//     let html =
//         r#"<div *ngIf="show"><app-card [title]="name" (click)="handler()"></app-card></div>"#;
//     let language = tree_sitter_html::LANGUAGE.into();
//     let mut parser = tree_sitter::Parser::new();
//     parser.set_language(&language).unwrap();
//     let tree = parser.parse(html.as_bytes(), None).unwrap();
//     let root = tree.root_node();
//     fn print_node(node: tree_sitter::Node, source: &str, indent: usize) {
//         let text = node.utf8_text(source.as_bytes()).unwrap_or("<error>");
//         let truncated = if text.len() > 50 { &text[..50] } else { text };
//         eprintln!(
//             "{}{:?} [named={}] \"{}\"",
//             " ".repeat(indent),
//             node.kind(),
//             node.is_named(),
//             truncated
//         );
//         let mut cursor = node.walk();
//         for child in node.children(&mut cursor) {
//             print_node(child, source, indent + 2);
//         }
//     }
//     print_node(root, html, 0);
// }

// ===== Legacy syntax tests (Tier 2) =====

#[test]
fn extracts_basic_tags() {
    let html = r#"<div><span>Hello</span></div>"#;
    let shape = extract_template_shape(html);
    assert!(shape.tags.contains(&"div".to_string()));
    assert!(shape.tags.contains(&"span".to_string()));
}

#[test]
fn extracts_structural_directives() {
    let html = r#"<div *ngIf="show"><li *ngFor="let item of items"></li></div>"#;
    let shape = extract_template_shape(html);
    assert!(shape.structural_directives.contains(&"ngIf".to_string()));
    assert!(shape.structural_directives.contains(&"ngFor".to_string()));
}

#[test]
fn extracts_property_bindings() {
    let html = r#"<app-card [title]="name" [hidden]="false"></app-card>"#;
    let shape = extract_template_shape(html);
    assert!(shape.prop_bindings.contains(&"title".to_string()));
    assert!(shape.prop_bindings.contains(&"hidden".to_string()));
}

#[test]
fn extracts_event_bindings() {
    let html = r#"<button (click)="handleClick()" (mousedown)="onDown()"></button>"#;
    let shape = extract_template_shape(html);
    assert!(shape.event_bindings.contains(&"click".to_string()));
    assert!(shape.event_bindings.contains(&"mousedown".to_string()));
}

#[test]
fn extracts_two_way_bindings() {
    let html = r#"<input [(ngModel)]="value">"#;
    let shape = extract_template_shape(html);
    assert!(shape.two_way_bindings.contains(&"ngModel".to_string()));
}

#[test]
fn counts_interpolations() {
    let html = r#"<div>{{ name }} has {{ count }} items</div>"#;
    let shape = extract_template_shape(html);
    assert_eq!(shape.interpolation_count, 2);
}

#[test]
fn detects_custom_elements() {
    let html = r#"<app-user-card></app-user-card><app-avatar></app-avatar>"#;
    let shape = extract_template_shape(html);
    assert!(shape.custom_elements.contains(&"app-user-card".to_string()));
    assert!(shape.custom_elements.contains(&"app-avatar".to_string()));
}

#[test]
fn empty_html_returns_empty_shape() {
    let shape = extract_template_shape("");
    assert!(shape.tags.is_empty());
    assert_eq!(shape.interpolation_count, 0);
}

#[test]
fn whitespace_only_returns_empty_shape() {
    let shape = extract_template_shape("   \n  \t  ");
    assert!(shape.tags.is_empty());
}

#[test]
fn to_marker_line_contains_tags() {
    let html = r#"<div><span>{{ count }}</span></div>"#;
    let shape = extract_template_shape(html);
    let line = shape.to_marker_line();
    assert!(line.starts_with("Φtpl:"));
    assert!(line.contains("div"));
    assert!(line.contains("span"));
}

#[test]
fn to_marker_line_contains_directives() {
    let html = r#"<div *ngIf="show"></div>"#;
    let shape = extract_template_shape(html);
    let line = shape.to_marker_line();
    assert!(line.contains("[ngIf]"));
}

#[test]
fn to_marker_line_contains_two_way_bindings() {
    let html = r#"<input [(ngModel)]="val">"#;
    let shape = extract_template_shape(html);
    let line = shape.to_marker_line();
    assert!(line.contains("[(ngModel)]"));
}

#[test]
fn to_marker_line_empty_when_no_content() {
    let shape = TemplateShape::default();
    assert_eq!(shape.to_marker_line(), "Φtpl:empty");
}

#[test]
fn depth_zero_extracts_only_root_element() {
    let html = r#"<div><span>Nested</span></div>"#;
    let shape = extract_template_shape_with_depth(html, 0);
    assert!(shape.tags.is_empty());
}

#[test]
fn depth_one_extracts_root_element_only() {
    let html = r#"<div><span>Text</span></div>"#;
    let shape = extract_template_shape_with_depth(html, 1);
    assert!(shape.tags.contains(&"div".to_string()));
    assert!(!shape.tags.contains(&"span".to_string()));
}

#[test]
fn depth_two_extracts_two_levels() {
    let html = r#"<div><span>Text</span></div>"#;
    let shape = extract_template_shape_with_depth(html, 2);
    assert!(shape.tags.contains(&"div".to_string()));
    assert!(shape.tags.contains(&"span".to_string()));
}

#[test]
fn complex_template_all_features() {
    let html = r#"
<div class="container">
  <h1>{{ title }}</h1>
  <app-card *ngIf="showCard"
    [data]="cardData"
    (select)="onSelect($event)"
    [(ngModel)]="selected">
  </app-card>
  <div *ngFor="let item of items">
    {{ item.name }}
  </div>
</div>"#;
    let shape = extract_template_shape(html);
    assert!(shape.tags.contains(&"div".to_string()));
    assert!(shape.tags.contains(&"h1".to_string()));
    assert!(shape.custom_elements.contains(&"app-card".to_string()));
    assert!(shape.structural_directives.contains(&"ngIf".to_string()));
    assert!(shape.structural_directives.contains(&"ngFor".to_string()));
    assert!(shape.prop_bindings.contains(&"data".to_string()));
    assert!(shape.event_bindings.contains(&"select".to_string()));
    assert!(shape.two_way_bindings.contains(&"ngModel".to_string()));
    assert!(shape.interpolation_count >= 2);
}

// ===== Phase 2.5: Modern syntax tests =====

#[test]
fn detects_at_if_control_flow() {
    let html = r#"@if (isLoggedIn) { <span>Hello</span> }"#;
    let shape = extract_template_shape(html);
    assert!(shape.control_flow_blocks.contains(&"if".to_string()));
}

#[test]
fn detects_at_else_control_flow() {
    let html = r#"@if (cond) { <span>A</span> } @else { <span>B</span> }"#;
    let shape = extract_template_shape(html);
    assert!(shape.control_flow_blocks.contains(&"if".to_string()));
    assert!(shape.control_flow_blocks.contains(&"else".to_string()));
}

#[test]
fn detects_at_for_control_flow() {
    let html = r#"@for (item of items; track item.id) { <li>{{ item.name }}</li> }"#;
    let shape = extract_template_shape(html);
    assert!(shape.control_flow_blocks.contains(&"for".to_string()));
    assert!(shape.tags.contains(&"li".to_string()));
}

#[test]
fn detects_at_empty_control_flow() {
    let html = r#"@for (item of items; track item.id) { <li>{{ item.name }}</li> } @empty { <p>None</p> }"#;
    let shape = extract_template_shape(html);
    assert!(shape.control_flow_blocks.contains(&"for".to_string()));
    assert!(shape.control_flow_blocks.contains(&"empty".to_string()));
}

#[test]
fn detects_at_switch_and_at_case() {
    let html = r#"@switch (mode) { @case ('a') { <app-a /> } @case ('b') { <app-b /> } }"#;
    let shape = extract_template_shape(html);
    assert!(shape.control_flow_blocks.contains(&"switch".to_string()));
    assert!(shape.control_flow_blocks.contains(&"case".to_string()));
}

#[test]
fn detects_at_default_in_switch() {
    let html = r#"@switch (mode) { @default { <p>fallback</p> } }"#;
    let shape = extract_template_shape(html);
    assert!(shape.control_flow_blocks.contains(&"switch".to_string()));
    assert!(shape.control_flow_blocks.contains(&"default".to_string()));
}

#[test]
fn detects_at_defer_with_trigger() {
    let html = r#"@defer (on viewport) { <app-heavy /> }"#;
    let shape = extract_template_shape(html);
    assert!(shape.defer_blocks.contains(&"viewport".to_string()));
    assert!(!shape.defer_blocks.contains(&"default".to_string()));
}

#[test]
fn detects_at_defer_default() {
    let html = r#"@defer { <app-heavy /> }"#;
    let shape = extract_template_shape(html);
    assert!(shape.defer_blocks.contains(&"default".to_string()));
}

#[test]
fn detects_defer_sub_blocks() {
    let html = r#"@defer (on viewport) { <app-heavy /> } @placeholder { <div>Loading</div> } @loading { <app-spinner /> } @error { <p>Error</p> }"#;
    let shape = extract_template_shape(html);
    assert!(shape.defer_blocks.contains(&"viewport".to_string()));
    assert!(shape.defer_blocks.contains(&"placeholder".to_string()));
    assert!(shape.defer_blocks.contains(&"loading".to_string()));
    assert!(shape.defer_blocks.contains(&"error".to_string()));
}

#[test]
fn detects_at_let_declarations() {
    let html = r#"@let greeting = 'Hello'; @let user = user$ | async;"#;
    let shape = extract_template_shape(html);
    // Both @let declarations are in the same text node; after dedup, only 1 entry.
    assert_eq!(shape.let_declarations.len(), 1);
    assert!(shape.let_declarations.contains(&"let".to_string()));
}

#[test]
fn modern_template_marker_line_includes_at_tokens() {
    let html = r#"@if (isLoggedIn) { <span>Hello</span> } @else { <span>Bye</span> }"#;
    let shape = extract_template_shape(html);
    let line = shape.to_marker_line();
    assert!(line.starts_with("Φtpl:"));
    assert!(line.contains("@if"));
    assert!(line.contains("@else"));
    assert!(line.contains("span"));
}

#[test]
fn extracts_self_closing_xhtml_component() {
    let html = r#"<app-avatar [user]="user" />"#;
    let shape = extract_template_shape(html);
    assert!(shape.custom_elements.contains(&"app-avatar".to_string()));
    assert!(shape.prop_bindings.contains(&"user".to_string()));
}

#[test]
fn extracts_self_closing_xhtml_in_container() {
    let html = r#"<div><app-avatar [user]="user" (click)="onClick()" /></div>"#;
    let shape = extract_template_shape(html);
    assert!(shape.tags.contains(&"div".to_string()));
    assert!(shape.custom_elements.contains(&"app-avatar".to_string()));
    assert!(shape.prop_bindings.contains(&"user".to_string()));
    assert!(shape.event_bindings.contains(&"click".to_string()));
}

#[test]
fn extracts_self_closing_in_control_flow() {
    let html = r#"@if (cond) { <app-heavy [config]="cfg" /> }"#;
    let shape = extract_template_shape(html);
    assert!(shape.custom_elements.contains(&"app-heavy".to_string()));
    assert!(shape.prop_bindings.contains(&"config".to_string()));
    assert!(shape.control_flow_blocks.contains(&"if".to_string()));
}

#[test]
fn extracts_void_element_with_bindings() {
    let html = r#"<input [(ngModel)]="value" />"#;
    let shape = extract_template_shape(html);
    assert!(shape.tags.contains(&"input".to_string()));
    assert!(shape.two_way_bindings.contains(&"ngModel".to_string()));
}

#[test]
fn extracts_multiple_self_closing_at_root() {
    let html = r#"<app-a /><app-b />"#;
    let shape = extract_template_shape(html);
    assert!(shape.custom_elements.contains(&"app-a".to_string()));
    assert!(shape.custom_elements.contains(&"app-b".to_string()));
}

#[test]
fn marker_line_includes_self_closing_components() {
    let html = r#"<app-avatar [user]="user" (click)="onClick()" />"#;
    let shape = extract_template_shape(html);
    let line = shape.to_marker_line();
    assert!(line.contains("app-avatar"));
    assert!(line.contains("[user]"));
    assert!(line.contains("(click)"));
}

#[test]
fn mixed_legacy_and_modern() {
    let html = r#"<div *ngIf="legacyFlag">
  <span>{{ name }}</span>
</div>
@if (modernFlag) {
  <app-modern [data]="name" />
}"#;
    let shape = extract_template_shape(html);
    // Legacy
    assert!(shape.structural_directives.contains(&"ngIf".to_string()));
    // Modern
    assert!(shape.control_flow_blocks.contains(&"if".to_string()));
    // Both tags
    assert!(shape.tags.contains(&"div".to_string()));
    assert!(shape.custom_elements.contains(&"app-modern".to_string()));
    // Bindings
    assert!(shape.prop_bindings.contains(&"data".to_string()));
}

#[test]
fn marker_line_shows_both_legacy_and_modern() {
    let html = r#"<div *ngIf="show">
  <span>{{ name }}</span>
</div>
@if (modern) {
  <span>Modern</span>
}"#;
    let shape = extract_template_shape(html);
    let line = shape.to_marker_line();
    // Both legacy and modern should appear
    assert!(line.contains("[ngIf]"));
    assert!(line.contains("@if"));
}

#[test]
fn at_if_does_not_false_positive_on_at_symbol_in_text() {
    // Ensure @ in email addresses doesn't trigger @if detection
    let html = r#"<span>{{ email }}</span>"#;
    let shape = extract_template_shape(html);
    assert!(shape.control_flow_blocks.is_empty());
}

#[test]
fn complex_modern_template_all_features() {
    let html = r#"
@if (isLoggedIn) {
  <app-avatar [user]="user" (avatarClick)="onClick()" />
  <div class="info">
    <p>{{ user.name }} - {{ user.email }}</p>
  </div>
} @else {
  <button (click)="login()">Log in</button>
}

@for (item of items; track item.id) {
  <span>{{ item.label }}</span>
} @empty {
  <p>No items.</p>
}

@switch (view) {
  @case ('grid') { <app-grid [data]="items" /> }
  @default { <p>Select mode</p> }
}

@defer (on viewport) {
  <app-heavy [config]="cfg" />
} @placeholder {
  <div>Loading...</div>
}

@let greeting = 'Hello';
"#;
    let shape = extract_template_shape(html);

    // Control flow
    assert!(shape.control_flow_blocks.contains(&"if".to_string()));
    assert!(shape.control_flow_blocks.contains(&"else".to_string()));
    assert!(shape.control_flow_blocks.contains(&"for".to_string()));
    assert!(shape.control_flow_blocks.contains(&"empty".to_string()));
    assert!(shape.control_flow_blocks.contains(&"switch".to_string()));
    assert!(shape.control_flow_blocks.contains(&"case".to_string()));
    assert!(shape.control_flow_blocks.contains(&"default".to_string()));

    // Defer
    assert!(shape.defer_blocks.contains(&"viewport".to_string()));
    assert!(shape.defer_blocks.contains(&"placeholder".to_string()));

    // Let declarations
    assert!(shape.let_declarations.contains(&"let".to_string()));

    // Tags
    assert!(shape.custom_elements.contains(&"app-avatar".to_string()));
    assert!(shape.custom_elements.contains(&"app-grid".to_string()));
    assert!(shape.custom_elements.contains(&"app-heavy".to_string()));
    assert!(shape.tags.contains(&"div".to_string()));
    assert!(shape.tags.contains(&"button".to_string()));

    // Bindings
    assert!(shape.prop_bindings.contains(&"user".to_string()));
    assert!(shape.prop_bindings.contains(&"data".to_string()));
    assert!(shape.prop_bindings.contains(&"config".to_string()));
    assert!(shape.event_bindings.contains(&"avatarClick".to_string()));
    assert!(shape.event_bindings.contains(&"click".to_string()));

    // Interpolations
    assert!(shape.interpolation_count >= 3);

    // Marker line
    let line = shape.to_marker_line();
    assert!(line.starts_with("Φtpl:"));
    assert!(line.contains("@if"));
    assert!(line.contains("@for"));
    assert!(line.contains("@switch"));
    assert!(line.contains("@else"));
    assert!(line.contains("@defer(viewport)"));
    assert!(line.contains("@defer(placeholder)"));
    assert!(line.contains("@let"));
}

/// FAANG AUDIT regression: `@if`/`@for` inside string literals or
/// identifiers must NOT be captured into `if_conditions`/`for_loops`.
///
/// Previously `extract_at_if_condition` / `extract_at_for_loop` used a
/// bare `text.find("@if")` / `text.find("@for")` which matched `@if`/`@for`
/// inside string literals (e.g. `{{ "@if (x)" }}`) or identifiers like
/// `@formatter`, producing false control-flow markers even though
/// `control_flow_blocks` (via `contains_at_keyword`) correctly rejected them.
#[test]
fn at_if_for_inside_string_not_captured() {
    // `@if`/`@for` appear only inside a string literal and an identifier.
    let html = r#"<div>{{ "@if (x)" }} {{ "@for (y of z)" }} @formatter</div>"#;
    let shape = extract_template_shape(html);

    // The word-boundary check must reject these — no control-flow blocks.
    assert!(
        shape.control_flow_blocks.is_empty(),
        "string-literal @if/@for must not be detected as control flow: {:?}",
        shape.control_flow_blocks
    );
    assert!(
        shape.if_conditions.is_empty(),
        "string-literal @if must not be captured as a condition: {:?}",
        shape.if_conditions
    );
    assert!(
        shape.for_loops.is_empty(),
        "string-literal @for must not be captured as a loop: {:?}",
        shape.for_loops
    );
}

/// FAANG AUDIT regression: real `@if`/`@for` blocks ARE captured into
/// `if_conditions`/`for_loops` (the positive case for the word-boundary fix).
#[test]
fn at_if_for_blocks_captured() {
    let html = r#"@if (isLoading) { <div>Loading</div> } @for (item of items; track item.id) { <p>{{ item.name }}</p> }"#;
    let shape = extract_template_shape(html);

    assert!(
        shape.if_conditions.contains(&"isLoading".to_string()),
        "real @if condition should be captured: {:?}",
        shape.if_conditions
    );
    assert!(
        shape
            .for_loops
            .contains(&("item".to_string(), "items".to_string())),
        "real @for loop should be captured: {:?}",
        shape.for_loops
    );
}
