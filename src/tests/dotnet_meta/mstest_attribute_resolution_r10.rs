use crate::compression::Fidelity;
use crate::ir::compiler::IRCompiler;
use crate::layers::meta::semantic::{SemanticEdge, SemanticRelation};
use std::collections::BTreeSet;

#[test]
fn r10_attribute_resolver_preserves_neutral_identity_dispositions_and_suffix_rules() {
    use crate::ir::csharp_resolution::{
        CSharpAliasContext, CSharpAliasDirective, CSharpOrdinaryImportContext, CSharpResolution,
        CSharpResolutionBasis, CSharpSourceSpan, CSharpTypeIdentity,
        resolve_attribute_type_reference,
    };
    use crate::ir::csharp_symbols::CSharpSourceSymbolIndex;

    let identity = CSharpTypeIdentity::known_external(
        "Microsoft.VisualStudio.TestTools.UnitTesting.TestClassAttribute",
    );
    let known = vec![identity.clone()];
    let symbols = CSharpSourceSymbolIndex::default();
    let ordinary = CSharpOrdinaryImportContext {
        ordinary_imports: vec![vec![
            "Microsoft".into(),
            "VisualStudio".into(),
            "TestTools".into(),
            "UnitTesting".into(),
        ]],
        ..Default::default()
    };
    let aliases = CSharpAliasContext {
        directives: vec![CSharpAliasDirective {
            alias: "Marker".into(),
            target: identity.qualified_name.clone(),
            scope_depth: 0,
            file: "Tests.cs".into(),
            span: CSharpSourceSpan {
                start_byte: 0,
                end_byte: 10,
            },
        }],
    };

    for (written, expected_basis) in [
        (
            "Microsoft.VisualStudio.TestTools.UnitTesting.TestClass",
            CSharpResolutionBasis::ExplicitQualified,
        ),
        (
            "global::Microsoft.VisualStudio.TestTools.UnitTesting.TestClassAttribute",
            CSharpResolutionBasis::GlobalQualified,
        ),
        ("TestClass", CSharpResolutionBasis::OrdinaryImport),
        ("Marker", CSharpResolutionBasis::TypeAlias),
    ] {
        let resolved =
            resolve_attribute_type_reference(written, &ordinary, &aliases, &known, &symbols);
        assert!(
            matches!(
                &resolved,
                CSharpResolution::Resolved {
                    identity: actual,
                    basis,
                    ..
                } if actual == &identity && *basis == expected_basis
            ),
            "unexpected attribute resolution for {written}: {resolved:?}"
        );
    }
    assert!(matches!(
        resolve_attribute_type_reference(
            "Missing",
            &CSharpOrdinaryImportContext::default(),
            &CSharpAliasContext::default(),
            &known,
            &symbols,
        ),
        CSharpResolution::Unresolved(_)
    ));
    assert!(matches!(
        resolve_attribute_type_reference("TestClass<", &ordinary, &aliases, &known, &symbols,),
        CSharpResolution::Unsupported(_)
    ));
}

fn compile_edges(source: &str) -> Vec<SemanticEdge> {
    let (language, query) =
        crate::compression::language::language_for_extension("cs").expect("C# language");
    let mut compiler = IRCompiler::new();
    compiler
        .compile_focused(
            source,
            "r10-mstest-attribute-resolution",
            Some("C:/repo/Tests.cs"),
            language,
            query,
            Fidelity::High,
            None,
            None,
        )
        .expect("C# compilation");
    compiler.semantic_edges
}

fn test_classes(edges: &[SemanticEdge]) -> BTreeSet<String> {
    edges
        .iter()
        .filter(|edge| {
            edge.relation == SemanticRelation::Defines
                && edge.subject.domain == "dotnet"
                && edge.subject.entity_type == "TestClass"
        })
        .map(|edge| edge.subject.name.clone())
        .collect()
}

#[test]
fn r10_mstest_attributes_use_shared_resolution_for_suffix_spelling_aliases_and_partials() {
    let edges = compile_edges(
        r#"
using Microsoft.VisualStudio.TestTools.UnitTesting;
using MsTest = Microsoft.VisualStudio.TestTools.UnitTesting;
using TestMarker = Microsoft.VisualStudio.TestTools.UnitTesting.TestClassAttribute;

namespace Application
{
    [TestClass] public class ImportedTests { }
    [TestClassAttribute] public class ExplicitSuffixTests { }
    [Microsoft.VisualStudio.TestTools.UnitTesting.TestClass]
    public class QualifiedTests { }
    [global::Microsoft.VisualStudio.TestTools.UnitTesting.TestClassAttribute]
    public class GlobalTests { }
    [MsTest.TestClass] public class NamespaceAliasTests { }
    [TestMarker] public class TypeAliasTests { }
    [TestClass] public partial class PartialTests { }
    public partial class PartialTests { }
}
"#,
    );

    assert_eq!(
        test_classes(&edges),
        BTreeSet::from([
            "ExplicitSuffixTests".into(),
            "GlobalTests".into(),
            "ImportedTests".into(),
            "NamespaceAliasTests".into(),
            "PartialTests".into(),
            "QualifiedTests".into(),
            "TypeAliasTests".into(),
        ])
    );
}

#[test]
fn r10_mstest_attribute_shadows_and_competing_imports_fail_closed() {
    let edges = compile_edges(
        r#"
using Microsoft.VisualStudio.TestTools.UnitTesting;
using Other;

namespace Other { public class TestClassAttribute : System.Attribute { } }
namespace Application
{
    [TestClass] public class AmbiguousTests { }

    namespace Shadow
    {
        public class TestClassAttribute : System.Attribute { }
        [TestClass] public class LocalShadowTests { }
    }
}
"#,
    );

    assert!(
        test_classes(&edges).is_empty(),
        "source shadows and competing imports cannot authorize MSTest identity"
    );
}
