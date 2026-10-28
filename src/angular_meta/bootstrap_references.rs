//! Written bootstrap associations only; no activation or provider effectiveness.
use crate::ir::typescript_resolution::declaration_facets::{
    ExpressionFacet, ExpressionValue, InvocationFacet,
};
use crate::ir::typescript_resolution::demanded_occurrences::DemandedValue;
use crate::ir::typescript_resolution::{DeclarationId, Namespace, Resolution, TypeScriptAuthority};
use crate::layers::meta::canonical_endpoint::{CanonicalEndpoint, InvocationOccurrence};
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};

pub(crate) const DEMANDED_APIS: &[(&str, &str)] = &[
    ("@angular/platform-browser", "bootstrapApplication"),
    ("@angular/core", "mergeApplicationConfig"),
];

pub(super) fn call<'a>(
    authority: &'a TypeScriptAuthority,
    facet: &'a InvocationFacet,
) -> Option<&'a [ExpressionFacet]> {
    let expression = &facet.expression;
    if facet.source != authority.source
        || facet.source_hash != authority.source_hash
        || authority
            .invocation_facets
            .iter()
            .filter(|other| {
                other.source == facet.source
                    && other.expression.start == expression.start
                    && other.expression.end == expression.end
            })
            .count()
            != 1
    {
        return None;
    }
    let ExpressionValue::Call(callee, args) = &expression.value else {
        return None;
    };
    let Some(DemandedValue::Structure { kind, children }) = authority.demanded_at(
        &facet.source,
        &facet.source_hash,
        expression.start,
        expression.end,
    ) else {
        return None;
    };
    let expected: Vec<_> = std::iter::once(("callee".into(), callee.start, callee.end))
        .chain(
            args.iter()
                .enumerate()
                .map(|(i, a)| (format!("argument:{i}"), a.start, a.end)),
        )
        .collect();
    if kind != "call" || children != &expected {
        return None;
    }
    let mut previous = expression.start;
    if !expected.iter().all(|(_, start, end)| {
        let valid = previous <= *start && start < end && *end <= expression.end;
        previous = *end;
        valid
    }) {
        return None;
    }
    let ExpressionValue::Reference(Resolution::Resolved(api)) = &callee.value else {
        return None;
    };
    let span = crate::ir::invocation_evidence::SourceSpan::new(callee.start, callee.end).ok()?;
    if authority.value_reference_at(span) != Some(api)
        || authority.demanded_reference_at(
            &facet.source,
            &facet.source_hash,
            callee.start,
            callee.end,
        ) != Some(api)
        || !super::class_role_authority::public_entry_api(
            authority,
            api,
            "@angular/platform-browser",
            "@angular/platform-browser",
            "bootstrapApplication",
            Namespace::Value,
        )
    {
        return None;
    }
    Some(args)
}
fn target<'a>(
    authority: &'a TypeScriptAuthority,
    args: &[ExpressionFacet],
    position: usize,
) -> Option<&'a DeclarationId> {
    let argument = args.get(position)?;
    if args[..=position]
        .iter()
        .any(|arg| matches!(arg.value, ExpressionValue::Spread(_)))
    {
        return None;
    }
    let ExpressionValue::Reference(Resolution::Resolved(expected)) = &argument.value else {
        return None;
    };
    let actual = authority.demanded_reference_at(
        &authority.source,
        &authority.source_hash,
        argument.start,
        argument.end,
    )?;
    let span =
        crate::ir::invocation_evidence::SourceSpan::new(argument.start, argument.end).ok()?;
    (actual == expected && authority.value_reference_at(span) == Some(actual)).then_some(actual)
}
fn component(authority: &TypeScriptAuthority, id: &DeclarationId) -> bool {
    let mut facets = authority
        .declaration_facets
        .iter()
        .filter(|facet| facet.declaration == *id);
    let Some(facet) = facets.next() else {
        return false;
    };
    if facets.next().is_some() {
        return false;
    }
    let Some(DemandedValue::Declaration {
        id: actual,
        decorators,
        ..
    }) = authority.demanded_at(&id.source, &id.source_hash, id.start, id.end)
    else {
        return false;
    };
    if actual != id
        || *decorators
            != facet
                .decorators
                .iter()
                .map(|d| (d.start, d.end))
                .collect::<Vec<_>>()
    {
        return false;
    }
    let role = super::class_role_authority::declaration_role(authority, id);
    if role != Some(super::decorators::ClassKind::Component) {
        return false;
    }
    facet.decorators.iter().all(|decorator| {
        let expected:Vec<_>=std::iter::once(("callee".into(),decorator.callee.start,decorator.callee.end))
            .chain(decorator.arguments.iter().enumerate().map(|(i,a)|(format!("argument:{i}"),a.start,a.end))).collect();
        matches!(authority.demanded_at(&id.source,&id.source_hash,decorator.start,decorator.end),
            Some(DemandedValue::Structure {kind,children}) if kind=="decorator" && children==&expected)
        && matches!(&decorator.callee.value,ExpressionValue::Reference(Resolution::Resolved(api))
            if authority.demanded_reference_at(&id.source,&id.source_hash,decorator.callee.start,decorator.callee.end)==Some(api))
    })
}
fn config(authority: &TypeScriptAuthority, id: &DeclarationId) -> bool {
    let Some(facet) = authority
        .declaration_facets
        .iter()
        .find(|facet| facet.declaration == *id)
    else {
        return false;
    };
    super::typed_provider_root::admits_qualified(
        authority,
        facet,
        "@angular/core",
        "ApplicationConfig",
    ) && (facet
        .initializer
        .as_ref()
        .is_some_and(|i| matches!(i.value, ExpressionValue::Object(_)))
        || super::config_composition::result_args(authority, facet).is_some())
}
/// Applicability shares the exact retained callee/call path with projection.
pub(crate) fn has_call(authority: &TypeScriptAuthority) -> bool {
    authority
        .invocation_facets
        .iter()
        .any(|facet| call(authority, facet).is_some())
}
pub(crate) fn project(authority: &TypeScriptAuthority) -> Vec<SemanticEdge> {
    let mut edges = Vec::new();
    for facet in &authority.invocation_facets {
        let Some(args) = call(authority, facet) else {
            continue;
        };
        let expression = &facet.expression;
        let mut subject = EntityRef::new("angular", "ApplicationBootstrap", "bootstrapApplication")
            .with_file(facet.source.to_string_lossy().into_owned());
        subject.canonical = Some(CanonicalEndpoint::InvocationOccurrence {
            invocation_occurrence: InvocationOccurrence {
                source: facet.source.clone(),
                source_hash: facet.source_hash.clone(),
                start: expression.start,
                end: expression.end,
            },
        });
        edges.extend(super::bootstrap_inline_providers::project_call(
            authority,
            facet,
            args,
            subject.clone(),
        ));
        for (position, kind, relation) in [
            (0, "Component", SemanticRelation::DeclaresBootstrapRoot),
            (
                1,
                "ApplicationConfig",
                SemanticRelation::UsesApplicationConfig,
            ),
        ] {
            let Some(id) = target(authority, args, position) else {
                continue;
            };
            if !(if position == 0 {
                component(authority, id)
            } else {
                config(authority, id)
            }) {
                continue;
            }
            edges.push(SemanticEdge {
                subject: subject.clone(),
                object: super::string_token_authority::owner(id, kind),
                relation,
                layer: "angular",
                call_evidence: None,
            });
        }
    }
    edges
}
pub(crate) fn admits(authority: &TypeScriptAuthority, edge: &SemanticEdge) -> bool {
    let same = |a: &EntityRef, b: &EntityRef| {
        a.domain == b.domain
            && a.entity_type == b.entity_type
            && a.name == b.name
            && a.file == b.file
            && a.canonical == b.canonical
    };
    project(authority).iter().any(|expected| {
        expected.relation == edge.relation
            && expected.layer == edge.layer
            && edge.call_evidence.is_none()
            && same(&expected.subject, &edge.subject)
            && same(&expected.object, &edge.object)
    })
}
