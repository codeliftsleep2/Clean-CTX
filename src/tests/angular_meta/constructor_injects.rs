// src/tests/angular_meta/constructor_injects.rs
//
// RED-DI regressions: Angular constructor injection is modifier-independent.
//
// Semantic invariant under test:
//
//   within a class already recognized as an Angular DI class, a *typed*
//   constructor parameter is a constructor injection — whether or not it also
//   declares a TypeScript parameter property.
//
// `private` / `public` / `protected` / `readonly` control TypeScript property
// generation; they do not decide whether Angular injects the parameter.
//
// These tests exercise the extraction source of the `injects` list that feeds
// `class_to_semantic_edges`. Edge-level and workspace-level equivalents live
// in `src/tests/angular_meta/constructor_di_edges.rs`.

use crate::angular_meta::constructor_injects::extract_constructor_injects;
use crate::angular_meta::decorators::{extract_decorators, extract_graph_entries};
use crate::compression::Fidelity;

/// Extract the injected types, treating "no injects" as an empty list.
fn injects(raw: &str) -> Vec<String> {
    extract_constructor_injects(raw).unwrap_or_default()
}

/// The `injects` slot of `extract_graph_entries` — the exact input consumed
/// by `class_to_semantic_edges` for `SemanticRelation::Injects`.
fn graph_injects(raw: &str) -> Vec<String> {
    extract_graph_entries(raw)
        .map(|(_, _, _, injects, _)| injects)
        .unwrap_or_default()
}

// ── RED-DI1 — bare typed constructor parameter ────────────────────────

#[test]
fn red_di1_bare_typed_constructor_parameter() {
    let raw = r#"
        @Injectable()
        export class Consumer {
            constructor(fooService: FooService) {}
        }
    "#;
    assert_eq!(injects(raw), vec!["FooService".to_string()]);
    assert_eq!(
        graph_injects(raw),
        vec!["FooService".to_string()],
        "the graph-entry injects list (semantic-edge input) must carry the bare parameter"
    );
}

#[test]
fn red_di1_bare_parameter_with_body_assignment_is_recognized() {
    // The live symptom shape: the parameter is stored manually instead of
    // through parameter-property shorthand.
    let raw = r#"
        @Injectable()
        export class Consumer {
            fooService: FooService;
            constructor(fooService: FooService) {
                this.fooService = fooService;
            }
        }
    "#;
    assert_eq!(injects(raw), vec!["FooService".to_string()]);
}

// ── RED-DI2/3/4 — parameter-property shorthand controls (must stay green) ──

#[test]
fn red_di2_private_shorthand_parameter_property_control() {
    let raw = r#"
        @Injectable()
        export class Consumer {
            constructor(private fooService: FooService) {}
        }
    "#;
    assert_eq!(injects(raw), vec!["FooService".to_string()]);
}

#[test]
fn red_di3_readonly_shorthand_parameter_property_control() {
    let raw = r#"
        @Injectable()
        export class Consumer {
            constructor(readonly fooService: FooService) {}
        }
    "#;
    assert_eq!(injects(raw), vec!["FooService".to_string()]);
}

#[test]
fn red_di3_public_and_protected_shorthand_controls() {
    let public_form = r#"
        @Injectable()
        export class Consumer {
            constructor(public fooService: FooService) {}
        }
    "#;
    let protected_form = r#"
        @Injectable()
        export class Consumer {
            constructor(protected fooService: FooService) {}
        }
    "#;
    assert_eq!(injects(public_form), vec!["FooService".to_string()]);
    assert_eq!(injects(protected_form), vec!["FooService".to_string()]);
}

#[test]
fn red_di4_private_readonly_shorthand_parameter_property_control() {
    let raw = r#"
        @Injectable()
        export class Consumer {
            constructor(private readonly fooService: FooService) {}
        }
    "#;
    assert_eq!(injects(raw), vec!["FooService".to_string()]);
}

// ── RED-DI5 — multiple mixed parameters ───────────────────────────────

#[test]
fn red_di5_multiple_mixed_parameters_yield_each_type_once() {
    let raw = r#"
        @Injectable()
        export class Consumer {
            constructor(
                private alpha: AlphaService,
                beta: BetaService,
                readonly gamma: GammaService
            ) {}
        }
    "#;
    assert_eq!(
        injects(raw),
        vec![
            "AlphaService".to_string(),
            "BetaService".to_string(),
            "GammaService".to_string()
        ],
        "every parameter contributes exactly one injection, in declaration order"
    );
}

// ── RED-DI8 — parameter decorators preserve existing behaviour ───────

#[test]
fn red_di8_optional_decorator_preserves_type_identity() {
    let raw = r#"
        @Injectable()
        export class Consumer {
            constructor(@Optional() foo: FooService) {}
        }
    "#;
    assert_eq!(injects(raw), vec!["FooService".to_string()]);
}

#[test]
fn red_di8_inject_token_decorator_still_uses_the_parameter_type() {
    // The existing semantic model uses the parameter TYPE as the injection
    // identity; `@Inject(TOKEN)` does not replace it.
    let raw = r#"
        @Injectable()
        export class Consumer {
            constructor(@Inject(API_TOKEN) api: ApiClient) {}
        }
    "#;
    assert_eq!(injects(raw), vec!["ApiClient".to_string()]);
}

#[test]
fn red_di8_stacked_decorators_and_modifiers_combine() {
    let raw = r#"
        @Injectable()
        export class Consumer {
            constructor(@Optional() @Inject(API_TOKEN) private api: ApiClient) {}
        }
    "#;
    assert_eq!(injects(raw), vec!["ApiClient".to_string()]);
}

// ── RED-DI9 — formatting independence ─────────────────────────────────

