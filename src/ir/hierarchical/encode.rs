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

use super::*;
use crate::ir::identity::{
    ClassId, FieldId, InterfaceId, MethodId, PatternTarget, pattern_target, validate_identity_graph,
};
use std::collections::HashMap;

mod support;
use support::*;

/// Convert a flat `CompiledIR` instruction stream into a `HierarchicalIR`.
///
/// The converter validates the migrated identity graph, then scans
/// instructions while collecting:
/// - DefClass → creates a ClassNode identified by ClassId
/// - DefMethod → creates a MethodNode under its declared ClassId owner
/// - DefField → creates a FieldNode under its declared ClassId owner
/// - Param → attached to its target MethodId after definition placement
/// - Return → attached to its target MethodId after definition placement
/// - FieldType → attached to its target FieldId after definition placement
/// - MethodModifiers → appended to its MethodId as typed occurrences
/// - ClassModifiers → appended to its ClassId as typed occurrences
/// - ControlSummary → appended to its MethodId as typed occurrences
/// - Flags → appended to its target MethodId as one preserved occurrence
/// - ClassFlags → appended to its target ClassId as one preserved occurrence
/// - Extends → set on the class named by its child ID
/// - Implements → added to the class named by its target ID
/// - Injects → appended to its target ClassId as one preserved occurrence
/// - DefInterface → creates a distinct InterfaceNode
/// - DefInterfaceMethod/Field → create members under InterfaceId owners
/// - Import → added to top-level imports
/// - TypeAlias → added to top-level type_aliases
/// - Call → appended to the top-level calls table (the caller is carried
///   explicitly, so no scope derivation is needed)
/// - Pattern → added to the class or method inferred from its current schema
pub fn ir_to_hierarchical(ir: &CompiledIR) -> HierarchicalIR {
    try_ir_to_hierarchical(ir).unwrap_or_else(|error| {
        panic!("hierarchical projection requires valid semantic identity: {error}")
    })
}

