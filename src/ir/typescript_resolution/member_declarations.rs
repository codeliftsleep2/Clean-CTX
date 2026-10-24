//! Source-owned direct field initializers; no framework roles or lexical bindings.
use super::{DeclarationId, Resolver, TypeScriptAuthority};
use crate::ir::invocation_evidence::SourceSpan;
use std::path::Path;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct MemberDeclaration {
    pub owner: DeclarationId,
    pub member: DeclarationId,
    pub initializer: (usize, usize),
    pub unique: bool,
}

pub(super) fn capture(
    resolver: &mut Resolver,
    file: &Path,
    source: &str,
    declarations: &[DeclarationId],
) -> Vec<MemberDeclaration> {
    let Some(tree) = resolver.source_tree(file) else {
        return Vec::new();
    };
    let mut result = Vec::new();
    for owner in declarations.iter().filter(|id| {
        matches!(
            id.kind.as_str(),
            "class_declaration" | "abstract_class_declaration"
        )
    }) {
        let Some(class) = tree
            .root_node()
            .named_descendant_for_byte_range(owner.start, owner.end)
            .filter(|node| node.start_byte() == owner.start && node.end_byte() == owner.end)
        else {
            continue;
        };
        let Some(body) = class.child_by_field_name("body") else {
            continue;
        };
        let mut cursor = body.walk();
        let fields: Vec<_> = body.named_children(&mut cursor).collect();
        for field in &fields {
            if field.kind() != "public_field_definition" {
                continue;
            }
            let Some(name) = field
                .child_by_field_name("name")
                .filter(|name| name.kind() == "property_identifier")
            else {
                continue;
            };
            let Ok(spelling) = name.utf8_text(source.as_bytes()) else {
                continue;
            };
            let Some(value) = field
                .child_by_field_name("value")
                .filter(|value| value.kind() == "call_expression")
            else {
                continue;
            };
            let unique = fields
                .iter()
                .filter(|other| {
                    other
                        .child_by_field_name("name")
                        .and_then(|name| name.utf8_text(source.as_bytes()).ok())
                        == Some(spelling)
                })
                .count()
                == 1;
            result.push(MemberDeclaration {
                owner: owner.clone(),
                member: DeclarationId {
                    source: owner.source.clone(),
                    source_hash: owner.source_hash.clone(),
                    start: field.start_byte(),
                    end: field.end_byte(),
                    name: spelling.to_owned(),
                    kind: field.kind().to_owned(),
                },
                initializer: (value.start_byte(), value.end_byte()),
                unique,
            });
        }
    }
    result
}

impl TypeScriptAuthority {
    pub(crate) fn member_at(
        &self,
        field: SourceSpan,
        initializer: SourceSpan,
    ) -> Option<&MemberDeclaration> {
        let mut matches = self.member_declarations.iter().filter(|evidence| {
            evidence.member.start == field.start_byte()
                && evidence.member.end == field.end_byte()
                && evidence.initializer == (initializer.start_byte(), initializer.end_byte())
        });
        let evidence = matches.next()?;
        if matches.next().is_some()
            || !evidence.unique
            || evidence.owner.source != self.source
            || evidence.member.source != self.source
            || evidence.owner.source_hash != self.source_hash
            || evidence.member.source_hash != self.source_hash
            || !(evidence.owner.start <= evidence.member.start
                && evidence.member.start < evidence.member.end
                && evidence.member.end <= evidence.owner.end
                && evidence.member.start <= evidence.initializer.0
                && evidence.initializer.0 < evidence.initializer.1
                && evidence.initializer.1 <= evidence.member.end)
            || self
                .member_declarations
                .iter()
                .filter(|other| {
                    other.owner == evidence.owner && other.member.name == evidence.member.name
                })
                .count()
                != 1
            || !self.declarations.contains(&evidence.owner)
            || !self.declarations.contains(&evidence.member)
        {
            return None;
        }
        Some(evidence)
    }
}
