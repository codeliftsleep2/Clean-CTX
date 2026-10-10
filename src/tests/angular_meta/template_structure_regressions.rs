//! RED regressions for truthful Angular 15+ template structure.
//!
//! These tests deliberately exercise the existing public compression entry
//! point. They must fail on behavioral assertions before the ordered template
//! representation is implemented; compilation failure is not valid RED.

use crate::angular_meta::template_compress::compress_template_to_string;
use crate::compression::Fidelity;

fn assert_tokens_in_order(rendered: &str, tokens: &[&str]) {
    let mut previous = 0;
    for token in tokens {
        let relative = rendered[previous..]
            .find(token)
            .unwrap_or_else(|| panic!("missing {token:?} in:\n{rendered}"));
        previous += relative + token.len();
    }
}

#[test]
fn angular_17_sequential_if_blocks_keep_their_conditions_and_children() {
    let source = r#"
@if (!items?.length || !users?.length) {
  <p class="empty">No {{ label }} found.</p>
}
@if (items?.length && users?.length && !isLoading) {
  <child-list [items]="items" (picked)="onPick($event)"></child-list>
  <child-detail [item]="selected" (closed)="onClose()"></child-detail>
}
"#;

    let rendered = compress_template_to_string(source, Fidelity::Medium);

    assert_tokens_in_order(
        &rendered,
        &[
            "!items?.length || !users?.length",
            "No {{ label }} found.",
            "items?.length && users?.length && !isLoading",
            "<child-list",
            "<child-detail",
        ],
    );
}

#[test]
fn angular_15_legacy_regions_keep_conditions_text_and_owned_children() {
    let source = r#"
<ng-container *ngIf="items?.length; else emptyState">
  <child-list
    *ngFor="let item of items; trackBy: trackItem"
    [item]="item"
    (picked)="onPick($event)">
  </child-list>
</ng-container>
<ng-template #emptyState>
  <p>No {{ label }} found.</p>
</ng-template>
"#;

    let rendered = compress_template_to_string(source, Fidelity::Medium);

    assert_tokens_in_order(
        &rendered,
        &[
            "items?.length; else emptyState",
            "let item of items; trackBy: trackItem",
            "<child-list",
            "[item]=\"item\"",
            "(picked)=\"onPick($event)\"",
            "No {{ label }} found.",
        ],
    );
}

#[test]
fn mixed_legacy_and_modern_control_flow_preserves_source_order() {
    let source = r#"
@if (ready) {
  <section *ngIf="selected; else noSelection">
    <child-detail [item]="selected"></child-detail>
  </section>
  <ng-template #noSelection>No selection.</ng-template>
}
<div *ngIf="showLegacy">
  @if (showModernChild) {
    <child-summary></child-summary>
  }
</div>
"#;

    let rendered = compress_template_to_string(source, Fidelity::High);

    assert_tokens_in_order(
        &rendered,
        &[
            "@if(ready)",
            "selected; else noSelection",
            "<child-detail",
            "No selection.",
            "showLegacy",
            "@if(showModernChild)",
            "<child-summary",
        ],
    );
}

#[test]
fn high_fidelity_keeps_visible_text_interpolations_and_accessibility_content() {
    let source = r#"
<article>
  <h1>{{ title }}</h1>
  <p>Error: {{ errorMessage }}</p>
  <img [src]="imageUrl" alt="Selected item preview">
</article>
"#;

    let rendered = compress_template_to_string(source, Fidelity::High);

    for required in [
        "{{ title }}",
        "Error: {{ errorMessage }}",
        "<img",
        "[src]=\"imageUrl\"",
        "alt=\"Selected item preview\"",
    ] {
        assert!(
            rendered.contains(required),
            "High fidelity lost {required:?}:\n{rendered}"
        );
    }
}
