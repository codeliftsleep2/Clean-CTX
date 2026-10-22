//! Binding identity groups, not a certificate of member/type compatibility.
use super::DeclarationId;
use std::collections::BTreeSet;

pub(super) fn mergeable(declarations: &BTreeSet<DeclarationId>) -> bool {
    let Some(first) = declarations.first() else {
        return false;
    };
    if declarations.len() < 2
        || declarations.iter().any(|id| {
            id.source != first.source
                || id.source_hash != first.source_hash
                || id.name != first.name
        })
    {
        return false;
    }
    let namespaces: Vec<_> = declarations
        .iter()
        .filter(|id| id.kind == "internal_module")
        .collect();
    if !namespaces.is_empty() {
        let others: Vec<_> = declarations
            .iter()
            .filter(|id| id.kind != "internal_module")
            .collect();
        if others.is_empty() {
            return true;
        }
        if namespaces
            .iter()
            .any(|ns| others.iter().any(|id| ns.start < id.end))
        {
            return false;
        }
        let classes = others
            .iter()
            .filter(|id| {
                matches!(
                    id.kind.as_str(),
                    "class_declaration" | "abstract_class_declaration"
                )
            })
            .count();
        let enums = others
            .iter()
            .filter(|id| id.kind == "enum_declaration")
            .count();
        let functions = others
            .iter()
            .filter(|id| {
                matches!(
                    id.kind.as_str(),
                    "function_signature" | "function_declaration"
                )
            })
            .count();
        return (classes == 1 && others.len() == 1)
            || (enums == 1 && others.len() == 1)
            || (functions == others.len()
                && others
                    .iter()
                    .filter(|id| id.kind == "function_declaration")
                    .count()
                    <= 1);
    }
    if declarations
        .iter()
        .all(|id| id.kind == "interface_declaration")
    {
        return true;
    }
    // Enum parts contribute to the same type/value symbol in one source
    // snapshot, just as admitted cross-source program parts do. This groups
    // declaration identity; member compatibility is a separate obligation.
    if declarations.iter().all(|id| id.kind == "enum_declaration") {
        return true;
    }
    // A class owns a value and a type; interfaces augment only its type
    // symbol. Namespace eligibility is filtered before this grouping step.
    if declarations.iter().all(|id| {
        matches!(
            id.kind.as_str(),
            "interface_declaration" | "class_declaration" | "abstract_class_declaration"
        )
    }) && declarations
        .iter()
        .filter(|id| id.kind != "interface_declaration")
        .count()
        == 1
    {
        return true;
    }
    // Overload signatures contribute to one value symbol. Multiple bodies
    // remain competing definitions; unrelated files never merge here.
    declarations.iter().all(|id| {
        matches!(
            id.kind.as_str(),
            "function_signature" | "function_declaration"
        )
    }) && declarations
        .iter()
        .filter(|id| id.kind == "function_declaration")
        .count()
        <= 1
}
