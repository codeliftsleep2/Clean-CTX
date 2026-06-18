// src/queries.rs
// TypeScript AST node types: class_declaration, method_definition, 
//   function_declaration, property_signature, throw_statement, 
//   if_statement, for_statement, while_statement, return_statement,
//   import_statement, import_declaration
// C# AST node types: class_declaration, method_declaration, 
//   interface_declaration, field_declaration, throw_statement,
//   for_statement, if_statement, while_statement,
//   return_statement, using_directive
//
// FAANG audit: expanded all queries to capture the full set of
// structural, control-flow, and Java/Rust-specific constructs.
// Every language now captures: class/enum/interface/record types,
// methods, fields, constructors, control flow, imports, and
// language-specific items (package, mod, macro, type alias, etc.).

pub const TS_QUERY: &str = r#"
    ; --- TypeScript/JavaScript structural captures ---
    (class_declaration) @class.root
    (method_definition) @method.root
    (function_declaration) @func.root
    (property_signature) @field.root
    ; --- TypeScript-specific type declarations ---
    (interface_declaration) @interface.root
    (enum_declaration) @enum.root
    (type_alias_declaration) @type.root
    ; --- Control flow captures ---
    (throw_statement) @throw.root
    (for_statement) @for.root
    (if_statement) @if.root
    (while_statement) @while.root
    (return_statement) @return.root
    ; --- Import captures ---
    (import_statement) @import.root
    ; --- Angular Meta-Layer forward-compat captures ---
    (decorator) @decorator.root
    (object) @object.root
"#;

pub const CS_QUERY: &str = r#"
    ; --- C# structural captures ---
    (class_declaration) @class.root
    (method_declaration) @method.root
    (interface_declaration) @interface.root
    (struct_declaration) @struct.root
    (enum_declaration) @enum.root
    (record_declaration) @record.root
    (field_declaration) @field.root
    (constructor_declaration) @constructor.root
    ; --- Control flow captures ---
    (throw_statement) @throw.root
    (for_statement) @for.root
    (if_statement) @if.root
    (while_statement) @while.root
    (do_statement) @do.root
    (return_statement) @return.root
    (switch_statement) @switch.root
    (try_statement) @try.root
    ; --- Import captures ---
    (using_directive) @import.root
"#;

// Rust AST node types: struct_item, enum_item, trait_item, impl_item,
//   function_item, type_item, field_declaration, use_declaration,
//   return_expression, if_expression, for_expression, while_expression,
//   match_expression, macro_invocation, mod_item
pub const RS_QUERY: &str = r#"
    ; Core structural captures
    (struct_item) @struct.root
    (enum_item) @enum.root
    (trait_item) @trait.root
    (impl_item) @impl.root
    (function_item) @method.root
    (type_item) @type.root
    (field_declaration) @field.root
    ; Import and module captures
    (use_declaration) @import.root
    (mod_item) @mod.root
    ; Control flow captures
    (return_expression) @return.root
    (if_expression) @if.root
    (for_expression) @for.root
    (while_expression) @while.root
    (match_expression) @match.root
    (loop_expression) @loop.root
    ; Exception / panic captures
    (macro_invocation
        macro: (identifier) @_panic_macro
        (#match? @_panic_macro "panic|unreachable|unimplemented|todo|assert")
    ) @throw.root
    ; Macro captures
    (macro_invocation) @macro.root
"#;

// Java AST node types: class_declaration, interface_declaration,
//   method_declaration, constructor_declaration, field_declaration,
//   enum_declaration, record_declaration, import_declaration,
//   package_declaration, if_statement, for_statement, while_statement,
//   do_statement, return_statement, throw_statement, try_statement,
//   switch_statement
pub const JAVA_QUERY: &str = r#"
    ; Core structural captures
    (class_declaration) @class.root
    (interface_declaration) @interface.root
    (enum_declaration) @enum.root
    (record_declaration) @record.root
    (method_declaration) @method.root
    (constructor_declaration) @constructor.root
    (field_declaration) @field.root
    ; Import and package
    (import_declaration) @import.root
    (package_declaration) @package.root
    ; Control flow captures
    (if_statement) @if.root
    (for_statement) @for.root
    (while_statement) @while.root
    (do_statement) @do.root
    (return_statement) @return.root
    (throw_statement) @throw.root
    (try_statement) @try.root
    (switch_statement) @switch.root
"#;