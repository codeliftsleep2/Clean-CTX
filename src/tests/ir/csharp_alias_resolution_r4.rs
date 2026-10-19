use super::*;
use crate::ir::csharp_resolution::{
    CSharpResolutionBasis, CSharpResolutionEvidence, CSharpTypeIdentityOrigin,
};

fn alias(name: &str, target: &str, scope_depth: usize, file: &str) -> CSharpAliasDirective {
    CSharpAliasDirective {
        alias: name.to_string(),
        target: target.to_string(),
        scope_depth,
        file: file.to_string(),
        span: CSharpSourceSpan {
            start_byte: scope_depth * 10,
            end_byte: scope_depth * 10 + 8,
        },
    }
}

#[test]
fn r4_aliases_resolve_lexically_with_cycles_ambiguity_and_evidence() {
    let source = CSharpSourceSymbolIndex::default();
    let external = [
        CSharpTypeIdentity::known_external("Microsoft.EntityFrameworkCore.DbContext"),
        CSharpTypeIdentity::known_external("Local.Framework.Context"),
        CSharpTypeIdentity::known_external_with_arity("Microsoft.AspNetCore.SignalR.Hub", 1),
    ];
    let aliases = CSharpAliasContext {
        directives: vec![
            alias("Ef", "Microsoft.EntityFrameworkCore", 0, "Global.cs"),
            alias(
                "ContextBase",
                "Microsoft.EntityFrameworkCore.DbContext",
                0,
                "Global.cs",
            ),
            alias("Data", "Ef", 0, "Global.cs"),
            alias(
                "Ctx",
                "Microsoft.EntityFrameworkCore.DbContext",
                0,
                "Outer.cs",
            ),
            alias("Ctx", "Local.Framework.Context", 1, "Inner.cs"),
            alias(
                "HubBase",
                "Microsoft.AspNetCore.SignalR.Hub<IClient>",
                0,
                "Global.cs",
            ),
        ],
    };

    let CSharpResolution::Resolved {
        identity,
        basis,
        evidence,
    } = resolve_alias_type_reference("Ef.DbContext", &aliases, &external, &source)
    else {
        panic!("namespace alias must resolve");
    };
    assert_eq!(
        identity.qualified_name,
        "Microsoft.EntityFrameworkCore.DbContext"
    );
    assert_eq!(basis, CSharpResolutionBasis::NamespaceAlias);
    assert_eq!(
        evidence,
        vec![CSharpResolutionEvidence {
            file: "Global.cs".to_string(),
            span: CSharpSourceSpan {
                start_byte: 0,
                end_byte: 8,
            },
        }]
    );

    let CSharpResolution::Resolved { basis, .. } =
        resolve_alias_type_reference("ContextBase", &aliases, &external, &source)
    else {
        panic!("type alias must resolve");
    };
    assert_eq!(basis, CSharpResolutionBasis::TypeAlias);

    let CSharpResolution::Resolved {
        identity, evidence, ..
    } = resolve_alias_type_reference("Data.DbContext", &aliases, &external, &source)
    else {
        panic!("alias chain must resolve");
    };
    assert_eq!(
        identity.qualified_name,
        "Microsoft.EntityFrameworkCore.DbContext"
    );
    assert_eq!(evidence.len(), 2);

    let CSharpResolution::Resolved { identity, .. } =
        resolve_alias_type_reference("Ctx", &aliases, &external, &source)
    else {
        panic!("inner alias must shadow outer alias");
    };
    assert_eq!(identity.qualified_name, "Local.Framework.Context");

    let CSharpResolution::Resolved { identity, .. } =
        resolve_alias_type_reference("HubBase", &aliases, &external, &source)
    else {
        panic!("generic type alias must preserve definition arity");
    };
    assert_eq!(identity.generic_arity, 1);
    assert_eq!(identity.origin, CSharpTypeIdentityOrigin::KnownExternal);

    let duplicate = CSharpAliasContext {
        directives: vec![
            alias("Dup", "First.Type", 1, "One.cs"),
            alias("Dup", "Second.Type", 1, "Two.cs"),
        ],
    };
    assert_eq!(
        resolve_alias_type_reference("Dup", &duplicate, &external, &source),
        CSharpResolution::Ambiguous(CSharpResolutionReason::DuplicateAlias)
    );

    let cycle = CSharpAliasContext {
        directives: vec![alias("A", "B", 0, "A.cs"), alias("B", "A", 0, "B.cs")],
    };
    assert_eq!(
        resolve_alias_type_reference("A", &cycle, &external, &source),
        CSharpResolution::Unsupported(CSharpResolutionReason::AliasCycle)
    );

    let missing = CSharpAliasContext {
        directives: vec![alias("Missing", "Unknown.Namespace.Type", 0, "Missing.cs")],
    };
    assert_eq!(
        resolve_alias_type_reference("Missing", &missing, &external, &source),
        CSharpResolution::Unresolved(CSharpResolutionReason::MissingAuthority)
    );
}
