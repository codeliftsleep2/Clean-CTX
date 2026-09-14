use crate::angular_meta::formly::{FormlyKind, expand_phi, extract_formly_shape, has_formly};
use crate::angular_meta::run_meta_layer_with_config;
use crate::compression::Fidelity;
use crate::config::MetaLayerConfig;

fn render(source: &str, fidelity: Fidelity) -> String {
    extract_formly_shape(source, fidelity)
        .map(|shape| shape.render(fidelity))
        .unwrap_or_default()
}

fn imported(fields: &str) -> String {
    format!(
        "import {{ FormlyFieldConfig }} from '@ngx-formly/core';\n\
         fields: FormlyFieldConfig[] = {fields};"
    )
}

#[test]
fn red_fm1_detection_via_ngx_formly_import() {
    let source = imported("[{ key: 'name', type: 'input' }]");
    assert!(has_formly(&source));
    assert!(render(&source, Fidelity::Medium).contains("Φffield:name type=input"));
}

#[test]
fn red_fm2_type_usage_detects_without_import() {
    let source = "fields: FormlyFieldConfig[] = [{ key: 'name', type: 'input' }];";
    assert!(has_formly(source));
}

#[test]
fn red_fm3_ordinary_key_type_objects_do_not_activate() {
    let source = "const fields = [{ key: 'name', type: 'input' }];";
    assert!(!has_formly(source));
    assert!(render(source, Fidelity::High).is_empty());
}

#[test]
fn red_fm4_low_counts_top_level_fields_only() {
    let source = imported(
        "[{ key: 'a', type: 'input' }, { key: 'b', type: 'select' }, \
         { key: 'c', type: 'checkbox' }]",
    );
    let output = render(&source, Fidelity::Low);
    assert_eq!(output, "// --- Φ Formly Meta ---\nΦffield:count=3\n");
    assert!(!output.contains("Φffield:a"));
}

#[test]
fn red_fm5_medium_emits_top_level_fields_only() {
    let source = imported(
        "[{ key: 'a', type: 'input' }, { key: 'b', type: 'select' }, \
         { key: 'c', type: 'checkbox' }]",
    );
    let output = render(&source, Fidelity::Medium);
    assert!(output.contains("Φffield:a type=input"));
    assert!(output.contains("Φffield:b type=select"));
    assert!(output.contains("Φffield:c type=checkbox"));
    assert!(!output.contains("Φfgroup:"));
    assert!(!output.contains("Φfexpr:"));
}

#[test]
fn red_fm6_multiline_formatting_is_equivalent() {
    let compact = imported("[{key:'a',type:'input'},{key:\"b\",type:`select`}]");
    let formatted = imported(
        "[\n  {\n    key: 'a',\n    type: 'input',\n  },\n  {\n    key: \"b\",\n    type: `select`,\n  },\n]",
    );
    assert_eq!(
        render(&compact, Fidelity::High),
        render(&formatted, Fidelity::High)
    );
}

#[test]
fn red_fm7_field_group_preserves_child_order() {
    let source = imported(
        "[{ key: 'address', fieldGroup: [\
         { key: 'street', type: 'input' }, { key: 'city', type: 'input' }\
         ] }]",
    );
    assert!(render(&source, Fidelity::High).contains("Φfgroup:address fields=[street,city]"));
}

#[test]
fn red_fm8_nested_groups_preserve_hierarchy() {
    let source = imported(
        "[{ key: 'profile', fieldGroup: [{ key: 'address', fieldGroup: [\
         { key: 'street', type: 'input' }] }] }]",
    );
    let output = render(&source, Fidelity::High);
    let profile = output
        .find("Φfgroup:profile fields=[address]")
        .expect("profile group");
    let address = output
        .find("Φfgroup:address fields=[street]")
        .expect("address group");
    assert!(profile < address);
}

#[test]
fn red_fm9_hide_expression_is_structural_only() {
    let source = imported("[{ key: 'state', hideExpression: model => !model.country }]");
    let output = render(&source, Fidelity::High);
    assert!(output.contains("Φfexpr:state → hideExpression"));
    assert!(!output.contains("!model.country"));
}

#[test]
fn red_fm10_expression_properties_emit_only_controlled_property() {
    let source = imported(
        "[{ key: 'details', expressionProperties: {\
         'props.disabled': model => !model.enabled,\
         'props.label': model => model.secret } }]",
    );
    let output = render(&source, Fidelity::High);
    assert!(output.contains("Φfexpr:details → props.disabled"));
    assert!(!output.contains("model.enabled"));
    assert!(!output.contains("props.label"));
}

