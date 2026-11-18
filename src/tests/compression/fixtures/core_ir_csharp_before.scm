
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
    ; --- Private source-bound invocation evidence (same parse) ---
    (invocation_expression) @invoke.root
    (invocation_expression function: (_) @invoke.function) @invoke.root
    (invocation_expression function: (identifier) @invoke.terminal) @invoke.root
    (invocation_expression function: (generic_name
        (identifier) @invoke.terminal
        (type_argument_list (type) @invoke.type_argument))) @invoke.root
    (invocation_expression function: (generic_name
        (identifier) @invoke.terminal)) @invoke.root
    (invocation_expression function: (member_access_expression
        expression: (_) @invoke.receiver
        name: (identifier) @invoke.terminal)) @invoke.root
    (invocation_expression function: (member_access_expression
        expression: (_) @invoke.receiver
        name: (generic_name (identifier) @invoke.terminal))) @invoke.root
    (invocation_expression function: (member_access_expression
        name: (generic_name (type_argument_list (type) @invoke.type_argument)))) @invoke.root
    (invocation_expression arguments: (argument_list
        (argument name: (identifier)? @invoke.argument_name) @invoke.argument)) @invoke.root
    (method_declaration name: (identifier) @invoke.declaration_name)
        @invoke.declaration @invoke.declaration.callable
    (constructor_declaration name: (identifier) @invoke.declaration_name)
        @invoke.declaration @invoke.declaration.callable
    (local_function_statement name: (identifier) @invoke.declaration_name)
        @invoke.declaration @invoke.declaration.callable
    (local_declaration_statement (variable_declaration
        (variable_declarator name: (identifier) @invoke.declaration_name
            "=" (_)) @invoke.declaration @invoke.declaration.variable))
    (field_declaration (variable_declaration
        (variable_declarator name: (identifier) @invoke.declaration_name
            "=" (_)) @invoke.declaration @invoke.declaration.field))
    (property_declaration name: (identifier) @invoke.declaration_name
        "=" value: (_)) @invoke.declaration @invoke.declaration.property
    (property_declaration name: (identifier) @invoke.declaration_name
        (arrow_expression_clause)) @invoke.declaration @invoke.declaration.property


    ; Namespace scopes are captured structurally so semantic resolution never
    ; reconstructs lexical ownership from strings.
    (namespace_declaration) @csharp.namespace_root
    (file_scoped_namespace_declaration) @csharp.namespace_root
    (namespace_declaration name: (_) @csharp.namespace_name)
    (file_scoped_namespace_declaration name: (_) @csharp.namespace_name)
    ; Declaration identifiers are the single C# canonical/builtin name authority.
    (class_declaration name: (identifier) @csharp.class_name)
    (interface_declaration name: (identifier) @csharp.interface_name)
    (struct_declaration name: (identifier) @csharp.struct_name)
    (enum_declaration name: (identifier) @csharp.enum_name)
    (record_declaration name: (identifier) @csharp.record_name)
    (type_parameter_list) @csharp.type_parameters
    ; Direct declaration ownership is retained through the query match identity.
    (class_declaration
        (type_parameter_list
            (type_parameter) @csharp.class_type_parameter)) @csharp.generic_class
    ; Class/struct/interface base lists expose entries through the type supertype.
    (base_list (type) @csharp.base_type)
    ; record_base is aliased to base_list. Capture each direct named child
    ; without guessing which concrete _name form the grammar selected.
    (record_declaration
        (base_list
            (_) @csharp.record_base_candidate))
    ; A positional record may invoke its record base constructor. Capture only
    ; the typed name field. Projection suppresses the enclosing candidate by
    ; span, so constructor arguments never enter the written type.
    (record_declaration
        (base_list
            (primary_constructor_base_type
                type: (_) @csharp.base_type)))
    ; Attribute identity remains neutral. The R0 collector separates the
    ; written name from any argument payload without assigning framework meaning.
    (attribute) @csharp.attribute
    ; Declaration attributes are captured separately so resolution can attach
    ; them to their owning class without confusing method/property attributes.
    (class_declaration
        (attribute_list
            (attribute) @csharp.class_attribute))
    ; Method attributes retain their declaration owner so framework consumers
    ; can authorize action semantics without scanning written class text.
    (method_declaration
        (attribute_list
            (attribute) @csharp.method_attribute))
