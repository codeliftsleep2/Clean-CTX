//! Class-field inject dependencies from retained value endpoints, not spellings.
use super::{
    class_role_authority as roles, decorators::ClassKind, inject_evidence::ClassInitializerEvidence,
};
use crate::ir::invocation_evidence::{
    ArgumentQualifier, InvocationEvidenceView, SourceSpan, WrittenCallee,
};
use crate::ir::typescript_resolution::{DeclarationId, TypeScriptAuthority};
use crate::layers::meta::canonical_endpoint::CanonicalEndpoint;
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};

pub(crate) fn project(
    source: &str,
    authority: &TypeScriptAuthority,
    classes: &[ClassInitializerEvidence],
    invocations: &InvocationEvidenceView<'_>,
) -> Vec<SemanticEdge> {
    let mut edges = Vec::new();
    for class in classes {
        let Some(start) = class
            .class_span
            .end_byte()
            .checked_sub(class.raw_class.len())
        else {
            continue;
        };
        let Some(role) = roles::class_role(source, &class.raw_class, start, authority) else {
            continue;
        };
        let owner_type = match role.kind {
            ClassKind::Component => "Component",
            ClassKind::Service => "Service",
            ClassKind::Directive => "Directive",
            _ => continue,
        };
        let owners: Vec<_> = authority
            .declaration_facets
            .iter()
            .filter(|facet| {
                facet.declaration.source == authority.source
                    && facet.declaration.name == class.consumer
                    && facet.declaration.start >= start
                    && facet.declaration.end == class.class_span.end_byte()
            })
            .collect();
        let [owner] = owners.as_slice() else {
            continue;
        };
        for field in &class.fields {
            if field.receiver.is_empty()
                || field.optional_call
                || field.field_span.start_byte() < class.class_span.start_byte()
                || field.field_span.end_byte() > class.class_span.end_byte()
                || field.initializer_span.start_byte() < field.field_span.start_byte()
                || field.initializer_span.end_byte() > field.field_span.end_byte()
                || field.invocation_span != Some(field.initializer_span)
            {
                continue;
            }
            let calls: Vec<_> = invocations
                .occurrences()
                .iter()
                .filter(|call| call.span() == field.initializer_span)
                .collect();
            let [call] = calls.as_slice() else {
                continue;
            };
            let callee_span = match call.callee() {
                WrittenCallee::Free { terminal } => *terminal,
                WrittenCallee::Member { receiver, terminal } => {
                    let Ok(span) = SourceSpan::new(receiver.start_byte(), terminal.end_byte())
                    else {
                        continue;
                    };
                    span
                }
                WrittenCallee::Unsupported { .. } => continue,
            };
            // Resolve the entire qualification. A member named inject on an
            // unrelated receiver cannot borrow the Angular export's role.
            let Some(api) = authority.value_reference_at(callee_span) else {
                continue;
            };
            if !roles::public_api(authority, api, "@angular/core", "inject") {
                continue;
            }
            let Some(argument) = call.arguments().first() else {
                continue;
            };
            if argument.qualifier() != &ArgumentQualifier::Plain
                || invocations.slice(argument.span()).is_err()
            {
                continue;
            }
            let Some(token) = authority.value_reference_at(argument.span()) else {
                if let Some((_, _, name)) =
                    authority
                        .unbound_value_sites
                        .iter()
                        .find(|(start, end, _)| {
                            *start == argument.span().start_byte()
                                && *end == argument.span().end_byte()
                        })
                {
                    // Preserve the established symbolic relationship, without
                    // inventing a declaration endpoint or canonical annotation.
                    edges.push(SemanticEdge {
                        relation: SemanticRelation::Injects,
                        subject: EntityRef::new("angular", owner_type, &owner.declaration.name)
                            .with_file(owner.declaration.source.to_string_lossy().into_owned()),
                        object: EntityRef::new("angular", "Service", name),
                        layer: "angular",
                        call_evidence: None,
                    });
                }
                continue;
            };
            edges.push(canonical_edge(&owner.declaration, owner_type, token));
        }
    }
    edges
}

/// Constructor and field syntax share the same owner/token dependency fact.
pub(super) fn canonical_edge(
    owner: &DeclarationId,
    owner_type: &'static str,
    token: &DeclarationId,
) -> SemanticEdge {
    let mut subject = EntityRef::new("angular", owner_type, &owner.name)
        .with_file(owner.source.to_string_lossy().into_owned());
    // Injects is the existing owner-to-token dependency fact, not a call
    // census. Multiple fields using the same token retain one fact.
    subject.canonical = Some(CanonicalEndpoint::Declaration {
        declaration: owner.into(),
        site: Some((owner.start, owner.end)),
    });
    let mut object = EntityRef::new("angular", "Service", &token.name)
        .with_file(token.source.to_string_lossy().into_owned());
    object.canonical = Some(CanonicalEndpoint::Declaration {
        declaration: token.into(),
        site: None,
    });
    SemanticEdge {
        relation: SemanticRelation::Injects,
        subject,
        object,
        layer: "angular",
        call_evidence: None,
    }
}
