//! Validate retained relation endpoints against their consumed native capture.
use super::{DeclarationId, TypeScriptAuthority};
use crate::layers::meta::canonical_endpoint::CanonicalEndpoint;
use crate::layers::meta::semantic::{EntityRef, SemanticEdge};

impl TypeScriptAuthority {
    pub(crate) fn admits_relation_evidence(&self, edges: &[SemanticEdge]) -> bool {
        #[cfg(all(test, feature = "angular"))]
        let _observation = crate::angular_meta::admission_work_metrics::begin();
        // One synchronous admission owns this lazy projection. Never retained
        // across calls, source changes, publication or snapshot lifecycles.
        #[cfg(feature = "angular")]
        let mut resolver_projection: Option<Vec<SemanticEdge>> = None;
        #[cfg(feature = "angular")]
        let mut activate_projection: Option<Vec<SemanticEdge>> = None;
        #[cfg(feature = "angular")]
        let mut match_guard_projection: Option<Vec<SemanticEdge>> = None;
        #[cfg(feature = "angular")]
        let mut deactivate_projection: Option<Vec<SemanticEdge>> = None;
        #[cfg(feature = "angular")]
        let mut load_guard_projection: Option<Vec<SemanticEdge>> = None;
        #[cfg(feature = "angular")]
        let mut activate_child_projection: Option<Vec<SemanticEdge>> = None;
        #[cfg(feature = "angular")]
        let mut eager_projection: Option<Vec<SemanticEdge>> = None;
        #[cfg(feature = "angular")]
        let mut lazy_projection: Option<Vec<SemanticEdge>> = None;
        #[cfg(feature = "angular")]
        let mut children_projection: Option<Vec<SemanticEdge>> = None;
        #[cfg(feature = "angular")]
        let mut providers_projection: Option<Vec<SemanticEdge>> = None;
        #[cfg(feature = "angular")]
        let mut bootstrap_projection: Option<Vec<SemanticEdge>> = None;
        #[cfg(feature = "angular")]
        let mut config_provider_projection: Option<Vec<SemanticEdge>> = None;
        #[cfg(feature = "angular")]
        let mut config_composition_projection: Option<Vec<SemanticEdge>> = None;
        #[cfg(feature = "angular")]
        let mut imported_provider_projection: Option<Vec<SemanticEdge>> = None;
        edges.iter().all(|edge| {
            if edge.relation == crate::layers::meta::semantic::SemanticRelation::ResolvedBy
                && (edge.subject.canonical.is_some() || edge.object.canonical.is_some())
            {
                #[cfg(feature = "angular")]
                {
                    let projected = resolver_projection
                        .get_or_insert_with(|| crate::angular_meta::route_resolvers::project(self));
                    return crate::angular_meta::route_resolvers::admits_projected(projected, edge);
                }
                #[cfg(not(feature = "angular"))]
                {
                    return false;
                }
            }
            if matches!(
                edge.relation,
                crate::layers::meta::semantic::SemanticRelation::GuardedByCanMatch
                    | crate::layers::meta::semantic::SemanticRelation::GuardedByCanDeactivate
                    | crate::layers::meta::semantic::SemanticRelation::GuardedByCanLoad
                    | crate::layers::meta::semantic::SemanticRelation::GuardedByCanActivateChild
            ) || (edge.relation == crate::layers::meta::semantic::SemanticRelation::GuardedBy
                && (edge.subject.canonical.is_some() || edge.object.canonical.is_some()))
            {
                #[cfg(feature = "angular")]
                {
                    use crate::layers::meta::semantic::SemanticRelation;
                    let slot = match edge.relation {
                        SemanticRelation::GuardedBy => &mut activate_projection,
                        SemanticRelation::GuardedByCanMatch => &mut match_guard_projection,
                        SemanticRelation::GuardedByCanDeactivate => &mut deactivate_projection,
                        SemanticRelation::GuardedByCanLoad => &mut load_guard_projection,
                        SemanticRelation::GuardedByCanActivateChild => {
                            &mut activate_child_projection
                        }
                        _ => unreachable!("guard-only dispatch"),
                    };
                    let projected = slot.get_or_insert_with(|| {
                        crate::angular_meta::route_guards::project_relation(self, edge.relation)
                    });
                    return crate::angular_meta::route_guards::admits_projected(projected, edge);
                }
                #[cfg(not(feature = "angular"))]
                {
                    return false;
                }
            }
            if edge.relation == crate::layers::meta::semantic::SemanticRelation::LoadsChildren
                && (edge.subject.canonical.is_some() || edge.object.canonical.is_some())
            {
                #[cfg(feature = "angular")]
                {
                    let projected = children_projection.get_or_insert_with(|| {
                        crate::angular_meta::route_load_children::project(self)
                    });
                    return crate::angular_meta::route_load_children::admits_projected(
                        projected, edge,
                    );
                }
                #[cfg(not(feature = "angular"))]
                {
                    return false;
                }
            }
            if edge.relation == crate::layers::meta::semantic::SemanticRelation::RouteMapsTo
                && (edge.subject.canonical.is_some() || edge.object.canonical.is_some())
            {
                #[cfg(feature = "angular")]
                {
                    let eager = eager_projection.get_or_insert_with(|| {
                        crate::angular_meta::route_components::project(self)
                    });
                    if crate::angular_meta::route_components::admits_projected(eager, edge) {
                        return true;
                    }
                    let lazy = lazy_projection.get_or_insert_with(|| {
                        crate::angular_meta::route_lazy_components::project(self)
                    });
                    return crate::angular_meta::route_lazy_components::admits_projected(
                        lazy, edge,
                    );
                }
                #[cfg(not(feature = "angular"))]
                {
                    return false;
                }
            }
            if edge.relation
                == crate::layers::meta::semantic::SemanticRelation::IncludesApplicationConfig
            {
                #[cfg(feature = "angular")]
                {
                    if edge.call_evidence.is_some() {
                        return false;
                    }
                    let projected = config_composition_projection.get_or_insert_with(|| {
                        crate::angular_meta::config_composition::project(self)
                    });
                    return crate::angular_meta::config_composition::admits_projected(
                        projected, edge,
                    );
                }
                #[cfg(not(feature = "angular"))]
                {
                    return false;
                }
            }
            if matches!(
                edge.relation,
                crate::layers::meta::semantic::SemanticRelation::DeclaresBootstrapRoot
                    | crate::layers::meta::semantic::SemanticRelation::DeclaresBootstrapModule
                    | crate::layers::meta::semantic::SemanticRelation::UsesApplicationConfig
            ) || edge.subject.entity_type == "ApplicationBootstrap"
            {
                #[cfg(feature = "angular")]
                {
                    let projected = bootstrap_projection.get_or_insert_with(|| {
                        crate::angular_meta::bootstrap_references::project(self)
                    });
                    return crate::angular_meta::bootstrap_references::admits_projected(
                        projected, edge,
                    );
                }
                #[cfg(not(feature = "angular"))]
                {
                    return false;
                }
            }
            #[cfg(feature = "angular")]
            if edge.subject.entity_type == "ApplicationConfig"
                && edge.relation
                    == crate::layers::meta::semantic::SemanticRelation::RegistersProvider
            {
                let projected = config_provider_projection.get_or_insert_with(|| {
                    crate::angular_meta::application_config_providers::project(self)
                });
                return crate::angular_meta::application_config_providers::admits_projected(
                    projected, edge,
                );
            }
            #[cfg(feature = "angular")]
            if edge.subject.entity_type == "Route"
                && edge.relation
                    == crate::layers::meta::semantic::SemanticRelation::RegistersProvider
            {
                let projected = providers_projection
                    .get_or_insert_with(|| crate::angular_meta::route_providers::project(self));
                return crate::angular_meta::route_providers::admits_projected(projected, edge);
            }
            #[cfg(feature = "angular")]
            if edge.relation == crate::layers::meta::semantic::SemanticRelation::RegistersProvider
                && matches!(
                    edge.subject.entity_type,
                    "Component" | "Directive" | "Module"
                )
                && edge.subject.canonical.is_some()
            {
                return crate::angular_meta::provider_position::admits_registration(
                    self,
                    edge,
                    &mut imported_provider_projection,
                );
            }
            if edge.relation
                == crate::layers::meta::semantic::SemanticRelation::RegistersViewProvider
            {
                #[cfg(feature = "angular")]
                {
                    return crate::angular_meta::provider_position::admits_registration(
                        self,
                        edge,
                        &mut imported_provider_projection,
                    );
                }
                #[cfg(not(feature = "angular"))]
                {
                    return false;
                }
            }
            if edge.relation
                == crate::layers::meta::semantic::SemanticRelation::ProviderUsesExisting
                || (matches!(
                    edge.relation,
                    crate::layers::meta::semantic::SemanticRelation::ProviderUsesClass
                        | crate::layers::meta::semantic::SemanticRelation::ProviderUsesFactory
                ) && (edge.subject.canonical.is_some() || edge.object.canonical.is_some()))
            {
                #[cfg(feature = "angular")]
                {
                    return crate::angular_meta::provider_pair_authority::admits(
                        self,
                        edge,
                        &mut providers_projection,
                        &mut bootstrap_projection,
                        &mut config_provider_projection,
                        &mut imported_provider_projection,
                    );
                }
                #[cfg(not(feature = "angular"))]
                {
                    return false;
                }
            }
            if matches!(
                edge.relation,
                crate::layers::meta::semantic::SemanticRelation::HasInput
                    | crate::layers::meta::semantic::SemanticRelation::HasOutput
            ) && (edge.subject.canonical.is_some() || edge.object.canonical.is_some())
            {
                return self.admits_member_pair(edge);
            }
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

    fn admits_member_pair(&self, edge: &SemanticEdge) -> bool {
        let (
            Some(CanonicalEndpoint::Declaration {
                declaration: owner,
                site: owner_site,
            }),
            Some(CanonicalEndpoint::Declaration {
                declaration: member,
                site: Some((start, end)),
            }),
        ) = (&edge.subject.canonical, &edge.object.canonical)
        else {
            return false;
        };
        let (Ok(field), Ok(initializer)) = (
            crate::ir::invocation_evidence::SourceSpan::new(member.start, member.end),
            crate::ir::invocation_evidence::SourceSpan::new(*start, *end),
        ) else {
            return false;
        };
        let Some(pair) = self.member_at(field, initializer) else {
            return false;
        };
        let expected_owner: crate::layers::meta::canonical_endpoint::CanonicalDeclaration =
            (&pair.owner).into();
        let expected_member: crate::layers::meta::canonical_endpoint::CanonicalDeclaration =
            (&pair.member).into();
        *owner == expected_owner
            && *member == expected_member
            && *owner_site == Some((pair.owner.start, pair.owner.end))
            && edge.subject.name == pair.owner.name
            && edge.object.name == pair.member.name
            && edge.subject.file.as_deref() == pair.owner.source.to_str()
            && edge.object.file.as_deref() == pair.member.source.to_str()
    }

    fn admits_endpoint(&self, entity: &EntityRef, endpoint: &CanonicalEndpoint) -> bool {
        let declaration = match endpoint {
            CanonicalEndpoint::RouteOccurrence { .. }
            | CanonicalEndpoint::InvocationOccurrence { .. } => return false,
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
