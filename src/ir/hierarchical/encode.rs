// src/ir/hierarchical/encode.rs
//
// Hierarchical ENCODING: flat `CompiledIR` instructions -> `HierarchicalIR`.
//
// Split out of `src/ir/hierarchical.rs` (active-file size policy): the module
// had exceeded the 615-line ceiling. The `hierarchical` module re-exports the
// projection entry points so every existing path keeps resolving.
//
// Child-module access: a private item of an ancestor module is visible in its
// descendants, so `use super::*` supplies the node types (`HierarchicalIR`,
// `HierarchicalCall`, `ClassNode`, `MethodNode`, `FieldNode`, `PatternEntry`)
// and the shared `find_class_by_id` helper.

use super::identity::{ClassId, MethodId, validate_first_slice};
use super::*;
use std::collections::HashMap;

/// Convert a flat `CompiledIR` instruction stream into a `HierarchicalIR`.
///
/// The converter validates the first-slice identity graph, then scans
/// instructions while collecting:
/// - DefClass → creates a ClassNode identified by ClassId
/// - DefMethod → creates a MethodNode under its declared ClassId owner
/// - DefField → creates a FieldNode inside current class
/// - Param → attached to its target MethodId after definition placement
/// - Return → attached to its target MethodId after definition placement
/// - FieldType → set as current field's field_type
/// - Flags → accumulated (stable union) into the current method's flags
/// - ClassFlags → set as current class's class_flags
/// - Extends → set as current class's extends
/// - Implements → added to current class's implements
/// - Injects → added to current class's injects
/// - DefInterface → creates a ClassNode with synthetic=false and name
/// - Import → added to top-level imports
/// - TypeAlias → added to top-level type_aliases
/// - Call → appended to the top-level calls table (the caller is carried
///   explicitly, so no scope derivation is needed)
/// - Pattern → added to current scope (class or method), storing args as-is
pub fn ir_to_hierarchical(ir: &CompiledIR) -> HierarchicalIR {
    try_ir_to_hierarchical(ir).unwrap_or_else(|error| {
        panic!("hierarchical projection requires valid semantic identity: {error}")
    })
}

