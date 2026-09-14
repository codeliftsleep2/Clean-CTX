use super::{FieldKind, FormArtifact, ReactiveFormShape, extract_reactive_form_shape};
use crate::compression::Fidelity;

fn source(body: &str) -> String {
    format!(
        "import {{ FormBuilder, FormControl, FormGroup, Validators }} from '@angular/forms';\n\
         export class Example {{\n\
         constructor(private fb: FormBuilder) {{}}\n\
         {body}\n\
         }}"
    )
}

fn shape(body: &str) -> ReactiveFormShape {
    extract_reactive_form_shape(&source(body), Fidelity::High).expect("reactive form shape")
}

fn render(body: &str, fidelity: Fidelity) -> String {
    extract_reactive_form_shape(&source(body), fidelity)
        .expect("reactive form shape")
        .render(fidelity)
}

#[test]
fn red_m1_empty_initializer_merges_into_populated_form() {
    let output = render(
        "form: FormGroup = new FormGroup({});\n\
         this.form = this.fb.group({ a: [''], b: [''] });",
        Fidelity::High,
    );

    assert_eq!(output.matches("Φform:form").count(), 1, "{output}");
    assert!(output.contains("Φform:form ctrls=[a,b]"), "{output}");
}

#[test]
fn red_m2_repeated_empty_reconstruction_emits_one_form() {
    let output = render(
        "this.form = this.fb.group({});\n\
         this.form = this.fb.group({});\n\
         this.form = this.fb.group({});",
        Fidelity::High,
    );

    assert_eq!(output, "// --- Φ Forms Meta ---\n  Φform:form\n");
}

#[test]
fn red_m3_branch_evidence_is_unioned_in_source_order() {
    let output = render(
        "if (a) { this.form = this.fb.group({ alpha: [''] }); }\n\
         else { this.form = this.fb.group({ beta: [''] }); }",
        Fidelity::High,
    );

    assert!(output.contains("Φform:form ctrls=[alpha,beta]"), "{output}");
    assert_eq!(output.matches("Φform:form").count(), 1, "{output}");
}

#[test]
fn red_m4_overlapping_fields_and_validators_are_deduplicated() {
    let output = render(
        "this.form = this.fb.group({ common: ['', Validators.required], alpha: [''] });\n\
         this.form = this.fb.group({ common: ['', Validators.required], beta: [''] });",
        Fidelity::High,
    );

    assert!(
        output.contains("Φform:form ctrls=[common,alpha,beta]"),
        "{output}"
    );
    assert_eq!(output.matches("Φcontrol:common").count(), 1, "{output}");
    assert_eq!(
        output.matches("Φvalidator:common → [required]").count(),
        1,
        "{output}"
    );
}

#[test]
fn red_m5_repeated_standalone_control_merges_validator_evidence() {
    let output = render(
        "this.search = new FormControl('');\n\
         this.search = new FormControl('', Validators.required);",
        Fidelity::High,
    );

    assert_eq!(output.matches("Φcontrol:search").count(), 1, "{output}");
    assert!(
        output.contains("Φvalidator:search → [required]"),
        "{output}"
    );
}

#[test]
fn red_m6_normalized_order_is_deterministic() {
    let body = "this.form = this.fb.group({ common: [''], alpha: [''] });\n\
                this.form = this.fb.group({ common: [''], beta: [''] });";
    let expected = render(body, Fidelity::High);

    for _ in 0..8 {
        assert_eq!(render(body, Fidelity::High), expected);
    }
}

#[test]
fn compatible_nested_group_and_array_evidence_merges_recursively() {
    let output = render(
        "this.form = this.fb.group({\n\
         address: this.fb.group({ street: [''] }),\n\
         aliases: this.fb.array([this.fb.group({ first: [''] })])\n\
         });\n\
         this.form = this.fb.group({\n\
         address: this.fb.group({ city: [''] }),\n\
         aliases: this.fb.array([this.fb.group({ second: [''] })])\n\
         });",
        Fidelity::High,
    );

    assert_eq!(output.matches("Φform:address").count(), 1, "{output}");
    assert!(
        output.contains("Φform:address ctrls=[street,city]"),
        "{output}"
    );
    assert_eq!(output.matches("Φarray:aliases").count(), 1, "{output}");
    assert!(
        output.contains("Φarray:aliases ctrls=[first,second]"),
        "{output}"
    );
}

#[test]
fn repeated_standalone_array_merges_nested_group_evidence() {
    let output = render(
        "this.items = this.fb.array([this.fb.group({ alpha: [''] })]);\n\
         this.items = this.fb.array([this.fb.group({ beta: [''] })]);",
        Fidelity::High,
    );

    assert_eq!(output.matches("Φarray:items").count(), 1, "{output}");
    assert!(
        output.contains("Φarray:items ctrls=[alpha,beta]"),
        "{output}"
    );
}

#[test]
fn red_g1_nested_group_survives_as_recursive_structure() {
    let shape = shape(
        "profileForm = this.fb.group({\n\
         name: this.fb.control(''),\n\
         address: this.fb.group({ street: this.fb.control(''), city: this.fb.control('') })\n\
         });",
    );
    let FormArtifact::Form(form) = &shape.artifacts[0] else {
        panic!("expected top-level form")
    };
    let address = form
        .fields
        .iter()
        .find(|field| field.name == "address")
        .expect("address field");
    let FieldKind::Group(fields) = &address.kind else {
        panic!("address must remain a group")
    };

    assert_eq!(
        fields
            .iter()
            .map(|field| field.name.as_str())
            .collect::<Vec<_>>(),
        ["street", "city"]
    );
}

