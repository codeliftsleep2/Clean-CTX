//! Legitimate captured artifacts -> resolved partial authority -> artifact projection.
use super::artifact;
use crate::compression::Fidelity;
use crate::ir::compiler::IRCompiler;
use crate::ir::csharp_resolution::resolve_compilation_units;
use crate::ir::opcodes::{CoreOp, ExecutionContextKind};
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};
use crate::work_observations_af_perf_002 as work;

fn edge(
    relation: SemanticRelation,
    kind: &'static str,
    target_kind: &'static str,
    target: &str,
    file: &str,
    subject_file: &str,
    layer: &'static str,
) -> SemanticEdge {
    let mut subject = EntityRef::new("dotnet", kind, "Role");
    subject.file = Some(subject_file.into());
    let mut object = EntityRef::new("dotnet", target_kind, target);
    object.file = Some(file.into());
    SemanticEdge {
        relation,
        subject,
        object,
        layer,
        call_evidence: None,
    }
}

#[test]
fn af002_a2_ownership_partial_role_oracles() {
    for owners in [1, 2, 4, 8, 16] {
        for (kind, base, payload) in [
            (
                "DbContext",
                " : global::Microsoft.EntityFrameworkCore.DbContext",
                "",
            ),
            (
                "MapperProfile",
                " : global::AutoMapper.Profile",
                "public Role() { CreateMap<Source, Destination>(); }",
            ),
            (
                "Hub",
                " : global::Microsoft.AspNetCore.SignalR.Hub",
                "public void Echo() {} public void Echo(int value) {}",
            ),
            (
                "Controller",
                " : global::Microsoft.AspNetCore.Mvc.ControllerBase",
                "[global::Microsoft.AspNetCore.Mvc.HttpGetAttribute] public void Echo() {}",
            ),
            ("TestClass", "", ""),
        ] {
            let mut compiler = IRCompiler::new();
            compiler.add_language_layer(Box::new(crate::ir::layers::csharp::CSharpLayer::new()));
            let files: Vec<_> = (0..owners)
                .map(|n| format!("C:/repo/A2Owner{n:02}.cs"))
                .collect();
            let artifacts: Vec<_> = files
                .iter()
                .enumerate()
                .map(|(n, file)| {
                    let attribute = if kind == "TestClass" && n == 0 {
                        "[global::Microsoft.VisualStudio.TestTools.UnitTesting.TestClassAttribute]"
                    } else {
                        ""
                    };
                    let payload = payload
                        .replace("Echo", &format!("Echo{n}"))
                        .replace("public Role()", &format!("public void Configure{n}()"));
                    let source = format!(
                        "{attribute} public partial class Role{} {{ {payload} }}",
                        if n == 0 { base } else { "" }
                    );
                    artifact(&mut compiler, &source, file)
                })
                .collect();
            let units: Vec<_> = artifacts
                .iter()
                .map(|a| a.csharp_resolution_unit().unwrap().clone())
                .collect();
            let classes = resolve_compilation_units(
                &units,
                &crate::dotnet_meta::semantic::known_csharp_type_identities(),
            );
            assert_eq!(classes.len(), 1, "one legitimate composed partial symbol");
            let class = &classes[0];
            assert_eq!(class.files, files, "deterministic physical part order");
            assert_eq!(
                class.raw_parts.iter().map(|(f, _)| f).collect::<Vec<_>>(),
                files.iter().collect::<Vec<_>>()
            );
            assert_eq!(
                class.method_ids.iter().map(|(f, _)| f).collect::<Vec<_>>(),
                files.iter().collect::<Vec<_>>()
            );
            eprintln!(
                "AF002_A2_AUTHORITY kind={kind} owners={owners} classes={} class_file_entries={} method_owner_entries={} method_ids={} estimate_a2a_class_sweeps={} estimate_a2b_class_sweeps={} estimate_scope=analytical-not-implemented",
                classes.len(),
                class.files.len(),
                class.method_ids.len(),
                class
                    .method_ids
                    .iter()
                    .map(|(_, ids)| ids.len())
                    .sum::<usize>(),
                owners * classes.len(),
                classes.len()
            );
            for (index, artifact) in artifacts.iter().enumerate() {
                let file = &files[index];
                for fidelity in [Fidelity::High, Fidelity::Low] {
                    work::reset();
                    let start = std::time::Instant::now();
                    let edges = artifact
                        .project_csharp_generation_semantics_for_fidelity(&[], &classes, fidelity)
                        .unwrap();
                    let projected = artifact.project_csharp_generation_ir(&classes);
                    let counts = work::snapshot();
                    let actual: Vec<_> = edges
                        .iter()
                        .filter(|e| e.subject.domain == "dotnet" && e.subject.name == "Role")
                        .map(|e| serde_json::to_value(e).unwrap())
                        .collect();
                    let registration =
                        crate::layers::meta::semantic::SEMANTIC_ROLE_REGISTRATION_LAYER;
                    let expected = match kind {
                        "DbContext" => vec![edge(
                            SemanticRelation::Defines,
                            "DbContext",
                            "DbContext",
                            "Role",
                            file,
                            file,
                            registration,
                        )],
                        "MapperProfile" => vec![
                            edge(
                                SemanticRelation::MapsFrom,
                                "MapperProfile",
                                "Entity",
                                "Source",
                                file,
                                file,
                                "dotnet",
                            ),
                            edge(
                                SemanticRelation::MapsTo,
                                "MapperProfile",
                                "Entity",
                                "Destination",
                                file,
                                file,
                                "dotnet",
                            ),
                        ],
                        "Hub" if fidelity != Fidelity::Low => vec![edge(
                            SemanticRelation::HubMethodTargets,
                            "Hub",
                            "HubMethod",
                            &format!("Echo{index}"),
                            file,
                            file,
                            "dotnet",
                        )],
                        "Controller" => {
                            let mut result = Vec::new();
                            if index == 0 {
                                result.push(edge(
                                    SemanticRelation::Defines,
                                    "Controller",
                                    "Controller",
                                    "Role",
                                    file,
                                    file,
                                    registration,
                                ));
                            }
                            if fidelity != Fidelity::Low {
                                result.push(edge(
                                    SemanticRelation::ControllerAction,
                                    "Controller",
                                    "Action",
                                    &format!("Echo{index}"),
                                    file,
                                    &files[0],
                                    "dotnet",
                                ));
                            }
                            result
                        }
                        "TestClass" if index == 0 => vec![edge(
                            SemanticRelation::Defines,
                            "TestClass",
                            "TestClass",
                            "Role",
                            file,
                            file,
                            "dotnet",
                        )],
                        _ => vec![],
                    };
                    assert_eq!(
                        actual,
                        expected
                            .iter()
                            .map(|e| serde_json::to_value(e).unwrap())
                            .collect::<Vec<_>>(),
                        "ordered exact oracle kind={kind} owner={index} fidelity={fidelity:?}"
                    );
                    let contexts: Vec<_> = projected
                        .instructions
                        .iter()
                        .filter_map(|op| match op {
                            CoreOp::ExecutionContext(id, ExecutionContextKind::Realtime) => {
                                Some(id.clone())
                            }
                            _ => None,
                        })
                        .collect();
                    let expected_ids: Vec<_> = if kind == "Hub" {
                        class
                            .method_ids
                            .iter()
                            .filter(|(owner, _)| owner == file)
                            .flat_map(|(_, ids)| ids.iter().cloned())
                            .collect()
                    } else {
                        vec![]
                    };
                    assert_eq!(
                        contexts, expected_ids,
                        "exact ordered IR IDs, independent of edge fidelity"
                    );
                    eprintln!(
                        "AF002_A2_PARTIAL kind={kind} owners={owners} owner={index} fidelity={fidelity:?} class_visits={:?} files={:?} method_owners={} raw_parts={:?} context_methods={} action_methods={} us={} boundary=artifact-projection",
                        counts.csharp_role_class_visits,
                        counts.csharp_class_file_visits,
                        counts.csharp_method_owner_visits,
                        counts.csharp_raw_part_visits,
                        counts.csharp_context_method_visits,
                        counts.csharp_aspnet_method_visits,
                        start.elapsed().as_micros()
                    );
                }
            }
        }
    }
}

