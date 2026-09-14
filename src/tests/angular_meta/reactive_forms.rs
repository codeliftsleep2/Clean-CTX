use crate::angular_meta::reactive_forms::{
    ReactiveFormKind, expand_phi, extract_reactive_form_shape, has_reactive_forms,
};
use crate::angular_meta::run_meta_layer_with_config;
use crate::compression::Fidelity;
use crate::config::MetaLayerConfig;

fn render(source: &str, fidelity: Fidelity) -> String {
    extract_reactive_form_shape(source, fidelity)
        .map(|shape| shape.render(fidelity))
        .unwrap_or_default()
}

fn builder_source(body: &str) -> String {
    format!(
        "import {{ FormBuilder, Validators }} from '@angular/forms';\n\
         export class Example {{\n\
         constructor(private fb: FormBuilder) {{}}\n{body}\n}}"
    )
}

fn untyped_builder_source(body: &str) -> String {
    format!(
        "import {{ UntypedFormBuilder, Validators }} from '@angular/forms';\n\
         export class Example {{\n\
         constructor(private fb: UntypedFormBuilder) {{}}\n{body}\n}}"
    )
}

#[test]
fn red_u1_untyped_form_builder_activates_extraction() {
    let source = "import {\n\
                  UntypedFormBuilder,\n\
                  Validators\n\
                  } from '@angular/forms';\n\
                  export class Example {\n\
                  constructor(private fb: UntypedFormBuilder) {}\n\
                  init() {\n\
                  this.form = this.fb.group({\n\
                  name: this.fb.control('', Validators.required)\n\
                  });\n\
                  }\n\
                  }";

    assert_eq!(
        render(source, Fidelity::High),
        concat!(
            "// --- Φ Forms Meta ---\n",
            "  Φform:form ctrls=[name]\n",
            "  Φcontrol:name\n",
            "  Φvalidator:name → [required]\n",
        )
    );
}

#[test]
fn red_u2_typed_and_untyped_form_builders_have_identical_metadata() {
    let body = "form = this.fb.group({ name: this.fb.control('', Validators.required) });";
    assert_eq!(
        render(&builder_source(body), Fidelity::High),
        render(&untyped_builder_source(body), Fidelity::High)
    );
}

#[test]
fn red_u3_untyped_form_group_matches_typed_form_group() {
    let typed = "import { FormGroup, FormControl } from '@angular/forms';\n\
                 form = new FormGroup({ name: new FormControl('') });";
    let untyped = "import { UntypedFormGroup, UntypedFormControl } from '@angular/forms';\n\
         form = new UntypedFormGroup({ name: new UntypedFormControl('') });";

    assert_eq!(
        render(typed, Fidelity::High),
        render(untyped, Fidelity::High)
    );
}

#[test]
fn red_u4_untyped_form_control_matches_typed_form_control() {
    let typed = "import { FormControl } from '@angular/forms';\n\
                 name = new FormControl('', Validators.required);";
    let untyped = "import { UntypedFormControl } from '@angular/forms';\n\
                   name = new UntypedFormControl('', Validators.required);";

    assert_eq!(
        render(typed, Fidelity::High),
        render(untyped, Fidelity::High)
    );
    assert!(render(untyped, Fidelity::High).contains("Φcontrol:name"));
}

#[test]
fn red_u5_untyped_form_array_matches_typed_form_array() {
    let typed = "import { FormArray } from '@angular/forms'; items = new FormArray([]);";
    let untyped =
        "import { UntypedFormArray } from '@angular/forms'; items = new UntypedFormArray([]);";

    assert_eq!(
        render(typed, Fidelity::High),
        render(untyped, Fidelity::High)
    );
    assert!(render(untyped, Fidelity::High).contains("Φarray:items"));
}

#[test]
fn red_u6_aliased_untyped_form_builder_import_is_supported() {
    let source = "import { UntypedFormBuilder as LegacyBuilder } from '@angular/forms';\n\
         constructor(private fb: LegacyBuilder) {}\n\
         form = this.fb.group({ name: [''] });";

    assert!(render(source, Fidelity::Medium).contains("Φform:form ctrls=[name]"));
}

#[test]
fn red_u7_mixed_typed_and_untyped_imports_are_discovered_once() {
    let source = "import { FormControl, UntypedFormBuilder, UntypedFormArray } from '@angular/forms';\n\
         constructor(private fb: UntypedFormBuilder) {}\n\
         form = this.fb.group({\n\
         name: new FormControl(''),\n\
         items: new UntypedFormArray([])\n\
         });";

    assert_eq!(
        render(source, Fidelity::High),
        "// --- Φ Forms Meta ---\n  Φform:form ctrls=[name,items]\n  Φcontrol:name\n  Φarray:items\n"
    );
}

