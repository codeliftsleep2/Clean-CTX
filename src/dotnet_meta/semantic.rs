// src/dotnet_meta/semantic.rs
//
// .NET-specific semantic edge construction helpers.
//
// These follow the same scanning patterns as the existing .NET meta-layer
// extractors (aspnet, efcore, signalr, automapper) but produce SemanticEdge
// objects instead of Phi marker strings.
//
// Phase 3 contract: zero duplication of existing Phi output. Semantic edges
// are a separate projection of the same framework information.

use crate::compression::Fidelity;
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};

const EF_CORE_DB_CONTEXT: &str = "Microsoft.EntityFrameworkCore.DbContext";
const AUTOMAPPER_PROFILE: &str = "AutoMapper.Profile";
const SIGNALR_HUB: &str = "Microsoft.AspNetCore.SignalR.Hub";
const ASPNET_CONTROLLER: &str = "Microsoft.AspNetCore.Mvc.Controller";
const ASPNET_CONTROLLER_BASE: &str = "Microsoft.AspNetCore.Mvc.ControllerBase";
const ASPNET_API_CONTROLLER_ATTRIBUTE: &str = "Microsoft.AspNetCore.Mvc.ApiControllerAttribute";
const ASPNET_CONTROLLER_ATTRIBUTE: &str = "Microsoft.AspNetCore.Mvc.ControllerAttribute";
const ASPNET_ROUTE_ATTRIBUTE: &str = "Microsoft.AspNetCore.Mvc.RouteAttribute";
const ASPNET_HTTP_ATTRIBUTES: &[&str] = &[
    "Microsoft.AspNetCore.Mvc.HttpGetAttribute",
    "Microsoft.AspNetCore.Mvc.HttpPostAttribute",
    "Microsoft.AspNetCore.Mvc.HttpPutAttribute",
    "Microsoft.AspNetCore.Mvc.HttpDeleteAttribute",
    "Microsoft.AspNetCore.Mvc.HttpPatchAttribute",
    "Microsoft.AspNetCore.Mvc.HttpHeadAttribute",
    "Microsoft.AspNetCore.Mvc.HttpOptionsAttribute",
];
const MSTEST_TEST_CLASS_ATTRIBUTE: &str =
    "Microsoft.VisualStudio.TestTools.UnitTesting.TestClassAttribute";

pub(crate) fn known_csharp_type_identities() -> Vec<crate::ir::csharp_resolution::CSharpTypeIdentity>
{
    vec![
        crate::ir::csharp_resolution::CSharpTypeIdentity::known_external(EF_CORE_DB_CONTEXT),
        crate::ir::csharp_resolution::CSharpTypeIdentity::known_external(AUTOMAPPER_PROFILE),
        crate::ir::csharp_resolution::CSharpTypeIdentity::known_external(SIGNALR_HUB),
        crate::ir::csharp_resolution::CSharpTypeIdentity::known_external_with_arity(SIGNALR_HUB, 1),
        crate::ir::csharp_resolution::CSharpTypeIdentity::known_external(ASPNET_CONTROLLER),
        crate::ir::csharp_resolution::CSharpTypeIdentity::known_external(ASPNET_CONTROLLER_BASE),
        crate::ir::csharp_resolution::CSharpTypeIdentity::known_external(
            ASPNET_API_CONTROLLER_ATTRIBUTE,
        ),
        crate::ir::csharp_resolution::CSharpTypeIdentity::known_external(
            ASPNET_CONTROLLER_ATTRIBUTE,
        ),
        crate::ir::csharp_resolution::CSharpTypeIdentity::known_external(ASPNET_ROUTE_ATTRIBUTE),
        crate::ir::csharp_resolution::CSharpTypeIdentity::known_external(
            MSTEST_TEST_CLASS_ATTRIBUTE,
        ),
    ]
    .into_iter()
    .chain(ASPNET_HTTP_ATTRIBUTES.iter().map(|identity| {
        crate::ir::csharp_resolution::CSharpTypeIdentity::known_external(*identity)
    }))
    .collect()
}

