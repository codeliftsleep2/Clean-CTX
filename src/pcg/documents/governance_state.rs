//! Exact source-owned protected state; offsets and incidental source hashes are not policy.
use super::{membership, model::Candidate, profile::Trusted, scan::Capture};
use crate::pcg::{Error, Result, identity, require};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

pub(super) const PROTECTED: &str = "docs/architecture/knowledge-governance/protected.csv";

pub(super) fn read(capture: &mut Capture, trusted: &Trusted) -> Result<Value> {
    let corpus = capture.read(trusted, membership::REGISTRY)?;
    let membership =
        membership::parse(&corpus, trusted).map_err(|_| Error("governance_membership"))?;
    let bytes = capture.read(trusted, PROTECTED)?;
    require(
        !bytes.starts_with(&[0xef, 0xbb, 0xbf]),
        "governance_protection_encoding",
    )?;
    let text = std::str::from_utf8(&bytes).map_err(|_| Error("governance_protection_encoding"))?;
    let mut lines = text.lines();
    require(
        membership::columns(lines.next().unwrap_or_default())
            == Some(vec![
                "id".into(),
                "role".into(),
                "label".into(),
                "selector".into(),
            ]),
        "governance_protection_schema",
    )?;
    let mut selectors = BTreeSet::new();
    let mut protected = Vec::new();
    for line in lines {
        let fields = membership::columns(line).ok_or(Error("governance_protection_schema"))?;
        let [id, role, label, selector] = fields.as_slice() else {
            return Err(Error("governance_protection_schema"));
        };
        require(
            selectors.insert((id.clone(), role.clone(), label.clone(), selector.clone())),
            "governance_protection_duplicate",
        )?;
        protected.push(json!({"id":id,"role":role,"label":if label.is_empty(){Value::Null}else{json!(label)},"selector":selector}));
    }
    protected.sort_by_key(identity::canonical);
    let view = project(&capture.nodes, &protected, false)?;
    Ok(json!({"membership":membership,"protection":protected,"protected":view}))
}

pub(super) fn project(
    nodes: &BTreeMap<String, Candidate>,
    selectors: &[Value],
    observe_previous: bool,
) -> Result<Value> {
    project_with_work(nodes, selectors, observe_previous).map(|(value, _)| value)
}

/// Count actual association derivations, not the number of emitted components.
pub(super) fn project_with_work(
    nodes: &BTreeMap<String, Candidate>,
    selectors: &[Value],
    observe_previous: bool,
) -> Result<(Value, usize)> {
    let mut result = Vec::new();
    let mut units = BTreeMap::new();
    let mut association_derivations = 0;
    for selected in selectors {
        let id = selected["id"]
            .as_str()
            .ok_or(Error("governance_protection_schema"))?;
        let Some(node) = nodes.get(id) else {
            require(observe_previous, "governance_protection_identity")?;
            result.push(json!({"selection":selected,"missing_identity":true}));
            continue;
        };
        let Some(knowledge) = node.knowledge.as_ref() else {
            require(observe_previous, "governance_protection_unrepresented")?;
            result.push(json!({"selection":selected,"missing_representation":true}));
            continue;
        };
        let found: Vec<_> = knowledge
            .components
            .iter()
            .filter(|component| {
                json!(component.binding.role) == selected["role"]
                    && json!(component.binding.label) == selected["label"]
                    && json!(component.binding.selector) == selected["selector"]
            })
            .collect();
        if found.is_empty() && observe_previous {
            // Describe the removal against accepted obligations. This observation
            // grants nothing: proposed selectors remain strict and the changed
            // state still requires exact predecessor-bound operator authorization.
            result.push(json!({"selection":selected,"missing_component":true}));
        } else {
            require(found.len() == 1, "governance_protection_component")?;
            let component = found[0];
            result.push(json!({"selection":selected,"path":component.binding.path,"content_sha256":component.content_sha}));
        }
        // Derive the complete governing association set once per protected unit.
        units.entry(id).or_insert_with(|| {
            association_derivations += 1;
            let mut qualifications: Vec<_> = knowledge.components.iter()
                .filter(|component| component.binding.role == "qualification")
                .map(|component| json!({"role":component.binding.role,"label":component.binding.label,
                    "selector":component.binding.selector,"path":component.binding.path,"content_sha256":component.content_sha}))
                .collect();
            qualifications.sort_by_key(identity::canonical);
            let relationships: BTreeSet<_> = node.declaration.relationships.iter()
                .map(|edge| (edge.kind.clone(), edge.target.clone())).collect();
            // Declaration and component artifact source hashes include unrelated
            // source bytes. Protect selected content above, not those source hashes.
            let component_paths: BTreeSet<_> = knowledge.components.iter().map(|component| component.binding.path.as_str()).collect();
            let mut evidence: Vec<_> = node.evidence.iter().filter(|association|
                association["kind"] == "artifact" && association["path"].as_str()
                    .is_some_and(|path| path != node.path && !component_paths.contains(path)))
                .map(|association| json!({"path":association["path"],"sha256":association["sha256"]})).collect();
            evidence.sort_by_key(identity::canonical);
            json!({"id":id,"owner":node.path,"kind":node.declaration.kind,"status":node.declaration.status,
                "scope":knowledge.scope,"relationships":relationships,"qualifications":qualifications,"evidence":evidence})
        });
    }
    Ok((
        json!({"components":result,"units":units.into_values().collect::<Vec<_>>()}),
        association_derivations,
    ))
}
