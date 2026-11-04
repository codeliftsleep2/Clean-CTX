//! Independent role population and method count, using legitimate partial artifacts.
use super::artifact;
use crate::ir::compiler::IRCompiler;
use crate::ir::csharp_resolution::resolve_compilation_units;
use crate::ir::opcodes::{CoreOp, ExecutionContextKind};
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};
use crate::work_observations_af_perf_002 as work;

#[test]
fn af002_a2_density_roles_and_methods() {
    let mut dimensions: Vec<_> = [0, 1, 2, 4, 8, 16]
        .into_iter()
        .map(|roles| (16, roles, 1))
        .collect();
    dimensions.extend(
        [1, 2, 4, 8, 16, 64, 256]
            .into_iter()
            .map(|methods| (1, 1, methods)),
    );
    for (population, roles, methods) in dimensions {
        let mut compiler = IRCompiler::new();
        compiler.add_language_layer(Box::new(crate::ir::layers::csharp::CSharpLayer::new()));
        let files = ["C:/repo/Density0.cs", "C:/repo/Density1.cs"];
        let artifacts: Vec<_> = files
            .iter()
            .enumerate()
            .map(|(part, file)| {
                let source: String = (0..population)
                    .map(|index| {
                        let base = if part == 0 && index < roles {
                            " : global::Microsoft.AspNetCore.SignalR.Hub"
                        } else {
                            ""
                        };
                        let body: String = (0..methods)
                            .map(|method| format!("public void P{part}M{method}() {{}} "))
                            .collect();
                        format!("public partial class Class{index}{base} {{ {body} }} ")
                    })
                    .collect();
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
        assert_eq!(classes.len(), population);
        for (part, item) in artifacts.iter().enumerate() {
            work::reset();
            let start = std::time::Instant::now();
            let edges = item
                .project_csharp_generation_semantics(&[], &classes)
                .unwrap();
            let projected = item.project_csharp_generation_ir(&classes);
            let counts = work::snapshot();
            // Projection follows supplied authority order, independently generated member names.
            let mut expected = Vec::new();
            let mut expected_ids = Vec::new();
            for class in &classes {
                let index: usize = class.owner.strip_prefix("Class").unwrap().parse().unwrap();
                assert!(index < population);
                assert_eq!(class.files, files.map(str::to_owned));
                if index >= roles {
                    continue;
                }
                for method in 0..methods {
                    let mut subject = EntityRef::new("dotnet", "Hub", &class.owner);
                    subject.file = Some(files[part].into());
                    let mut object =
                        EntityRef::new("dotnet", "HubMethod", format!("P{part}M{method}"));
                    object.file = Some(files[part].into());
                    expected.push(
                        serde_json::to_value(SemanticEdge {
                            relation: SemanticRelation::HubMethodTargets,
                            subject,
                            object,
                            layer: "dotnet",
                            call_evidence: None,
                        })
                        .unwrap(),
                    );
                }
                expected_ids.extend(
                    class
                        .method_ids
                        .iter()
                        .filter(|(file, _)| file == files[part])
                        .flat_map(|(_, ids)| ids.iter().cloned()),
                );
            }
            let actual: Vec<_> = edges
                .iter()
                .filter(|e| e.subject.domain == "dotnet")
                .map(|e| serde_json::to_value(e).unwrap())
                .collect();
            assert_eq!(
                actual, expected,
                "exact ordered positive/negative role facts"
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
            assert_eq!(contexts, expected_ids, "ordered source-owned method IDs");
            assert_eq!(contexts.len(), roles * methods);
            eprintln!(
                "AF002_A2_DENSITY classes={population} roles={roles} methods_per_part={methods} owner={part} class_visits={:?} files={:?} method_owners={} raw_parts={:?} context_methods={} us={} boundary=artifact-projection",
                counts.csharp_role_class_visits,
                counts.csharp_class_file_visits,
                counts.csharp_method_owner_visits,
                counts.csharp_raw_part_visits,
                counts.csharp_context_method_visits,
                start.elapsed().as_micros()
            );
        }
    }
}
