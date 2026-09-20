// src/ir/semantic_projection.rs
//
// Language-agnostic projection of IR facts onto the semantic-edge model.
//
// The WorkspaceIndex consumes `SemanticEdge` values (cross-file semantic
// facts). Framework meta-layers produce their own edges during Pass 3; the
// GENERIC facts below are shared by every language and are projected here, at
// the compilation boundary, so a new language producer needs no semantic
// projection of its own:
//
//   CoreOp::DefMethod        → registering occurrence for the callable entity
//   CoreOp::Call             → SemanticRelation::Calls (with call evidence)
//
// Identity model (unchanged, Model C): an entity is (domain, entity_type,
// name). Generic callables are `builtin` / `Method`; a call relationship is
// method-level (`Process --Calls--> Save`), never attributed to an enclosing
// class. Explicit argument count travels as edge EVIDENCE
// (`CallEvidence::explicit_arg_count`) and never enters an entity name, so no
// `Foo/2` or `Foo$arity2` semantic name can be produced.
//
// A callee that is never declared in the compiled workspace still appears as
// the OBJECT of a `Calls` edge (it is honestly unresolved); only the caller is
// required to exist in the IR (enforced by validator rule E011), which is why
// an unknown caller id projects no edge rather than a fabricated subject.

use std::collections::HashMap;

use super::opcodes::CoreOp;
use crate::layers::meta::semantic::{CallEvidence, EntityRef, SemanticEdge, SemanticRelation};

/// Domain of generic (non-framework) entities.
pub const BUILTIN_DOMAIN: &str = "builtin";
/// Entity type of a generic callable.
pub const METHOD_ENTITY_TYPE: &str = "Method";
/// Provenance layer recorded on generic semantic edges.
pub const BUILTIN_LAYER: &str = "builtin";

fn method_entity(name: &str, file: &str) -> EntityRef {
    EntityRef::new(BUILTIN_DOMAIN, METHOD_ENTITY_TYPE, name).with_file(file.to_string())
}

/// Map every declared callable id to its name.
fn declared_method_names(instructions: &[CoreOp]) -> HashMap<&str, &str> {
    instructions
        .iter()
        .filter_map(|op| match op {
            CoreOp::DefMethod(_class_id, method_id, name)
            | CoreOp::DefInterfaceMethod(_class_id, method_id, name)
                if !name.is_empty() =>
            {
                Some((method_id.as_str(), name.as_str()))
            }
            _ => None,
        })
        .collect()
}

/// Project every callable declaration into a registering occurrence.
///
/// The self-referential `Defines` shape is the established entity-registration
/// carrier: `WorkspaceIndex::add_edges` normalizes it into a registered entity
/// occurrence with file provenance and keeps it out of the relationship graph
/// (so it can never create a cycle or a dependency edge). Declarations are
/// projected, never fabricated from call sites: a callee with no declaration
/// in the compiled file gets no occurrence, while external callees still
/// remain reachable as `Calls` edge objects.
pub fn project_method_declarations(instructions: &[CoreOp], file: &str) -> Vec<SemanticEdge> {
    instructions
        .iter()
        .filter_map(|op| match op {
            CoreOp::DefMethod(_class_id, _method_id, name)
            | CoreOp::DefInterfaceMethod(_class_id, _method_id, name)
                if !name.is_empty() =>
            {
                Some(SemanticEdge {
                    relation: SemanticRelation::Defines,
                    subject: method_entity(name, file),
                    object: method_entity(name, file),
                    layer: BUILTIN_LAYER,
                    call_evidence: None,
                })
            }
            _ => None,
        })
        .collect()
}

/// Project every native call fact into a `Calls` semantic edge.
///
/// The subject is the caller callable's declared name (resolved from the IR by
/// id, never from source text); the object is the callee name exactly as
/// written at the call site; the explicit argument count AND its spread
/// qualifier are carried as `CallEvidence`, so both the written arity and the
/// fact that the count is not exact participate in edge-occurrence identity.
///
/// The projection is deliberately structural: it claims a callee NAME, an
/// observed written arity, and whether that arity is exact — never overload
/// resolution, receiver typing, or a resolved declaration identity.
pub fn project_calls(instructions: &[CoreOp], file: &str) -> Vec<SemanticEdge> {
    let callers = declared_method_names(instructions);
    instructions
        .iter()
        .filter_map(|op| match op {
            CoreOp::Call(caller_id, callee, explicit_arg_count, has_spread) => {
                let caller_name = *callers.get(caller_id.as_str())?;
                Some(SemanticEdge {
                    relation: SemanticRelation::Calls,
                    subject: method_entity(caller_name, file),
                    object: method_entity(callee, file),
                    layer: BUILTIN_LAYER,
                    call_evidence: Some(CallEvidence::new(*explicit_arg_count, *has_spread)),
                })
            }
            _ => None,
        })
        .collect()
}

/// Project the complete generic semantic surface of one compiled file:
/// callable declarations first (registration occurrences), then call facts.
///
/// Appending preserves the relative order of every edge already produced by
/// the meta-layer pass, so existing entity occurrence ordering is unchanged.
pub fn project_generic_facts(instructions: &[CoreOp], file: &str) -> Vec<SemanticEdge> {
    let mut edges = project_method_declarations(instructions, file);
    edges.extend(project_calls(instructions, file));
    edges
}

#[cfg(test)]
#[path = "../tests/ir/semantic_projection.rs"]
mod tests;
