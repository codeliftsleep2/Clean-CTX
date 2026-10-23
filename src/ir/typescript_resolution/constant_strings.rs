//! Question-demanded immutable string routes, interpreted from retained facts.
//! No filesystem reads, arbitrary evaluation or mutable initializer promotion.
use super::declaration_facets::{ExpressionFacet, ExpressionValue};
use super::{DeclarationId, Resolution, TypeScriptAuthority};
use std::collections::BTreeSet;

impl TypeScriptAuthority {
    pub(crate) fn string_value(&self, expression: &ExpressionFacet) -> Option<Vec<u16>> {
        self.string_value_inner(expression, &mut BTreeSet::new())
    }

    fn string_value_inner(
        &self,
        expression: &ExpressionFacet,
        active: &mut BTreeSet<DeclarationId>,
    ) -> Option<Vec<u16>> {
        match &expression.value {
            ExpressionValue::StringValue(units) => Some(units.clone()),
            ExpressionValue::Reference(Resolution::Resolved(id))
                if id.kind == "variable_declarator" && active.insert(id.clone()) =>
            {
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
                    facet
                        .initializer
                        .as_ref()
                        .and_then(|initializer| self.string_value_inner(initializer, active))
                });
                active.remove(id);
                result
            }
            _ => None,
        }
    }
}
