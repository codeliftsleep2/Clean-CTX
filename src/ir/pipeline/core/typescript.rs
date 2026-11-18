//! TypeScript class identity from the declaration's own name field.
use super::CapturedNode;
use std::collections::HashMap;

// Supplemental IR-only evidence: the compression/diff query and the raw
// decorator-bearing source needed by language/framework layers stay unchanged.
pub(super) const DECLARATION_QUERY: &str = r#"
    (class_declaration name: (type_identifier) @ts.identity.name) @ts.identity.owner
    (abstract_class_declaration name: (type_identifier) @ts.identity.name) @ts.identity.owner

    (class_declaration body: (class_body
        (public_field_definition name: (property_identifier) @angular.initializer.receiver
            value: (_) @angular.initializer.value) @angular.initializer.field)) @angular.initializer.owner
    (abstract_class_declaration body: (class_body
        (public_field_definition name: (property_identifier) @angular.initializer.receiver
            value: (_) @angular.initializer.value) @angular.initializer.field)) @angular.initializer.owner
    (call_expression "?." @angular.initializer.optional) @angular.initializer.optional_call
    (call_expression arguments: (arguments
        (identifier) @angular.initializer.identifier_argument)) @angular.initializer.argument_call
"#;

pub(super) fn apply_declaration_names(captures: &mut [CapturedNode]) {
    let names: HashMap<usize, &str> = captures
        .iter()
        .filter(|capture| capture.name == "ts.identity.name")
        .map(|capture| (capture.match_index, capture.raw_text.as_str()))
        .collect();
    let owners: HashMap<(usize, usize), String> = captures
        .iter()
        .filter(|capture| capture.name == "ts.identity.owner")
        .filter_map(|capture| {
            names
                .get(&capture.match_index)
                .map(|name| ((capture.start_byte, capture.end_byte), (*name).to_string()))
        })
        .collect();
    for capture in captures
        .iter_mut()
        .filter(|capture| capture.name == "class.root")
    {
        if let Some(name) = owners.get(&(capture.start_byte, capture.end_byte)) {
            capture.text.clone_from(name);
        }
    }
}

#[cfg(test)]
#[path = "../../../tests/ir/angular_inject_evidence_capture.rs"]
mod angular_inject_evidence_capture_tests;
