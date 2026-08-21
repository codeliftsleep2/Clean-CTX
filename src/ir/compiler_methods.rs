// src/ir/compiler_methods.rs
//
// Forward alias resolution — the only method that remains after the
// PassPipeline migration. The equivalent method-IR and import-IR
// emission functions now live on PassContext in pipeline.rs.
//
// The following functions were migrated to PassContext in pipeline.rs:
//   - parse_method_sig      → PassContext::parse_method_sig()
//   - find_body_start       → find_body_start_in()
//   - extract_method_body   → extract_method_body()
//   - emit_method_ir        → PassContext::emit_method_ir()
//   - emit_import_ir        → PassContext::emit_import_ir()

use super::opcodes::CoreOp;

/// F-FULL-08: Post-process the IR stream to resolve forward-declared class
/// aliases. When class B extends class A, but A is defined later in the file,
/// the TypeScript layer emits `Extends("C2", "A")` where "A" is a raw class
/// name (not an alias ID). This function builds a mapping from class name →
/// alias ID from the `DefClass` ops in the stream, then rewrites any
/// `Extends`/`Implements` ops that reference a raw class name.
pub(super) fn resolve_forward_aliases(instructions: &mut [CoreOp]) {
    // First pass: build the class-name → alias-id mapping from DefClass ops.
    let mut name_to_alias: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    for op in instructions.iter() {
        if let CoreOp::DefClass(alias_id, name) = op {
            // name is the extracted class name like "FooService" (no modifiers).
            // Store the mapping so we can resolve Extends("C1", "Foo") → Extends("C1", "C2").
            if !name_to_alias.contains_key(name) {
                name_to_alias.insert(name.clone(), alias_id.clone());
            }
        }
    }

    // Second pass: rewrite Extends/Implements ops that reference raw class names.
    for op in instructions.iter_mut() {
        match op {
            CoreOp::Extends(_, target) => {
                // If target looks like a raw class name (not an alias ID starting with "C"),
                // try to resolve it to the alias ID.
                if !target.starts_with('C') {
                    if let Some(alias) = name_to_alias.get(target.as_str()) {
                        *target = alias.clone();
                    }
                }
            }
            CoreOp::Implements(_, target) if !target.starts_with('C') => {
                if let Some(alias) = name_to_alias.get(target.as_str()) {
                    *target = alias.clone();
                }
            }
            _ => {}
        }
    }
}
