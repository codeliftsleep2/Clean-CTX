use crate::ir::opcodes::{CoreOp, PatternFact};

pub(super) fn is_pattern_annotation(op: &CoreOp) -> bool {
    matches!(op, CoreOp::PatternFacts(..) | CoreOp::Flags(..))
}

pub(super) fn pattern_annotation_owner(op: &CoreOp) -> Option<&str> {
    match op {
        CoreOp::PatternFacts(method, _) | CoreOp::Flags(method, _) => Some(method),
        _ => None,
    }
}

pub(super) fn is_override_annotation(op: &CoreOp, method_id: &str) -> bool {
    match op {
        CoreOp::PatternFacts(method, facts) => {
            method == method_id && facts.contains(&PatternFact::Override)
        }
        CoreOp::Flags(method, values) => {
            method == method_id && values.iter().any(|value| value == "OVERRIDE")
        }
        _ => false,
    }
}

fn op_references_method(op: &CoreOp, method_id: &str) -> bool {
    match op {
        CoreOp::Param(mid, _, _, _)
        | CoreOp::Return(mid, _)
        | CoreOp::PatternFacts(mid, _)
        | CoreOp::Flags(mid, _)
        | CoreOp::MethodModifiers(mid, _)
        | CoreOp::ControlSummary(mid, _)
        | CoreOp::DataFlow(mid, _, _)
        | CoreOp::SideEffect(mid, _)
        | CoreOp::ExecutionContext(mid, _)
        | CoreOp::ControlFlow(mid, _, _)
        | CoreOp::Body(mid, _, _, _) => mid == method_id,
        CoreOp::Call(caller, _, _, _) => caller == method_id,
        _ => false,
    }
}

pub(super) fn trailing_region_references_call(
    slice: &[CoreOp],
    offset: usize,
    method_id: &str,
) -> bool {
    let mut index = offset;
    while index < slice.len() {
        match &slice[index] {
            CoreOp::Call(caller, _, _, _) if caller == method_id => return true,
            op if op_references_method(op, method_id) => index += 1,
            _ => break,
        }
    }
    false
}

pub(super) fn count_trailing_annotations(
    slice: &[CoreOp],
    offset: usize,
    method_id: &str,
) -> usize {
    let mut count = 0;
    while offset + count < slice.len() {
        match &slice[offset + count] {
            CoreOp::PatternFacts(mid, _)
            | CoreOp::Flags(mid, _)
            | CoreOp::MethodModifiers(mid, _)
            | CoreOp::ControlSummary(mid, _)
                if mid == method_id =>
            {
                count += 1
            }
            _ => break,
        }
    }
    count
}

fn op_is_unrepresentable_method_ref(op: &CoreOp, method_id: &str) -> bool {
    match op {
        CoreOp::DataFlow(mid, _, _)
        | CoreOp::SideEffect(mid, _)
        | CoreOp::ExecutionContext(mid, _)
        | CoreOp::ControlFlow(mid, _, _)
        | CoreOp::Body(mid, _, _, _) => mid == method_id,
        CoreOp::Call(caller, _, _, _) => caller == method_id,
        _ => false,
    }
}

pub(super) fn trailing_region_references_method(
    slice: &[CoreOp],
    offset: usize,
    method_id: &str,
) -> bool {
    let mut index = offset;
    while index < slice.len() {
        match &slice[index] {
            CoreOp::PatternFacts(mid, _)
            | CoreOp::Flags(mid, _)
            | CoreOp::MethodModifiers(mid, _)
            | CoreOp::ControlSummary(mid, _)
                if mid == method_id =>
            {
                index += 1
            }
            _ => break,
        }
    }
    slice
        .get(index)
        .is_some_and(|op| op_is_unrepresentable_method_ref(op, method_id))
}
