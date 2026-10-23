//! Validate retained relation endpoints against their consumed native capture.
use super::{DeclarationId, TypeScriptAuthority};
use crate::layers::meta::canonical_endpoint::CanonicalEndpoint;
use crate::layers::meta::semantic::{EntityRef, SemanticEdge};

impl TypeScriptAuthority {
    pub(crate) fn admits_relation_evidence(&self, edges: &[SemanticEdge]) -> bool {
        edges.iter().all(|edge| {
            let (subject, object) = match (&edge.subject.canonical, &edge.object.canonical) {
                (None, None) => return true,
                (Some(subject), Some(object)) => (subject, object),
                _ => return false,
            };
            let CanonicalEndpoint::Declaration { declaration, site } = subject else {
                return false;
            };
            declaration.source == self.source
                && declaration.source_hash == self.source_hash
                && site.is_some_and(|(start, end)| start < end && end <= declaration.end)
                && self.admits_endpoint(&edge.subject, subject)
                && self.admits_endpoint(&edge.object, object)
        })
    }

    fn admits_endpoint(&self, entity: &EntityRef, endpoint: &CanonicalEndpoint) -> bool {
        let declaration = match endpoint {
            CanonicalEndpoint::Declaration { declaration, .. } => declaration,
            CanonicalEndpoint::StringValue {
                string_value,
                proof,
            } => {
                return entity.name
                    == crate::layers::meta::canonical_endpoint::string_label(string_value)
                    && entity.file.as_deref() == proof.source.to_str()
                    && proof.source == self.source
                    && proof.source_hash == self.source_hash
                    && proof.start < proof.end
                    && self.admits_literal(proof, string_value);
            }
        };
        if entity.name != declaration.name || entity.file.as_deref() != declaration.source.to_str()
        {
            return false;
        }
        let matches = |id: &DeclarationId| {
            id.source == declaration.source
                && id.source_hash == declaration.source_hash
                && id.start == declaration.start
                && id.end == declaration.end
                && id.name == declaration.name
                && id.kind == declaration.kind
        };
        self.constructor_parameters.iter().any(|parameter|
            parameter.runtime_class().is_some_and(matches))
            || self.declaration_facets
            .iter()
            .any(|facet| matches(&facet.declaration))
            || self.reference_sites.iter().any(|(_, _, resolution)| {
                matches!(resolution, super::Resolution::Resolved(id) if matches(id))
            })
            || self
                .package_exports
                .iter()
                .any(|package| matches(&package.declaration))
    }

    fn admits_literal(
        &self,
        proof: &crate::layers::meta::canonical_endpoint::LiteralProof,
        units: &[u16],
    ) -> bool {
        use super::declaration_facets::{ExpressionFacet, ExpressionValue};
        fn contains(
            authority: &TypeScriptAuthority,
            expression: &ExpressionFacet,
            start: usize,
            end: usize,
            units: &[u16],
        ) -> bool {
            if expression.start == start && expression.end == end {
                return authority.string_value(expression).as_deref() == Some(units);
            }
            match &expression.value {
                ExpressionValue::Array(items) => items
                    .iter()
                    .any(|item| contains(authority, item, start, end, units)),
                ExpressionValue::Object(fields) => fields
                    .iter()
                    .any(|(_, item)| contains(authority, item, start, end, units)),
                ExpressionValue::Spread(value) => contains(authority, value, start, end, units),
                _ => false,
            }
        }
        self.constructor_parameters
            .iter()
            .filter(|parameter| {
                parameter.owner.source == proof.source
                    && parameter.owner.source_hash == proof.source_hash
            })
            .flat_map(|parameter| &parameter.decorator_facets)
            .flat_map(|decorator| &decorator.arguments)
            .any(|argument| contains(self, argument, proof.start, proof.end, units))
            || self
                .declaration_facets
                .iter()
                .filter(|facet| {
                    facet.declaration.source == proof.source
                        && facet.declaration.source_hash == proof.source_hash
                })
                .flat_map(|facet| &facet.decorators)
                .flat_map(|decorator| &decorator.arguments)
                .any(|argument| contains(self, argument, proof.start, proof.end, units))
    }
}

impl From<&DeclarationId> for crate::layers::meta::canonical_endpoint::CanonicalDeclaration {
    fn from(id: &DeclarationId) -> Self {
        Self {
            source: id.source.clone(),
            source_hash: id.source_hash.clone(),
            start: id.start,
            end: id.end,
            name: id.name.clone(),
            kind: id.kind.clone(),
        }
    }
}