pub(crate) fn project_resolved_mstest_edges(
    classes: &[crate::ir::csharp_resolution::CSharpResolvedClass],
    file: &str,
) -> Vec<SemanticEdge> {
    classes
        .iter()
        .filter(|class| class.files.first().is_some_and(|owner| owner == file))
        .filter(|class| is_resolved_mstest_test_class(class))
        .map(|class| {
            let mut entity = EntityRef::new("dotnet", "TestClass", &class.owner);
            entity.file = Some(file.to_string());
            SemanticEdge {
                relation: SemanticRelation::Defines,
                subject: entity.clone(),
                object: entity,
                layer: "dotnet",
                call_evidence: None,
            }
        })
        .collect()
}

fn is_resolved_mstest_test_class(
    class: &crate::ir::csharp_resolution::CSharpResolvedClass,
) -> bool {
    class.attributes.iter().any(|attribute| {
        matches!(
            &attribute.resolution,
            crate::ir::csharp_resolution::CSharpResolution::Resolved { identity, .. }
                if identity.qualified_name == MSTEST_TEST_CLASS_ATTRIBUTE
                    && identity.generic_arity == 0
                    && identity.origin
                        == crate::ir::csharp_resolution::CSharpTypeIdentityOrigin::KnownExternal
        )
    })
}

pub(crate) fn is_resolved_dbcontext(
    class: &crate::ir::csharp_resolution::CSharpResolvedClass,
) -> bool {
    matches!(
        &class.ancestry,
        crate::ir::csharp_resolution::CSharpResolution::Resolved { identity, .. }
            if identity.qualified_name == EF_CORE_DB_CONTEXT
                && identity.generic_arity == 0
                && identity.origin
                    == crate::ir::csharp_resolution::CSharpTypeIdentityOrigin::KnownExternal
    )
}

pub(crate) fn project_resolved_dbcontext_edges(
    classes: &[crate::ir::csharp_resolution::CSharpResolvedClass],
    file: &str,
    fidelity: Fidelity,
) -> Vec<SemanticEdge> {
    let mut edges = Vec::new();
    for class in classes.iter().filter(|class| {
        is_resolved_dbcontext(class) && class.files.iter().any(|owner| owner == file)
    }) {
        let mut role = EntityRef::new("dotnet", "DbContext", &class.owner);
        role.file = Some(file.to_string());
        edges.push(SemanticEdge {
            relation: SemanticRelation::Defines,
            subject: role.clone(),
            object: role,
            layer: crate::layers::meta::semantic::SEMANTIC_ROLE_REGISTRATION_LAYER,
            call_evidence: None,
        });
        for (_, raw_class) in class.raw_parts.iter().filter(|(owner, _)| owner == file) {
            if let Some(analysis) =
                crate::dotnet_meta::efcore::analyze_authorized_efcore(raw_class, fidelity)
            {
                edges.extend(analysis.semantic_edges.into_iter().map(|mut edge| {
                    edge.subject.file = Some(file.to_string());
                    edge.object.file = Some(file.to_string());
                    edge
                }));
            }
        }
    }
    edges
}

fn is_resolved_automapper_profile(
    class: &crate::ir::csharp_resolution::CSharpResolvedClass,
) -> bool {
    matches!(
        &class.ancestry,
        crate::ir::csharp_resolution::CSharpResolution::Resolved { identity, .. }
            if identity.qualified_name == AUTOMAPPER_PROFILE
                && identity.generic_arity == 0
                && identity.origin
                    == crate::ir::csharp_resolution::CSharpTypeIdentityOrigin::KnownExternal
    )
}

fn is_resolved_signalr_hub(class: &crate::ir::csharp_resolution::CSharpResolvedClass) -> bool {
    matches!(
        &class.ancestry,
        crate::ir::csharp_resolution::CSharpResolution::Resolved { identity, .. }
            if identity.qualified_name == SIGNALR_HUB
                && matches!(identity.generic_arity, 0 | 1)
                && identity.origin
                    == crate::ir::csharp_resolution::CSharpTypeIdentityOrigin::KnownExternal
    )
}

