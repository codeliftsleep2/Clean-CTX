use super::*;
use crate::ir::compiler::IRCompiler;
use crate::ir::csharp_resolution::resolve_compilation_units;
use crate::ir::opcodes::{CoreOp, ExecutionContextKind};
use crate::ir::pipeline::{CoreIRPass, IRPass, PassContext};
use crate::mcp::semantic_compilation::artifact::SemanticCompilationArtifact;
use std::collections::BTreeSet;
use std::path::Path;

fn artifact(compiler: &mut IRCompiler, source: &str, file: &str) -> SemanticCompilationArtifact {
    let (language, query) = crate::compression::language::language_for_extension("cs").unwrap();
    let compiled = compiler
        .compile_focused(
            source,
            file,
            Some(file),
            language,
            query,
            Fidelity::High,
            None,
            None,
        )
        .unwrap();
    SemanticCompilationArtifact::from_compiler(
        compiled,
        compiler,
        crate::cache::content_hash(source.as_bytes()),
        file.to_string(),
        source,
        Fidelity::High,
        Path::new(file),
        &crate::tests::test_config(),
        "cs",
        &[],
        None,
    )
    .unwrap()
}

#[test]
fn compilation_identity_exact_spans_preserve_overload_and_constructor_ir_ids() {
    let source = "class C { public C() {} public void M(int x) {} public void M(string x) {} }";
    let (language, query) = crate::compression::language::language_for_extension("cs").unwrap();
    let mut state = PassContext::new(source.into(), "identity".into(), Fidelity::High);
    state.language = Some(language);
    state.query_string = query.into();
    CoreIRPass::new().run(&mut state).unwrap();
    let ids = state
        .captures
        .iter()
        .filter(|cap| matches!(cap.name.as_str(), "method.root" | "constructor.root"))
        .map(|cap| {
            let (id, name) = &state.csharp_declarations.methods[&(cap.start_byte, cap.end_byte)];
            assert!(state.instructions.iter().any(|op| {
                matches!(op, CoreOp::DefMethod(_, actual_id, actual_name)
                    if actual_id == id && actual_name == name)
            }));
            assert_eq!(&source[cap.start_byte..cap.end_byte], cap.raw_text);
            id.clone()
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        ids.len(),
        3,
        "overloads and constructor retain distinct IR IDs"
    );
}

#[test]
fn compilation_identity_artifacts_compose_cross_file_partials_and_replace_generations() {
    for framework in [
        "Microsoft.AspNetCore.Mvc.ControllerBase",
        "Microsoft.AspNetCore.SignalR.Hub",
        "AutoMapper.Profile",
        "Microsoft.EntityFrameworkCore.DbContext",
    ] {
        for generic_in_base in [false, true] {
            let mut compiler = IRCompiler::new();
            compiler.add_language_layer(Box::new(crate::ir::layers::csharp::CSharpLayer::new()));
            let generic = "protected void M<T>() {} protected void M<T,U>(int x) {} protected void N<A,B,C>() {}";
            let base_source = format!(
                "namespace Bases {{ public partial class Root : global::{framework} {{ {} }} }}",
                if generic_in_base { generic } else { "" }
            );
            let other_source = format!(
                "namespace Bases {{ public partial class Root {{ {} }} }}",
                if generic_in_base { "" } else { generic }
            );
            let base = artifact(&mut compiler, &base_source, "C:/repo/Base.cs");
            let other = artifact(&mut compiler, &other_source, "C:/repo/Other.cs");
            let leaf_source = r#"
namespace App {
    public class Middle : global::Bases.Root {}
    public class Leaf : Middle {
        public Leaf() { CreateMap<Source, Destination>(); }
        [global::Microsoft.AspNetCore.Mvc.HttpGet] public void Run() {}
    }
}
"#;
            let leaf = artifact(&mut compiler, leaf_source, "C:/repo/Leaf.cs");
            let units =
                [&base, &other, &leaf].map(|item| item.csharp_resolution_unit().unwrap().clone());
            let known = known_csharp_type_identities();
            let classes = resolve_compilation_units(&units, &known);
            let root = classes.iter().find(|class| class.owner == "Root").unwrap();
            assert_eq!(root.files.len(), 2, "partial symbol must compose");
            let descendant = classes.iter().find(|class| class.owner == "Leaf").unwrap();
            let admitted = match framework {
                "Microsoft.AspNetCore.Mvc.ControllerBase" => {
                    is_resolved_aspnet_controller(descendant)
                }
                "Microsoft.AspNetCore.SignalR.Hub" => is_resolved_signalr_hub(descendant),
                "AutoMapper.Profile" => is_resolved_automapper_profile(descendant),
                _ => is_resolved_dbcontext(descendant),
            };
            assert!(admitted, "{framework}: {:?}", descendant.ancestry);
            let edges = leaf
                .project_csharp_generation_semantics(&[], &classes)
                .unwrap();
            let relation = match framework {
                "Microsoft.AspNetCore.Mvc.ControllerBase" => SemanticRelation::ControllerAction,
                "Microsoft.AspNetCore.SignalR.Hub" => SemanticRelation::HubMethodTargets,
                "AutoMapper.Profile" => SemanticRelation::MapsTo,
                _ => SemanticRelation::Defines,
            };
            let target = match relation {
                SemanticRelation::ControllerAction | SemanticRelation::HubMethodTargets => "Run",
                SemanticRelation::MapsTo => "Destination",
                _ => "Leaf",
            };
            assert!(
                edges.iter().any(|edge| {
                    edge.subject.domain == "dotnet"
                        && edge.subject.name == "Leaf"
                        && edge.relation == relation
                        && edge.object.name == target
                }),
                "emitted payload missing for {framework}"
            );
            if framework == "Microsoft.AspNetCore.SignalR.Hub" {
                let projected = leaf.project_csharp_generation_ir(&classes);
                let expected = leaf
                    .compiled()
                    .instructions
                    .iter()
                    .filter_map(|op| match op {
                        CoreOp::DefMethod(_, id, _) => Some(id.clone()),
                        _ => None,
                    })
                    .collect::<BTreeSet<_>>();
                assert_eq!(expected.len(), 2);
                let actual = projected
                    .instructions
                    .iter()
                    .filter_map(|op| match op {
                        CoreOp::ExecutionContext(id, ExecutionContextKind::Realtime) => {
                            Some(id.clone())
                        }
                        _ => None,
                    })
                    .collect::<BTreeSet<_>>();
                assert_eq!(actual, expected, "constructor and method IDs remain valid");
            }
            // Replace only the base artifact; keep the existing leaf/other artifacts.
            let replacement = artifact(
                &mut compiler,
                "namespace Bases { public partial class Root { public void M<T,U,V,W>() {} } }",
                "C:/repo/Base.cs",
            );
            let replaced_units = [
                replacement.csharp_resolution_unit().unwrap().clone(),
                units[1].clone(),
                units[2].clone(),
            ];
            let replaced = resolve_compilation_units(&replaced_units, &known);
            let edges = leaf
                .project_csharp_generation_semantics(&[], &replaced)
                .unwrap();
            assert!(
                !edges.iter().any(|edge| {
                    edge.subject.domain == "dotnet"
                        && edge.subject.name == "Leaf"
                        && edge.relation == relation
                }),
                "removed authority must retract {framework} payload"
            );
            assert!(
                leaf.project_csharp_generation_ir(&replaced)
                    .instructions
                    .iter()
                    .all(|op| {
                        !matches!(
                            op,
                            CoreOp::ExecutionContext(_, ExecutionContextKind::Realtime)
                        )
                    })
            );
            // Restoring the old owned unit must restore exactly the original truth.
            let mut restored = resolve_compilation_units(&units, &known);
            let mut original = classes;
            // Symbol index iteration order is unspecified; compare complete records.
            restored.sort_by(|left, right| left.owner.cmp(&right.owner));
            original.sort_by(|left, right| left.owner.cmp(&right.owner));
            assert_eq!(restored, original);
        }
    }
}

#[test]
fn compilation_identity_mstest_class_attributes_survive_generic_members() {
    let mut compiler = IRCompiler::new();
    let item = artifact(
        &mut compiler,
        "[global::Microsoft.VisualStudio.TestTools.UnitTesting.TestClass] public class Tests<T,U> { public Tests() {} public void M<A,B,C,D>() {} }",
        "C:/repo/Tests.cs",
    );
    let classes = resolve_compilation_units(
        &[item.csharp_resolution_unit().unwrap().clone()],
        &known_csharp_type_identities(),
    );
    let edges = item
        .project_csharp_generation_semantics(&[], &classes)
        .unwrap();
    assert!(edges.iter().any(|edge| {
        edge.relation == SemanticRelation::Defines && edge.subject.entity_type == "TestClass"
    }));
}
