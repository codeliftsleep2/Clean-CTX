//! Question-demanded immutable string routes, interpreted from retained facts.
//! No filesystem reads, arbitrary evaluation or mutable initializer promotion.
use super::declaration_facets::{ExpressionFacet, ExpressionValue};
use super::{DeclarationId, Resolution, TypeScriptAuthority};
use std::collections::BTreeSet;

impl TypeScriptAuthority {
    pub(crate) fn string_value(&self, expression: &ExpressionFacet) -> Option<Vec<u16>> {
        self.string_value_inner(expression, &mut BTreeSet::new(), None)
    }

    pub(crate) fn demanded_string_value(
        &self,
        source: &std::path::Path,
        hash: &str,
        expression: &ExpressionFacet,
    ) -> Option<Vec<u16>> {
        self.string_value_inner(expression, &mut BTreeSet::new(), Some((source, hash)))
    }

    fn string_value_inner(
        &self,
        expression: &ExpressionFacet,
        active: &mut BTreeSet<DeclarationId>,
        qualified: Option<(&std::path::Path, &str)>,
    ) -> Option<Vec<u16>> {
        match &expression.value {
            ExpressionValue::StringValue(units) => {
                if let Some((source, hash)) = qualified {
                    let super::demanded_occurrences::DemandedValue::Literal { kind, text } =
                        self.demanded_at(source, hash, expression.start, expression.end)?
                    else {
                        return None;
                    };
                    if kind != "string"
                        || super::string_literal::decode(text).as_ref() != Some(units)
                    {
                        return None;
                    }
                }
                Some(units.clone())
            }
            ExpressionValue::Reference(Resolution::Resolved(id))
                if id.kind == "variable_declarator" && active.insert(id.clone()) =>
            {
                if let Some((source, hash)) = qualified
                    && self.demanded_reference_at(source, hash, expression.start, expression.end)
                        != Some(id)
                {
                    active.remove(id);
                    return None;
                }
                let mut facets = self
                    .declaration_facets
                    .iter()
                    .filter(|facet| facet.declaration == *id);
                let result = facets.next().and_then(|facet| {
                    if facets.next().is_some() {
                        return None;
                    }
                    // Only immutable const declarations receive an initializer
                    // facet from the authorized native snapshot producer.
                    facet.initializer.as_ref().and_then(|initializer| {
                        if qualified.is_some()
                            && (initializer.start < id.start
                                || initializer.start >= initializer.end
                                || initializer.end > id.end)
                        {
                            return None;
                        }
                        self.string_value_inner(
                            initializer,
                            active,
                            qualified.map(|_| (id.source.as_path(), id.source_hash.as_str())),
                        )
                    })
                });
                active.remove(id);
                result
            }
            _ => None,
        }
    }
}
