//! Ephemeral Angular Template selector relationships.
//!
//! This request-local catalog reads current Template sources and current
//! Component declarations. Nothing here is stored in WorkspaceIndex.

use crate::compression::Fidelity;
use crate::layers::meta::semantic::{EntityRef, SemanticRelation};
use crate::mcp::McpState;
use crate::workspace::index::WorkspaceIndex;
use crate::workspace::scope::WorkspaceScope;
use serde_json::{Value, json};
use std::collections::BTreeSet;

pub(super) struct Catalog {
    edges: Vec<Value>,
    diagnostics: Vec<Value>,
    sources: Vec<TemplateSourceStatus>,
}

struct TemplateSourceStatus {
    component: EntityRef,
    template: EntityRef,
    available: bool,
}

impl Catalog {
    pub(super) fn build(
        state: &McpState,
        index: &WorkspaceIndex,
        scope: Option<&WorkspaceScope>,
        discovery_complete: bool,
    ) -> Self {
        let mut component_keys = BTreeSet::new();
        for keys in index.file_map().values() {
            for key in keys {
                if key.0 == "angular" && key.1 == "Component" {
                    component_keys.insert(key.clone());
                }
            }
        }

        let mut edges = Vec::new();
        let mut diagnostics = Vec::new();
        let mut sources = Vec::new();
        let mut seen_links = BTreeSet::new();
        for (_, _, component_name) in component_keys {
            let links = match scope {
                Some(scope) => index.forward_edges_by_identity_in_scope(
                    "angular",
                    "Component",
                    &component_name,
                    scope,
                ),
                None => index.forward_edges_by_identity("angular", "Component", &component_name),
            };
            for link in links
                .into_iter()
                .filter(|edge| edge.relation == SemanticRelation::HasTemplate)
            {
                let link_key = (
                    link.subject.name.clone(),
                    link.subject.file.clone(),
                    link.object.name.clone(),
                );
                if !seen_links.insert(link_key) {
                    continue;
                }
                let Some(source) = template_source(state, link) else {
                    sources.push(TemplateSourceStatus {
                        component: link.subject.clone(),
                        template: link.object.clone(),
                        available: false,
                    });
                    diagnostics.push(json!({
                        "code": "linked_template_missing",
                        "component": link.subject.clone(),
                        "template": link.object.clone(),
                    }));
                    continue;
                };
                let ordered = crate::angular_meta::template_ordered::extract(&source);
                let available = ordered.is_supported();
                sources.push(TemplateSourceStatus {
                    component: link.subject.clone(),
                    template: link.object.clone(),
                    available,
                });
                if !available {
                    diagnostics.push(json!({
                        "code": "template_parse_unsupported",
                        "component": link.subject.clone(),
                        "template": link.object.clone(),
                    }));
                    continue;
                }
                for (ordinal, occurrence) in ordered.selector_occurrences().iter().enumerate() {
                    let candidates =
                        selector_candidates(index, scope, &occurrence.written_selector);
                    let status = if !discovery_complete {
                        "incomplete"
                    } else {
                        match candidates.len() {
                            0 => "unresolved",
                            1 => "unique",
                            _ => "ambiguous",
                        }
                    };
                    let unique = discovery_complete && candidates.len() == 1;
                    if status != "unique" {
                        let code = match status {
                            "unresolved" => "selector_unresolved",
                            "ambiguous" => "selector_ambiguous",
                            _ => "component_discovery_incomplete",
                        };
                        diagnostics.push(json!({
                            "code": code,
                            "component": link.subject.clone(),
                            "template": link.object.clone(),
                            "selector": occurrence.written_selector,
                            "occurrence": ordinal,
                            "candidates": candidates.clone(),
                        }));
                    }
                    let evidence = json!({
                        "template_occurrence": occurrence,
                        "resolution": {
                            "status": status,
                            "discovery_complete": discovery_complete,
                            "candidates": candidates.clone(),
                        }
                    });
                    edges.push(json!({
                        "relation": "UsesSelector",
                        "subject": link.object.clone(),
                        "object": EntityRef::new(
                            "angular",
                            "SelectorRef",
                            &occurrence.written_selector,
                        ),
                        "layer": "angular",
                        "occurrence": ordinal,
                        "evidence": evidence.clone(),
                    }));
                    if unique {
                        edges.push(json!({
                            "relation": "UsesComponent",
                            "subject": link.subject.clone(),
                            "object": candidates[0],
                            "layer": "angular",
                            "occurrence": ordinal,
                            "evidence": evidence,
                        }));
                    }
                }
            }
        }
        Self {
            edges,
            diagnostics,
            sources,
        }
    }

