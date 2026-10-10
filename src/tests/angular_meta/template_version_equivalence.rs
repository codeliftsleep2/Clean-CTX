//! RED contract for equivalent Angular 15+ template capabilities.
//!
//! Legacy structural directives and Angular 17+ blocks retain their written
//! syntax, but equivalent behavior must expose the same normalized control
//! markers and owned content through the public template compressor.

use crate::angular_meta::template_compress::compress_template_to_string;
use crate::compression::Fidelity;

struct EquivalentTemplate {
    name: &'static str,
    legacy: &'static str,
    modern: &'static str,
    normalized_behavior: &'static [&'static str],
    owned_content: &'static [&'static str],
}

const CASES: &[EquivalentTemplate] = &[
    EquivalentTemplate {
        name: "conditional child",
        legacy: r#"<child-card *ngIf="ready" [item]="item">Ready {{ item.name }}</child-card>"#,
        modern: r#"@if (ready) { <child-card [item]="item">Ready {{ item.name }}</child-card> }"#,
        normalized_behavior: &["@if(ready)"],
        owned_content: &["<child-card", "[item]=\"item\"", "Ready {{ item.name }}"],
    },
    EquivalentTemplate {
        name: "repeated child",
        legacy: r#"<child-row *ngFor="let item of items; trackBy: trackItem" [item]="item"></child-row>"#,
        modern: r#"@for (item of items; track item.id) { <child-row [item]="item"></child-row> }"#,
        normalized_behavior: &["@for(item of items)"],
        owned_content: &["<child-row", "[item]=\"item\""],
    },
    EquivalentTemplate {
        name: "switched child",
        legacy: r#"
<ng-container [ngSwitch]="status">
  <child-ready *ngSwitchCase="'ready'"></child-ready>
  <child-fallback *ngSwitchDefault></child-fallback>
</ng-container>
"#,
        modern: r#"
@switch (status) {
  @case ('ready') { <child-ready></child-ready> }
  @default { <child-fallback></child-fallback> }
}
"#,
        normalized_behavior: &["@switch(status)", "@case('ready')", "@default"],
        owned_content: &["<child-ready", "<child-fallback"],
    },
];

fn assert_contains_all(case: &str, syntax: &str, rendered: &str, expected: &[&str]) {
    for token in expected {
        assert!(
            rendered.contains(token),
            "{case} ({syntax}) lost {token:?}:\n{rendered}"
        );
    }
}

#[test]
fn angular_15_and_17_equivalents_expose_the_same_normalized_behavior_and_ownership() {
    for case in CASES {
        let legacy = compress_template_to_string(case.legacy, Fidelity::Medium);
        let modern = compress_template_to_string(case.modern, Fidelity::Medium);

        assert_contains_all(case.name, "legacy", &legacy, case.normalized_behavior);
        assert_contains_all(case.name, "modern", &modern, case.normalized_behavior);
        assert_contains_all(case.name, "legacy", &legacy, case.owned_content);
        assert_contains_all(case.name, "modern", &modern, case.owned_content);
    }
}

#[test]
fn equivalent_templates_retain_visible_content_at_every_structural_fidelity() {
    for fidelity in [Fidelity::Medium, Fidelity::High, Fidelity::Edit] {
        for case in CASES {
            let legacy = compress_template_to_string(case.legacy, fidelity);
            let modern = compress_template_to_string(case.modern, fidelity);
            assert_contains_all(case.name, "legacy", &legacy, case.owned_content);
            assert_contains_all(case.name, "modern", &modern, case.owned_content);
        }
    }
}

#[test]
fn source_specific_loop_details_are_preserved_without_false_equivalence() {
    let repeated = &CASES[1];
    let legacy = compress_template_to_string(repeated.legacy, Fidelity::Medium);
    let modern = compress_template_to_string(repeated.modern, Fidelity::Medium);

    assert!(legacy.contains("let item of items; trackBy: trackItem"), "{legacy}");
    assert!(modern.contains("track item.id"), "{modern}");
    assert!(!legacy.contains("track item.id"), "{legacy}");
    assert!(!modern.contains("trackBy: trackItem"), "{modern}");
}
