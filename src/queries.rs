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
    ; --- TypeScript class property captures ---
    ; Class properties are distinct AST nodes from interface property
    ; signatures. Without these captures, adding a property to a class
    ; produced a false negative in diff_commits. F-01 diff audit.
    ; NOTE: `public_field_definition` is a valid node type in
    ; tree-sitter-typescript v0.23. `property_definition` and
    ; `method_signature` were NOT — including them caused the entire
    ; TS query to fail to compile, which broke the diff path for every
    ; TypeScript file.
    (public_field_definition) @field.root
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
    ; Audit fix: capture enum variant members so the diff snapshot can
    ; detect additions/removals of enum values. Without this capture, enum
    ; variant changes are a false negative in diff_commits.
    (enum_member_declaration) @field.root
    (constructor_declaration) @constructor.root
    ; --- C# property/event/indexer/operator captures ---
    ; Properties are a distinct AST node from fields in C#. Without this
    ; capture, adding a property to a class produced a false negative in
    ; diff_commits (the class appeared unchanged). F-01 diff audit.
    (property_declaration) @field.root
    (event_declaration) @field.root
    (event_field_declaration) @field.root
    (indexer_declaration) @field.root
    (operator_declaration) @field.root
    (destructor_declaration) @field.root
    (conversion_operator_declaration) @field.root
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

// Generic invocation captures (native call facts) for C#.
//
// These patterns are deliberately NOT part of `CS_QUERY`: the compression and
// diff paths (`compress_file`, `build_snapshot`) consume `CS_QUERY` captures
// positionally and would otherwise gain unknown capture names. The IR
// compilation path concatenates this query onto its base query and walks both
// in ONE tree-sitter parse (see `crate::ir::calls`).
//
// Structure, not text: each invocation form binds the NAME node that is
// written at the call site (`@call.callee`) and, in the second pattern of each
// pair, every individual `argument` node (`@call.argument`). The explicit
// argument count is therefore the number of observed argument nodes — never a
// `split(',')`, regex, or line scan — so nested commas inside generic
// arguments, object creations, lambdas, and array initializers can never
// inflate it.
//
// Both patterns of a pair are required: the argument-less pattern enumerates
// invocations with ZERO arguments (`Foo()`), and the argument pattern
// enumerates their arity. `match_index` groups the captures of one query
// match so the producer can associate each argument with its callee.
//
// `function:` is the invocation's function node and `name:` is a member
// access's final name node, so `items.OrderBy(a, b)` yields the callee
// `OrderBy` with argc 2 — the receiver is never part of the argument list.
// Unsupported function shapes (conditional access `a?.Foo()`, parenthesized
// and cast expressions) are deliberately not matched: an unmatched form
// produces no fact rather than a guessed one.
pub const CS_CALL_QUERY: &str = r#"
    ; --- Generic invocation captures (native call facts) ---
    (invocation_expression
        function: (identifier) @call.callee
        arguments: (argument_list))
    (invocation_expression
        function: (identifier) @call.callee
        arguments: (argument_list (argument) @call.argument))
    (invocation_expression
        function: (member_access_expression name: (identifier) @call.callee)
        arguments: (argument_list))
    (invocation_expression
        function: (member_access_expression name: (identifier) @call.callee)
        arguments: (argument_list (argument) @call.argument))
    (invocation_expression
        function: (generic_name (identifier) @call.callee)
        arguments: (argument_list))
    (invocation_expression
        function: (generic_name (identifier) @call.callee)
        arguments: (argument_list (argument) @call.argument))
    (invocation_expression
        function: (member_access_expression name: (generic_name (identifier) @call.callee))
        arguments: (argument_list))
    (invocation_expression
        function: (member_access_expression name: (generic_name (identifier) @call.callee))
        arguments: (argument_list (argument) @call.argument))
"#;