    pub(super) fn diagnostics(&self, domain: &str, entity_type: &str, name: &str) -> Vec<Value> {
        if domain != "angular" {
            return Vec::new();
        }
        let field = match entity_type {
            "Component" => "component",
            "Template" => "template",
            "SelectorRef" => "selector",
            _ => return Vec::new(),
        };
        self.diagnostics
            .iter()
            .filter(|diagnostic| {
                if field == "selector" {
                    diagnostic[field] == name
                } else {
                    identity_matches(&diagnostic[field], domain, entity_type, name)
                }
            })
            .cloned()
            .collect()
    }

    pub(super) fn source_complete(
        &self,
        domain: &str,
        entity_type: &str,
        name: &str,
        discovery_complete: bool,
    ) -> bool {
        if !discovery_complete || domain != "angular" {
            return false;
        }
        let matching: Vec<_> = self
            .sources
            .iter()
            .filter(|source| match entity_type {
                "Component" => identity_ref_matches(&source.component, domain, entity_type, name),
                "Template" => identity_ref_matches(&source.template, domain, entity_type, name),
                _ => false,
            })
            .collect();
        !matching.is_empty() && matching.iter().all(|source| source.available)
    }

    pub(super) fn selectors(&self) -> BTreeSet<String> {
        self.edges
            .iter()
            .filter(|edge| edge["relation"] == "UsesSelector")
            .filter_map(|edge| edge["object"]["name"].as_str().map(str::to_string))
            .collect()
    }

    pub(super) fn forward(&self, domain: &str, entity_type: &str, name: &str) -> Vec<Value> {
        self.edges
            .iter()
            .filter(|edge| identity_matches(&edge["subject"], domain, entity_type, name))
            .cloned()
            .collect()
    }

    pub(super) fn reverse(&self, domain: &str, entity_type: &str, name: &str) -> Vec<Value> {
        self.edges
            .iter()
            .filter(|edge| identity_matches(&edge["object"], domain, entity_type, name))
            .cloned()
            .collect()
    }
}

fn identity_matches(entity: &Value, domain: &str, entity_type: &str, name: &str) -> bool {
    entity["domain"] == domain && entity["entity_type"] == entity_type && entity["name"] == name
}

fn identity_ref_matches(entity: &EntityRef, domain: &str, entity_type: &str, name: &str) -> bool {
    entity.domain == domain && entity.entity_type == entity_type && entity.name == name
}

fn selector_candidates(
    index: &WorkspaceIndex,
    scope: Option<&WorkspaceScope>,
    selector: &str,
) -> Vec<EntityRef> {
    let mut seen = BTreeSet::new();
    index
        .resolve_selector(selector)
        .into_iter()
        .filter(|candidate| {
            candidate.domain == "angular"
                && candidate.entity_type == "Component"
                && scope.is_none_or(|scope| {
                    candidate
                        .file
                        .as_deref()
                        .is_some_and(|file| scope.admits(file))
                })
        })
        .filter(|candidate| seen.insert((candidate.name.clone(), candidate.file.clone())))
        .cloned()
        .collect()
}

fn template_source(
    state: &McpState,
    link: &crate::layers::meta::semantic::SemanticEdge,
) -> Option<String> {
    if link.object.name.starts_with("inline:") {
        let declaring_file = link.subject.file.as_deref()?;
        let source = state.read_source(declaring_file).ok()?;
        inline_template(&source, &link.subject.name)
    } else {
        state
            .read_source(&link.object.name)
            .ok()
            .map(|source| source.to_string())
    }
}

fn inline_template(source: &str, component_name: &str) -> Option<String> {
    let language = crate::compression::language::safe_typescript_language()?;
    let captures = crate::compression::capture_pipeline::run_capture_pipeline_nodes(
        language,
        crate::queries::TS_QUERY,
        source,
        Fidelity::Low,
        |_, raw, _| Some(raw.to_string()),
    )
    .ok()?;
    captures
        .iter()
        .filter(|capture| capture.name == "class.root")
        .find_map(|capture| {
            let entry = capture.to_entry();
            let raw_class = crate::meta_util::class_source_from_capture(source, &entry);
            let (name, kind, _, _, _) =
                crate::angular_meta::decorators::extract_graph_entries(raw_class)?;
            if name != component_name
                || kind != crate::angular_meta::decorators::ClassKind::Component
            {
                return None;
            }
            crate::angular_meta::decorators::extract_decorators(raw_class, Fidelity::Low)?
                .inline_template
        })
}
