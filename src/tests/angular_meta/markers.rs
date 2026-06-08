// src/tests/angular_meta/markers.rs
//
// Tests for the `Φ` marker construction and expansion.

use crate::angular_meta::markers::{
    build_component_line, build_directive_line, build_injects_line, build_input_line,
    build_module_line, build_output_line, build_pipe_line, build_service_line, expand_phi,
    expand_phi_in_line, ComponentFields,
};

#[test]
fn component_line_emits_selector_and_template() {
    let fields = ComponentFields {
        selector: Some("app-foo".to_string()),
        template_url: Some("./foo.html".to_string()),
        ..Default::default()
    };
    let line = build_component_line("Foo", &fields);
    assert_eq!(line, "Φcmp:Foo sel=app-foo tpl=./foo.html");
}

#[test]
fn component_line_uses_inline_template_when_no_url() {
    let fields = ComponentFields {
        template: Some("<h1>Hi</h1>".to_string()),
        ..Default::default()
    };
    let line = build_component_line("Foo", &fields);
    assert!(line.contains("tpl=<h1>Hi</h1>"));
    assert!(!line.contains("tplUrl"));
}

#[test]
fn component_line_joins_multiple_style_urls() {
    let fields = ComponentFields {
        style_urls: Some(vec!["./a.css".to_string(), "./b.css".to_string()]),
        ..Default::default()
    };
    let line = build_component_line("Foo", &fields);
    assert!(line.contains("sty=[./a.css,./b.css]"));
}

#[test]
fn component_line_emits_class_name_only_when_no_fields() {
    let fields = ComponentFields::default();
    let line = build_component_line("Foo", &fields);
    assert_eq!(line, "Φcmp:Foo");
}

#[test]
fn service_line_emits_provided_in_scope() {
    let line = build_service_line("UserService", Some("root"));
    assert_eq!(line, "Φsvc:UserService scope=root");
}

#[test]
fn service_line_emits_class_only_when_no_provided_in() {
    let line = build_service_line("Foo", None);
    assert_eq!(line, "Φsvc:Foo");
}

#[test]
fn module_line_emits_all_three_lists() {
    let decl = vec!["FooCmp".to_string()];
    let imp = vec!["CommonModule".to_string()];
    let exp = vec!["FooCmp".to_string()];
    let line = build_module_line("AppModule", &decl, &imp, &exp);
    assert!(line.contains("decl=[FooCmp]"));
    assert!(line.contains("imp=[CommonModule]"));
    assert!(line.contains("exp=[FooCmp]"));
}

#[test]
fn directive_line_emits_selector() {
    let line = build_directive_line("Highlight", Some("[appH]"));
    assert_eq!(line, "Φdir:Highlight sel=[appH]");
}

#[test]
fn pipe_line_emits_name() {
    let line = build_pipe_line("UpperPipe", Some("upper"));
    assert_eq!(line, "Φpipe:UpperPipe name=upper");
}

#[test]
fn input_line_emits_alias() {
    let line = build_input_line("userId", Some("id"));
    assert_eq!(line, "Φin:userId alias=id");
}

#[test]
fn output_line_emits_alias() {
    let line = build_output_line("userDeleted", Some("deleted"));
    assert_eq!(line, "Φout:userDeleted alias=deleted");
}

#[test]
fn injects_line_joins_types() {
    let line = build_injects_line(&["UserService".to_string(), "HttpClient".to_string()]);
    assert_eq!(line, "Φinjects:[UserService,HttpClient]");
}

#[test]
fn expand_phi_in_line_rewrites_component_marker() {
    let line = "Φcmp:UserCard sel=app-user-card tpl=./user-card.component.html";
    let expanded = expand_phi_in_line(line);
    assert_eq!(expanded, "@Component UserCard sel=app-user-card tpl=./user-card.component.html");
}

#[test]
fn expand_phi_in_line_rewrites_service_marker() {
    let line = "Φsvc:UserService scope=root";
    let expanded = expand_phi_in_line(line);
    assert_eq!(expanded, "@Injectable UserService scope=root");
}

#[test]
fn expand_phi_in_line_rewrites_input_marker() {
    let line = "Φin:userId";
    let expanded = expand_phi_in_line(line);
    assert_eq!(expanded, "@Input userId");
}

#[test]
fn expand_phi_in_line_preserves_unknown_phi_tokens() {
    let line = "Φunknown:foo bar=baz";
    let expanded = expand_phi_in_line(line);
    // Unknown tokens pass through.
    assert_eq!(expanded, "Φunknown:foo bar=baz");
}

#[test]
fn expand_phi_single_token() {
    assert_eq!(expand_phi("Φcmp"), Some("@Component"));
    assert_eq!(expand_phi("Φsvc"), Some("@Injectable"));
    assert_eq!(expand_phi("Φin"), Some("@Input"));
    assert_eq!(expand_phi("Φout"), Some("@Output"));
    assert_eq!(expand_phi("Φunknown"), None);
}

#[test]
fn phi_in_line_rewrite_is_idempotent_only_known_tokens() {
    // Multiple known tokens in one line.
    let line = "Φcmp:Foo sel=a;Φin:bar;Φout:baz";
    let expanded = expand_phi_in_line(line);
    assert_eq!(
        expanded,
        "@Component Foo sel=a;@Input bar;@Output baz"
    );
}