// Generic invocation captures (native call facts) for TypeScript.
//
// Exactly the `CS_CALL_QUERY` contract, expressed in TypeScript grammar:
// deliberately NOT part of `TS_QUERY` (so the compression and diff paths, which
// consume `TS_QUERY` captures positionally, can never observe a call capture),
// and structural — the callee NAME node plus one capture per explicitly written
// argument — so the observed arity is never derived from `split(',')`, a regex,
// a line scan, or a second parse.
//
// TypeScript invocation syntax maps onto the existing normalized fact:
//   * `foo(...)`      -> callee `foo`   (identifier)
//   * `this.foo(...)` -> callee `foo`   (member property; the receiver is
//                        never an argument and never enters the count)
//   * `foo<T>(...)`   -> callee `foo`   (`type_arguments` is a sibling field of
//                        the call, never part of the name)
//   * `foo(...args)`  -> one explicitly written argument (spread element)
//
// Both patterns of a pair are required: the argument-less pattern enumerates
// invocations with ZERO arguments (`foo()`), and the argument patterns
// enumerate their arity. `match_index` groups the captures of one query match
// so the producer can associate each argument with its callee.
//
// A spread element binds `@call.spread`, not `@call.argument`: it is still ONE
// written argument, but it also expands at run time, so the invocation's
// written count is no longer an exact arity. The producer records it as one
// argument AND qualifies the fact (`has_spread`), which is what keeps
// `foo(a)` and `foo(...args)` distinguishable even though both write one
// argument node.
//
// Unmatched forms emit NO fact rather than a guessed one, and they belong to
// two clearly different classes:
//
// SEMANTIC / MODEL BOUNDARIES — the current normalized fact cannot express
// them (not a parsing gap):
//   * `new Foo(...)` is a `new_expression`: object creation, not an invocation
//     of a named method. The callee would be the constructed TYPE, and the
//     `Calls` model is method-name-level (`builtin` / `Method` entities), so
//     construction is out of scope rather than unmatched.
//   * tagged template calls (`tag` + backtick) carry a `template_string` in
//     their `arguments` field — a substitution tuple, not an argument list.
//     The normalized fact counts explicitly written argument nodes in an
//     `arguments` list, which such a call does not have.
//   * an invocation with no precisely identified callable owner (a class
//     property initializer, a top-level statement): the caller must be a real
//     callable declaration id, so ownership is never guessed.
//
// CURRENT PRODUCER COVERAGE GAPS — recoverable later; these are NOT claims
// that the forms are unrepresentable:
//   * computed member calls with a non-literal property: `foo[bar]()` is a
//     `subscript_expression` and has no statically written callee name. The
//     literal form `foo["save"]()` IS structurally recoverable — it is a
//     coverage gap of these patterns, not a property of the model.
//   * `#private` member calls: `this.#save()` names its callee precisely (a
//     `private_property_identifier`); the patterns above simply do not bind
//     that node kind yet.
//   * arrow-function bodies: `const f = () => save();` emits no fact because
//     no callable declaration is captured for the arrow function, so the
//     invocation has no stable `builtin` / `Method` caller identity. The exact
//     limitation is caller identity, not arrow-function syntax.
pub const TS_CALL_QUERY: &str = r#"
    ; --- Generic invocation captures (native call facts) ---
    (call_expression
        function: (identifier) @call.callee
        arguments: (arguments))
    (call_expression
        function: (identifier) @call.callee
        arguments: (arguments (expression) @call.argument))
    (call_expression
        function: (identifier) @call.callee
        arguments: (arguments (spread_element) @call.spread))
    (call_expression
        function: (member_expression property: (property_identifier) @call.callee)
        arguments: (arguments))
    (call_expression
        function: (member_expression property: (property_identifier) @call.callee)
        arguments: (arguments (expression) @call.argument))
    (call_expression
        function: (member_expression property: (property_identifier) @call.callee)
        arguments: (arguments (spread_element) @call.spread))
"#;

// Generic invocation captures (native call facts) for Java.
//
// Exactly the `CS_CALL_QUERY` contract, expressed in Java grammar: deliberately
// NOT part of `JAVA_QUERY` (so the compression and diff paths can never observe
// a call capture), and structural — the callee NAME node plus one capture per
// explicitly written argument — so the observed arity is never derived from
// `split(',')`, a regex, a line scan, or a second parse.
//
// Java's `method_invocation` carries the callee name in its `name` field for
// BOTH receiver-less (`foo(...)`) and receiver-ful (`obj.foo(...)`,
// `this.foo(...)`, `super.foo(...)`) forms, so one pair of patterns covers
// every invocation shape. `type_arguments` is a sibling field, so `foo<T>(...)`
// yields the callee `foo`.
//
// Unmatched forms emit NO fact rather than a guessed one, and they belong to
// two clearly different classes:
//
// SEMANTIC / MODEL BOUNDARIES — the current normalized fact cannot express
// them (not a parsing gap):
//   * `new Foo(...)` is an `object_creation_expression`: object creation is not
//     an invocation of a named method, and the `Calls` model is method-name-
//     level (`builtin` / `Method` entities), so construction is out of scope.
//   * `super(...)` / `this(...)` is an `explicit_constructor_invocation`, which
//     has no `name` field at all — it is a constructor invocation, not a named
//     method call.
//   * a method reference (`Foo::bar`) is a method VALUE, never an invocation.
//   * an invocation with no precisely identified callable owner (a field
//     initializer, a static initializer): the caller must be a real callable
//     declaration id, so ownership is never guessed.
//
// CURRENT PRODUCER COVERAGE GAPS: none inside `method_invocation` — the callee
// is always the required `name` identifier, so the pair of patterns above
// covers every invocation shape (receiver-less, `this.`, `super.`, `obj.`, and
// explicit type-argument forms alike).
pub const JAVA_CALL_QUERY: &str = r#"
    ; --- Generic invocation captures (native call facts) ---
    (method_invocation
        name: (identifier) @call.callee
        arguments: (argument_list))
    (method_invocation
        name: (identifier) @call.callee
        arguments: (argument_list (expression) @call.argument))
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
    ; Enum variants are distinct AST nodes. Without this capture, adding
    ; or removing a variant produced a false negative in diff_commits.
    ; F-01 diff audit.
    (enum_variant) @field.root
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
//   switch_expression
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
    ; Audit fix: tree-sitter-java uses `switch_expression` (not
    ; `switch_statement`) since Java 14 unified switch expression syntax.
    (switch_expression) @switch.root
"#;