/// Checked projection from canonical IR to the hierarchical representation.
///
/// The approved first slice resolves `DefClass`, `DefMethod`, `Param`, and
/// `Return` through typed stable identities. Their attribution is independent
/// of instruction order, and invalid identity graphs fail before a partial
/// hierarchy can be returned.
pub fn try_ir_to_hierarchical(
    ir: &CompiledIR,
) -> Result<HierarchicalIR, HierarchicalProjectionError> {
    let identities = validate_first_slice(ir)?;
    let mut classes: Vec<ClassNode> = Vec::with_capacity(identities.classes.len());
    let mut imports: Vec<Vec<String>> = Vec::new();
    let mut type_aliases: Vec<Vec<String>> = Vec::new();
    let mut calls: Vec<HierarchicalCall> = Vec::new();
    let mut method_locations: HashMap<MethodId, (usize, usize)> =
        HashMap::with_capacity(identities.methods.len());
    let mut pending_methods: HashMap<ClassId, Vec<(MethodId, String)>> = HashMap::new();

    // Track current scope
    let mut current_class_idx: Option<usize> = None;

    for op in &ir.instructions {
        match op {
            CoreOp::DefClass(id, name) => {
                classes.push(ClassNode {
                    id: id.clone(),
                    name: name.clone(),
                    methods: Vec::new(),
                    fields: Vec::new(),
                    class_flags: None,
                    extends: None,
                    implements: Vec::new(),
                    injects: Vec::new(),
                    patterns: Vec::new(),
                    synthetic: false,
                });
                current_class_idx = Some(classes.len() - 1);

                let class_id = ClassId::from_serialized(id);
                if let Some(methods) = pending_methods.remove(&class_id) {
                    let class_idx = classes.len() - 1;
                    for (method_id, method_name) in methods {
                        let method_idx =
                            push_method(&mut classes[class_idx], &method_id, method_name);
                        method_locations.insert(method_id, (class_idx, method_idx));
                    }
                }
            }

            CoreOp::DefMethod(cid, mid, name) => {
                let class_id = ClassId::from_serialized(cid);
                let method_id = MethodId::from_serialized(mid);
                if let Some(class_idx) = find_class_by_id(&classes, cid) {
                    let method_idx = push_method(&mut classes[class_idx], &method_id, name.clone());
                    method_locations.insert(method_id, (class_idx, method_idx));
                    current_class_idx = Some(class_idx);
                } else {
                    // The owner is valid but appears later. Preserve the
                    // declaration until its class node is materialized.
                    pending_methods
                        .entry(class_id)
                        .or_default()
                        .push((method_id, name.clone()));
                    current_class_idx = None;
                }
            }

            CoreOp::DefField(cid, fid, name) => {
                if let Some(class_idx) = find_class_by_id(&classes, cid) {
                    classes[class_idx].fields.push(FieldNode {
                        id: fid.clone(),
                        name: name.clone(),
                        field_type: None,
                    });
                    current_class_idx = Some(class_idx);
                } else {
                    // Field with no matching class — create synthetic class
                    classes.push(ClassNode {
                        id: cid.clone(),
                        name: format!("__synthetic_{}", cid),
                        methods: Vec::new(),
                        fields: vec![FieldNode {
                            id: fid.clone(),
                            name: name.clone(),
                            field_type: None,
                        }],
                        class_flags: None,
                        extends: None,
                        implements: Vec::new(),
                        injects: Vec::new(),
                        patterns: Vec::new(),
                        synthetic: true,
                    });
                    current_class_idx = Some(classes.len() - 1);
                }
            }

            // Attached by stable MethodId after every definition is placed.
            CoreOp::Param(..) | CoreOp::Return(..) => {}

            CoreOp::FieldType(fid, ty) => {
                if let Some(c_idx) = current_class_idx {
                    for fi in 0..classes[c_idx].fields.len() {
                        if classes[c_idx].fields[fi].id == *fid {
                            classes[c_idx].fields[fi].field_type = Some(ty.clone());
                            break;
                        }
                    }
                }
            }

            CoreOp::Flags(tid, flags) => {
                if let Some(c_idx) = current_class_idx {
                    for mi in 0..classes[c_idx].methods.len() {
                        if classes[c_idx].methods[mi].id == *tid {
                            // ACCUMULATE, never assign: `CoreOp::Flags` has
                            // multiple legitimate producers for one method id,
                            // so the flat stream legitimately carries more
                            // than one op here (see `accumulate_flags`).
                            accumulate_flags(
                                classes[c_idx].methods[mi]
                                    .flags
                                    .get_or_insert_with(Vec::new),
                                flags,
                            );
                            break;
                        }
                    }
                }
            }

            CoreOp::ClassFlags(cid, flags) => {
                if let Some(c_idx) = find_class_by_id(&classes, cid) {
                    classes[c_idx].class_flags = Some(flags.clone());
                    current_class_idx = Some(c_idx);
                }
            }

            CoreOp::Extends(child, parent) => {
                if let Some(c_idx) = find_class_by_id(&classes, child) {
                    classes[c_idx].extends = Some(parent.clone());
                    current_class_idx = Some(c_idx);
                }
            }

            CoreOp::Implements(cid, iid) => {
                if let Some(c_idx) = find_class_by_id(&classes, cid) {
                    classes[c_idx].implements.push(iid.clone());
                    current_class_idx = Some(c_idx);
                }
            }

            CoreOp::Injects(cid, deps) => {
                if let Some(c_idx) = find_class_by_id(&classes, cid) {
                    classes[c_idx].injects.extend(deps.clone());
                    current_class_idx = Some(c_idx);
                }
            }

            CoreOp::DefInterface(id, name) => {
                classes.push(ClassNode {
                    id: id.clone(),
                    name: name.clone(),
                    methods: Vec::new(),
                    fields: Vec::new(),
                    class_flags: None,
                    extends: None,
                    implements: Vec::new(),
                    injects: Vec::new(),
                    patterns: Vec::new(),
                    synthetic: false,
                });
                current_class_idx = Some(classes.len() - 1);
            }

            CoreOp::Import(alias, module, named) => {
                imports.push(vec![alias.clone(), module.clone(), named.clone()]);
            }

            CoreOp::TypeAlias(alias, original) => {
                type_aliases.push(vec![alias.clone(), original.clone()]);
            }

            // Edit Mode: verbatim method body
            CoreOp::Body(mid, text, start, end) => {
                if let Some(c_idx) = current_class_idx {
                    for mi in 0..classes[c_idx].methods.len() {
                        if classes[c_idx].methods[mi].id == *mid {
                            classes[c_idx].methods[mi].body = Some(text.clone());
                            // Span pairing invariant: producer guarantees
                            // both-or-neither, so assignment preserves it.
                            classes[c_idx].methods[mi].body_start = *start;
                            classes[c_idx].methods[mi].body_end = *end;
                            break;
                        }
                    }
                }
            }

            // R-43a: Execution semantics — stored as method-level metadata.
            CoreOp::ControlFlow(mid, kind, target) => {
                if let Some(c_idx) = current_class_idx {
                    for mi in 0..classes[c_idx].methods.len() {
                        if classes[c_idx].methods[mi].id == *mid {
                            classes[c_idx].methods[mi]
                                .control_flow
                                .push(vec![kind.clone(), target.clone()]);
                            break;
                        }
                    }
                }
            }

            CoreOp::DataFlow(mid, direction, target) => {
                if let Some(c_idx) = current_class_idx {
                    for mi in 0..classes[c_idx].methods.len() {
                        if classes[c_idx].methods[mi].id == *mid {
                            classes[c_idx].methods[mi]
                                .data_flow
                                .push(vec![direction.clone(), target.clone()]);
                            break;
                        }
                    }
                }
            }

            CoreOp::SideEffect(mid, effect_type) => {
                if let Some(c_idx) = current_class_idx {
                    for mi in 0..classes[c_idx].methods.len() {
                        if classes[c_idx].methods[mi].id == *mid {
                            classes[c_idx].methods[mi].side_effect = Some(effect_type.clone());
                            break;
                        }
                    }
                }
            }

            CoreOp::ExecutionContext(mid, context_type) => {
                if let Some(c_idx) = current_class_idx {
                    for mi in 0..classes[c_idx].methods.len() {
                        if classes[c_idx].methods[mi].id == *mid {
                            classes[c_idx].methods[mi].execution_context =
                                Some(context_type.clone());
                            break;
                        }
                    }
                }
            }

            // Structural invocations (native call graph). Flat table: the
            // caller is carried explicitly, so no class/method scope is
            // needed to place the fact.
            CoreOp::Call(caller, callee, argc, has_spread) => {
                calls.push(HierarchicalCall {
                    caller: caller.clone(),
                    callee: callee.clone(),
                    explicit_arg_count: *argc,
                    has_spread: *has_spread,
                });
            }

            CoreOp::Pattern(name, args) => {
                // Parse pattern args to find the correct parent by class/method ID.
                // PatternOp::to_tuple() format: [class_id, method_id?, ...args]
                // All method-level patterns have class_id at index 0 and method_id at index 1.
                // Method IDs always start with "M" (generated by IRCompiler::next_id("M")).
                // Class-level patterns have class_id at index 0 and no method_id.
                // If args[1] does not start with "M", it's a class-level pattern argument.
                let class_id = args.first().cloned();
                let method_id = args.get(1).filter(|s| s.starts_with('M')).cloned();

                if let Some(cid) = class_id {
                    if let Some(c_idx) = find_class_by_id(&classes, &cid) {
                        if let Some(mid) = method_id {
                            // Method-level pattern: find the method by ID within this class
                            if let Some(m_idx) =
                                classes[c_idx].methods.iter().position(|m| m.id == mid)
                            {
                                classes[c_idx].methods[m_idx].patterns.push(PatternEntry {
                                    name: name.clone(),
                                    args: args.clone(),
                                });
                                current_class_idx = Some(c_idx);
                            }
                        } else {
                            // Class-level pattern (no method_id)
                            classes[c_idx].patterns.push(PatternEntry {
                                name: name.clone(),
                                args: args.clone(),
                            });
                            current_class_idx = Some(c_idx);
                        }
                    }
                }
            }
        }
    }

    debug_assert!(pending_methods.is_empty());

    for (instruction, op) in ir.instructions.iter().enumerate() {
        let (raw_method, operation) = match op {
            CoreOp::Param(method, ..) => (method, "SIG"),
            CoreOp::Return(method, _) => (method, "RET"),
            _ => continue,
        };
        let method_id = MethodId::from_serialized(raw_method);
        let (class_idx, method_idx) =
            method_locations.get(&method_id).copied().ok_or_else(|| {
                HierarchicalProjectionError::UnresolvedIdentity {
                    operation,
                    expected: ProjectionIdentityKind::Method,
                    id: raw_method.clone(),
                    instruction,
                }
            })?;

        match op {
            CoreOp::Param(_, parameter_id, ty, name) => {
                classes[class_idx].methods[method_idx].params.push(vec![
                    parameter_id.clone(),
                    ty.clone(),
                    name.clone(),
                ]);
            }
            CoreOp::Return(_, ty) => {
                classes[class_idx].methods[method_idx].return_type = Some(ty.clone());
            }
            _ => unreachable!("only Param and Return reach stable attachment"),
        }
    }

    Ok(HierarchicalIR {
        classes,
        imports,
        type_aliases,
        calls,
    })
}

