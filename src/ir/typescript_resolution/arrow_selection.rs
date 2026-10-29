//! Demand-scoped written import/then/value-export selection, not callback execution.
use super::declaration_facets::{ExpressionFacet, ExpressionValue};
use super::demanded_occurrences::DemandedValue;
use super::{DeclarationId, TypeScriptAuthority};
use std::path::PathBuf;
mod acquire;
pub(super) use acquire::capture;
pub(crate) type Span = (usize, usize);
type Structure = (Span, &'static str, Vec<(String, usize, usize)>);
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct ArrowSelection {
    pub source: PathBuf,
    pub source_hash: String,
    pub outer: Span,
    pub parameters: Span,
    pub returned: Span,
    pub access: Span,
    pub import: Span,
    pub import_keyword: Span,
    pub literal: Span,
    pub then_property: Span,
    pub selector: Span,
    pub parameter: Span,
    pub member: Span,
    pub receiver: Span,
    pub property: Span,
    pub module: PathBuf,
    pub specifier: String,
    pub export: String,
    pub target: DeclarationId,
}
impl ArrowSelection {
    pub(super) fn structures(&self) -> Vec<Structure> {
        let child = |name: &str, s: Span| (name.to_owned(), s.0, s.1);
        vec![
            (
                self.outer,
                "arrow-selection",
                vec![
                    child("zero-parameters", self.parameters),
                    child("expression", self.returned),
                ],
            ),
            (
                self.returned,
                "call",
                vec![
                    child("callee", self.access),
                    child("argument:0", self.selector),
                ],
            ),
            (
                self.access,
                "member",
                vec![
                    child("receiver", self.import),
                    child("then", self.then_property),
                ],
            ),
            (
                self.import,
                "dynamic-import",
                vec![
                    child("callee", self.import_keyword),
                    child("argument:0", self.literal),
                ],
            ),
            (
                self.selector,
                "selector-arrow",
                vec![
                    child("parameter", self.parameter),
                    child("expression", self.member),
                ],
            ),
            (
                self.member,
                "selected-member",
                vec![
                    child("receiver", self.receiver),
                    child(&format!("export:{}", self.export), self.property),
                ],
            ),
            (
                self.receiver,
                "selector-binding",
                vec![
                    child("selector", self.selector),
                    child("declaration", self.parameter),
                ],
            ),
        ]
    }
}
fn contains(a: Span, b: Span) -> bool {
    a.0 <= b.0 && b.0 < b.1 && b.1 <= a.1
}
fn field<'a>(
    v: &'a ExpressionFacet,
    span: Span,
    out: &mut Vec<(&'a ExpressionFacet, &'a ExpressionFacet)>,
) {
    match &v.value {
        ExpressionValue::Object(fields) => {
            for (name, value) in fields {
                if name == "loadComponent" && (value.start, value.end) == span {
                    out.push((v, value));
                }
                field(value, span, out);
            }
        }
        ExpressionValue::Array(values) => {
            for value in values {
                field(value, span, out);
            }
        }
        _ => {}
    }
}
impl TypeScriptAuthority {
    /// Replay consumed evidence only. No resolver or filesystem selection fallback.
    pub(crate) fn arrow_selection_at(&self, span: Span) -> Option<&ArrowSelection> {
        let mut found = self.arrow_selections.iter().filter(|p| p.outer == span);
        let p = found.next()?;
        if found.next().is_some()
            || p.source != self.source
            || p.source_hash != self.source_hash
            || !self.capture_current
            || self.disposition.is_some()
            || !self.witnesses.iter().all(super::Witness::current)
        {
            return None;
        }
        let mut fields = Vec::new();
        for facet in self.declaration_facets.iter().filter(|f| {
            f.declaration.source == p.source && f.declaration.source_hash == p.source_hash
        }) {
            if let Some(value) = &facet.initializer {
                field(value, span, &mut fields);
            }
            for d in &facet.decorators {
                for value in &d.arguments {
                    field(value, span, &mut fields);
                }
            }
        }
        if fields.len() != 1 {
            return None;
        }
        let (object, value) = fields[0];
        if !contains((object.start, object.end), p.outer)
            || !matches!(&value.value,ExpressionValue::Unsupported(kind) if kind=="arrow_function")
            || !self.admits_demanded_structure(&p.source, &p.source_hash, object)
        {
            return None;
        }
        for (parent, child) in [
            (p.outer, p.parameters),
            (p.outer, p.returned),
            (p.returned, p.access),
            (p.returned, p.selector),
            (p.access, p.import),
            (p.access, p.then_property),
            (p.import, p.import_keyword),
            (p.import, p.literal),
            (p.selector, p.parameter),
            (p.selector, p.member),
            (p.member, p.receiver),
            (p.member, p.property),
        ] {
            if !contains(parent, child) {
                return None;
            }
        }
        if p.outer.0 != p.parameters.0 {
            return None;
        }
        if p.parameters.1 > p.returned.0
            || p.access.0 != p.import.0
            || p.import.1 > p.then_property.0
            || p.access.1 > p.selector.0
            || p.parameter.1 > p.member.0
            || p.receiver.1 > p.property.0
            || p.member.1 != p.selector.1
            || p.returned.1 != p.outer.1
        {
            return None;
        }
        for (span, kind, children) in p.structures() {
            if !matches!(self.demanded_at(&p.source,&p.source_hash,span.0,span.1),Some(DemandedValue::Structure {kind:k,children:c}) if k==kind&&c==&children)
            {
                return None;
            }
        }
        if !matches!(self.demanded_at(&p.source,&p.source_hash,p.parameters.0,p.parameters.1),Some(DemandedValue::Literal{kind,text}) if kind=="zero-parameters"&&text=="()")
        {
            return None;
        }
        if !matches!(self.demanded_at(&p.source,&p.source_hash,p.then_property.0,p.then_property.1),Some(DemandedValue::Literal{kind,text}) if kind=="property_identifier"&&text=="then")
        {
            return None;
        }
        if !matches!(self.demanded_at(&p.source,&p.source_hash,p.literal.0,p.literal.1),Some(DemandedValue::Literal{kind,text}) if kind=="string"&&super::string_value::decode(text).as_deref()==Some(p.specifier.as_str()))
        {
            return None;
        }
        if self.demanded_reference_at(&p.source, &p.source_hash, p.property.0, p.property.1)
            != Some(&p.target)
        {
            return None;
        }
        if !(p.specifier.starts_with("./") || p.specifier.starts_with("../")) {
            return None;
        }
        if !matches!(self.demanded_at(&p.source,&p.source_hash,p.parameter.0,p.parameter.1),Some(DemandedValue::Literal{kind,..}) if kind=="selector-parameter")
        {
            return None;
        }
        if !matches!(self.demanded_at(&p.source,&p.source_hash,p.import_keyword.0,p.import_keyword.1),Some(DemandedValue::Literal{kind,text}) if kind=="import"&&text=="import")
        {
            return None;
        }
        let mut modules = self
            .module_requests
            .iter()
            .filter(|m| (m.start, m.end) == p.literal);
        let module = modules.next()?;
        if modules.next().is_some()
            || !module.mode_valid
            || module.kind != super::module_resolution::ModuleRequestKind::Import
            || module.specifier != p.specifier
            || module.resolved.as_ref().ok() != Some(&p.module)
        {
            return None;
        }
        let witnessed = |file: &std::path::Path, hash: Option<&str>| {
            self.witnesses.iter().any(|w|matches!(w,super::Witness::Content{path,hash:h} if path==file&&hash.is_none_or(|expected|h==expected)))
        };
        if !witnessed(&p.module, None) || !witnessed(&p.target.source, Some(&p.target.source_hash))
        {
            return None;
        }
        Some(p)
    }
}
