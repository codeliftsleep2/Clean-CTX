//! Neutral C# attribute-reference resolution and suffix elision.

use super::{
    CSharpAliasContext, CSharpOrdinaryImportContext, CSharpResolution, CSharpTypeIdentity,
    resolve_alias_type_reference, resolve_explicit_type_reference,
    resolve_ordinary_import_type_reference,
};
use crate::ir::csharp_symbols::CSharpSourceSymbolIndex;

/// Resolve a C# attribute reference without assigning framework meaning.
pub fn resolve_attribute_type_reference(
    written: &str,
    ordinary: &CSharpOrdinaryImportContext,
    aliases: &CSharpAliasContext,
    known_external: &[CSharpTypeIdentity],
    source_symbols: &CSharpSourceSymbolIndex,
) -> CSharpResolution<CSharpTypeIdentity> {
    let exact = resolve_form(written, ordinary, aliases, known_external, source_symbols);
    if written
        .rsplit(['.', ':'])
        .next()
        .is_some_and(|name| name.ends_with("Attribute"))
        || matches!(
            &exact,
            CSharpResolution::Resolved { .. } | CSharpResolution::Ambiguous(_)
        )
    {
        return exact;
    }
    resolve_form(
        &format!("{written}Attribute"),
        ordinary,
        aliases,
        known_external,
        source_symbols,
    )
}

fn resolve_form(
    written: &str,
    ordinary: &CSharpOrdinaryImportContext,
    aliases: &CSharpAliasContext,
    known_external: &[CSharpTypeIdentity],
    source_symbols: &CSharpSourceSymbolIndex,
) -> CSharpResolution<CSharpTypeIdentity> {
    let head = written.split(['.', '<']).next().unwrap_or_default().trim();
    if aliases
        .directives
        .iter()
        .any(|directive| directive.alias == head)
    {
        resolve_alias_type_reference(written, aliases, known_external, source_symbols)
    } else if written.starts_with("global::") || written.contains('.') {
        resolve_explicit_type_reference(written, known_external, source_symbols)
    } else {
        resolve_ordinary_import_type_reference(written, ordinary, known_external, source_symbols)
    }
}