fn push_method(class: &mut ClassNode, method_id: &MethodId, name: String) -> usize {
    class.methods.push(MethodNode {
        id: method_id.as_str().to_owned(),
        name,
        params: Vec::new(),
        return_type: None,
        flags: None,
        patterns: Vec::new(),
        body: None,
        body_start: None,
        body_end: None,
        control_flow: Vec::new(),
        data_flow: Vec::new(),
        side_effect: None,
        execution_context: None,
    });
    class.methods.len() - 1
}

/// Merge one `CoreOp::Flags` op's values into a method's accumulated flags.
///
/// `CoreOp::Flags` has multiple legitimate producers for the SAME method id:
///
/// * the language layer's declaration/modifier flags (`STATIC`, `ASYNC`,
///   `PRIVATE`, `PROTECTED`, `ABSTRACT`, `EXPORT`, …), emitted while the
///   declaration capture is dispatched, and
/// * the core pipeline's accumulated control-flow flags (`IF`, `LOOP`, `RET`,
///   `THROW`), collected by `PassContext::current_method_flags` and flushed
///   into one op by `flush_method_flags` when the declaration ends
///   (`src/ir/pipeline.rs`).
///
/// The flat stream therefore carries more than one `Flags` op for one method,
/// while the hierarchical representation has exactly one
/// [`MethodNode::flags`] field. The projection must ACCUMULATE: assigning the
/// last op discarded the other producer's entire family, so a method whose
/// body contains control flow rendered `fl:RET` with none of its declaration
/// modifiers.
///
/// Semantics: **first-seen stable order + deduplication**. A value keeps the
/// position where the flat stream first mentioned it, and a value already
/// present is not appended again — the result is deterministic and never
/// depends on an unordered set's iteration order.
///
/// Scope note: this preserves INFORMATION (every value the flat stream named
/// is present), not flat-op multiplicity. A later hierarchical → flat decode
/// re-emits the projected form — one combined `Flags` op — which is the
/// established `MethodNode::flags` semantics.
fn accumulate_flags(target: &mut Vec<String>, incoming: &[String]) {
    for flag in incoming {
        if !target.iter().any(|existing| existing == flag) {
            target.push(flag.clone());
        }
    }
}
