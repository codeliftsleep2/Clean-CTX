//! Angular interpretation of retained constructor parameter meanings.
use super::{
    class_role_authority as roles, decorators::ClassKind, inject_evidence::ClassInitializerEvidence,
};
use crate::ir::typescript_resolution::TypeScriptAuthority;
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};

pub(crate) fn project(
    source: &str,
    authority: &TypeScriptAuthority,
    classes: &[ClassInitializerEvidence],
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
            ClassKind::Pipe => "Pipe",
            _ => continue,
        };
        for parameter in authority.constructor_parameters.iter().filter(|parameter| {
            parameter.owner.source == authority.source
                && parameter.owner.name == class.consumer
                && parameter.owner.start >= start
                && parameter.owner.end == class.class_span.end_byte()
        }) {
            let mut overrides = Vec::new();
            let mut legacy = !matches!(
                parameter.shape.as_str(),
                "type_identifier" | "nested_type_identifier" | "generic_type"
            );
            // Old records with only decorator spans have no API-role proof.
            let mut unproved_decorator =
                parameter.decorators.len() != parameter.decorator_facets.len();
            for decorator in &parameter.decorator_facets {
                use crate::ir::typescript_resolution::Resolution;
                use crate::ir::typescript_resolution::declaration_facets::ExpressionValue;
                let ExpressionValue::Reference(Resolution::Resolved(api)) = &decorator.callee.value
                else {
                    unproved_decorator = true;
                    continue;
                };
                if roles::public_api(authority, api, "@angular/core", "Inject") {
                    overrides.push(decorator);
                } else if roles::public_api(authority, api, "@angular/core", "Attribute") {
                    // Attribute is a different dependency meaning; retain its
                    // established noncanonical form until that slice migrates.
                    unproved_decorator = true;
                }
                // Proved foreign decorators cannot impersonate Angular Inject.
                // Optional/Self/SkipSelf/Host modify scope, not token identity.
            }
            if !overrides.is_empty() {
                // A failed runtime reference or competing override cannot
                // authorize the parameter annotation as a replacement token.
                let [decorator] = overrides.as_slice() else {
                    continue;
                };
                let [argument] = decorator.arguments.as_slice() else {
                    continue;
                };
                if unproved_decorator {
                    continue;
                }
                if let Some(units) = authority.string_value(argument) {
                    edges.push(super::string_token_authority::injection_edge(
                        &parameter.owner,
                        owner_type,
                        argument,
                        &units,
                    ));
                    continue;
                }
                match &argument.value {
                    crate::ir::typescript_resolution::declaration_facets::ExpressionValue::Reference(
                        crate::ir::typescript_resolution::Resolution::Resolved(token)) => {
                        edges.push(super::inject_authority::canonical_edge(&parameter.owner, owner_type, token));
                        continue;
                    }
                    crate::ir::typescript_resolution::declaration_facets::ExpressionValue::Unsupported(_) => {
                        // Non-reference token syntax remains the explicitly
                        // unmigrated legacy projection, never canonical proof.
                        legacy = true;
                    }
                    _ => continue,
                }
            }
            legacy |= unproved_decorator;
            if !legacy && let Some(token) = parameter.runtime_class() {
                edges.push(super::inject_authority::canonical_edge(
                    &parameter.owner,
                    owner_type,
                    token,
                ));
                continue;
            }
            let symbolic = if legacy {
                source
                    .get(parameter.parameter.0..parameter.parameter.1)
                    .and_then(super::constructor_injects::constructor_param_inject_type)
            } else if parameter.unbound {
                parameter.path.first().cloned()
            } else {
                None
            };
            if let Some(name) = symbolic {
                edges.push(SemanticEdge {
                    relation: SemanticRelation::Injects,
                    subject: EntityRef::new("angular", owner_type, &parameter.owner.name)
                        .with_file(parameter.owner.source.to_string_lossy().into_owned()),
                    object: EntityRef::new("angular", "Service", name),
                    layer: "angular",
                    call_evidence: None,
                });
            }
        }
    }
    edges
}
