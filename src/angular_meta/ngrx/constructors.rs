//! Closed same-file constructor dispatch evidence; no module or general resolver.
use super::legacy::ClassActionEvidence;
use crate::compression::capture_pipeline::CapturedNode;
use crate::ir::invocation_evidence::{InvocationOccurrence, SourceSpan};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const QUERY: &str = r#"
(call_expression function: (member_expression property: (property_identifier) @ngrx.ctor.dispatch)
    arguments: (arguments . (new_expression) @ngrx.ctor.new)) @ngrx.ctor.call
(method_definition "static") @ngrx.ctor.static_method
(call_expression "?.") @ngrx.ctor.optional_call
(new_expression constructor: (identifier) @ngrx.ctor.endpoint) @ngrx.ctor.new_owner
(program [(class_declaration) @ngrx.ctor.program_owner
    (export_statement declaration: (class_declaration) @ngrx.ctor.program_owner)])
(class_declaration name: (type_identifier) @ngrx.ctor.class_name) @ngrx.ctor.class_owner
(class_declaration body: (class_body (method_definition
    name: (property_identifier) @ngrx.ctor.ctor_name
    parameters: (formal_parameters (required_parameter
        (accessibility_modifier) @ngrx.ctor.access
        pattern: (identifier) @ngrx.ctor.store_field
        type: (type_annotation (generic_type name: (type_identifier) @ngrx.ctor.store_type))))))) @ngrx.ctor.store_owner
(class_declaration body: (class_body (method_definition body: (statement_block
    (expression_statement (call_expression
        function: (member_expression !optional_chain
            object: (member_expression !optional_chain object: (this)
                property: (property_identifier) @ngrx.ctor.receiver)
            property: (property_identifier) @ngrx.ctor.terminal)
        arguments: (arguments . (new_expression constructor: (identifier) @ngrx.ctor.target) .)) @ngrx.ctor.owned_call))))) @ngrx.ctor.owner
(class_declaration body: (class_body (public_field_definition name: (_) @ngrx.ctor.other_member))) @ngrx.ctor.member_owner
(class_declaration body: (class_body (method_definition name: (_) @ngrx.ctor.other_member))) @ngrx.ctor.member_owner
(for_in_statement kind: ["var" "let" "const"]) @ngrx.ctor.unsupported

"#;

#[derive(Debug, Clone)]
pub(crate) struct ConstructorDispatch {
    pub start_byte: usize,
    pub file: Option<String>,
    pub source_hash: String,
    pub owner_name: Option<String>,
    pub action_name: Option<String>,
    // Exact declaration proof stays compilation-local; never a new public entity.
    #[allow(dead_code)]
    pub action_span: Option<SourceSpan>,
}
fn span(c: &CapturedNode) -> SourceSpan {
    SourceSpan::new(c.start_byte, c.end_byte).expect("ordered parser span")
}

