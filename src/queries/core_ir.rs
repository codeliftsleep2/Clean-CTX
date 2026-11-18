//! Immutable query definitions shared by IR producers and the capture engine.
//! No source, workspace, runtime configuration or fidelity participates.
pub(crate) fn complete_query_definition(base_query: &str) -> String {
    let mut capture_query = capture_query(base_query);
    if base_query == crate::queries::CS_QUERY {
        capture_query.push('\n');
        capture_query.push_str(crate::queries::CS_SEMANTIC_QUERY);
    }
    if base_query == crate::queries::TS_QUERY {
        capture_query.push('\n');
        capture_query.push_str(TYPESCRIPT_DECLARATION_QUERY);
        capture_query.push('\n');
        capture_query.push_str(NGRX_LEGACY_QUERY);
        capture_query.push_str(NGRX_SELECTOR_DECLARATIONS_QUERY);
        capture_query.push_str(NGRX_TRANSITIONS_QUERY);
        capture_query.push_str(NGRX_CONSTRUCTORS_QUERY);
        capture_query.push_str(NGRX_SERVICE_OWNERS_QUERY);
    }
    capture_query
}

pub(crate) fn call_capture_query(base_query: &str) -> Option<&'static str> {
    if base_query == crate::queries::CS_QUERY {
        Some(crate::queries::CS_CALL_QUERY)
    } else if base_query == crate::queries::TS_QUERY {
        Some(crate::queries::TS_CALL_QUERY)
    } else if base_query == crate::queries::JAVA_QUERY {
        Some(crate::queries::JAVA_CALL_QUERY)
    } else if base_query == crate::queries::RS_QUERY {
        Some(crate::queries::RS_CALL_QUERY)
    } else {
        None
    }
}

pub(crate) fn capture_query(base_query: &str) -> String {
    match call_capture_query(base_query) {
        Some(call_query) => format!("{base_query}\n{call_query}"),
        None => base_query.to_string(),
    }
}

pub(crate) const TYPESCRIPT_DECLARATION_QUERY: &str = r#"
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

