use crate::compression::Fidelity;
use crate::ir::compiler::IRCompiler;
use crate::layers::meta::semantic::{SemanticEdge, SemanticRelation};
use std::collections::BTreeSet;

fn compile_edges(source: &str) -> Vec<SemanticEdge> {
    let (language, query) =
        crate::compression::language::language_for_extension("cs").expect("C# language");
    let mut compiler = IRCompiler::new();
    compiler
        .compile_focused(
            source,
            "r7-automapper-resolution",
            Some("C:/repo/Profiles.cs"),
            language,
            query,
            Fidelity::High,
            None,
            None,
        )
        .expect("C# compilation");
    compiler.semantic_edges
}

fn mappings(edges: &[SemanticEdge]) -> BTreeSet<(String, &'static str, String)> {
    edges
        .iter()
        .filter(|edge| {
            edge.subject.domain == "dotnet"
                && edge.subject.entity_type == "MapperProfile"
                && matches!(
                    edge.relation,
                    SemanticRelation::MapsFrom | SemanticRelation::MapsTo
                )
        })
        .map(|edge| {
            (
                edge.subject.name.clone(),
                match edge.relation {
                    SemanticRelation::MapsFrom => "from",
                    SemanticRelation::MapsTo => "to",
                    _ => unreachable!("mapping filter admits only mapping relations"),
                },
                edge.object.name.clone(),
            )
        })
        .collect()
}

#[test]
fn r7_automapper_profiles_use_shared_resolution_for_spelling_ancestry_and_partials() {
    let edges = compile_edges(
        r#"
using AutoMapper;
using Mapper = AutoMapper;
using ProfileBase = AutoMapper.Profile;

namespace Application
{
    public class ImportedProfile : Profile { void Configure() { CreateMap<A, ADto>(); } }
    public class QualifiedProfile : AutoMapper.Profile { void Configure() { CreateMap<B, BDto>(); } }
    public class GlobalProfile : global::AutoMapper.Profile { void Configure() { CreateMap<C, CDto>(); } }
    public class NamespaceAliasProfile : Mapper.Profile { void Configure() { CreateMap<D, DDto>(); } }
    public class TypeAliasProfile : ProfileBase { void Configure() { CreateMap<E, EDto>(); } }
    public class IntermediateProfile : Profile { }
    public class DerivedProfile : IntermediateProfile { void Configure() { CreateMap<F, FDto>(); } }
    public partial class PartialProfile : Profile { }
    public partial class PartialProfile { void Configure() { CreateMap<G, GDto>(); } }
    public class EmptyProfile : Profile { }

    namespace Shadow
    {
        public class Profile { }
        public class FakeProfile : Profile { void Configure() { CreateMap<Bad, BadDto>(); } }
    }
}
"#,
    );

    assert_eq!(
        mappings(&edges),
        BTreeSet::from([
            ("DerivedProfile".into(), "from", "F".into()),
            ("DerivedProfile".into(), "to", "FDto".into()),
            ("GlobalProfile".into(), "from", "C".into()),
            ("GlobalProfile".into(), "to", "CDto".into()),
            ("ImportedProfile".into(), "from", "A".into()),
            ("ImportedProfile".into(), "to", "ADto".into()),
            ("NamespaceAliasProfile".into(), "from", "D".into()),
            ("NamespaceAliasProfile".into(), "to", "DDto".into()),
            ("PartialProfile".into(), "from", "G".into()),
            ("PartialProfile".into(), "to", "GDto".into()),
            ("QualifiedProfile".into(), "from", "B".into()),
            ("QualifiedProfile".into(), "to", "BDto".into()),
            ("TypeAliasProfile".into(), "from", "E".into()),
            ("TypeAliasProfile".into(), "to", "EDto".into()),
        ])
    );
    assert!(
        mappings(&edges)
            .iter()
            .all(|(profile, _, entity)| { profile != "FakeProfile" && entity != "Bad" }),
        "a local Profile shadow cannot authorize AutoMapper relationships"
    );
}

#[test]
fn r7_automapper_competing_imports_fail_closed() {
    let edges = compile_edges(
        r#"
using AutoMapper;
using Other;
namespace Other { public class Profile { } }
namespace Application
{
    public class AmbiguousProfile : Profile
    {
        void Configure() { CreateMap<Bad, BadDto>(); }
    }
}
"#,
    );

    assert!(
        mappings(&edges).is_empty(),
        "competing source and known-external imports cannot authorize mappings"
    );
}