#[test]
fn red_u8_untyped_form_builder_preserves_fidelity_gating() {
    let body = "form = this.fb.group({ name: ['', Validators.required] });";
    let typed = builder_source(body);
    let untyped = untyped_builder_source(body);

    for fidelity in [Fidelity::Low, Fidelity::Medium, Fidelity::High] {
        assert_eq!(render(&typed, fidelity), render(&untyped, fidelity));
    }
}

#[test]
fn red_u9_existing_typed_forms_output_is_unchanged() {
    let source = "import { FormBuilder, FormGroup, FormControl, FormArray } from '@angular/forms';\n\
         constructor(private fb: FormBuilder) {}\n\
         form = new FormGroup({\n\
         name: new FormControl(''),\n\
         items: new FormArray([]),\n\
         alias: this.fb.control('')\n\
         });";

    assert_eq!(
        render(source, Fidelity::High),
        concat!(
            "// --- Φ Forms Meta ---\n",
            "  Φform:form ctrls=[name,items,alias]\n",
            "  Φcontrol:name\n",
            "  Φarray:items\n",
            "  Φcontrol:alias\n",
        )
    );
}

#[test]
fn red_f1_detection_requires_angular_forms() {
    let source = "class Example { form = this.fb.group({ name: [''] }); }";
    assert!(!has_reactive_forms(source));
    assert!(render(source, Fidelity::High).is_empty());
}

#[test]
fn red_f2_forms_import_alone_is_insufficient() {
    let source = "import { FormBuilder } from '@angular/forms'; class Example {}";
    assert!(!has_reactive_forms(source));
    assert!(render(source, Fidelity::High).is_empty());
}

#[test]
fn red_f3_low_form_summary_has_identity_only() {
    let source = builder_source("form = this.fb.group({ name: [''], email: [''] });");
    let output = render(&source, Fidelity::Low);
    assert_eq!(output, "// --- Φ Forms Meta ---\n  Φform:form\n");
}

#[test]
fn red_f4_medium_control_summary() {
    let source = builder_source("form = this.fb.group({ name: [''], email: [''] });");
    let output = render(&source, Fidelity::Medium);
    assert!(output.contains("Φform:form ctrls=[name,email]"));
    assert!(output.contains("Φcontrol:name"));
    assert!(output.contains("Φcontrol:email"));
}

#[test]
fn red_f5_explicit_form_group_and_control() {
    let source = "import { FormGroup, FormControl } from '@angular/forms';\n\
                  form = new FormGroup({ name: new FormControl('') });";
    let output = render(source, Fidelity::Medium);
    assert!(output.contains("Φform:form ctrls=[name]"));
    assert!(output.contains("Φcontrol:name"));
}

#[test]
fn red_f6_constructor_form_builder_alias() {
    let source = "import { FormBuilder } from '@angular/forms';\n\
                  constructor(private formBuilder: FormBuilder) {}\n\
                  form = this.formBuilder.group({ name: [''] });";
    assert!(render(source, Fidelity::Medium).contains("Φcontrol:name"));
}

#[test]
fn red_f7_injected_form_builder_alias() {
    let source = "import { FormBuilder } from '@angular/forms';\n\
                  private fb = inject(FormBuilder);\n\
                  form = this.fb.group({ name: [''] });";
    assert!(render(source, Fidelity::Medium).contains("Φcontrol:name"));
}

#[test]
fn red_f8_form_arrays_from_constructor_and_builder() {
    let explicit = "import { FormArray } from '@angular/forms'; items = new FormArray([]);";
    assert!(render(explicit, Fidelity::Medium).contains("Φarray:items"));

    let builder = builder_source("items = this.fb.array([]);");
    assert!(render(&builder, Fidelity::Medium).contains("Φarray:items"));
}

#[test]
fn red_f9_builtin_validators_are_concise() {
    let source = builder_source(
        "form = this.fb.group({\n\
         name: ['', Validators.required],\n\
         email: ['', Validators.email],\n\
         zip: ['', Validators.pattern(/x/)],\n\
         code: ['', Validators.minLength(3)]\n\
         });",
    );
    let output = render(&source, Fidelity::High);
    assert!(output.contains("Φvalidator:name → [required]"));
    assert!(output.contains("Φvalidator:email → [email]"));
    assert!(output.contains("Φvalidator:zip → [pattern]"));
    assert!(output.contains("Φvalidator:code → [minLength]"));
}

