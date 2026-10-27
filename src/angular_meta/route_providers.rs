//! Exact native Routes-root/object membership feeding shared provider policies.
use crate::ir::typescript_resolution::TypeScriptAuthority;
use crate::ir::typescript_resolution::declaration_facets::{
    DeclarationFacet, ExpressionFacet, ExpressionValue,
};
use crate::layers::meta::canonical_endpoint::{CanonicalEndpoint, RouteOccurrence};
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};

fn contains(parent: &ExpressionFacet, child: &ExpressionFacet) -> bool {
    parent.start <= child.start && child.start < child.end && child.end <= parent.end
}

fn structure(authority: &TypeScriptAuthority, value: &ExpressionFacet) -> bool {
    authority.admits_demanded_structure(&authority.source, &authority.source_hash, value)
}

fn unique<'a>(fields: &'a [(String, ExpressionFacet)], name: &str) -> Option<&'a ExpressionFacet> {
    let mut found = fields.iter().filter(|(key, _)| key.as_str() == name);
    let (_, value) = found.next()?;
    found.next().is_none().then_some(value)
}

fn root(authority: &TypeScriptAuthority, facet: &DeclarationFacet) -> bool {
    super::typed_provider_root::admits(authority, facet, "@angular/router", "Routes")
        && facet
            .initializer
            .as_ref()
            .is_some_and(|value| matches!(value.value, ExpressionValue::Array(_)))
}

fn objects(
    authority: &TypeScriptAuthority,
    root: &DeclarationFacet,
    array: &ExpressionFacet,
    parent_path: &str,
    output: &mut Vec<SemanticEdge>,
) {
    if !structure(authority, array) {
        return;
    }
    let ExpressionValue::Array(values) = &array.value else {
        return;
    };
    for object in values {
        if !contains(array, object) || !structure(authority, object) {
            continue;
        }
        let ExpressionValue::Object(fields) = &object.value else {
            continue;
        };
        // Spreads/computed/duplicate relevant fields cannot establish a unique object.
        if fields.iter().any(|(name, _)| name.is_empty())
            || ["path", "providers", "children"].iter().any(|name| {
                fields
                    .iter()
                    .filter(|(key, _)| key.as_str() == *name)
                    .count()
                    > 1
            })
        {
            continue;
        }
        let segment = if let Some(path) = unique(fields, "path") {
            if !contains(object, path) {
                continue;
            }
            let Some(units) =
                authority.demanded_string_value(&authority.source, &authority.source_hash, path)
            else {
                continue;
            };
            let Ok(value) = String::from_utf16(&units) else {
                continue;
            };
            value
        } else {
            String::new()
        };
        let path = if parent_path.is_empty() {
            segment
        } else if segment.is_empty() {
            parent_path.to_owned()
        } else {
            format!("{parent_path}/{segment}")
        };
        if let Some(providers) = unique(fields, "providers")
            && contains(object, providers)
        {
            let proof = RouteOccurrence {
                root: (&root.declaration).into(),
                source: authority.source.clone(),
                source_hash: authority.source_hash.clone(),
                start: object.start,
                end: object.end,
            };
            let mut subject = EntityRef::new("angular", "Route", &path)
                .with_file(authority.source.to_string_lossy().into_owned());
            subject.canonical = Some(CanonicalEndpoint::RouteOccurrence {
                route_occurrence: proof,
            });
            output.extend(super::provider_imported::project_entries(
                authority,
                &root.declaration,
                subject,
                providers,
                false,
                SemanticRelation::RegistersProvider,
                true,
            ));
        }
        if let Some(children) = unique(fields, "children")
            && contains(object, children)
            && matches!(children.value, ExpressionValue::Array(_))
        {
            objects(authority, root, children, &path, output);
        }
    }
}

pub(crate) fn project(authority: &TypeScriptAuthority) -> Vec<SemanticEdge> {
    let mut edges = Vec::new();
    for facet in &authority.declaration_facets {
        if root(authority, facet)
            && let Some(array) = &facet.initializer
        {
            objects(authority, facet, array, "", &mut edges);
        }
    }
    edges
}

/// Reproduce the exact root/object/field/entry join, including subject identity.
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
            && same(&expected.subject, &edge.subject)
            && same(&expected.object, &edge.object)
    })
}
