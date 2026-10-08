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
//   constructor CoreOp::Param → HasConstructorParameterType
//   CoreOp::Extends          → SemanticRelation::Extends (Class → Class)
//   CoreOp::Implements       → SemanticRelation::Implements (Class → Interface)
//   CoreOp::InterfaceExtends → SemanticRelation::Extends (Interface → Interface)
//   CoreOp::BaseTypeRef      → HasBaseType (Class → unresolved TypeRef)
//
// Identity model (unchanged, Model C): an entity is (domain, entity_type,
// name). Generic callables are `builtin` / `Method`; a call relationship is
// method-level (`Process --Calls--> Save`), never attributed to an enclosing
// class. Explicit argument count travels as edge EVIDENCE
// (`CallEvidence::explicit_arg_count`) and never enters an entity name, so no
// `Foo/2` or `Foo$arity2` semantic name can be produced.

/// Generation of the language-agnostic canonical-fact projection. Increment
/// when identical canonical facts can emit different generic semantic edges.
pub(crate) const GENERIC_SEMANTIC_PROJECTION_GENERATION: u32 = 1;

/// Generation of C#-specific generic semantic projection. This is separate
/// from the generic generation so a C#-only edge change does not invalidate
/// durable semantic snapshots for unrelated languages.
pub(crate) const CSHARP_SEMANTIC_PROJECTION_GENERATION: u32 = 1;
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
/// Entity type of a generic class declaration.
pub const CLASS_ENTITY_TYPE: &str = "Class";
/// Entity type of a generic interface/protocol declaration. Rust traits are
/// normalized to this role when they occur as an `Implements` target because
/// `CoreOp::Implements` intentionally carries no source-language kind.
pub const INTERFACE_ENTITY_TYPE: &str = "Interface";
/// Entity type of an unresolved written type reference.
pub const TYPE_REF_ENTITY_TYPE: &str = "TypeRef";
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

fn named_entity(entity_type: &'static str, name: &str, file: &str) -> EntityRef {
    EntityRef::new(BUILTIN_DOMAIN, entity_type, name).with_file(file.to_string())
}

/// Project canonical structural inheritance facts onto the workspace graph.
///
/// Canonical operands may be either internal aliases or unresolved written
/// names. Declared aliases are translated back to their source names; an
/// external target remains the written name and is not presented as resolved.
/// Interface inheritance intentionally uses the existing generic `Extends`
/// relation with typed Interface endpoints rather than adding a second public
/// relation that duplicates the endpoint types.
pub fn project_inheritance(instructions: &[CoreOp], file: &str) -> Vec<SemanticEdge> {
    let classes: HashMap<&str, &str> = instructions
        .iter()
        .filter_map(|op| match op {
            CoreOp::DefClass(id, name) if !name.is_empty() => Some((id.as_str(), name.as_str())),
            _ => None,
        })
        .collect();
    let interfaces: HashMap<&str, &str> = instructions
        .iter()
        .filter_map(|op| match op {
            CoreOp::DefInterface(id, name) if !name.is_empty() => {
                Some((id.as_str(), name.as_str()))
            }
            _ => None,
        })
        .collect();

    let class_name = |id: &str| classes.get(id).copied();
    let interface_name = |id: &str| interfaces.get(id).copied();

    instructions
        .iter()
        .filter_map(|op| {
            let (relation, subject_type, subject_name, object_type, object_name) = match op {
                CoreOp::Extends(owner, target) => (
                    SemanticRelation::Extends,
                    CLASS_ENTITY_TYPE,
                    class_name(owner)?,
                    CLASS_ENTITY_TYPE,
                    classes.get(target.as_str()).copied().unwrap_or(target),
                ),
                CoreOp::Implements(owner, target) => (
                    SemanticRelation::Implements,
                    CLASS_ENTITY_TYPE,
                    class_name(owner)?,
                    INTERFACE_ENTITY_TYPE,
                    interfaces
                        .get(target.as_str())
                        .or_else(|| classes.get(target.as_str()))
                        .copied()
                        .unwrap_or(target),
                ),
                CoreOp::InterfaceExtends(owner, target) => (
                    SemanticRelation::Extends,
                    INTERFACE_ENTITY_TYPE,
                    interface_name(owner)?,
                    INTERFACE_ENTITY_TYPE,
                    interfaces.get(target.as_str()).copied().unwrap_or(target),
                ),
                CoreOp::BaseTypeRef(owner, written_type) => (
                    SemanticRelation::HasBaseType,
                    CLASS_ENTITY_TYPE,
                    class_name(owner)?,
                    TYPE_REF_ENTITY_TYPE,
                    written_type.as_str(),
                ),
                _ => return None,
            };
            Some(SemanticEdge {
                relation,
                subject: named_entity(subject_type, subject_name, file),
                object: named_entity(object_type, object_name, file),
                layer: BUILTIN_LAYER,
                call_evidence: None,
            })
        })
        .collect()
}

/// Project constructor parameter types as class-level query relationships.
///
/// Constructor identity is derived only from canonical ownership: a
/// `DefMethod` is a constructor when its owning `DefClass` has the same name.
/// Every `Param` occurrence remains in canonical IR. The workspace index may
/// deduplicate repeated same-type parameters into the existential class-level
/// fact that the class has at least one constructor parameter of that type.
pub fn project_constructor_parameter_types(
    instructions: &[CoreOp],
    file: &str,
) -> Vec<SemanticEdge> {
    let classes: HashMap<&str, &str> = instructions
        .iter()
        .filter_map(|op| match op {
            CoreOp::DefClass(class_id, name) if !name.is_empty() => {
                Some((class_id.as_str(), name.as_str()))
            }
            _ => None,
        })
        .collect();
    let constructors: HashMap<&str, &str> = instructions
        .iter()
        .filter_map(|op| match op {
            CoreOp::DefMethod(class_id, method_id, method_name) => {
                let class_name = *classes.get(class_id.as_str())?;
                (method_name == class_name).then_some((method_id.as_str(), class_name))
            }
            _ => None,
        })
        .collect();

    instructions
        .iter()
        .filter_map(|op| match op {
            CoreOp::Param(method_id, _parameter_id, parameter_type, _parameter_name)
                if !parameter_type.is_empty() && parameter_type != super::opcodes::TYPE_VOID =>
            {
                let class_name = *constructors.get(method_id.as_str())?;
                Some(SemanticEdge {
                    relation: SemanticRelation::HasConstructorParameterType,
                    subject: EntityRef::new(BUILTIN_DOMAIN, CLASS_ENTITY_TYPE, class_name)
                        .with_file(file.to_string()),
                    object: EntityRef::new(BUILTIN_DOMAIN, TYPE_REF_ENTITY_TYPE, parameter_type)
                        .with_file(file.to_string()),
                    layer: BUILTIN_LAYER,
                    call_evidence: None,
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
    edges.extend(project_constructor_parameter_types(instructions, file));
    edges.extend(project_inheritance(instructions, file));
    edges
}

#[cfg(test)]
#[path = "../tests/ir/semantic_projection.rs"]
mod tests;