pub(crate) const NGRX_LEGACY_QUERY: &str = r#"
    (program (import_statement
        (import_clause (named_imports
            (import_specifier name: (identifier) @ngrx.legacy.import_name !alias)))
        source: (string . (string_fragment) @ngrx.legacy.import_source .)) @ngrx.legacy.import)
    (program (import_statement (import_clause (named_imports
        (import_specifier alias: (identifier) @ngrx.legacy.alias_binding)))))
    (program [
        (class_declaration (class_heritage (implements_clause
            (type_identifier) @ngrx.legacy.interface))) @ngrx.legacy.owner
        (abstract_class_declaration (class_heritage (implements_clause
            (type_identifier) @ngrx.legacy.interface))) @ngrx.legacy.owner
        (export_statement declaration: [
            (class_declaration (class_heritage (implements_clause
                (type_identifier) @ngrx.legacy.interface))) @ngrx.legacy.owner
            (abstract_class_declaration (class_heritage (implements_clause
                (type_identifier) @ngrx.legacy.interface))) @ngrx.legacy.owner
        ])
    ])
    [(class_declaration type_parameters: (type_parameters
        (type_parameter name: (type_identifier) @ngrx.legacy.shadow))) @ngrx.legacy.shadow_owner
     (abstract_class_declaration type_parameters: (type_parameters
        (type_parameter name: (type_identifier) @ngrx.legacy.shadow))) @ngrx.legacy.shadow_owner]
    (program [
        (interface_declaration name: (type_identifier) @ngrx.legacy.local_binding)
        (type_alias_declaration name: (type_identifier) @ngrx.legacy.local_binding)
        (class_declaration name: (type_identifier) @ngrx.legacy.local_binding)
        (abstract_class_declaration name: (type_identifier) @ngrx.legacy.local_binding)
        (enum_declaration name: (identifier) @ngrx.legacy.local_binding)
        (export_statement declaration: [
            (interface_declaration name: (type_identifier) @ngrx.legacy.local_binding)
            (type_alias_declaration name: (type_identifier) @ngrx.legacy.local_binding)
            (class_declaration name: (type_identifier) @ngrx.legacy.local_binding)
            (abstract_class_declaration name: (type_identifier) @ngrx.legacy.local_binding)
            (enum_declaration name: (identifier) @ngrx.legacy.local_binding)
        ])
    ])
    (program [(lexical_declaration kind: "const"
        (variable_declarator name: (identifier) @ngrx.type.const_name
            value: (_) @ngrx.type.const_value) @ngrx.type.const)
      (export_statement declaration: (lexical_declaration kind: "const"
        (variable_declarator name: (identifier) @ngrx.type.const_name
            value: (_) @ngrx.type.const_value) @ngrx.type.const))])
    [(class_declaration body: (class_body
        (public_field_definition "readonly" name: (property_identifier) @ngrx.type.member
            value: (identifier) @ngrx.type.reference) @ngrx.type.field)) @ngrx.type.owner
     (abstract_class_declaration body: (class_body
        (public_field_definition "readonly" name: (property_identifier) @ngrx.type.member
            value: (identifier) @ngrx.type.reference) @ngrx.type.field)) @ngrx.type.owner]
    [(class_declaration body: (class_body
        (public_field_definition name: (property_identifier) @ngrx.type.any_member))) @ngrx.type.any_owner
     (abstract_class_declaration body: (class_body
        (public_field_definition name: (property_identifier) @ngrx.type.any_member))) @ngrx.type.any_owner
     (class_declaration body: (class_body
        (method_definition name: (property_identifier) @ngrx.type.any_member))) @ngrx.type.any_owner
     (abstract_class_declaration body: (class_body
        (method_definition name: (property_identifier) @ngrx.type.any_member))) @ngrx.type.any_owner]
    [(class_declaration body: (class_body
        (public_field_definition name: [(computed_property_name) (string) (number)]))) @ngrx.type.unsupported_owner
     (abstract_class_declaration body: (class_body
        (public_field_definition name: [(computed_property_name) (string) (number)]))) @ngrx.type.unsupported_owner
     (class_declaration body: (class_body
        (method_definition name: [(computed_property_name) (string) (number)]))) @ngrx.type.unsupported_owner
     (abstract_class_declaration body: (class_body
        (method_definition name: [(computed_property_name) (string) (number)]))) @ngrx.type.unsupported_owner
     (class_declaration body: (class_body
        (method_definition parameters: (formal_parameters
            (required_parameter (accessibility_modifier)))))) @ngrx.type.unsupported_owner
     (abstract_class_declaration body: (class_body
        (method_definition parameters: (formal_parameters
            (required_parameter (accessibility_modifier)))))) @ngrx.type.unsupported_owner]
    (public_field_definition "static") @ngrx.type.static_field
    (variable_declarator name: (_) @ngrx.type.binding_pattern)
    (variable_declarator name: (identifier) @ngrx.type.binding)
    [(required_parameter pattern: (_) @ngrx.type.binding_pattern)
     (optional_parameter pattern: (_) @ngrx.type.binding_pattern)]
    [(required_parameter pattern: (identifier) @ngrx.type.binding)
     (optional_parameter pattern: (identifier) @ngrx.type.binding)
     (required_parameter name: (identifier) @ngrx.type.binding)
     (optional_parameter name: (identifier) @ngrx.type.binding)]
    [(class_declaration name: (type_identifier) @ngrx.type.binding)
     (abstract_class_declaration name: (type_identifier) @ngrx.type.binding)
     (class name: (type_identifier) @ngrx.type.binding)
     (function_declaration name: (identifier) @ngrx.type.binding)
     (function_expression name: (identifier) @ngrx.type.binding)
     (generator_function_declaration name: (identifier) @ngrx.type.binding)
     (generator_function name: (identifier) @ngrx.type.binding)
     (type_parameter name: (type_identifier) @ngrx.type.binding)
     (enum_declaration name: (identifier) @ngrx.type.binding)
     (internal_module name: (_) @ngrx.type.binding_pattern)]
    (arrow_function parameter: (identifier) @ngrx.type.binding)
    (import_require_clause (identifier) @ngrx.type.binding)
    (import_alias (identifier) @ngrx.type.binding)
    (import_clause (identifier) @ngrx.type.binding)
    (namespace_import (identifier) @ngrx.type.binding)
    (import_specifier name: (identifier) @ngrx.type.binding !alias)
    (import_specifier alias: (identifier) @ngrx.type.binding)
    (catch_clause parameter: (_) @ngrx.type.binding_pattern)
    (catch_clause parameter: (identifier) @ngrx.type.binding)
    (ERROR) @ngrx.type.invalid

"#;

pub(crate) const NGRX_SELECTOR_DECLARATIONS_QUERY: &str = r#"
(program (import_statement
    (import_clause (named_imports
        (import_specifier name: (identifier) @ngrx.selector.import_name !alias)))
    source: (string . (string_fragment) @ngrx.selector.import_source .)) @ngrx.selector.import)
