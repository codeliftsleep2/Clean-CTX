// src/tests/angular_meta/markers.rs
//
// Tests for the `Φ` marker construction and expansion.

use crate::angular_meta::markers::{
    build_component_line, build_directive_line, build_injects_line, build_input_line,
    build_model_line, build_module_line, build_output_line, build_pipe_line, build_service_line,
    expand_phi, expand_phi_in_line, ComponentFields, InjectsLine, InputLine, ModelLine, ModuleLine,
    OutputLine, PhiLine, PhiLineKind, PipeLine, ServiceLine,
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
fn model_line_emits_field_name() {
    let line = build_model_line("checked", None);
    assert_eq!(line, "Φmodel:checked");
}

#[test]
fn model_line_emits_alias() {
    let line = build_model_line("checked", Some("isChecked"));
    assert_eq!(line, "Φmodel:checked alias=isChecked");
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
    assert_eq!(expand_phi("Φmodel"), Some("@Model"));
    assert_eq!(expand_phi("Φunknown"), None);
}

#[test]
fn phi_in_line_rewrite_is_idempotent_only_known_tokens() {
    // Multiple known tokens in one line.
    let line = "Φcmp:Foo sel=a;Φin:bar;Φout:baz;Φmodel:checked";
    let expanded = expand_phi_in_line(line);
    assert_eq!(
        expanded,
        "@Component Foo sel=a;@Input bar;@Output baz;@Model checked"
    );
}

// ---------------------------------------------------------------------------
// Track C — PhiLineKind / PhiLine structural tests (F-ANG-06)
// ---------------------------------------------------------------------------

/// Every `PhiLineKind` variant must have a unique `marker_prefix` and
/// `expansion`. This catches copy-paste errors where two variants map
/// to the same token or expansion string.
#[test]
fn phi_line_kind_uniqueness() {
    let all_kinds = PhiLineKind::all_in_expand_order();
    // ANGULAR_HTML_COMPRESSION_PLAN: added TemplateBinding, TemplateDirective,
    // TemplateComponent — 14 original + 3 new = 17.
    assert_eq!(all_kinds.len(), 17, "expected 17 PhiLineKind variants");

    // Check that marker_prefix is unique across all variants.
    let mut prefixes = std::collections::HashSet::new();
    for &kind in all_kinds {
        let prefix = kind.marker_prefix();
        assert!(
            prefixes.insert(prefix),
            "duplicate marker_prefix {:?} for kind {:?}",
            prefix,
            kind
        );
    }

    // Check that expansion is unique across all variants.
    let mut expansions = std::collections::HashSet::new();
    for &kind in all_kinds {
        let expansion = kind.expansion();
        assert!(
            expansions.insert(expansion),
            "duplicate expansion {:?} for kind {:?}",
            expansion,
            kind
        );
    }
}

/// The expansion table is a bijection: every marker prefix maps to
/// exactly one expansion, and every expansion maps back to exactly one
/// prefix. This is the same invariant that the old hand-maintained
/// `expand_phi` match table enforced.
#[test]
fn phi_vocab_is_bijective() {
    // Forward: each kind → unique expansion.
    let mut exp_to_kind = std::collections::HashMap::new();
    for &kind in PhiLineKind::all_in_expand_order() {
        let exp = kind.expansion();
        if let Some(&prev) = exp_to_kind.get(exp) {
            panic!(
                "expansion {:?} maps to both {:?} and {:?}",
                exp, prev, kind
            );
        }
        exp_to_kind.insert(exp, kind);
    }

    // Reverse: each expansion → unique kind.
    let mut prefix_to_kind = std::collections::HashMap::new();
    for &kind in PhiLineKind::all_in_expand_order() {
        let prefix = kind.marker_prefix();
        if let Some(&prev) = prefix_to_kind.get(prefix) {
            panic!(
                "prefix {:?} maps to both {:?} and {:?}",
                prefix, prev, kind
            );
        }
        prefix_to_kind.insert(prefix, kind);
    }
}

/// For each builder-backed `PhiLine` impl, verify that the rendered
/// output starts with the correct `Φ…:` prefix and that
/// `expand_phi_in_line` reverses the prefix to the expected expansion.
/// This is the round-trip invariant: `render()` → `expand_phi_in_line`.
#[test]
fn phi_line_round_trip() {
    // --- Component (with fields) ---
    let fields = ComponentFields {
        selector: Some("app-test".to_string()),
        ..Default::default()
    };
    let line = crate::angular_meta::markers::ComponentLine {
        class_name: "MyCmp",
        fields: &fields,
    };
    assert_eq!(line.kind(), PhiLineKind::Component);
    let rendered = line.render();
    assert!(rendered.starts_with("Φcmp:"));
    let expanded = expand_phi_in_line(&rendered);
    assert!(expanded.starts_with("@Component MyCmp"));

    // --- Component (no fields) ---
    let fields = ComponentFields::default();
    let line = crate::angular_meta::markers::ComponentLine {
        class_name: "Bare",
        fields: &fields,
    };
    let rendered = line.render();
    assert_eq!(rendered, "Φcmp:Bare");
    let expanded = expand_phi_in_line(&rendered);
    assert!(expanded.starts_with("@Component Bare"));

    // --- Service ---
    let line = ServiceLine {
        class_name: "Svc",
        provided_in: Some("root"),
    };
    assert_eq!(line.kind(), PhiLineKind::Service);
    let rendered = line.render();
    assert!(rendered.starts_with("Φsvc:"));
    let expanded = expand_phi_in_line(&rendered);
    assert!(expanded.starts_with("@Injectable Svc"));

    // --- Module ---
    let decl = vec!["CmpA".to_string()];
    let line = ModuleLine {
        class_name: "Mod",
        decl: &decl,
        imp: &[],
        exp: &[],
    };
    assert_eq!(line.kind(), PhiLineKind::Module);
    let rendered = line.render();
    assert!(rendered.starts_with("Φmod:"));
    let expanded = expand_phi_in_line(&rendered);
    assert!(expanded.starts_with("@NgModule Mod"));

    // --- Directive ---
    let line = crate::angular_meta::markers::DirectiveLine {
        class_name: "Dir",
        selector: Some("[appDir]"),
    };
    let rendered = line.render();
    assert!(rendered.starts_with("Φdir:"));
    let expanded = expand_phi_in_line(&rendered);
    assert!(expanded.starts_with("@Directive Dir"));

    // --- Pipe ---
    let line = PipeLine {
        class_name: "Pip",
        name: Some("myPipe"),
    };
    assert_eq!(line.kind(), PhiLineKind::Pipe);
    let rendered = line.render();
    assert!(rendered.starts_with("Φpipe:"));
    let expanded = expand_phi_in_line(&rendered);
    assert!(expanded.starts_with("@Pipe Pip"));

    // --- Input ---
    let line = InputLine {
        field_name: "value",
        alias: Some("val"),
    };
    assert_eq!(line.kind(), PhiLineKind::Input);
    let rendered = line.render();
    assert!(rendered.starts_with("Φin:"));
    let expanded = expand_phi_in_line(&rendered);
    assert!(expanded.starts_with("@Input value"));

    // --- Output ---
    let line = OutputLine {
        field_name: "changed",
        alias: None,
    };
    assert_eq!(line.kind(), PhiLineKind::Output);
    let rendered = line.render();
    assert!(rendered.starts_with("Φout:"));
    let expanded = expand_phi_in_line(&rendered);
    assert!(expanded.starts_with("@Output changed"));

    // --- Model ---
    let line = ModelLine {
        field_name: "checked",
        alias: Some("isOn"),
    };
    assert_eq!(line.kind(), PhiLineKind::Model);
    let rendered = line.render();
    assert!(rendered.starts_with("Φmodel:"));
    let expanded = expand_phi_in_line(&rendered);
    assert!(expanded.starts_with("@Model checked"));

    // --- Injects ---
    let types = vec!["HttpClient".to_string(), "Router".to_string()];
    let line = InjectsLine { types: &types };
    assert_eq!(line.kind(), PhiLineKind::Injects);
    let rendered = line.render();
    assert!(rendered.starts_with("Φinjects:"));
    let expanded = expand_phi_in_line(&rendered);
    assert!(expanded.starts_with("@Inject [HttpClient,Router]"));
}