#[test]
fn af002_a2_ownership_testing_off_preserves_non_testing_consumers() {
    use crate::mcp::semantic_compilation::artifact::SemanticCompilationArtifact;
    let mut compiler = IRCompiler::new();
    compiler.add_language_layer(Box::new(crate::ir::layers::csharp::CSharpLayer::new()));
    let source = "[global::Microsoft.VisualStudio.TestTools.UnitTesting.TestClassAttribute] public class Role : global::Microsoft.AspNetCore.SignalR.Hub { public void Echo() {} }";
    let file = "C:/repo/A2TestingOff.cs";
    let mut config = crate::tests::test_config();
    config
        .meta_layers
        .entry("dotnet".into())
        .or_default()
        .testing
        .enabled = false;
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
    let item = SemanticCompilationArtifact::from_compiler(
        compiled,
        &mut compiler,
        crate::cache::content_hash(source.as_bytes()),
        file.into(),
        source,
        Fidelity::High,
        std::path::Path::new(file),
        &config,
        "cs",
        &[],
        None,
    )
    .unwrap();
    let classes = resolve_compilation_units(
        &[item.csharp_resolution_unit().unwrap().clone()],
        &crate::dotnet_meta::semantic::known_csharp_type_identities(),
    );
    work::reset();
    let edges = item
        .project_csharp_generation_semantics(&[], &classes)
        .unwrap();
    let counts = work::snapshot();
    assert_eq!(
        counts.csharp_role_entries[4], 0,
        "testing-off branch unreachable"
    );
    let actual: Vec<_> = edges
        .iter()
        .filter(|e| e.subject.domain == "dotnet" && e.subject.name == "Role")
        .map(|e| serde_json::to_value(e).unwrap())
        .collect();
    assert_eq!(
        actual,
        vec![
            serde_json::to_value(edge(
                SemanticRelation::HubMethodTargets,
                "Hub",
                "HubMethod",
                "Echo",
                file,
                file,
                "dotnet"
            ))
            .unwrap()
        ]
    );
}
