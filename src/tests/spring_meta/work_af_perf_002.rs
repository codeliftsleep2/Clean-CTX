//! Production compile_focused Spring work, separate from HTTP/publication timing.
use crate::compression::Fidelity;
use crate::ir::compiler::IRCompiler;
use crate::layers::meta::semantic::SemanticRelation;
use crate::work_observations_af_perf_002 as work;
#[test]
fn af002_spring_class_and_method_growth() {
    let mut axes: Vec<_> = [1, 2, 4, 8, 16, 64, 256]
        .into_iter()
        .map(|n| (n, 1))
        .collect();
    axes.extend([2, 4, 8, 16, 64, 256].into_iter().map(|n| (1, n)));
    axes.push((16, 16));
    for (classes, methods) in axes {
        let source: String = (0..classes)
            .map(|c| {
                let members: String = (0..methods)
                    .map(|m| {
                        format!(r#"@GetMapping("/p{c}_{m}") String get{m}() {{ return "ok"; }} "#)
                    })
                    .collect();
                format!("@RestController class Controller{c} {{ {members} }}\n")
            })
            .collect();
        for fidelity in [Fidelity::Low, Fidelity::High] {
            let (language, query) =
                crate::compression::language::language_for_extension("java").unwrap();
            let mut compiler = IRCompiler::new();
            work::reset();
            crate::spring_meta::reset_detection_count();
            crate::spring_meta::reset_evaluation_count();
            let started = std::time::Instant::now();
            compiler
                .compile_focused(
                    &source,
                    "spring-work",
                    Some("Workload.java"),
                    language,
                    query,
                    fidelity,
                    None,
                    None,
                )
                .unwrap();
            let elapsed = started.elapsed().as_micros();
            let edges: Vec<_> = compiler
                .semantic_edges
                .iter()
                .filter(|e| e.layer == "spring")
                .collect();
            let expected: Vec<_> = if fidelity == Fidelity::Low {
                Vec::new()
            } else {
                (0..classes)
                    .flat_map(|c| {
                        (0..methods)
                            .map(move |m| (format!("Controller{c}"), format!("GET /p{c}_{m}")))
                    })
                    .collect()
            };
            assert_eq!(edges.len(), expected.len(), "actual captured Spring output");
            for (edge, (controller, endpoint)) in edges.iter().zip(expected) {
                assert_eq!(edge.relation, SemanticRelation::EndpointMapsTo);
                assert_eq!(
                    (
                        edge.subject.domain,
                        edge.subject.entity_type,
                        edge.subject.name.as_str()
                    ),
                    ("spring", "Controller", controller.as_str())
                );
                assert_eq!(
                    (
                        edge.object.domain,
                        edge.object.entity_type,
                        edge.object.name.as_str()
                    ),
                    ("spring", "Endpoint", endpoint.as_str())
                );
                assert_eq!(edge.subject.file, edge.object.file);
                assert_eq!(
                    edge.subject.file.as_deref(),
                    Some("Workload.java"),
                    "exact caller-supplied provenance"
                );
            }
            let c = work::snapshot();
            eprintln!(
                "AF002_SPRING classes={classes} methods={methods} fidelity={fidelity:?} source_bytes={} detection_calls={} evaluations={} detection_parses={} detection_parse_bytes={} method_scan_calls={} method_scan_bytes={} emitted_edges={} us={elapsed} boundary=production-compile-focused",
                source.len(),
                crate::spring_meta::detection_count(),
                crate::spring_meta::evaluation_count(),
                c.spring_detection_parses,
                c.spring_detection_parse_bytes,
                c.spring_method_scan_calls,
                c.spring_method_scan_bytes,
                edges.len()
            );
        }
    }
}

#[test]
fn af002_spring_plain_java_negative_control() {
    let source = r#"public class Plain { String get() { return "ok"; } }"#;
    let (language, query) = crate::compression::language::language_for_extension("java").unwrap();
    let mut compiler = IRCompiler::new();
    work::reset();
    crate::spring_meta::reset_detection_count();
    crate::spring_meta::reset_evaluation_count();
    compiler
        .compile_focused(
            source,
            "plain-work",
            Some("Plain.java"),
            language,
            query,
            Fidelity::High,
            None,
            None,
        )
        .unwrap();
    assert!(
        compiler
            .semantic_edges
            .iter()
            .all(|edge| edge.layer != "spring")
    );
    let counts = work::snapshot();
    eprintln!(
        "AF002_SPRING_NEGATIVE detection_calls={} evaluations={} detection_parses={} method_scan_calls={}",
        crate::spring_meta::detection_count(),
        crate::spring_meta::evaluation_count(),
        counts.spring_detection_parses,
        counts.spring_method_scan_calls
    );
}
