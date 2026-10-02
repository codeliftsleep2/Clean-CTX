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
    // First pass: build distinct class/interface name maps. An implements
    // target is interface-owned and must never resolve through a class map.
    let mut name_to_alias: std::collections::HashMap<String, Option<String>> =
        std::collections::HashMap::new();
    let mut interface_name_to_alias: std::collections::HashMap<String, Option<String>> =
        std::collections::HashMap::new();
    let mut class_aliases = std::collections::HashSet::new();
    let mut interface_aliases = std::collections::HashSet::new();
    for op in instructions.iter() {
        match op {
            CoreOp::DefClass(alias_id, name) => {
                class_aliases.insert(alias_id.clone());
                name_to_alias
                    .entry(name.clone())
                    .and_modify(|alias| *alias = None)
                    .or_insert_with(|| Some(alias_id.clone()));
            }
            CoreOp::DefInterface(alias_id, name) => {
                interface_aliases.insert(alias_id.clone());
                interface_name_to_alias
                    .entry(name.clone())
                    .and_modify(|alias| *alias = None)
                    .or_insert_with(|| Some(alias_id.clone()));
            }
            _ => {}
        }
    }
    // Second pass: rewrite relationship targets that reference raw names.
    // `BaseTypeRef` is the C# uncertainty boundary: refine it only when the
    // complete same-file declaration set proves the target kind. Otherwise
    // retain the written target and its neutral opcode unchanged.
    for op in instructions.iter_mut() {
        if let CoreOp::BaseTypeRef(owner, written_type) = op.clone() {
            let class_alias = name_to_alias
                .get(written_type.as_str())
                .and_then(Option::as_ref);
            let interface_alias = interface_name_to_alias
                .get(written_type.as_str())
                .and_then(Option::as_ref);
            if let (Some(alias), None) = (class_alias, interface_alias) {
                *op = CoreOp::Extends(owner, alias.clone());
            } else if let (None, Some(alias)) = (class_alias, interface_alias) {
                *op = CoreOp::Implements(owner, alias.clone());
            }
            continue;
        }
        match op {
            CoreOp::Extends(_, target) => {
                if !class_aliases.contains(target.as_str()) {
                    if let Some(Some(alias)) = name_to_alias.get(target.as_str()) {
                        *target = alias.clone();
                    }
                }
            }
            CoreOp::Implements(_, target)
                if !interface_aliases.contains(target.as_str()) =>
            {
                if let Some(Some(alias)) = interface_name_to_alias.get(target.as_str()) {
                    *target = alias.clone();
                }
            }
            CoreOp::InterfaceExtends(_, target)
                if !interface_aliases.contains(target.as_str()) =>
            {
                if let Some(Some(alias)) = interface_name_to_alias.get(target.as_str()) {
                    *target = alias.clone();
                }
            }
            _ => {}
        }
    }
}