fn is_resolved_aspnet_controller(
    class: &crate::ir::csharp_resolution::CSharpResolvedClass,
) -> bool {
    matches!(
        &class.ancestry,
        crate::ir::csharp_resolution::CSharpResolution::Resolved { identity, .. }
            if matches!(
                identity.qualified_name.as_str(),
                ASPNET_CONTROLLER | ASPNET_CONTROLLER_BASE
            )
                && identity.generic_arity == 0
                && identity.origin
                    == crate::ir::csharp_resolution::CSharpTypeIdentityOrigin::KnownExternal
    ) || class.attributes.iter().any(|attribute| {
        resolved_attribute_is(attribute, ASPNET_API_CONTROLLER_ATTRIBUTE)
            || resolved_attribute_is(attribute, ASPNET_CONTROLLER_ATTRIBUTE)
    })
}

fn resolved_attribute_is(
    attribute: &crate::ir::csharp_resolution::CSharpResolvedAttribute,
    expected: &str,
) -> bool {
    matches!(
        &attribute.resolution,
        crate::ir::csharp_resolution::CSharpResolution::Resolved { identity, .. }
            if identity.qualified_name == expected
                && identity.generic_arity == 0
                && identity.origin
                    == crate::ir::csharp_resolution::CSharpTypeIdentityOrigin::KnownExternal
    )
}

pub(crate) fn project_resolved_aspnet_edges(
    classes: &[crate::ir::csharp_resolution::CSharpResolvedClass],
    file: &str,
    fidelity: Fidelity,
) -> Vec<SemanticEdge> {
    let mut edges = Vec::new();
    for class in classes.iter().filter(|class| {
        is_resolved_aspnet_controller(class) && class.files.iter().any(|owner| owner == file)
    }) {
        let Some(symbol_owner) = class.files.first() else {
            continue;
        };
        // A partial class is one resolved source symbol even though payload
        // facts remain attributed to their physical parts. Register its role
        // once under the deterministic first composed owner.
        if symbol_owner == file {
            let mut role = EntityRef::new("dotnet", "Controller", &class.owner);
            role.file = Some(file.to_string());
            edges.push(SemanticEdge {
                relation: SemanticRelation::Defines,
                subject: role.clone(),
                object: role,
                layer: crate::layers::meta::semantic::SEMANTIC_ROLE_REGISTRATION_LAYER,
                call_evidence: None,
            });
        }
        let mut controller = EntityRef::new("dotnet", "Controller", &class.owner);
        controller.file = Some(symbol_owner.clone());
        for attribute in class.attributes.iter().filter(|attribute| {
            attribute.file == file && resolved_attribute_is(attribute, ASPNET_ROUTE_ATTRIBUTE)
        }) {
            let Some(route) = first_quoted_argument(&attribute.raw) else {
                continue;
            };
            let mut target = EntityRef::new("dotnet", "Route", route);
            target.file = Some(file.to_string());
            edges.push(SemanticEdge {
                relation: SemanticRelation::HasRoute,
                subject: controller.clone(),
                object: target,
                layer: "dotnet",
                call_evidence: None,
            });
        }
        if fidelity != Fidelity::Low {
            for method in class.methods.iter().filter(|method| method.file == file) {
                if !method.attributes.iter().any(|attribute| {
                    ASPNET_HTTP_ATTRIBUTES
                        .iter()
                        .any(|identity| resolved_attribute_is(attribute, identity))
                }) {
                    continue;
                }
                let mut target = EntityRef::new("dotnet", "Action", &method.name);
                target.file = Some(file.to_string());
                edges.push(SemanticEdge {
                    relation: SemanticRelation::ControllerAction,
                    subject: controller.clone(),
                    object: target,
                    layer: "dotnet",
                    call_evidence: None,
                });
            }
        }
    }
    edges
}

fn first_quoted_argument(raw: &str) -> Option<&str> {
    let start = raw.find('"')? + 1;
    let end = raw[start..].find('"')?;
    Some(&raw[start..start + end])
}

