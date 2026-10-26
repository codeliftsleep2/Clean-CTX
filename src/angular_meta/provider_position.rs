//! Explicit Angular metadata positions sharing the established provider lifecycle.
use super::decorators::ClassKind;
use crate::ir::typescript_resolution::declaration_facets::{ExpressionFacet, ExpressionValue};
use crate::ir::typescript_resolution::{Resolution, TypeScriptAuthority};
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Position {
    Ordinary,
    View,
}

impl Position {
    pub(super) fn field(self) -> &'static str {
        match self {
            Self::Ordinary => "providers",
            Self::View => "viewProviders",
        }
    }
    pub(super) fn relation(self) -> SemanticRelation {
        match self {
            Self::Ordinary => SemanticRelation::RegistersProvider,
            Self::View => SemanticRelation::RegistersViewProvider,
        }
    }
    /// Exact Angular owner identity and position policy, shared by every consumer.
    pub(super) fn owner_type(
        self,
        authority: &TypeScriptAuthority,
        owner: &crate::ir::typescript_resolution::DeclarationId,
    ) -> Option<&'static str> {
        if owner.source != authority.source || owner.source_hash != authority.source_hash {
            return None;
        }
        let kind = super::class_role_authority::declaration_role(authority, owner)?;
        if !self.admits(kind) {
            return None;
        }
        match kind {
            ClassKind::Component => Some("Component"),
            ClassKind::Directive => Some("Directive"),
            ClassKind::Module => Some("Module"),
            _ => None,
        }
    }

    pub(super) fn admits(self, kind: ClassKind) -> bool {
        match self {
            Self::Ordinary => matches!(
                kind,
                ClassKind::Component | ClassKind::Directive | ClassKind::Module
            ),
            Self::View => kind == ClassKind::Component,
        }
    }
}

fn contains(container: &ExpressionFacet, value: &ExpressionFacet) -> bool {
    container.start <= value.start && value.start < value.end && value.end <= container.end
}

/// Replay one exact position/terminal path, never an endpoint cross-product.
pub(super) fn registration_token(
    authority: &TypeScriptAuthority,
    name: &str,
    start: usize,
    end: usize,
    position: Position,
) -> Option<EntityRef> {
    let (owner, array) =
        super::string_token_authority::provider_array_at(authority, name, position)?;
    if owner.source != authority.source
        || owner.source_hash != authority.source_hash
        || array.end > owner.end
    {
        return None;
    }
    let entry = super::provider_composition::entry_at(
        authority,
        array,
        position.owner_type(authority, owner).is_some(),
        start,
        end,
    )?;
    registration_entry(
        authority,
        owner,
        super::provider_composition::Entry {
            source: &authority.source,
            hash: &authority.source_hash,
            facet: entry,
            crossed: false,
        },
    )
}

pub(super) fn registration_entry(
    authority: &TypeScriptAuthority,
    owner: &crate::ir::typescript_resolution::DeclarationId,
    occurrence: super::provider_composition::Entry<'_>,
) -> Option<EntityRef> {
    use super::provider_pair_authority::{Strategy, canonical_token_in, pair_entry};
    let entry = occurrence.facet;
    if matches!(entry.value, ExpressionValue::Reference(_)) {
        return pair_entry(authority, owner, occurrence, Strategy::Class).map(|(token, _)| token);
    }
    let ExpressionValue::Object(fields) = &entry.value else {
        return None;
    };
    if fields.iter().any(|(key, _)| key.is_empty()) {
        return None;
    }
    let unique = |field: &str| {
        let mut values = fields.iter().filter(|(key, _)| key == field);
        let (_, value) = values.next()?;
        values.next().is_none().then_some(value)
    };
    let token = unique("provide")?;
    let strategies: Vec<_> = fields
        .iter()
        .filter(|(key, _)| {
            matches!(
                key.as_str(),
                "useClass" | "useFactory" | "useExisting" | "useValue"
            )
        })
        .collect();
    let [(field, target)] = strategies.as_slice() else {
        return None;
    };
    if !contains(entry, token)
        || !contains(entry, target)
        || (token.start < target.end && target.start < token.end)
    {
        return None;
    }
    let strategy = match field.as_str() {
        "useClass" => Some(Strategy::Class),
        "useFactory" => Some(Strategy::Factory),
        "useExisting" => Some(Strategy::Existing),
        _ => None,
    };
    if let Some(strategy) = strategy {
        // Existing callable-variable compatibility proves registration only;
        // it does not acquire canonical factory-target or result identity.
        if field == "useFactory"
            && let ExpressionValue::Reference(Resolution::Resolved(id)) = &target.value
            && id.kind == "variable_declarator"
        {
            return (occurrence.reference(authority, target) == Some(id))
                .then(|| canonical_token_in(authority, owner, occurrence, token))
                .flatten();
        }
        return pair_entry(authority, owner, occurrence, strategy).map(|(token, _)| token);
    }
    match &target.value {
        ExpressionValue::Reference(Resolution::Resolved(id)) => {
            if occurrence.reference(authority, target) != Some(id) {
                return None;
            }
        }
        ExpressionValue::Literal
        | ExpressionValue::StringValue(_)
        | ExpressionValue::Boolean(_) => {
            if occurrence.source != authority.source.as_path() {
                occurrence.literal(authority, target)?;
            }
        }
        _ => return None,
    }
    canonical_token_in(authority, owner, occurrence, token)
}

pub(crate) fn admits_registration(authority: &TypeScriptAuthority, edge: &SemanticEdge) -> bool {
    let position = match edge.relation {
        SemanticRelation::RegistersViewProvider => Position::View,
        SemanticRelation::RegistersProvider => Position::Ordinary,
        _ => return false,
    };
    if edge.layer != "angular" {
        return false;
    }
    let Some((owner, array)) =
        super::string_token_authority::provider_array_at(authority, &edge.subject.name, position)
    else {
        return false;
    };
    let Some(owner_type) = position.owner_type(authority, owner) else {
        return false;
    };
    let expected = super::string_token_authority::owner(owner, owner_type);
    let same = |left: &EntityRef, right: &EntityRef| {
        left.domain == right.domain
            && left.entity_type == right.entity_type
            && left.name == right.name
            && left.file == right.file
            && left.canonical == right.canonical
    };
    if !same(&expected, &edge.subject) {
        return false;
    }
    super::provider_imported::admits(authority, edge)
        || super::provider_composition::entries(
            authority,
            array,
            position.owner_type(authority, owner).is_some(),
        )
        .iter()
        .any(|entry| {
            registration_token(authority, &owner.name, entry.start, entry.end, position)
                .is_some_and(|token| same(&token, &edge.object))
        })
}