/// Checked projection from canonical IR to the hierarchical representation.
///
/// The migrated slices resolve class, method, field, signature, body, and
/// method-metadata relationships through typed stable identities. Their
/// attribution is independent of instruction order, and invalid identity
/// graphs fail before a partial hierarchy can be returned.
pub fn try_ir_to_hierarchical(
    ir: &CompiledIR,
) -> Result<HierarchicalIR, HierarchicalProjectionError> {
    let identities = validate_identity_graph(ir)?;
    let mut classes: Vec<ClassNode> = Vec::with_capacity(identities.classes.len());
    let mut interfaces: Vec<InterfaceNode> = Vec::with_capacity(identities.interfaces.len());
    let mut imports: Vec<Vec<String>> = Vec::new();
    let mut type_aliases: Vec<Vec<String>> = Vec::new();
    let mut calls: Vec<HierarchicalCall> = Vec::new();
    let mut class_locations: HashMap<ClassId, usize> =
        HashMap::with_capacity(identities.classes.len());
    let mut method_locations: HashMap<MethodId, MethodLocation> =
        HashMap::with_capacity(identities.methods.len());
    let mut field_locations: HashMap<FieldId, FieldLocation> =
        HashMap::with_capacity(identities.fields.len());
    let mut pending_methods: HashMap<ClassId, Vec<(MethodId, String)>> = HashMap::new();
    let mut pending_fields: HashMap<ClassId, Vec<(FieldId, String)>> = HashMap::new();
    let mut pending_interface_methods: HashMap<InterfaceId, Vec<(MethodId, String)>> =
        HashMap::new();
    let mut pending_interface_fields: HashMap<InterfaceId, Vec<(FieldId, String)>> = HashMap::new();
    let mut interface_locations: HashMap<InterfaceId, usize> = HashMap::new();

    for op in &ir.instructions {
        match op {
            CoreOp::DefClass(id, name) => {
                classes.push(ClassNode {
                    id: id.clone(),
                    name: name.clone(),
                    methods: Vec::new(),
                    fields: Vec::new(),
                    modifiers: Vec::new(),
                    class_flags: Vec::new(),
                    extends: None,
                    implements: Vec::new(),
                    injects: Vec::new(),
                    patterns: Vec::new(),
                    synthetic: false,
                });
                let class_id = ClassId::from_serialized(id);
                class_locations.insert(class_id.clone(), classes.len() - 1);
                if let Some(methods) = pending_methods.remove(&class_id) {
                    let class_idx = classes.len() - 1;
                    for (method_id, method_name) in methods {
                        let method_idx =
                            push_method(&mut classes[class_idx], &method_id, method_name);
                        method_locations
                            .insert(method_id, MethodLocation::Class(class_idx, method_idx));
                    }
                }
                if let Some(fields) = pending_fields.remove(&class_id) {
                    let class_idx = classes.len() - 1;
                    for (field_id, field_name) in fields {
                        let field_idx = push_field(&mut classes[class_idx], &field_id, field_name);
                        field_locations
                            .insert(field_id, FieldLocation::Class(class_idx, field_idx));
                    }
                }
            }

            CoreOp::DefMethod(cid, mid, name) => {
                let class_id = ClassId::from_serialized(cid);
                let method_id = MethodId::from_serialized(mid);
                if let Some(class_idx) = find_class_by_id(&classes, cid) {
                    let method_idx = push_method(&mut classes[class_idx], &method_id, name.clone());
                    method_locations
                        .insert(method_id, MethodLocation::Class(class_idx, method_idx));
                } else {
                    // The owner is valid but appears later. Preserve the
                    // declaration until its class node is materialized.
                    pending_methods
                        .entry(class_id)
                        .or_default()
                        .push((method_id, name.clone()));
                }
            }

            CoreOp::DefField(cid, fid, name) => {
                let class_id = ClassId::from_serialized(cid);
                let field_id = FieldId::from_serialized(fid);
                if let Some(class_idx) = find_class_by_id(&classes, cid) {
                    let field_idx = push_field(&mut classes[class_idx], &field_id, name.clone());
                    field_locations.insert(field_id, FieldLocation::Class(class_idx, field_idx));
                } else {
                    // The owner is valid but appears later. Preserve the
                    // declaration until its class node is materialized.
                    pending_fields
                        .entry(class_id)
                        .or_default()
                        .push((field_id, name.clone()));
                }
            }

            // Attached by stable identity after every definition is placed.
            CoreOp::Param(..)
            | CoreOp::Return(..)
            | CoreOp::FieldType(..)
            | CoreOp::MethodModifiers(..)
            | CoreOp::ClassModifiers(..)
            | CoreOp::InterfaceModifiers(..)
            | CoreOp::ControlSummary(..)
            | CoreOp::PatternFacts(..)
            | CoreOp::Flags(..)
            | CoreOp::ClassFlags(..)
            | CoreOp::Extends(..)
            | CoreOp::InterfaceExtends(..)
            | CoreOp::Implements(..)
            | CoreOp::Injects(..)
            | CoreOp::Body(..)
            | CoreOp::ControlFlow(..)
            | CoreOp::DataFlow(..)
            | CoreOp::SideEffect(..)
            | CoreOp::ExecutionContext(..)
            | CoreOp::Pattern(..) => {}

            CoreOp::DefInterface(id, name) => {
                interfaces.push(InterfaceNode {
                    id: id.clone(),
                    name: name.clone(),
                    methods: Vec::new(),
                    fields: Vec::new(),
                    modifiers: Vec::new(),
                    extends: Vec::new(),
                });
                let interface_id = InterfaceId::from_serialized(id);
                let interface_idx = interfaces.len() - 1;
                interface_locations.insert(interface_id.clone(), interface_idx);
                if let Some(methods) = pending_interface_methods.remove(&interface_id) {
                    for (method_id, name) in methods {
                        let method_idx =
                            push_interface_method(&mut interfaces[interface_idx], &method_id, name);
                        method_locations.insert(
                            method_id,
                            MethodLocation::Interface(interface_idx, method_idx),
                        );
                    }
                }
                if let Some(fields) = pending_interface_fields.remove(&interface_id) {
                    for (field_id, name) in fields {
                        let field_idx =
                            push_interface_field(&mut interfaces[interface_idx], &field_id, name);
                        field_locations
                            .insert(field_id, FieldLocation::Interface(interface_idx, field_idx));
                    }
                }
            }

            CoreOp::DefInterfaceMethod(iid, mid, name) => {
                let interface_id = InterfaceId::from_serialized(iid);
                let method_id = MethodId::from_serialized(mid);
                if let Some(interface_idx) = interface_locations.get(&interface_id).copied() {
                    let method_idx = push_interface_method(
                        &mut interfaces[interface_idx],
                        &method_id,
                        name.clone(),
                    );
                    method_locations.insert(
                        method_id,
                        MethodLocation::Interface(interface_idx, method_idx),
                    );
                } else {
                    pending_interface_methods
                        .entry(interface_id)
                        .or_default()
                        .push((method_id, name.clone()));
                }
            }

            CoreOp::DefInterfaceField(iid, fid, name) => {
                let interface_id = InterfaceId::from_serialized(iid);
                let field_id = FieldId::from_serialized(fid);
                if let Some(interface_idx) = interface_locations.get(&interface_id).copied() {
                    let field_idx = push_interface_field(
                        &mut interfaces[interface_idx],
                        &field_id,
                        name.clone(),
                    );
                    field_locations
                        .insert(field_id, FieldLocation::Interface(interface_idx, field_idx));
                } else {
                    pending_interface_fields
                        .entry(interface_id)
                        .or_default()
                        .push((field_id, name.clone()));
                }
            }

            CoreOp::Import(alias, module, named) => {
                imports.push(vec![alias.clone(), module.clone(), named.clone()]);
            }

            CoreOp::TypeAlias(alias, original) => {
                type_aliases.push(vec![alias.clone(), original.clone()]);
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
        }
    }

    debug_assert!(pending_methods.is_empty());
    debug_assert!(pending_fields.is_empty());
    debug_assert!(pending_interface_methods.is_empty());
    debug_assert!(pending_interface_fields.is_empty());

    for (instruction, op) in ir.instructions.iter().enumerate() {
        match op {
            CoreOp::Param(raw_method, parameter_id, ty, name) => {
                method_mut(
                    &mut classes,
                    &mut interfaces,
                    &method_locations,
                    raw_method,
                    "SIG",
                    instruction,
                )?
                .params
                .push(vec![parameter_id.clone(), ty.clone(), name.clone()]);
            }
            CoreOp::Return(raw_method, ty) => {
                method_mut(
                    &mut classes,
                    &mut interfaces,
                    &method_locations,
                    raw_method,
                    "RET",
                    instruction,
                )?
                .return_type = Some(ty.clone());
            }
            CoreOp::FieldType(raw_field, ty) => {
                let field_id = FieldId::from_serialized(raw_field);
                let location = field_locations.get(&field_id).copied().ok_or_else(|| {
                    HierarchicalProjectionError::UnresolvedIdentity {
                        operation: "FIELD_T",
                        expected: ProjectionIdentityKind::Field,
                        id: raw_field.clone(),
                        instruction,
                    }
                })?;
                field_mut(&mut classes, &mut interfaces, location).field_type = Some(ty.clone());
            }
            CoreOp::MethodModifiers(raw_method, modifiers) => {
                method_mut(
                    &mut classes,
                    &mut interfaces,
                    &method_locations,
                    raw_method,
                    "MOD_M",
                    instruction,
                )?
                .modifiers
                .push(modifiers.clone());
            }
            CoreOp::ClassModifiers(raw_class, modifiers) => {
                let class_idx = class_location(&class_locations, raw_class, "MOD_C", instruction)?;
                classes[class_idx].modifiers.push(modifiers.clone());
            }
            CoreOp::InterfaceModifiers(raw_interface, modifiers) => {
                let interface_idx =
                    interface_location(&interface_locations, raw_interface, "MOD_I", instruction)?;
                interfaces[interface_idx].modifiers.push(modifiers.clone());
            }
            CoreOp::ControlSummary(raw_method, summaries) => {
                method_mut(
                    &mut classes,
                    &mut interfaces,
                    &method_locations,
                    raw_method,
                    "CTRL_SUM",
                    instruction,
                )?
                .control_summaries
                .push(summaries.clone());
            }
            CoreOp::PatternFacts(raw_method, facts) => {
                method_mut(
                    &mut classes,
                    &mut interfaces,
                    &method_locations,
                    raw_method,
                    "PAT_FACT",
                    instruction,
                )?
                .pattern_facts
                .push(facts.clone());
            }
            CoreOp::Flags(raw_method, flags) => {
                method_mut(
                    &mut classes,
                    &mut interfaces,
                    &method_locations,
                    raw_method,
                    "FLAGS",
                    instruction,
                )?
                .flags
                .push(flags.clone());
            }
            CoreOp::Body(raw_method, text, start, end) => {
                let method = method_mut(
                    &mut classes,
                    &mut interfaces,
                    &method_locations,
                    raw_method,
                    "BODY",
                    instruction,
                )?;
                method.body = Some(text.clone());
                method.body_start = *start;
                method.body_end = *end;
            }
            CoreOp::ControlFlow(raw_method, kind, target) => {
                method_mut(
                    &mut classes,
                    &mut interfaces,
                    &method_locations,
                    raw_method,
                    "CTRL",
                    instruction,
                )?
                .control_flow
                .push(vec![kind.clone(), target.clone()]);
            }
            CoreOp::DataFlow(raw_method, direction, target) => {
                method_mut(
                    &mut classes,
                    &mut interfaces,
                    &method_locations,
                    raw_method,
                    "DATAFLOW",
                    instruction,
                )?
                .data_flow
                .push(vec![direction.clone(), target.clone()]);
            }
            CoreOp::SideEffect(raw_method, effect) => {
                method_mut(
                    &mut classes,
                    &mut interfaces,
                    &method_locations,
                    raw_method,
                    "EFFECT",
                    instruction,
                )?
                .side_effect
                .push(*effect);
            }
            CoreOp::ExecutionContext(raw_method, context) => {
                method_mut(
                    &mut classes,
                    &mut interfaces,
                    &method_locations,
                    raw_method,
                    "CTX",
                    instruction,
                )?
                .execution_context
                .push(*context);
            }
            CoreOp::ClassFlags(raw_class, flags) => {
                let class_idx =
                    class_location(&class_locations, raw_class, "FLAGS_C", instruction)?;
                classes[class_idx].class_flags.push(flags.clone());
            }
            CoreOp::Extends(raw_class, parent) => {
                let class_idx = class_location(&class_locations, raw_class, "EXT", instruction)?;
                classes[class_idx].extends = Some(parent.clone());
            }
            CoreOp::InterfaceExtends(raw_interface, parent) => {
                let interface_idx =
                    interface_location(&interface_locations, raw_interface, "EXT_I", instruction)?;
                interfaces[interface_idx].extends.push(parent.clone());
            }
            CoreOp::Implements(raw_class, interface) => {
                let class_idx = class_location(&class_locations, raw_class, "IMPL", instruction)?;
                classes[class_idx].implements.push(interface.clone());
            }
            CoreOp::Injects(raw_class, dependencies) => {
                let class_idx =
                    class_location(&class_locations, raw_class, "INJECTS", instruction)?;
                classes[class_idx].injects.push(dependencies.clone());
            }
            CoreOp::Pattern(name, args) => {
                let entry = PatternEntry {
                    name: name.clone(),
                    args: args.clone(),
                };
                match pattern_target(name, args, instruction)? {
                    PatternTarget::Class(class) => {
                        let class_idx = class_location(
                            &class_locations,
                            class.as_str(),
                            "PAT class",
                            instruction,
                        )?;
                        classes[class_idx].patterns.push(entry);
                    }
                    PatternTarget::Method { class, method } => {
                        let expected_class = class_location(
                            &class_locations,
                            class.as_str(),
                            "PAT class",
                            instruction,
                        )?;
                        let location = method_location(
                            &method_locations,
                            method.as_str(),
                            "PAT method",
                            instruction,
                        )?;
                        let MethodLocation::Class(class_idx, method_idx) = location else {
                            return Err(HierarchicalProjectionError::OwnerMismatch {
                                operation: "PAT",
                                id: method.as_str().to_owned(),
                                expected_owner: class.as_str().to_owned(),
                                instruction,
                            });
                        };
                        if class_idx != expected_class {
                            return Err(HierarchicalProjectionError::OwnerMismatch {
                                operation: "PAT",
                                id: method.as_str().to_owned(),
                                expected_owner: class.as_str().to_owned(),
                                instruction,
                            });
                        }
                        classes[class_idx].methods[method_idx].patterns.push(entry);
                    }
                }
            }
            _ => {}
        }
    }

    Ok(HierarchicalIR {
        classes,
        interfaces,
        imports,
        type_aliases,
        calls,
    })
}
