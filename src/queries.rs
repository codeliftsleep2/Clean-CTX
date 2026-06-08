// src/queries.rs
// TypeScript AST node types: class_declaration, method_definition, 
//   function_declaration, property_signature, throw_statement, 
//   if_statement, for_statement, while_statement, return_statement,
//   import_statement, import_declaration
// C# AST node types: class_declaration, method_declaration, 
//   interface_declaration, field_declaration, throw_statement,
//   for_statement, if_statement, while_statement,
//   return_statement, using_directive

pub const TS_QUERY: &str = r#"
    (class_declaration) @class.root
    (method_definition) @method.root
    (function_declaration) @func.root
    (throw_statement) @throw.root
    (for_statement) @for.root
    (if_statement) @if.root
    (while_statement) @while.root
    (return_statement) @return.root
    (import_statement) @import.root
    ; --- Angular Meta-Layer Phase 1 forward-compat captures ---
    ; These captures expose decorator/object AST nodes for downstream
    ; consumers. In Phase 1 the string-based Tier 1 extractor does
    ; not need them, but they are available via the standard capture
    ; pipeline. The default per-capture closure runs them through
    ; `compact_expression`, which is a safe no-op for these nodes.
    ; NOTE: `decorator_call_expression` is NOT a valid
    ; tree-sitter-typescript node type and must not be added here.
    (decorator) @decorator.root
    (object) @object.root
"#;

pub const CS_QUERY: &str = r#"
    (class_declaration) @class.root
    (method_declaration) @method.root
    (interface_declaration) @interface.root
    (field_declaration) @field.root
    (throw_statement) @throw.root
    (for_statement) @for.root
    (if_statement) @if.root
    (while_statement) @while.root
    (return_statement) @return.root
    (using_directive) @import.root
"#;