// src/ir/hierarchical/encode.rs
//
// Hierarchical ENCODING: flat `CompiledIR` instructions -> `HierarchicalIR`.
//
// Split out of `src/ir/hierarchical.rs` (active-file size policy): the module
// had exceeded the 615-line ceiling. This is a pure relocation -- the code
// below is byte-for-byte the previous implementation, and `hierarchical`
// re-exports `ir_to_hierarchical` so every existing path keeps resolving.
//
// Child-module access: a private item of an ancestor module is visible in its
// descendants, so `use super::*` supplies the node types (`HierarchicalIR`,
// `HierarchicalCall`, `ClassNode`, `MethodNode`, `FieldNode`, `PatternEntry`)
// and the shared `find_class_by_id` helper.

use super::*;

/// Convert a flat `CompiledIR` instruction stream into a `HierarchicalIR`.
///
/// The converter scans instructions in order, collecting:
/// - DefClass → creates a ClassNode (subsequent ops scoped to this class)
/// - DefMethod → creates a MethodNode inside current class
/// - DefField → creates a FieldNode inside current class
/// - Param → added to current method's params
/// - Return → set as current method's return_type
/// - FieldType → set as current field's field_type
/// - Flags → set as current method's flags
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
    let mut classes: Vec<ClassNode> = Vec::new();
    let mut imports: Vec<Vec<String>> = Vec::new();
    let mut type_aliases: Vec<Vec<String>> = Vec::new();
    let mut calls: Vec<HierarchicalCall> = Vec::new();

    // Track current scope
    let mut current_class_idx: Option<usize> = None;
    let mut current_method_idx: Option<usize> = None;

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
                current_method_idx = None;
            }

            CoreOp::DefMethod(cid, mid, name) => {
                if let Some(class_idx) = find_class_by_id(&classes, cid) {
                    classes[class_idx].methods.push(MethodNode {
                        id: mid.clone(),
                        name: name.clone(),
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
                    current_class_idx = Some(class_idx);
                    current_method_idx = Some(classes[class_idx].methods.len() - 1);
                } else {
                    // Method with no matching class — create a synthetic class
                    classes.push(ClassNode {
                        id: cid.clone(),
                        name: format!("__synthetic_{}", cid),
                        methods: vec![MethodNode {
                            id: mid.clone(),
                            name: name.clone(),
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
                        }],
                        fields: Vec::new(),
                        class_flags: None,
                        extends: None,
                        implements: Vec::new(),
                        injects: Vec::new(),
                        patterns: Vec::new(),
                        synthetic: true,
                    });
                    current_class_idx = Some(classes.len() - 1);
                    current_method_idx = Some(0);
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

            CoreOp::Param(mid, pid, ty, name) => {
                if let (Some(c_idx), Some(m_idx)) = (current_class_idx, current_method_idx) {
                    if classes[c_idx].methods[m_idx].id == *mid {
                        classes[c_idx].methods[m_idx].params.push(vec![
                            pid.clone(),
                            ty.clone(),
                            name.clone(),
                        ]);
                    } else {
                        // Method ID mismatch — search
                        for mi in 0..classes[c_idx].methods.len() {
                            if classes[c_idx].methods[mi].id == *mid {
                                classes[c_idx].methods[mi].params.push(vec![
                                    pid.clone(),
                                    ty.clone(),
                                    name.clone(),
                                ]);
                                current_method_idx = Some(mi);
                                break;
                            }
                        }
                    }
                }
            }

            CoreOp::Return(mid, ty) => {
                if let Some(c_idx) = current_class_idx {
                    for mi in 0..classes[c_idx].methods.len() {
                        if classes[c_idx].methods[mi].id == *mid {
                            classes[c_idx].methods[mi].return_type = Some(ty.clone());
                            current_method_idx = Some(mi);
                            break;
                        }
                    }
                }
            }

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
                            classes[c_idx].methods[mi].flags = Some(flags.clone());
                            current_method_idx = Some(mi);
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
                current_method_idx = None;
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
                            current_method_idx = Some(mi);
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
                            current_method_idx = Some(mi);
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
                            current_method_idx = Some(mi);
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
                            current_method_idx = Some(mi);
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
                            current_method_idx = Some(mi);
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
                                current_method_idx = Some(m_idx);
                            }
                        } else {
                            // Class-level pattern (no method_id)
                            classes[c_idx].patterns.push(PatternEntry {
                                name: name.clone(),
                                args: args.clone(),
                            });
                            current_class_idx = Some(c_idx);
                            current_method_idx = None;
                        }
                    }
                }
            }
        }
    }

    HierarchicalIR {
        classes,
        imports,
        type_aliases,
        calls,
    }
}
