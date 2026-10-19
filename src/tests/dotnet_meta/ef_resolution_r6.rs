use crate::compression::Fidelity;
use crate::ir::compiler::IRCompiler;
use crate::layers::meta::semantic::SemanticRelation;
use std::collections::BTreeSet;

fn compile_edges(source: &str) -> Vec<crate::layers::meta::semantic::SemanticEdge> {
    let (language, query) =
        crate::compression::language::language_for_extension("cs").expect("C# language");
    let mut compiler = IRCompiler::new();
    compiler
        .compile_focused(
            source,
            "r6-ef-resolution",
            Some("C:/repo/Contexts.cs"),
            language,
            query,
            Fidelity::High,
            None,
            None,
        )
        .expect("C# compilation");
    compiler.semantic_edges
}

#[test]
fn r6_ef_roles_use_shared_resolution_for_supported_spelling_and_ancestry() {
    let edges = compile_edges(
        r#"
using Microsoft.EntityFrameworkCore;
using Ef = Microsoft.EntityFrameworkCore;
using ContextBase = Microsoft.EntityFrameworkCore.DbContext;

namespace Application
{
    public class ImportedContext : DbContext {}
    public class QualifiedContext : Microsoft.EntityFrameworkCore.DbContext {}
    public class GlobalContext : global::Microsoft.EntityFrameworkCore.DbContext {}
    public class NamespaceAliasContext : Ef.DbContext {}
    public class TypeAliasContext : ContextBase {}
    public class IntermediateContext : DbContext {}
    public class DerivedContext : IntermediateContext {}
    public partial class PartialContext : DbContext {}
    public partial class PartialContext {}

    namespace Shadow
    {
        public class DbContext {}
        public class FakeContext : DbContext
        {
            public DbSet<FakeEntity> Entities { get; set; }
        }
        public class FakeEntity {}
    }
}
"#,
    );

    let roles = edges
        .iter()
        .filter(|edge| {
            edge.relation == SemanticRelation::Defines
                && edge.subject.domain == "dotnet"
                && edge.subject.entity_type == "DbContext"
        })
        .map(|edge| edge.subject.name.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        roles,
        BTreeSet::from([
            "DerivedContext",
            "GlobalContext",
            "ImportedContext",
            "IntermediateContext",
            "NamespaceAliasContext",
            "PartialContext",
            "QualifiedContext",
            "TypeAliasContext",
        ]),
        "all supported EF spellings and resolved ancestry must converge on one durable role"
    );
    assert!(
        edges.iter().all(|edge| {
            edge.subject.name != "FakeContext"
                || edge.subject.domain != "dotnet"
                || edge.subject.entity_type != "DbContext"
        }),
        "a local DbContext shadow must authorize neither an EF role nor EF relationships"
    );
}