pub(crate) fn collect(
    source: &str,
    file: Option<&str>,
    captures: &[CapturedNode],
    actions: &[ClassActionEvidence],
    calls: &[InvocationOccurrence],
) -> Vec<ConstructorDispatch> {
    let source_hash = crate::cache::content_hash(source.as_bytes());
    let mut matches: BTreeMap<usize, BTreeMap<&str, &CapturedNode>> = BTreeMap::new();
    let mut names: BTreeMap<&str, BTreeSet<SourceSpan>> = BTreeMap::new();
    for c in captures {
        matches.entry(c.match_index).or_default().insert(&c.name, c);
        if c.name == "ngrx.type.binding" {
            names.entry(&c.raw_text).or_default().insert(span(c));
        }
    }
    let plain: BTreeSet<_> = names.values().flatten().copied().collect();
    let unsupported = captures.iter().any(|c| {
        c.name == "ngrx.type.invalid"
            || c.name == "ngrx.ctor.unsupported"
            || (c.name == "ngrx.type.binding_pattern" && !plain.contains(&span(c)))
    }) || names.keys().any(|n| n.contains('\\'));
    let optional: BTreeSet<_> = captures
        .iter()
        .filter(|c| c.name == "ngrx.ctor.optional_call")
        .map(span)
        .collect();
    let program: BTreeSet<_> = captures
        .iter()
        .filter(|c| c.name == "ngrx.ctor.program_owner")
        .map(span)
        .collect();
    let canonical: BTreeMap<_, _> = captures
        .iter()
        .filter(|c| c.name == "class.root")
        .map(|c| (span(c), c.text.as_str()))
        .collect();
    let class_names: BTreeMap<_, _> = matches
        .values()
        .filter_map(|m| {
            Some((
                span(m.get("ngrx.ctor.class_owner")?),
                span(m.get("ngrx.ctor.class_name")?),
            ))
        })
        .collect();
    let endpoints: BTreeMap<_, _> = matches
        .values()
        .filter_map(|m| {
            Some((
                span(m.get("ngrx.ctor.new_owner")?),
                *m.get("ngrx.ctor.endpoint")?,
            ))
        })
        .collect();
    let imports: Vec<_> = matches
        .values()
        .filter_map(|m| {
            let name = m.get("ngrx.legacy.import_name")?;
            (name.raw_text == "Store")
                .then(|| m.get("ngrx.legacy.import_source").copied())
                .flatten()
        })
        .collect();
    let store_import = imports.len() == 1
        && imports[0].raw_text == "@ngrx/store"
        && names.get("Store").is_some_and(|s| s.len() == 1)
        && !captures
            .iter()
            .any(|c| c.name == "ngrx.legacy.local_binding" && c.raw_text == "Store");
    let mut stores: BTreeMap<SourceSpan, BTreeSet<(&str, SourceSpan)>> = BTreeMap::new();
    for m in matches.values() {
        if let (Some(owner), Some(ctor), Some(field), Some(ty)) = (
            m.get("ngrx.ctor.store_owner"),
            m.get("ngrx.ctor.ctor_name"),
            m.get("ngrx.ctor.store_field"),
            m.get("ngrx.ctor.store_type"),
        ) && ctor.raw_text == "constructor"
            && ty.raw_text == "Store"
            && program.contains(&span(owner))
        {
            stores
                .entry(span(owner))
                .or_default()
                .insert((&field.raw_text, span(field)));
        }
    }
    let mut members: BTreeMap<SourceSpan, BTreeSet<&str>> = BTreeMap::new();
    for m in matches.values() {
        if let (Some(owner), Some(member)) = (
            m.get("ngrx.ctor.member_owner"),
            m.get("ngrx.ctor.other_member"),
        ) {
            members
                .entry(span(owner))
                .or_default()
                .insert(&member.raw_text);
        }
    }
    let owned: BTreeMap<_, _> = matches
        .values()
        .filter_map(|m| {
            let call = m.get("ngrx.ctor.owned_call")?;
            let owner = m.get("ngrx.ctor.owner")?;
            let receiver = m.get("ngrx.ctor.receiver")?;
            let terminal = m.get("ngrx.ctor.terminal")?;
            (terminal.raw_text == "dispatch")
                .then_some((span(call), (span(owner), receiver.raw_text.as_str())))
        })
        .collect();
    let mut result = BTreeMap::new();
    for m in matches.values() {
        let (Some(call), Some(value), Some(terminal)) = (
            m.get("ngrx.ctor.call"),
            m.get("ngrx.ctor.new"),
            m.get("ngrx.ctor.dispatch"),
        ) else {
            continue;
        };
        if terminal.raw_text != "dispatch" {
            continue;
        }
        // Every syntactically direct constructor is recorded, even if authority
        // fails, so the raw summary cannot fabricate an Action named `new`.
        let mut record = ConstructorDispatch {
            start_byte: call.start_byte,
            file: file.map(Into::into),
            source_hash: source_hash.clone(),
            owner_name: None,
            action_name: None,
            action_span: None,
        };
        if !captures.iter().any(|c| {
            c.name == "ngrx.ctor.static_method"
                && c.start_byte <= call.start_byte
                && c.end_byte >= call.end_byte
        }) && !optional.contains(&span(call))
            && file.is_some()
            && !unsupported
            && store_import
            && let Some((owner, receiver)) = owned.get(&span(call))
            && program.contains(owner)
            && stores.get(owner).is_some_and(|fields| {
                fields.len() == 1 && fields.iter().any(|(name, _)| name == receiver)
            })
            && !members.get(owner).is_some_and(|fields| {
                fields.iter().any(|n| {
                    *n == *receiver
                        || n.starts_with('[')
                        || n.starts_with(char::from(39))
                        || n.starts_with('"')
                })
            })
            && let Some(endpoint) = endpoints.get(&span(value))
            && let Some(declarations) = names.get(endpoint.raw_text.as_str())
            && declarations.len() == 1
            && let Some(action) = actions
                .iter()
                .find(|a| class_names.get(&a.class_span) == declarations.first())
            && let Some(name) = canonical.get(owner)
            && calls.iter().any(|c| {
                c.span() == span(call)
                    && c.arguments().len() == 1
                    && c.arguments()[0].span() == span(value)
            })
        {
            record.owner_name = Some((*name).into());
            record.action_name = Some(action.name.clone());
            record.action_span = Some(action.class_span);
        }
        result.insert(call.start_byte, record);
    }
    result.into_values().collect()
}

/// Preserve modern/raw sites by exact source-start association, never by token
/// spelling. Constructor positions cannot manufacture Action declarations.
pub(crate) fn apply(
    mut shape: super::NgRxShape,
    starts: &[usize],
    records: &[ConstructorDispatch],
    source: &str,
    file: &str,
) -> super::NgRxShape {
    let hash = crate::cache::content_hash(source.as_bytes());
    let suppressed: BTreeSet<_> = records
        .iter()
        .filter(|r| r.source_hash == hash)
        .map(|r| r.start_byte)
        .collect();
    shape.dispatch_sites = shape
        .dispatch_sites
        .into_iter()
        .zip(starts)
        .filter_map(|(site, start)| (!suppressed.contains(start)).then_some(site))
        .collect();
    for record in records
        .iter()
        .filter(|r| r.source_hash == hash && r.file.as_deref() == Some(file))
    {
        if let (Some(owner), Some(action)) = (&record.owner_name, &record.action_name)
            && (shape.component_name.is_none() || shape.component_name.as_deref() == Some(owner))
        {
            shape.dispatch_sites.push(super::DispatchSite {
                action_name: action.clone(),
            });
        }
    }
    shape
}
