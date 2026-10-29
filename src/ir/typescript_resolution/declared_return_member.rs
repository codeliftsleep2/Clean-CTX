//! Demand-scoped explicit callable return and directly owned instance method joins.
use super::declaration_facets::ExpressionValue;
use super::demanded_occurrences::{DemandedOccurrence, DemandedValue};
use super::{DeclarationId, Resolution, Resolver, TypeScriptAuthority};
use crate::ir::invocation_evidence::SourceSpan;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DeclaredReturnMember {
    pub source: PathBuf,
    pub source_hash: String,
    pub outer: (usize, usize),
    pub access: (usize, usize),
    pub inner: (usize, usize),
    pub factory_site: (usize, usize),
    pub property: (usize, usize),
    pub factory: DeclarationId,
    pub return_site: (usize, usize),
    pub receiver: DeclarationId,
    pub method: DeclarationId,
}
fn span(node: tree_sitter::Node<'_>) -> (usize, usize) {
    (node.start_byte(), node.end_byte())
}
fn exact<'a>(node: tree_sitter::Node<'a>, id: &DeclarationId) -> Option<tree_sitter::Node<'a>> {
    if span(node) == (id.start, id.end) && node.kind() == id.kind {
        return Some(node);
    }
    let mut cursor = node.walk();
    node.named_children(&mut cursor)
        .filter(|n| n.start_byte() <= id.start && id.end <= n.end_byte())
        .find_map(|n| exact(n, id))
}
fn occurrence(
    id: &DeclarationId,
    range: (usize, usize),
    value: DemandedValue,
) -> DemandedOccurrence {
    DemandedOccurrence {
        source: id.source.clone(),
        source_hash: id.source_hash.clone(),
        start: range.0,
        end: range.1,
        value,
    }
}
fn structure(kind: &str, children: Vec<(String, usize, usize)>) -> DemandedValue {
    DemandedValue::Structure {
        kind: kind.into(),
        children,
    }
}
fn join(
    resolver: &mut Resolver,
    file: &Path,
    source: &str,
    outer: tree_sitter::Node<'_>,
    selected: &[(usize, usize)],
) -> Option<(DeclaredReturnMember, Vec<DemandedOccurrence>)> {
    let access = outer.child_by_field_name("function")?;
    if access.kind() != "member_expression" {
        return None;
    }
    let inner = access.child_by_field_name("object")?;
    let property = access.child_by_field_name("property")?;
    if inner.kind() != "call_expression" || property.kind() != "property_identifier" {
        return None;
    }
    // Optional/computed access is never the admitted ordinary chain.
    for node in [outer, access, inner] {
        let mut c = node.walk();
        if node.children(&mut c).any(|n| n.kind() == "optional_chain") {
            return None;
        }
    }
    let factory_site = inner.child_by_field_name("function")?;
    if !selected.contains(&span(factory_site)) {
        return None;
    }
    let arguments = inner.child_by_field_name("arguments")?;
    let mut c = arguments.walk();
    if arguments
        .named_children(&mut c)
        .any(|n| n.kind() != "comment")
    {
        return None;
    }
    let Resolution::Resolved(factory) = resolver.value_reference_for_invocation(
        file,
        source,
        SourceSpan::new(factory_site.start_byte(), factory_site.end_byte()).ok()?,
    )?
    else {
        return None;
    };
    if !matches!(
        factory.kind.as_str(),
        "function_signature" | "function_declaration"
    ) {
        return None;
    }
    let factory_source = resolver.host.read(&factory.source)?;
    if crate::cache::content_hash(factory_source.as_bytes()) != factory.source_hash {
        return None;
    }
    let tree = resolver.source_tree(&factory.source)?;
    let declaration = exact(tree.root_node(), &factory)?;
    // Competing callable declarations never select a convenient signature.
    let mut pending = vec![tree.root_node()];
    let mut signature_count = 0;
    while let Some(node) = pending.pop() {
        if matches!(node.kind(), "function_signature" | "function_declaration")
            && node
                .child_by_field_name("name")
                .and_then(|n| n.utf8_text(factory_source.as_bytes()).ok())
                == Some(factory.name.as_str())
        {
            signature_count += 1;
        }
        let mut cursor = node.walk();
        pending.extend(node.named_children(&mut cursor));
    }
    if signature_count != 1 {
        return None;
    }

    if declaration.child_by_field_name("type_parameters").is_some() {
        return None;
    }
    let annotation = declaration.child_by_field_name("return_type")?;
    let mut c = annotation.walk();
    let refs = annotation.named_children(&mut c).collect::<Vec<_>>();
    let [return_ref] = refs.as_slice() else {
        return None;
    };
    if !matches!(
        return_ref.kind(),
        "type_identifier" | "nested_type_identifier"
    ) {
        return None;
    }
    let Resolution::Resolved(receiver) = resolver.type_reference(
        &factory.source,
        &factory_source,
        SourceSpan::new(return_ref.start_byte(), return_ref.end_byte()).ok()?,
    )?
    else {
        return None;
    };
    if !matches!(
        receiver.kind.as_str(),
        "class_declaration" | "abstract_class_declaration"
    ) {
        return None;
    }
    let receiver_source = resolver.host.read(&receiver.source)?;
    if crate::cache::content_hash(receiver_source.as_bytes()) != receiver.source_hash {
        return None;
    }
    let receiver_tree = resolver.source_tree(&receiver.source)?;
    let class = exact(receiver_tree.root_node(), &receiver)?;
    if class.child_by_field_name("type_parameters").is_some() {
        return None;
    }
    let body = class.child_by_field_name("body")?;
    let name = property.utf8_text(source.as_bytes()).ok()?;
    let mut c = body.walk();
    let members = body
        .named_children(&mut c)
        .filter(|n| {
            n.child_by_field_name("name")
                .and_then(|n| n.utf8_text(receiver_source.as_bytes()).ok())
                == Some(name)
        })
        .collect::<Vec<_>>();
    let [method] = members.as_slice() else {
        return None;
    };
    if !matches!(method.kind(), "method_signature" | "method_definition")
        || method.child_by_field_name("type_parameters").is_some()
    {
        return None;
    }
    let mut c = method.walk();
    if method.children(&mut c).any(|n| {
        matches!(
            n.kind(),
            "static" | "private" | "protected" | "?" | "get" | "set"
        ) || (n.kind() == "accessibility_modifier"
            && n.utf8_text(receiver_source.as_bytes()).ok() != Some("public"))
    }) {
        return None;
    }
    let method_id = DeclarationId {
        source: receiver.source.clone(),
        source_hash: receiver.source_hash.clone(),
        start: method.start_byte(),
        end: method.end_byte(),
        name: name.into(),
        kind: method.kind().into(),
    };
    let evidence = DeclaredReturnMember {
        source: resolver.host.normalized(file)?,
        source_hash: crate::cache::content_hash(source.as_bytes()),
        outer: span(outer),
        access: span(access),
        inner: span(inner),
        factory_site: span(factory_site),
        property: span(property),
        factory: factory.clone(),
        return_site: span(*return_ref),
        receiver: receiver.clone(),
        method: method_id.clone(),
    };
    let occurrences = vec![
        DemandedOccurrence {
            source: evidence.source.clone(),
            source_hash: evidence.source_hash.clone(),
            start: property.start_byte(),
            end: property.end_byte(),
            value: DemandedValue::Literal {
                kind: "member-property".into(),
                text: name.into(),
            },
        },
        occurrence(
            &factory,
            (factory.start, factory.end),
            structure(
                "explicit-callable-return",
                vec![(
                    "return".into(),
                    return_ref.start_byte(),
                    return_ref.end_byte(),
                )],
            ),
        ),
        occurrence(
            &factory,
            span(*return_ref),
            DemandedValue::TypeReference(Resolution::Resolved(receiver.clone())),
        ),
        occurrence(
            &receiver,
            (receiver.start, receiver.end),
            structure(
                "direct-instance-method-owner",
                vec![(name.into(), method.start_byte(), method.end_byte())],
            ),
        ),
        occurrence(
            &method_id,
            (method_id.start, method_id.end),
            structure(
                "direct-instance-method",
                vec![(
                    "name".into(),
                    method.child_by_field_name("name")?.start_byte(),
                    method.child_by_field_name("name")?.end_byte(),
                )],
            ),
        ),
    ];
    Some((evidence, occurrences))
}
pub(super) fn capture(
    resolver: &mut Resolver,
    file: &Path,
    source: &str,
    selected: &[(usize, usize)],
) -> (Vec<DeclaredReturnMember>, Vec<DemandedOccurrence>) {
    if selected.is_empty() {
        return (Vec::new(), Vec::new());
    }
    let Some(tree) = resolver.source_tree(file) else {
        return (Vec::new(), Vec::new());
    };
    let mut pending = vec![tree.root_node()];
    let mut proofs = Vec::new();
    let mut occurrences = Vec::new();
    while let Some(node) = pending.pop() {
        let mut c = node.walk();
        pending.extend(node.named_children(&mut c));
        if node.kind() == "call_expression"
            && !node.has_error()
            && let Some((proof, sites)) = join(resolver, file, source, node, selected)
        {
            proofs.push(proof);
            for site in sites {
                if !occurrences.contains(&site) {
                    occurrences.push(site);
                }
            }
        }
    }
    (proofs, occurrences)
}
impl TypeScriptAuthority {
    pub(crate) fn declared_return_member_at(
        &self,
        access: SourceSpan,
    ) -> Option<&DeclaredReturnMember> {
        let mut proofs = self
            .declared_return_members
            .iter()
            .filter(|p| p.access == (access.start_byte(), access.end_byte()));
        let p = proofs.next()?;
        if proofs.next().is_some()
            || p.source != self.source
            || p.source_hash != self.source_hash
            || !self.capture_current
            || !self.witnesses.iter().all(super::Witness::current)
        {
            return None;
        }
        let contains = |a: (usize, usize), b: (usize, usize)| a.0 <= b.0 && b.0 < b.1 && b.1 <= a.1;
        if p.outer.0 != p.access.0
            || p.access.0 != p.inner.0
            || p.inner.0 != p.factory_site.0
            || p.access.1 != p.property.1
            || !contains(p.outer, p.access)
            || !contains(p.access, p.inner)
            || !contains(p.inner, p.factory_site)
            || !contains(p.access, p.property)
            || p.inner.1 > p.property.0
            || !contains((p.factory.start, p.factory.end), p.return_site)
            || !contains(
                (p.receiver.start, p.receiver.end),
                (p.method.start, p.method.end),
            )
            || p.method.source != p.receiver.source
            || p.method.source_hash != p.receiver.source_hash
        {
            return None;
        }
        let unique = |range: (usize, usize)| {
            let mut it = self.invocation_facets.iter().filter(|f| {
                f.source == p.source
                    && f.source_hash == p.source_hash
                    && (f.expression.start, f.expression.end) == range
            });
            let f = it.next()?;
            it.next().is_none().then_some(&f.expression)
        };
        let outer = unique(p.outer)?;
        let inner = unique(p.inner)?;
        if !self.admits_demanded_structure(&p.source, &p.source_hash, outer)
            || !self.admits_demanded_structure(&p.source, &p.source_hash, inner)
        {
            return None;
        }
        let ExpressionValue::Call(member, args) = &outer.value else {
            return None;
        };
        if (member.start, member.end) != p.access
            || !self.admits_demanded_structure(&p.source, &p.source_hash, member)
        {
            return None;
        }
        let ExpressionValue::Member(receiver, name) = &member.value else {
            return None;
        };
        if receiver.as_ref() != inner
            || (receiver.start, receiver.end) != p.inner
            || name != &p.method.name
            || args.iter().any(|a| !contains(p.outer, (a.start, a.end)))
        {
            return None;
        }
        let ExpressionValue::Call(factory, inner_args) = &inner.value else {
            return None;
        };
        if !inner_args.is_empty()
            || (factory.start, factory.end) != p.factory_site
            || !matches!(&factory.value,ExpressionValue::Reference(Resolution::Resolved(id)) if id==&p.factory)
            || self.demanded_reference_at(
                &p.source,
                &p.source_hash,
                p.factory_site.0,
                p.factory_site.1,
            ) != Some(&p.factory)
        {
            return None;
        }
        if self.demanded_at(&p.source, &p.source_hash, p.property.0, p.property.1)
            != Some(&DemandedValue::Literal {
                kind: "member-property".into(),
                text: p.method.name.clone(),
            })
        {
            return None;
        }
        if !matches!(
            p.factory.kind.as_str(),
            "function_signature" | "function_declaration"
        ) || !matches!(
            p.receiver.kind.as_str(),
            "class_declaration" | "abstract_class_declaration"
        ) || !matches!(
            p.method.kind.as_str(),
            "method_signature" | "method_definition"
        ) {
            return None;
        }
        let callable = structure(
            "explicit-callable-return",
            vec![("return".into(), p.return_site.0, p.return_site.1)],
        );
        let owner = structure(
            "direct-instance-method-owner",
            vec![(p.method.name.clone(), p.method.start, p.method.end)],
        );
        if self.demanded_at(
            &p.factory.source,
            &p.factory.source_hash,
            p.factory.start,
            p.factory.end,
        ) != Some(&callable)
            || self.demanded_at(
                &p.factory.source,
                &p.factory.source_hash,
                p.return_site.0,
                p.return_site.1,
            ) != Some(&DemandedValue::TypeReference(Resolution::Resolved(
                p.receiver.clone(),
            )))
            || self.demanded_at(
                &p.receiver.source,
                &p.receiver.source_hash,
                p.receiver.start,
                p.receiver.end,
            ) != Some(&owner)
        {
            return None;
        }
        let Some(DemandedValue::Structure { kind, children }) = self.demanded_at(
            &p.method.source,
            &p.method.source_hash,
            p.method.start,
            p.method.end,
        ) else {
            return None;
        };
        if kind != "direct-instance-method"
            || children.len() != 1
            || children[0].0 != "name"
            || !contains(
                (p.method.start, p.method.end),
                (children[0].1, children[0].2),
            )
        {
            return None;
        }
        Some(p)
    }
}