pub(crate) fn project_resolved_automapper_edges(
    classes: &[crate::ir::csharp_resolution::CSharpResolvedClass],
    file: &str,
) -> Vec<SemanticEdge> {
    let mut edges = Vec::new();
    for class in classes.iter().filter(|class| {
        is_resolved_automapper_profile(class) && class.files.iter().any(|owner| owner == file)
    }) {
        let mut profile = EntityRef::new("dotnet", "MapperProfile", &class.owner);
        profile.file = Some(file.to_string());
        for (_, raw_class) in class.raw_parts.iter().filter(|(owner, _)| owner == file) {
            for (source, destination) in extract_create_map_mappings(raw_class) {
                let mut source_entity = EntityRef::new("dotnet", "Entity", source);
                source_entity.file = Some(file.to_string());
                edges.push(SemanticEdge {
                    relation: SemanticRelation::MapsFrom,
                    subject: profile.clone(),
                    object: source_entity,
                    layer: "dotnet",
                    call_evidence: None,
                });
                let mut destination_entity = EntityRef::new("dotnet", "Entity", destination);
                destination_entity.file = Some(file.to_string());
                edges.push(SemanticEdge {
                    relation: SemanticRelation::MapsTo,
                    subject: profile.clone(),
                    object: destination_entity,
                    layer: "dotnet",
                    call_evidence: None,
                });
            }
        }
    }
    edges
}

pub(crate) fn project_resolved_signalr_edges(
    classes: &[crate::ir::csharp_resolution::CSharpResolvedClass],
    file: &str,
    fidelity: Fidelity,
) -> Vec<SemanticEdge> {
    if fidelity == Fidelity::Low {
        return Vec::new();
    }
    let mut edges = Vec::new();
    for class in classes
        .iter()
        .filter(|class| is_resolved_signalr_hub(class) && class.files.iter().any(|f| f == file))
    {
        let mut hub = EntityRef::new("dotnet", "Hub", &class.owner);
        hub.file = Some(file.to_string());
        let mut projected_methods = std::collections::BTreeSet::new();
        for (_, raw_class) in class.raw_parts.iter().filter(|(owner, _)| owner == file) {
            for method_name in extract_hub_methods(raw_class) {
                if !projected_methods.insert(method_name.clone()) {
                    continue;
                }
                let mut method = EntityRef::new("dotnet", "HubMethod", method_name);
                method.file = Some(file.to_string());
                edges.push(SemanticEdge {
                    relation: SemanticRelation::HubMethodTargets,
                    subject: hub.clone(),
                    object: method,
                    layer: "dotnet",
                    call_evidence: None,
                });
            }
        }
    }
    edges
}

pub(crate) fn project_resolved_signalr_execution_contexts(
    classes: &[crate::ir::csharp_resolution::CSharpResolvedClass],
    file: &str,
) -> Vec<crate::ir::opcodes::CoreOp> {
    classes
        .iter()
        .filter(|class| is_resolved_signalr_hub(class))
        .flat_map(|class| &class.method_ids)
        .filter(|(owner, _)| owner == file)
        .flat_map(|(_, methods)| methods)
        .map(|method| {
            crate::ir::opcodes::CoreOp::ExecutionContext(
                method.clone(),
                crate::ir::opcodes::ExecutionContextKind::Realtime,
            )
        })
        .collect()
}

pub(crate) fn has_resolved_dotnet_role(
    classes: &[crate::ir::csharp_resolution::CSharpResolvedClass],
) -> bool {
    classes.iter().any(|class| {
        is_resolved_dbcontext(class)
            || is_resolved_automapper_profile(class)
            || is_resolved_signalr_hub(class)
            || is_resolved_aspnet_controller(class)
            || is_resolved_mstest_test_class(class)
    })
}

/// Register EF Core DbContext roles from the shared resolved class authority.
/// Relationship payload remains independently gated and may be empty.
pub(crate) fn resolved_dbcontext_role_registrations(
    classes: &[crate::ir::csharp_resolution::CSharpResolvedClass],
) -> Vec<SemanticEdge> {
    classes
        .iter()
        .filter(|class| is_resolved_dbcontext(class))
        .map(|class| {
            let role = EntityRef::new("dotnet", "DbContext", &class.owner);
            SemanticEdge {
                relation: SemanticRelation::Defines,
                subject: role.clone(),
                object: role,
                layer: crate::layers::meta::semantic::SEMANTIC_ROLE_REGISTRATION_LAYER,
                call_evidence: None,
            }
        })
        .collect()
}

pub(crate) fn resolved_dbcontext_role_names(
    classes: &[crate::ir::csharp_resolution::CSharpResolvedClass],
) -> std::collections::HashSet<&str> {
    classes
        .iter()
        .filter(|class| is_resolved_dbcontext(class))
        .map(|class| class.owner.as_str())
        .collect()
}

