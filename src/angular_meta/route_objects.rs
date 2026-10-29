//! Exact native Routes-root/object membership shared by providers and eager Component membership.
use crate::ir::typescript_resolution::TypeScriptAuthority;
use crate::ir::typescript_resolution::declaration_facets::{
    DeclarationFacet, ExpressionFacet, ExpressionValue,
};
use crate::layers::meta::canonical_endpoint::{CanonicalEndpoint, RouteOccurrence};
use crate::layers::meta::semantic::EntityRef;

pub(super) fn contains(parent: &ExpressionFacet, child: &ExpressionFacet) -> bool {
    parent.start <= child.start && child.start < child.end && child.end <= parent.end
}

fn structure(authority: &TypeScriptAuthority, value: &ExpressionFacet) -> bool {
    authority.admits_demanded_structure(&authority.source, &authority.source_hash, value)
}

pub(super) fn unique<'a>(
    fields: &'a [(String, ExpressionFacet)],
    name: &str,
) -> Option<&'a ExpressionFacet> {
    let mut found = fields.iter().filter(|(key, _)| key.as_str() == name);
    let (_, value) = found.next()?;
    found.next().is_none().then_some(value)
}

fn root(authority: &TypeScriptAuthority, facet: &DeclarationFacet) -> bool {
    super::typed_provider_root::admits(authority, facet, "@angular/router", "Routes")
        && super::typed_provider_root::admits_qualified(
            authority,
            facet,
            "@angular/router",
            "Routes",
        )
        && facet
            .initializer
            .as_ref()
            .is_some_and(|value| matches!(value.value, ExpressionValue::Array(_)))
}

fn objects<'a>(
    authority: &'a TypeScriptAuthority,
    root: &'a DeclarationFacet,
    array: &'a ExpressionFacet,
    parent_path: &str,
    output: &mut Vec<RouteObject<'a>>,
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
        output.push(RouteObject {
            root,
            object,
            fields,
            path: path.clone(),
        });
        if let Some(children) = unique(fields, "children")
            && contains(object, children)
            && matches!(children.value, ExpressionValue::Array(_))
        {
            objects(authority, root, children, &path, output);
        }
    }
}

pub(super) struct RouteObject<'a> {
    pub root: &'a DeclarationFacet,
    pub object: &'a ExpressionFacet,
    pub fields: &'a [(String, ExpressionFacet)],
    pub path: String,
}
impl RouteObject<'_> {
    pub fn subject(&self, authority: &TypeScriptAuthority) -> EntityRef {
        let mut subject = EntityRef::new("angular", "Route", &self.path)
            .with_file(authority.source.to_string_lossy().into_owned());
        subject.canonical = Some(CanonicalEndpoint::RouteOccurrence {
            route_occurrence: RouteOccurrence {
                root: (&self.root.declaration).into(),
                source: authority.source.clone(),
                source_hash: authority.source_hash.clone(),
                start: self.object.start,
                end: self.object.end,
            },
        });
        subject
    }
}
pub(super) fn project(authority: &TypeScriptAuthority) -> Vec<RouteObject<'_>> {
    let mut objects_out = Vec::new();
    for facet in &authority.declaration_facets {
        if root(authority, facet)
            && let Some(array) = &facet.initializer
        {
            objects(authority, facet, array, "", &mut objects_out);
        }
    }
    objects_out
}