#[test]
fn red_f10_custom_validator_uses_stable_identifier() {
    let source = builder_source(
        "form = this.fb.group({ username: ['', [Validators.required, uniqueUsernameValidator]] });",
    );
    assert!(
        render(&source, Fidelity::High)
            .contains("Φvalidator:username → [required,uniqueUsernameValidator]")
    );
}

#[test]
fn red_f11_validators_are_high_only() {
    let source = builder_source("form = this.fb.group({ name: ['', Validators.required] });");
    assert!(!render(&source, Fidelity::Low).contains("Φvalidator:"));
    assert!(!render(&source, Fidelity::Medium).contains("Φvalidator:"));
    assert!(render(&source, Fidelity::High).contains("Φvalidator:name → [required]"));
}

#[test]
fn red_f12_nested_array_structure_is_concise() {
    let source = builder_source(
        "form = this.fb.group({ addresses: this.fb.array([this.fb.group({ street: [''], city: [''] })]) });",
    );
    let medium = render(&source, Fidelity::Medium);
    assert!(medium.contains("Φarray:addresses"));
    assert!(!medium.contains("Φarray:addresses ctrls="));
    let high = render(&source, Fidelity::High);
    assert!(high.contains("Φarray:addresses ctrls=[street,city]"));
    assert!(!high.contains("this.fb.group"));
}

#[test]
fn red_f13_formatter_variants_are_equivalent() {
    let single = builder_source("form=this.fb.group({name:[''],email:[''],});");
    let multiline =
        builder_source("form = this.fb.group(\n  {\n    name: [''],\n    email: [''],\n  },\n);");
    assert_eq!(
        render(&single, Fidelity::High),
        render(&multiline, Fidelity::High)
    );
}

#[test]
fn red_f14_configuration_opt_out_is_isolated() {
    let source = builder_source("form = this.fb.group({ name: [''] }); signalValue = signal(0);")
        + "\nimport { signal } from '@angular/core';";
    let mut config = MetaLayerConfig::default();
    config.reactive_forms.enabled = false;
    let output = run_meta_layer_with_config(&source, &[], Fidelity::Medium, Some(&config))
        .expect("signals remain enabled")
        .render();
    assert!(!output.contains("Forms Meta"));
    assert!(!output.contains("Φform:"));
    assert!(output.contains("Φsignal:signalValue"));
}

#[test]
fn red_f15_forms_do_not_duplicate_template_bindings() {
    let source = builder_source(
        "template = `<form [formGroup]=\"form\"><input formControlName=\"name\"></form>`;\n\
         form = this.fb.group({ name: [''] });",
    );
    let output = render(&source, Fidelity::High);
    assert!(output.contains("Φform:form"));
    assert!(!output.contains("Φtbind:"));
    assert!(!output.contains("formControlName"));
}

#[test]
fn marker_vocabulary_implements_shared_phi_contract() {
    use crate::angular_meta::phi::PhiMarker;
    assert_eq!(ReactiveFormKind::Form.marker_prefix(), "Φform:");
    assert_eq!(expand_phi("Φarray"), Some("FormArray"));
}

#[test]
fn aliased_angular_form_imports_are_supported() {
    let source = "import { FormBuilder as AngularFormBuilder } from '@angular/forms';\n\
                  private builder = inject(AngularFormBuilder);\n\
                  form = this.builder.group({ name: [''] });";
    assert!(render(source, Fidelity::Medium).contains("Φform:form ctrls=[name]"));
}

#[test]
fn validators_compose_is_flattened_conservatively() {
    let source = builder_source(
        "form = this.fb.group({ email: ['', Validators.compose([Validators.required, Validators.email])] });",
    );
    assert!(render(&source, Fidelity::High).contains("Φvalidator:email → [required,email]"));
}

#[test]
fn production_dispatch_reaches_forms_only_source() {
    use crate::layers::meta::{AngularMetaLayer, MetaLayer};
    use std::path::Path;

    let source = builder_source("form = this.fb.group({ name: [''] });");
    let layer = AngularMetaLayer::new();
    assert!(layer.is_applicable(&source, Path::new("account.component.ts"), None));
    let output = layer
        .enrich_with_path(
            &source,
            Path::new("account.component.ts"),
            &[],
            Fidelity::Medium,
            None,
        )
        .expect("forms-only source must reach production enrichment");
    assert!(output.rendered.contains("// --- Φ Forms Meta ---"));
    assert!(output.rendered.contains("Φform:form ctrls=[name]"));
}