(program (import_statement (import_clause (named_imports
    (import_specifier alias: (identifier) @ngrx.selector.alias_binding)))))
(program (export_statement declaration:
    (lexical_declaration kind: "const"
        (variable_declarator
            name: (identifier) @ngrx.selector.name
            value: (call_expression
                function: (identifier) @ngrx.selector.factory
                arguments: (arguments) @ngrx.selector.arguments) @ngrx.selector.initializer)
        @ngrx.selector.declaration)))
"#;

pub(crate) const NGRX_TRANSITIONS_QUERY: &str = r#"
(program (function_declaration parameters: (formal_parameters
        . (required_parameter !value pattern: (identifier) @ngrx.transition.state_parameter)
        . (required_parameter !value pattern: (identifier) @ngrx.transition.parameter
            type: (type_annotation (type_identifier) @ngrx.transition.parameter_type)) .)
    body: (statement_block
        (switch_statement value: (parenthesized_expression
            (member_expression !optional_chain
                object: (identifier) @ngrx.transition.receiver
                property: (property_identifier) @ngrx.transition.property))
            body: (switch_body
                (switch_case value: (identifier) @ngrx.transition.reference) @ngrx.transition.case)) @ngrx.transition.switch)) @ngrx.transition.owner)
(program (export_statement declaration: (function_declaration parameters: (formal_parameters
        . (required_parameter !value pattern: (identifier) @ngrx.transition.state_parameter)
        . (required_parameter !value pattern: (identifier) @ngrx.transition.parameter
            type: (type_annotation (type_identifier) @ngrx.transition.parameter_type)) .)
    body: (statement_block
        (switch_statement value: (parenthesized_expression
            (member_expression !optional_chain
                object: (identifier) @ngrx.transition.receiver
                property: (property_identifier) @ngrx.transition.property))
            body: (switch_body
                (switch_case value: (identifier) @ngrx.transition.reference) @ngrx.transition.case)) @ngrx.transition.switch)) @ngrx.transition.owner))
(for_in_statement kind: ["var" "let" "const"] left: (_) @ngrx.transition.unsupported_loop)

"#;

pub(crate) const NGRX_CONSTRUCTORS_QUERY: &str = r#"
(call_expression function: (member_expression property: (property_identifier) @ngrx.ctor.dispatch)
    arguments: (arguments . (new_expression) @ngrx.ctor.new)) @ngrx.ctor.call
(method_definition "static") @ngrx.ctor.static_method
(call_expression "?.") @ngrx.ctor.optional_call
(new_expression constructor: (identifier) @ngrx.ctor.endpoint) @ngrx.ctor.new_owner
(program [(class_declaration) @ngrx.ctor.program_owner
    (export_statement declaration: (class_declaration) @ngrx.ctor.program_owner)])
(class_declaration name: (type_identifier) @ngrx.ctor.class_name) @ngrx.ctor.class_owner
(class_declaration body: (class_body (method_definition
    name: (property_identifier) @ngrx.ctor.ctor_name
    parameters: (formal_parameters (required_parameter
        (accessibility_modifier) @ngrx.ctor.access
        pattern: (identifier) @ngrx.ctor.store_field
        type: (type_annotation (generic_type name: (type_identifier) @ngrx.ctor.store_type))))))) @ngrx.ctor.store_owner
(class_declaration body: (class_body (method_definition body: (statement_block
    (expression_statement (call_expression
        function: (member_expression !optional_chain
            object: (member_expression !optional_chain object: (this)
                property: (property_identifier) @ngrx.ctor.receiver)
            property: (property_identifier) @ngrx.ctor.terminal)
        arguments: (arguments . (new_expression constructor: (identifier) @ngrx.ctor.target) .)) @ngrx.ctor.owned_call))))) @ngrx.ctor.owner
(class_declaration body: (class_body (public_field_definition name: (_) @ngrx.ctor.other_member))) @ngrx.ctor.member_owner
(class_declaration body: (class_body (method_definition name: (_) @ngrx.ctor.other_member))) @ngrx.ctor.member_owner
(for_in_statement kind: ["var" "let" "const"]) @ngrx.ctor.unsupported

"#;

pub(crate) const NGRX_SERVICE_OWNERS_QUERY: &str = r#"
(class_declaration body: (class_body (method_definition
    body: (statement_block (expression_statement (call_expression) @ngrx.service.call)))
    @ngrx.service.method)) @ngrx.service.owner
"#;
