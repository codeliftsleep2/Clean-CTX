// src/ir/hierarchical/decode.rs
//
// Hierarchical DECODING: `HierarchicalIR` -> flat `Vec<CoreOp>` (the inverse
// of `encode`).
//
// Split out of `src/ir/hierarchical.rs` (active-file size policy): the module
// had exceeded the 615-line ceiling. This is a pure relocation -- the code
// below is byte-for-byte the previous implementation, and `hierarchical`
// re-exports `hierarchical_to_ir` so every existing path keeps resolving.
//
// Child-module access: a private item of an ancestor module is visible in its
// descendants, so `use super::*` supplies the node types (`HierarchicalIR`,
// `ClassNode`, `MethodNode`, `FieldNode`, `PatternEntry`) and the shared
// `find_class_by_id` helper.

use super::*;

/// Convert a `HierarchicalIR` back into a flat `Vec<CoreOp>` instruction stream.
///
/// This is the inverse of `ir_to_hierarchical`. The resulting instruction
/// order is: classes emit DefClass (unless synthetic), then class flags/
/// extends/implements/injects, then fields, then methods with their
/// params/return/flags/patterns, then imports, then type aliases.
///
/// NOTE: The original interleaving of instructions across methods/fields is
/// not preserved — the hierarchical format groups all instructions for a
/// given scope together. This is semantically equivalent because CoreOp
/// semantics don't depend on instruction ordering across scopes.
pub fn hierarchical_to_ir(hir: &HierarchicalIR) -> Vec<CoreOp> {
    let mut instructions = Vec::new();

    for class in &hir.classes {
        // Skip synthetic classes — they represent orphans that had no DefClass
        // in the original stream. Their methods/fields are emitted directly.
        if !class.synthetic {
            instructions.push(CoreOp::DefClass(class.id.clone(), class.name.clone()));

            // Class-level flag occurrences
            for flags in &class.class_flags {
                instructions.push(CoreOp::ClassFlags(class.id.clone(), flags.clone()));
            }

            // Extends
            if let Some(parent) = &class.extends {
                instructions.push(CoreOp::Extends(class.id.clone(), parent.clone()));
            }

            // Implements
            for iid in &class.implements {
                instructions.push(CoreOp::Implements(class.id.clone(), iid.clone()));
            }

            // Injection occurrences
            for dependencies in &class.injects {
                instructions.push(CoreOp::Injects(class.id.clone(), dependencies.clone()));
            }
        }

        // Fields (emitted for both synthetic and non-synthetic classes)
        for field in &class.fields {
            instructions.push(CoreOp::DefField(
                class.id.clone(),
                field.id.clone(),
                field.name.clone(),
            ));
            if let Some(ft) = &field.field_type {
                instructions.push(CoreOp::FieldType(field.id.clone(), ft.clone()));
            }
        }

        // Methods (emitted for both synthetic and non-synthetic classes)
        for method in &class.methods {
            instructions.push(CoreOp::DefMethod(
                class.id.clone(),
                method.id.clone(),
                method.name.clone(),
            ));

            // Params
            for param in &method.params {
                if param.len() >= 3 {
                    instructions.push(CoreOp::Param(
                        method.id.clone(),
                        param[0].clone(),
                        param[1].clone(),
                        param[2].clone(),
                    ));
                }
            }

            // Return type
            if let Some(rt) = &method.return_type {
                instructions.push(CoreOp::Return(method.id.clone(), rt.clone()));
            }

            // Method flag occurrences
            for flags in &method.flags {
                instructions.push(CoreOp::Flags(method.id.clone(), flags.clone()));
            }

            // Verbatim body
            if let Some(body) = &method.body {
                instructions.push(CoreOp::Body(
                    method.id.clone(),
                    body.clone(),
                    method.body_start,
                    method.body_end,
                ));
            }

            // Control-flow metadata
            for cf in &method.control_flow {
                if cf.len() >= 2 {
                    instructions.push(CoreOp::ControlFlow(
                        method.id.clone(),
                        cf[0].clone(),
                        cf[1].clone(),
                    ));
                }
            }

            // Data-flow metadata
            for df in &method.data_flow {
                if df.len() >= 2 {
                    instructions.push(CoreOp::DataFlow(
                        method.id.clone(),
                        df[0].clone(),
                        df[1].clone(),
                    ));
                }
            }

            // Side-effect annotations
            for se in &method.side_effect {
                instructions.push(CoreOp::SideEffect(method.id.clone(), se.clone()));
            }

            // Execution context annotations
            for ec in &method.execution_context {
                instructions.push(CoreOp::ExecutionContext(method.id.clone(), ec.clone()));
            }

            // Method-level patterns (args stored as-is)
            for pat in &method.patterns {
                instructions.push(CoreOp::Pattern(pat.name.clone(), pat.args.clone()));
            }
        }

        // Class-level patterns (only for non-synthetic classes)
        if !class.synthetic {
            for pat in &class.patterns {
                instructions.push(CoreOp::Pattern(pat.name.clone(), pat.args.clone()));
            }
        }
    }

    // Imports
    for imp in &hir.imports {
        if imp.len() >= 3 {
            instructions.push(CoreOp::Import(
                imp[0].clone(),
                imp[1].clone(),
                imp[2].clone(),
            ));
        }
    }

    // Type aliases
    for ta in &hir.type_aliases {
        if ta.len() >= 2 {
            instructions.push(CoreOp::TypeAlias(ta[0].clone(), ta[1].clone()));
        }
    }

    // Structural invocations (native call graph). Flat table re-emitted at
    // the end: the caller is carried explicitly, so the position is
    // independent of class/method nesting order.
    for call in &hir.calls {
        instructions.push(CoreOp::Call(
            call.caller.clone(),
            call.callee.clone(),
            call.explicit_arg_count,
            call.has_spread,
        ));
    }

    instructions
}