#[test]
fn red_fm11_dynamic_key_and_type_are_not_fabricated() {
    let source = imported("[{ key: getKey(), type: 'input' }, { key: 'stable', type: fieldType }]");
    let output = render(&source, Fidelity::Medium);
    assert!(!output.contains("getKey"));
    assert!(!output.contains("fieldType"));
    assert!(output.contains("Φffield:stable"));
    assert!(!output.contains("Φffield:stable type="));
}

#[test]
fn red_fm12_template_copy_is_excluded() {
    let source = imported(
        "[{ key: 'name', type: 'input', props: { label: 'Customer Name',\
         placeholder: 'Enter your full legal name' } }]",
    );
    let output = render(&source, Fidelity::High);
    assert!(!output.contains("Customer Name"));
    assert!(!output.contains("Enter your full legal name"));
}

#[test]
fn red_fm13_fidelity_gating_is_exact() {
    let source = imported(
        "[{ key: 'address', fieldGroup: [{ key: 'state', type: 'select',\
         hideExpression: model => !model.country }] }]",
    );
    let low = render(&source, Fidelity::Low);
    let medium = render(&source, Fidelity::Medium);
    let high = render(&source, Fidelity::High);
    assert!(low.contains("Φffield:count=1"));
    assert!(!low.contains("Φfgroup:") && !low.contains("Φfexpr:"));
    assert!(medium.contains("Φffield:address"));
    assert!(!medium.contains("Φfgroup:") && !medium.contains("Φfexpr:"));
    assert!(high.contains("Φfgroup:address fields=[state]"));
    assert!(high.contains("Φfexpr:state → hideExpression"));
}

#[test]
fn red_fm14_configuration_opt_out_is_isolated() {
    let source = imported("[{ key: 'name', type: 'input' }]")
        + "\nimport { signal } from '@angular/core'; signalValue = signal(0);";
    let mut config = MetaLayerConfig::default();
    config.formly.enabled = false;
    let output = run_meta_layer_with_config(&source, &[], Fidelity::High, Some(&config))
        .expect("signals remain enabled")
        .render();
    assert!(!output.contains("Formly Meta"));
    assert!(!output.contains("Φffield:"));
    assert!(output.contains("Φsignal:signalValue"));
}

#[test]
fn red_fm15_reactive_forms_and_formly_coexist() {
    let source = imported("[{ key: 'name', type: 'input' }]")
        + "\nimport { FormGroup, FormControl } from '@angular/forms';\
           form = new FormGroup({ name: new FormControl('') });";
    let output = run_meta_layer_with_config(source.as_str(), &[], Fidelity::High, None)
        .expect("both layers should activate")
        .render();
    assert!(output.contains("// --- Φ Forms Meta ---"));
    assert!(output.contains("Φcontrol:name"));
    assert!(output.contains("// --- Φ Formly Meta ---"));
    assert!(output.contains("Φffield:name type=input"));
    assert_eq!(output.matches("Φffield:name").count(), 1);
}

#[test]
fn plain_fields_assignment_is_supported_only_with_import_gate() {
    let source = "import { FormlyModule } from '@ngx-formly/core';\n\
                  this.fields = [{ key: 'name', type: 'input' }];";
    assert!(render(source, Fidelity::Medium).contains("Φffield:name type=input"));
}

#[test]
fn marker_vocabulary_implements_shared_phi_contract() {
    use crate::angular_meta::phi::PhiMarker;
    assert_eq!(FormlyKind::Field.marker_prefix(), "Φffield:");
    assert_eq!(expand_phi("Φfgroup"), Some("Formly field group"));
}

#[test]
fn production_dispatch_reaches_formly_only_source() {
    use crate::layers::meta::{AngularMetaLayer, MetaLayer};
    use std::path::Path;

    let source = imported("[{ key: 'name', type: 'input' }]");
    let layer = AngularMetaLayer::new();
    assert!(layer.is_applicable(&source, Path::new("fields.ts"), None));
    let output = layer
        .enrich_with_path(&source, Path::new("fields.ts"), &[], Fidelity::Medium, None)
        .expect("Formly-only source must reach production enrichment");
    assert!(output.rendered.contains("// --- Φ Formly Meta ---"));
}
