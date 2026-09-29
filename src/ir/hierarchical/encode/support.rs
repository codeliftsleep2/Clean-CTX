use super::*;

pub(super) fn class_location(
    class_locations: &HashMap<ClassId, usize>,
    raw_class: &str,
    operation: &'static str,
    instruction: usize,
) -> Result<usize, HierarchicalProjectionError> {
    class_locations
        .get(&ClassId::from_serialized(raw_class))
        .copied()
        .ok_or_else(|| HierarchicalProjectionError::UnresolvedIdentity {
            operation,
            expected: ProjectionIdentityKind::Class,
            id: raw_class.to_owned(),
            instruction,
        })
}

pub(super) fn method_location(
    method_locations: &HashMap<MethodId, MethodLocation>,
    raw_method: &str,
    operation: &'static str,
    instruction: usize,
) -> Result<MethodLocation, HierarchicalProjectionError> {
    method_locations
        .get(&MethodId::from_serialized(raw_method))
        .copied()
        .ok_or_else(|| HierarchicalProjectionError::UnresolvedIdentity {
            operation,
            expected: ProjectionIdentityKind::Method,
            id: raw_method.to_owned(),
            instruction,
        })
}

#[derive(Clone, Copy)]
pub(super) enum MethodLocation {
    Class(usize, usize),
    Interface(usize, usize),
}

#[derive(Clone, Copy)]
pub(super) enum FieldLocation {
    Class(usize, usize),
    Interface(usize, usize),
}

pub(super) fn method_mut<'a>(
    classes: &'a mut [ClassNode],
    interfaces: &'a mut [InterfaceNode],
    locations: &HashMap<MethodId, MethodLocation>,
    raw: &str,
    operation: &'static str,
    instruction: usize,
) -> Result<&'a mut MethodNode, HierarchicalProjectionError> {
    match method_location(locations, raw, operation, instruction)? {
        MethodLocation::Class(owner, member) => Ok(&mut classes[owner].methods[member]),
        MethodLocation::Interface(owner, member) => Ok(&mut interfaces[owner].methods[member]),
    }
}

pub(super) fn field_mut<'a>(
    classes: &'a mut [ClassNode],
    interfaces: &'a mut [InterfaceNode],
    location: FieldLocation,
) -> &'a mut FieldNode {
    match location {
        FieldLocation::Class(owner, member) => &mut classes[owner].fields[member],
        FieldLocation::Interface(owner, member) => &mut interfaces[owner].fields[member],
    }
}

pub(super) fn interface_location(
    locations: &HashMap<InterfaceId, usize>,
    raw: &str,
    operation: &'static str,
    instruction: usize,
) -> Result<usize, HierarchicalProjectionError> {
    locations
        .get(&InterfaceId::from_serialized(raw))
        .copied()
        .ok_or_else(|| HierarchicalProjectionError::UnresolvedIdentity {
            operation,
            expected: ProjectionIdentityKind::Interface,
            id: raw.to_owned(),
            instruction,
        })
}

pub(super) fn push_method(class: &mut ClassNode, method_id: &MethodId, name: String) -> usize {
    class.methods.push(new_method(method_id, name));
    class.methods.len() - 1
}

fn new_method(method_id: &MethodId, name: String) -> MethodNode {
    MethodNode {
        id: method_id.as_str().to_owned(),
        name,
        params: Vec::new(),
        return_type: None,
        modifiers: Vec::new(),
        control_summaries: Vec::new(),
        pattern_facts: Vec::new(),
        flags: Vec::new(),
        patterns: Vec::new(),
        body: None,
        body_start: None,
        body_end: None,
        control_flow: Vec::new(),
        data_flow: Vec::new(),
        side_effect: Vec::new(),
        execution_context: Vec::new(),
    }
}

pub(super) fn push_field(class: &mut ClassNode, field_id: &FieldId, name: String) -> usize {
    class.fields.push(FieldNode {
        id: field_id.as_str().to_owned(),
        name,
        field_type: None,
    });
    class.fields.len() - 1
}

pub(super) fn push_interface_method(
    interface: &mut InterfaceNode,
    method_id: &MethodId,
    name: String,
) -> usize {
    interface.methods.push(new_method(method_id, name));
    interface.methods.len() - 1
}

pub(super) fn push_interface_field(
    interface: &mut InterfaceNode,
    field_id: &FieldId,
    name: String,
) -> usize {
    interface.fields.push(FieldNode {
        id: field_id.as_str().to_owned(),
        name,
        field_type: None,
    });
    interface.fields.len() - 1
}