#[test]
fn red_g2_nested_validator_is_rendered_at_high_fidelity() {
    let output = render(
        "form = this.fb.group({ address: this.fb.group({\n\
         street: this.fb.control('', Validators.required) }) });",
        Fidelity::High,
    );

    assert!(
        output.contains("    Φvalidator:street → [required]"),
        "{output}"
    );
}

#[test]
fn red_g3_explicit_nested_form_group_is_recursive() {
    let output = render(
        "form = this.fb.group({ address: new FormGroup({\n\
         street: new FormControl('') }) });",
        Fidelity::High,
    );

    assert!(
        output.contains("  Φform:address ctrls=[street]"),
        "{output}"
    );
    assert!(output.contains("    Φcontrol:street"), "{output}");
}

#[test]
fn red_g4_two_level_group_nesting_preserves_hierarchy() {
    let output = render(
        "form = this.fb.group({ profile: this.fb.group({\n\
         address: this.fb.group({ street: [''] }) }) });",
        Fidelity::High,
    );

    assert!(
        output.contains("  Φform:profile ctrls=[address]"),
        "{output}"
    );
    assert!(
        output.contains("    Φform:address ctrls=[street]"),
        "{output}"
    );
    assert!(output.contains("      Φcontrol:street"), "{output}");
}

#[test]
fn red_g5_sibling_control_group_and_array_are_distinct() {
    let shape = shape(
        "form = this.fb.group({\n\
         name: [''],\n\
         address: this.fb.group({ street: [''] }),\n\
         aliases: this.fb.array([this.fb.control('')])\n\
         });",
    );
    let FormArtifact::Form(form) = &shape.artifacts[0] else {
        panic!("expected top-level form")
    };

    assert!(matches!(&form.fields[0].kind, FieldKind::Control));
    assert!(matches!(&form.fields[1].kind, FieldKind::Group(_)));
    assert!(matches!(&form.fields[2].kind, FieldKind::Array(_)));
}

#[test]
fn red_g6_group_inside_array_preserves_existing_summary() {
    let output = render(
        "form = this.fb.group({ addresses: this.fb.array([\n\
         this.fb.group({ street: [''], city: [''] })]) });",
        Fidelity::High,
    );

    assert!(
        output.contains("Φarray:addresses ctrls=[street,city]"),
        "{output}"
    );
}

#[test]
fn red_g7_array_inside_nested_group_is_preserved() {
    let output = render(
        "form = this.fb.group({ profile: this.fb.group({\n\
         aliases: this.fb.array([this.fb.group({ value: [''] })])\n\
         }) });",
        Fidelity::High,
    );

    assert!(
        output.contains("    Φarray:aliases ctrls=[value]"),
        "{output}"
    );
}

#[test]
fn red_g8_nested_detail_obeys_fidelity_gating() {
    let body = "profileForm = this.fb.group({ name: [''], address: this.fb.group({\n\
                street: ['', Validators.required], city: [''] }) });";
    let low = render(body, Fidelity::Low);
    let medium = render(body, Fidelity::Medium);
    let high = render(body, Fidelity::High);

    assert_eq!(low, "// --- Φ Forms Meta ---\n  Φform:profileForm\n");
    assert!(medium.contains("Φform:profileForm ctrls=[name,address]"));
    assert!(!medium.contains("Φform:address"));
    assert!(!medium.contains("Φcontrol:address"));
    assert!(!medium.contains("Φcontrol:street"));
    assert!(high.contains("Φform:address ctrls=[street,city]"));
    assert!(high.contains("Φvalidator:street → [required]"));
}

#[test]
fn red_c1_reconstructed_form_with_nested_group_is_normalized() {
    let body = "profileForm: FormGroup = new FormGroup({});\n\
                ngOnInit() { this.profileForm = this.fb.group({\n\
                name: ['', Validators.required],\n\
                address: this.fb.group({\n\
                street: ['', Validators.required], city: ['']\n\
                }) }); }";
    let output = render(body, Fidelity::High);

    assert_eq!(output.matches("Φform:profileForm").count(), 1, "{output}");
    assert!(
        output.contains("Φform:profileForm ctrls=[name,address]"),
        "{output}"
    );
    assert!(output.contains("Φcontrol:name"), "{output}");
    assert!(output.contains("Φvalidator:name → [required]"), "{output}");
    assert!(
        output.contains("Φform:address ctrls=[street,city]"),
        "{output}"
    );
    assert!(output.contains("Φcontrol:city"), "{output}");
    assert!(
        output.contains("Φvalidator:street → [required]"),
        "{output}"
    );
}

#[test]
fn reconstructed_nested_formatter_variants_are_equivalent() {
    let compact = "form=this.fb.group({address:this.fb.group({street:[''],city:[''],}),});";
    let multiline = concat!(
        "form = this.fb.group(\n{\n",
        "address: this.fb.group(\n{\nstreet: [''],\ncity: [''],\n},\n),\n",
        "},\n);"
    );

    assert_eq!(
        render(compact, Fidelity::High),
        render(multiline, Fidelity::High)
    );
}
