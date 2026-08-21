use super::*;

#[test]
fn low_fidelity_suppresses_fields() {
    assert_eq!(
        extract_field("private readonly userId: string = '';", Fidelity::Low),
        ""
    );
}

#[test]
fn medium_fidelity_keeps_name_and_type() {
    let out = extract_field("private readonly userId: string = '';", Fidelity::Medium);
    assert_eq!(out, "userId:string");
}

#[test]
fn high_fidelity_strips_only_initialiser() {
    let out = extract_field("private readonly userId: string = '';", Fidelity::High);
    assert!(out.starts_with("private readonly userId"));
    assert!(!out.contains("''"));
}

/// Regression: C# property captures include the `{ get; set; }` accessor
/// block. The compacted form must be `Name:string`, not
/// `Name:string { get; set; }`. F-01 diff audit.
#[test]
fn medium_fidelity_strips_property_accessors() {
    let out = extract_field("public string Name { get; set; }", Fidelity::Medium);
    assert_eq!(out, "Name:string");
}

#[test]
fn medium_fidelity_strips_property_accessors_with_modifiers() {
    let out = extract_field(
        "private string Name { get; private set; }",
        Fidelity::Medium,
    );
    assert_eq!(out, "Name:string");
}

#[test]
fn high_fidelity_strips_property_accessors() {
    let out = extract_field("public string Name { get; set; }", Fidelity::High);
    assert!(out.starts_with("public string Name"));
    assert!(!out.contains("{ get; set; }"));
}
