//! Actual index traversal operation; setup excluded, no optimization budget.
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};
use crate::work_observations_af_perf_002 as work;
use crate::workspace::index::WorkspaceIndex;
fn edge(from: &str, to: &str) -> SemanticEdge {
    SemanticEdge {
        relation: SemanticRelation::Injects,
        subject: EntityRef::new("angular", "Service", from),
        object: EntityRef::new("angular", "Service", to),
        layer: "angular",
        call_evidence: None,
    }
}
#[test]
fn af002_traversal_unrelated_and_invocation_dimensions() {
    let mut axes: Vec<_> = [1, 2, 4, 8, 16, 64, 256]
        .into_iter()
        .map(|n| (n, 1))
        .collect();
    axes.extend([2, 4, 8, 16, 64, 256].into_iter().map(|n| (1, n)));
    axes.push((16, 16));
    for (unrelated, queries) in axes {
        let mut index = WorkspaceIndex::new();
        let mut edges = vec![edge("Start", "Reached"), edge("Reached", "Last")];
        edges.extend((0..unrelated).map(|n| edge(&format!("Other{n}"), &format!("Target{n}"))));
        index.add_edges("traversal.ts", edges);
        for (name, depth) in [("Start", 1), ("Start", 0), ("Absent", 0), ("Last", 0)] {
            work::reset();
            let start = std::time::Instant::now();
            for _ in 0..queries {
                let actual = index.transitive_dependencies("angular", "Service", name, depth);
                let expected_names: &[&str] = match (name, depth) {
                    ("Start", 1) => &["Reached"],
                    ("Start", _) => &["Reached", "Last"],
                    _ => &[],
                };
                let expected: Vec<_> = expected_names
                    .iter()
                    .map(|n| ("angular".to_owned(), "Service".to_owned(), (*n).to_owned()))
                    .collect();
                assert_eq!(actual, expected, "exact ordered BFS identities");
            }
            let c = work::snapshot();
            eprintln!(
                "AF002_TRAVERSAL unrelated_edges={unrelated} invocations={queries} start={name} depth={depth} node_builds={} source_key_visits={} unique_nodes={} reached_edge_visits={} us={} boundary=index-transitive-dependencies",
                c.traversal_node_builds,
                c.traversal_source_keys,
                c.traversal_unique_nodes,
                c.traversal_adjacency_edges,
                start.elapsed().as_micros()
            );
        }
    }
}

#[test]
fn af002_traversal_reachable_density_and_lifecycle() {
    for nodes in [1, 2, 4, 8, 16, 64, 256] {
        for shape in ["chain", "star", "dense"] {
            let mut edges = Vec::new();
            if nodes == 1 {
                edges.push(edge("Node0", "Node0"));
            }
            for from in 0..nodes {
                for to in 0..nodes {
                    let include = match shape {
                        "chain" => to == from + 1,
                        "star" => from == 0 && to > 0,
                        _ => from != to,
                    };
                    if include {
                        edges.push(edge(&format!("Node{from}"), &format!("Node{to}")));
                    }
                }
            }
            let edge_count = edges.len();
            let mut index = WorkspaceIndex::new();
            index.add_edges("density.ts", edges);
            for depth in [1, 0] {
                work::reset();
                let started = std::time::Instant::now();
                let actual = index.transitive_dependencies("angular", "Service", "Node0", depth);
                let elapsed = started.elapsed().as_micros();
                let limit = if shape == "chain" && depth == 1 {
                    nodes.min(2)
                } else {
                    nodes
                };
                let expected: Vec<_> = (1..limit)
                    .map(|n| {
                        (
                            "angular".to_owned(),
                            "Service".to_owned(),
                            format!("Node{n}"),
                        )
                    })
                    .collect();
                assert_eq!(actual, expected, "exact {shape} BFS order at depth {depth}");
                let c = work::snapshot();
                eprintln!(
                    "AF002_TRAVERSAL_DENSITY shape={shape} nodes={nodes} edges={edge_count} depth={depth} node_builds={} source_key_visits={} unique_nodes={} expanded_edge_visits={} result_count={} us={elapsed}",
                    c.traversal_node_builds,
                    c.traversal_source_keys,
                    c.traversal_unique_nodes,
                    c.traversal_adjacency_edges,
                    actual.len()
                );
            }
        }
    }
    let root = tempfile::tempdir().unwrap();
    let a = root.path().join("a");
    let b = root.path().join("b");
    std::fs::create_dir_all(&a).unwrap();
    std::fs::create_dir_all(&b).unwrap();
    let scope =
        crate::workspace::scope::WorkspaceScope::new(Some(a.to_str().unwrap()), &[]).unwrap();
    let first = a.join("first.ts").to_string_lossy().into_owned();
    let duplicate = a.join("duplicate.ts").to_string_lossy().into_owned();
    let other = b.join("other.ts").to_string_lossy().into_owned();
    let mut index = WorkspaceIndex::new();
    index.add_edges(
        &first,
        vec![edge("Start", "Reached"), edge("Reached", "Last")],
    );
    index.add_edges(&duplicate, vec![edge("Start", "Reached")]);
    index.add_edges(&other, vec![edge("Reached", "External")]);
    let expected = vec![
        (
            "angular".to_owned(),
            "Service".to_owned(),
            "Reached".to_owned(),
        ),
        (
            "angular".to_owned(),
            "Service".to_owned(),
            "Last".to_owned(),
        ),
    ];
    assert_eq!(
        index.transitive_dependencies_in_scope("angular", "Service", "Start", 0, &scope),
        expected
    );
    index.remove_file(&duplicate);
    assert_eq!(
        index.transitive_dependencies_in_scope("angular", "Service", "Start", -1, &scope),
        expected
    );
    index.remove_file(&first);
    assert!(
        index
            .transitive_dependencies_in_scope("angular", "Service", "Start", 0, &scope)
            .is_empty()
    );
    assert!(
        index
            .transitive_dependencies_in_scope("angular", "Service", "Reached", 0, &scope)
            .is_empty(),
        "outside-only start is refused"
    );
    index.add_edges(
        &first,
        vec![edge("Start", "Reached"), edge("Reached", "Last")],
    );
    assert_eq!(
        index.transitive_dependencies_in_scope("angular", "Service", "Start", 0, &scope),
        expected
    );
}