#[test]
fn red_di9_multiline_and_trailing_comma_match_single_line() {
    let single_line = r#"
        @Injectable()
        export class Consumer {
            constructor(fooService: FooService, barService: BarService) {}
        }
    "#;
    let multiline = r#"
        @Injectable()
        export class Consumer {
            constructor(
                fooService: FooService,
                barService: BarService,
            ) {}
        }
    "#;
    let expected = vec!["FooService".to_string(), "BarService".to_string()];
    assert_eq!(injects(single_line), expected);
    assert_eq!(injects(multiline), expected);
}

#[test]
fn red_di9_line_breaks_inside_a_parameter_do_not_change_the_type() {
    let raw = "@Injectable()\nexport class Consumer {\n    constructor(\n        private\n        readonly\n        fooService\n        :\n        FooService\n    ) {}\n}\n";
    assert_eq!(injects(raw), vec!["FooService".to_string()]);
}

// ── RED-DI10 — no duplicate from modifier removal ─────────────────────

#[test]
fn red_di10_every_modifier_form_yields_exactly_one_type() {
    let forms = [
        "constructor(fooService: FooService) {}",
        "constructor(private fooService: FooService) {}",
        "constructor(public fooService: FooService) {}",
        "constructor(protected fooService: FooService) {}",
        "constructor(readonly fooService: FooService) {}",
        "constructor(private readonly fooService: FooService) {}",
        "constructor(readonly public fooService: FooService) {}",
    ];
    for form in forms {
        let raw = format!("@Injectable()\nexport class Consumer {{\n    {form}\n}}\n");
        assert_eq!(
            injects(&raw),
            vec!["FooService".to_string()],
            "form `{form}` must produce exactly one injection"
        );
    }
}

// ── Type-eligibility parity (unchanged rules) ─────────────────────────

#[test]
fn modifier_removal_does_not_change_the_extracted_type_identity() {
    let with_modifier = r#"
        @Injectable()
        export class Consumer {
            constructor(private service: GlobalService<Foo>) {}
        }
    "#;
    let without_modifier = r#"
        @Injectable()
        export class Consumer {
            constructor(service: GlobalService<Foo>) {}
        }
    "#;
    assert_eq!(injects(with_modifier), injects(without_modifier));
    // Pre-existing reduction: the leading identifier is the identity.
    assert_eq!(injects(without_modifier), vec!["GlobalService".to_string()]);
}

#[test]
fn qualified_type_names_keep_the_pre_existing_identity_rule() {
    // The pre-existing extractor reduces a type to its leading identifier
    // characters; the modifier fix changes WHEN that rule is applied (any
    // typed parameter) but not the rule itself.
    let with_modifier = r#"
        @Injectable()
        export class Consumer {
            constructor(private service: ns.FooService) {}
        }
    "#;
    let without_modifier = r#"
        @Injectable()
        export class Consumer {
            constructor(service: ns.FooService) {}
        }
    "#;
    assert_eq!(injects(with_modifier), injects(without_modifier));
    assert_eq!(injects(without_modifier), vec!["ns".to_string()]);
}

#[test]
fn type_eligibility_is_unchanged_by_modifier_removal() {
    // No new type filter is introduced by the fix: the pre-existing rule
    // (non-empty leading identifier of the declared type) applies to a bare
    // parameter exactly as it already did to a parameter property.
    let parameter_property = r#"
        @Injectable()
        export class Consumer {
            constructor(private name: string) {}
        }
    "#;
    let bare = r#"
        @Injectable()
        export class Consumer {
            constructor(name: string) {}
        }
    "#;
    assert_eq!(injects(parameter_property), injects(bare));
}

#[test]
fn untyped_parameters_contribute_nothing() {
    let untyped_only = r#"
        @Injectable()
        export class Consumer {
            constructor(fooService) {}
        }
    "#;
    let mixed = r#"
        @Injectable()
        export class Consumer {
            constructor(fooService: FooService, loader) {}
        }
    "#;
    assert!(injects(untyped_only).is_empty());
    assert_eq!(injects(mixed), vec!["FooService".to_string()]);
}

#[test]
fn constructor_without_parameters_has_no_injects() {
    let raw = r#"
        @Injectable()
        export class Consumer {
            constructor() {}
        }
    "#;
    assert!(injects(raw).is_empty());
    assert!(extract_constructor_injects(raw).is_none());
}

// ── Scanner guards preserved ──────────────────────────────────────────

#[test]
fn word_boundary_guard_skips_identifiers_containing_constructor() {
    let raw = r#"
        @Injectable()
        export class Consumer {
            constructorLike(): void {}
            constructor(fooService: FooService) {}
        }
    "#;
    assert_eq!(injects(raw), vec!["FooService".to_string()]);
}

#[test]
fn unterminated_constructor_yields_no_injects() {
    let raw = "@Injectable()\nexport class Consumer {\n    constructor(fooService: FooService";
    assert!(extract_constructor_injects(raw).is_none());
}

// ── Φinjects marker follows the same extraction ───────────────────────

#[test]
fn phi_injects_marker_includes_bare_parameter_injection() {
    // The marker is a projection of the SAME extraction (single source of
    // truth): `Φinjects:` is no longer limited to parameter properties.
    let raw = r#"
        @Injectable()
        export class Consumer {
            constructor(fooService: FooService) {}
        }
    "#;
    let lines = extract_decorators(raw, Fidelity::High)
        .map(|r| r.lines)
        .unwrap_or_default();
    assert!(
        lines
            .iter()
            .any(|l| l.starts_with("Φinjects:") && l.contains("FooService")),
        "expected Φinjects:[FooService] in {lines:?}"
    );
}