// ── Private Helpers (mirroring existing .NET extractor patterns) ──────────

/// Extract source/destination type pairs from CreateMap<TSource, TDest>() calls.
fn extract_create_map_mappings(class_source: &str) -> Vec<(String, String)> {
    let mut mappings = Vec::new();
    let mut search_start = 0;
    while let Some(pos) = class_source[search_start..].find("CreateMap<") {
        let actual_pos = search_start + pos;
        let rest = &class_source[actual_pos + "CreateMap<".len()..];
        if let Some(generic_end) = rest.find('>') {
            let types = rest[..generic_end].trim().to_string();
            if let Some(comma_pos) = types.find(',') {
                let source = types[..comma_pos].trim().to_string();
                let dest = types[comma_pos + 1..].trim().to_string();
                mappings.push((source, dest));
            }
        }
        search_start = actual_pos + 1;
    }
    mappings
}

/// Extract class name from a .NET class declaration.
/// Follows the same pattern as the existing private extract_class_name()
/// functions in aspnet.rs, efcore.rs, etc.
pub fn extract_class_name_from_class(source: &str) -> Option<String> {
    let patterns = [
        "public class ",
        "internal class ",
        "private class ",
        "protected class ",
        "class ",
    ];
    for pattern in &patterns {
        if let Some(pos) = source.find(pattern) {
            let start = pos + pattern.len();
            let rest = &source[start..];
            let end = rest
                .find(|c: char| c == ':' || c == '<' || c.is_whitespace() || c == '{')
                .unwrap_or(rest.len());
            let name = rest[..end].trim();
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
    }
    None
}

/// Extract hub method names from a SignalR Hub class.
fn extract_hub_methods(class_source: &str) -> Vec<String> {
    let mut methods = Vec::new();
    let mut search_start = 0;
    while let Some(pos) = class_source[search_start..].find("public ") {
        let actual_pos = search_start + pos;
        let rest = &class_source[actual_pos + "public ".len()..];
        if let Some(paren_pos) = rest.find('(') {
            let signature = &rest[..paren_pos];
            let method_name = signature
                .split_whitespace()
                .last()
                .unwrap_or("")
                .to_string();
            if !method_name.starts_with("On")
                && !method_name.starts_with("Dispose")
                && !method_name.is_empty()
            {
                methods.push(method_name);
            }
        }
        search_start = actual_pos + 1;
    }
    methods
}

#[cfg(all(test, feature = "dotnet"))]
#[path = "../tests/dotnet_meta/semantic.rs"]
mod tests;

#[cfg(all(test, feature = "dotnet", feature = "csharp"))]
#[path = "../tests/dotnet_meta/ef_resolution_r6.rs"]
mod tests_r6;

#[cfg(all(test, feature = "dotnet", feature = "csharp"))]
#[path = "../tests/dotnet_meta/automapper_resolution_r7.rs"]
mod tests_r7;

#[cfg(all(test, feature = "dotnet", feature = "csharp"))]
#[path = "../tests/dotnet_meta/signalr_resolution_r8.rs"]
mod tests_r8;

#[cfg(all(test, feature = "dotnet", feature = "csharp"))]
#[path = "../tests/dotnet_meta/aspnet_resolution_r9.rs"]
mod tests_r9;

#[cfg(all(test, feature = "dotnet", feature = "csharp"))]
#[path = "../tests/dotnet_meta/mstest_attribute_resolution_r10.rs"]
mod tests_r10;

#[cfg(all(test, feature = "dotnet", feature = "csharp"))]
#[path = "../tests/dotnet_meta/aspnet_attribute_resolution_r11.rs"]
mod tests_r11;

#[cfg(all(test, feature = "dotnet", feature = "csharp"))]
#[path = "../tests/dotnet_meta/csharp_convergence_r12.rs"]
mod tests_r12;

#[cfg(test)]
#[path = "../tests/dotnet_meta/csharp_compilation_identity.rs"]
mod tests_compilation_identity;

#[cfg(test)]
#[path = "../tests/dotnet_meta/csharp_compilation_identity_preservation.rs"]
mod tests_compilation_identity_preservation;
