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
    use super::provider_pair_authority::{Strategy, canonical_token, pair_at};
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
        super::provider_composition::component_owner(authority, owner),
        start,
        end,
    )?;
    if matches!(entry.value, ExpressionValue::Reference(_)) {
        return pair_at(authority, name, start, end, Strategy::Class, position)
            .map(|(token, _)| token);
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
            let span =
                crate::ir::invocation_evidence::SourceSpan::new(target.start, target.end).ok()?;
            return (authority.value_reference_at(span) == Some(id))
                .then(|| canonical_token(authority, owner, token))
                .flatten();
        }
        return pair_at(authority, name, start, end, strategy, position).map(|(token, _)| token);
    }
    match &target.value {
        ExpressionValue::Reference(Resolution::Resolved(id)) => {
            let span =
                crate::ir::invocation_evidence::SourceSpan::new(target.start, target.end).ok()?;
            if authority.value_reference_at(span) != Some(id) {
                return None;
            }
        }
        ExpressionValue::Literal
        | ExpressionValue::StringValue(_)
        | ExpressionValue::Boolean(_) => {}
        _ => return None,
    }
    canonical_token(authority, owner, token)
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
    let expected = super::string_token_authority::owner(owner, "Component");
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
    super::provider_composition::entries(
        authority,
        array,
        super::provider_composition::component_owner(authority, owner),
    )
    .iter()
    .any(|entry| {
        registration_token(authority, &owner.name, entry.start, entry.end, position)
            .is_some_and(|token| same(&token, &edge.object))
    })
}
