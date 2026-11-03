//! Private additive immediate mapping joins. No framework role or public key contract.
use super::*;
use crate::ir::typescript_resolution::{TypeScriptAuthority, Witness};
type Span = (usize, usize);
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct ResolveMapping {
    pub source: std::path::PathBuf,
    pub source_hash: String,
    pub owner: Span,
    pub field_pair: Span,
    pub field_key: Span,
    pub object: Span,
    pub pair: Span,
    pub key: Span,
    pub name: String,
    pub value: Span,
}
fn span(n: tree_sitter::Node<'_>) -> Span {
    (n.start_byte(), n.end_byte())
}
fn contains(a: Span, b: Span) -> bool {
    a.0 <= b.0 && b.0 < b.1 && b.1 <= a.1
}
fn named_key(n: tree_sitter::Node<'_>, source: &str) -> Option<String> {
    let raw = n.utf8_text(source.as_bytes()).ok()?;
    match n.kind() {
        "property_identifier" | "identifier" => Some(raw.to_owned()),
        "string" => super::super::string_value::decode(raw),
        _ => None,
    }
}
fn reference_syntax(n: tree_sitter::Node<'_>) -> bool {
    n.kind() == "identifier"
        || (n.kind() == "member_expression"
            && n.named_child_count() == 2
            && n.child_by_field_name("object")
                .is_some_and(|v| v.kind() == "identifier" || reference_syntax(v))
            && n.child_by_field_name("property")
                .is_some_and(|v| v.kind() == "property_identifier"))
}
impl Demand<'_> {
    fn key_occurrence(&mut self, n: tree_sitter::Node<'_>, file: &Path, source: &str) {
        self.occurrence(
            n,
            file,
            source,
            DemandedValue::Literal {
                kind: format!("mapping-key:{}", n.kind()),
                text: n
                    .utf8_text(source.as_bytes())
                    .unwrap_or_default()
                    .to_owned(),
            },
        );
    }
    fn pair_occurrence(
        &mut self,
        pair: tree_sitter::Node<'_>,
        key: tree_sitter::Node<'_>,
        value: tree_sitter::Node<'_>,
        file: &Path,
        source: &str,
    ) {
        self.key_occurrence(key, file, source);
        self.occurrence(
            pair,
            file,
            source,
            DemandedValue::Structure {
                kind: "explicit-mapping".into(),
                children: vec![
                    ("key".into(), key.start_byte(), key.end_byte()),
                    ("value".into(), value.start_byte(), value.end_byte()),
                ],
            },
        );
    }
    pub(super) fn resolve_object(
        &mut self,
        owner: tree_sitter::Node<'_>,
        field: tree_sitter::Node<'_>,
        object: tree_sitter::Node<'_>,
        file: &Path,
        source: &str,
    ) -> ExpressionFacet {
        let field_key = field.child_by_field_name("key").unwrap();
        self.pair_occurrence(field, field_key, object, file, source);
        let mut fields = Vec::new();
        let mut cursor = object.walk();
        for pair in object
            .named_children(&mut cursor)
            .filter(|n| n.kind() != "comment")
        {
            let nodes = pair
                .child_by_field_name("key")
                .zip(pair.child_by_field_name("value"));
            if pair.kind() == "pair"
                && let Some((key, value)) = nodes
                && let Some(name) = named_key(key, source)
                && reference_syntax(value)
                && !pair.has_error()
            {
                self.pair_occurrence(pair, key, value, file, source);
                let expression = self.expression(value, file, source);
                self.mappings.push(ResolveMapping {
                    source: file.to_path_buf(),
                    source_hash: crate::cache::content_hash(source.as_bytes()),
                    owner: span(owner),
                    field_pair: span(field),
                    field_key: span(field_key),
                    object: span(object),
                    pair: span(pair),
                    key: span(key),
                    name: name.clone(),
                    value: span(value),
                });
                fields.push((name, expression));
            } else {
                // Preserve unsupported occurrence, without recursively acquiring its contents.
                fields.push((
                    String::new(),
                    ExpressionFacet {
                        start: pair.start_byte(),
                        end: pair.end_byte(),
                        value: ExpressionValue::Unsupported(pair.kind().into()),
                    },
                ));
            }
        }
        self.occurrence(
            object,
            file,
            source,
            DemandedValue::Structure {
                kind: "object".into(),
                children: fields
                    .iter()
                    .map(|(n, v)| (n.clone(), v.start, v.end))
                    .collect(),
            },
        );
        ExpressionFacet {
            start: object.start_byte(),
            end: object.end_byte(),
            value: ExpressionValue::Object(fields),
        }
    }
}
fn find<'a>(v: &'a ExpressionFacet, wanted: Span, out: &mut Vec<&'a ExpressionFacet>) {
    if (v.start, v.end) == wanted {
        out.push(v);
    }
    match &v.value {
        ExpressionValue::Array(vs) => {
            for v in vs {
                find(v, wanted, out);
            }
        }
        ExpressionValue::Object(fs) => {
            for (_, v) in fs {
                find(v, wanted, out);
            }
        }
        _ => {}
    }
}
impl TypeScriptAuthority {
    /// Exact replay of retained language joins; no source parsing or resolution fallback.
    pub(crate) fn resolve_mapping_at(
        &self,
        pair: Span,
    ) -> Option<(&ResolveMapping, &DeclarationId)> {
        let mut proofs = self
            .resolve_mappings
            .iter()
            .filter(|p| p.pair == pair && p.source == self.source);
        let p = proofs.next()?;
        if proofs.next().is_some()
            || p.source_hash != self.source_hash
            || !self.capture_current
            || self.disposition.is_some()
            || !self.witnesses.iter().all(Witness::current)
        {
            return None;
        }
        for (outer, inner) in [
            (p.owner, p.field_pair),
            (p.field_pair, p.field_key),
            (p.field_pair, p.object),
            (p.object, p.pair),
            (p.pair, p.key),
            (p.pair, p.value),
        ] {
            if !contains(outer, inner) {
                return None;
            }
        }
        if p.field_key.1 > p.object.0 || p.key.1 > p.value.0 {
            return None;
        }
        let decode = |s: Span| -> Option<String> {
            let DemandedValue::Literal { kind, text } =
                self.demanded_at(&p.source, &p.source_hash, s.0, s.1)?
            else {
                return None;
            };
            match kind.as_str() {
                "mapping-key:property_identifier" | "mapping-key:identifier" => Some(text.clone()),
                "mapping-key:string" => super::super::string_value::decode(text),
                _ => None,
            }
        };
        if decode(p.field_key)? != "resolve" || decode(p.key)? != p.name {
            return None;
        }
        for (s, key, value) in [
            (p.field_pair, p.field_key, p.object),
            (p.pair, p.key, p.value),
        ] {
            let expected = DemandedValue::Structure {
                kind: "explicit-mapping".into(),
                children: vec![
                    ("key".into(), key.0, key.1),
                    ("value".into(), value.0, value.1),
                ],
            };
            if self.demanded_at(&p.source, &p.source_hash, s.0, s.1) != Some(&expected) {
                return None;
            }
        }
        let mut owners = Vec::new();
        for f in self.declaration_facets.iter().filter(|f| {
            f.declaration.source == p.source && f.declaration.source_hash == p.source_hash
        }) {
            if let Some(v) = &f.initializer {
                find(v, p.owner, &mut owners);
            }
            for d in &f.decorators {
                for v in &d.arguments {
                    find(v, p.owner, &mut owners);
                }
            }
        }
        if owners.len() != 1 {
            return None;
        }
        let owner = owners[0];
        let ExpressionValue::Object(fields) = &owner.value else {
            return None;
        };
        if !self.admits_demanded_structure(&p.source, &p.source_hash, owner) {
            return None;
        }
        let mut nested = fields
            .iter()
            .filter(|(n, v)| n == "resolve" && (v.start, v.end) == p.object);
        let (_, object) = nested.next()?;
        if nested.next().is_some() {
            return None;
        }
        if !self.admits_demanded_structure(&p.source, &p.source_hash, object) {
            return None;
        }
        let ExpressionValue::Object(fields) = &object.value else {
            return None;
        };
        if fields
            .iter()
            .filter(|(n, v)| n == &p.name && (v.start, v.end) == p.value)
            .count()
            != 1
        {
            return None;
        }
        // An independently valid key, pair, value or field cannot be borrowed.
        if self
            .resolve_mappings
            .iter()
            .filter(|q| {
                q.source == p.source
                    && q.source_hash == p.source_hash
                    && (q.key == p.key || q.value == p.value || q.pair == p.pair)
            })
            .count()
            != 1
        {
            return None;
        }
        let Some(DemandedValue::Reference(Resolution::Resolved(id))) =
            self.demanded_at(&p.source, &p.source_hash, p.value.0, p.value.1)
        else {
            return None;
        };
        let value = &fields
            .iter()
            .find(|(n, v)| n == &p.name && (v.start, v.end) == p.value)?
            .1;
        if !matches!(&value.value,ExpressionValue::Reference(Resolution::Resolved(target)) if target==id)
        {
            return None;
        }
        if id.start >= id.end || (id.source == self.source && id.source_hash != self.source_hash) {
            return None;
        }
        if id.source!=self.source && !self.witnesses.iter().any(|w|matches!(w,Witness::Content {path,hash} if path==&id.source && hash==&id.source_hash)) {return None;}
        Some((p, id))
    }
}
