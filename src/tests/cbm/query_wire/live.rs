use super::*;

// ── Fresh-process live probes over the SYNTHETIC fixture repo ────────

fn cbm_binary_exists() -> bool {
    let name = if cfg!(windows) {
        "codebase-memory-mcp.exe"
    } else {
        "codebase-memory-mcp"
    };
    std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).any(|dir| dir.join(name).is_file()))
        .unwrap_or(false)
}

fn live_config() -> CbmConfig {
    CbmConfig {
        enabled: true,
        ..Default::default()
    }
}

fn wait_ready(bridge: &mut GraphBridge) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
    loop {
        match bridge.ensure_indexed() {
            Ok(crate::cbm::bridge::IndexingStatus::Ready) => return,
            Ok(crate::cbm::bridge::IndexingStatus::StillIndexing { .. }) => {
                assert!(
                    std::time::Instant::now() < deadline,
                    "timed out waiting for CBM indexing of the query fixture"
                );
                std::thread::sleep(std::time::Duration::from_millis(250));
            }
            Err(e) => {
                eprintln!("Indexing failed (continuing): {e}");
                return;
            }
        }
    }
}

/// caller → callee is the ONLY relationship; nothing derives from this repo.
const FIXTURE_RS: &str = r#"
pub fn tw_probe_callee() -> i32 { 7 }
pub fn tw_probe_caller() -> i32 { tw_probe_callee() + 1 }
"#;

fn fresh_fixture_bridge() -> (tempfile::TempDir, GraphBridge) {
    let root = tempfile::Builder::new()
        .prefix("cleanctx_query_wire_")
        .tempdir()
        .expect("fixture tempdir");
    std::fs::write(root.path().join("query_fixture.rs"), FIXTURE_RS)
        .expect("write query fixture source");
    let mut bridge = GraphBridge::try_create(&live_config(), root.path());
    wait_ready(&mut bridge);
    (root, bridge)
}

/// THE regression: typed `graph_query` now surfaces relationship rows AS
/// EDGES. Pre-fix this exact projection collapsed to duplicated column-0
/// nodes with `"edges":[]`.
#[serial(cbm_live)]
#[test]
fn live_typed_graph_query_returns_edges_for_relationship_projection() {
    if !cbm_binary_exists() {
        eprintln!("Skipping — CBM not installed");
        return;
    }
    let (_root, mut bridge) = fresh_fixture_bridge();

    let qr = bridge.query_graph(
        "MATCH (a)-[r:CALLS]->(b) WHERE a.name = 'tw_probe_caller' \
         RETURN a.name, type(r), b.name",
    );
    assert!(bridge.take_last_error().is_none(), "query must succeed");
    assert_eq!(
        qr.edges.len(),
        1,
        "relationship rows become edges: {:?}",
        qr.edges
    );
    assert_eq!(qr.edges[0].from, "tw_probe_caller");
    // Endpoint identity is EXACTLY what the projection asks for: `.name`
    // yields the bare symbol (matches the directed-CALLS raw capture where
    // b.name arrived bare, e.g. "try_create").
    assert_eq!(qr.edges[0].to, "tw_probe_callee");
    assert_eq!(qr.edges[0].label, "CALLS");
    assert!(qr.nodes.is_empty(), "shape conversion synthesizes no nodes");

    // Same relationship, QUALIFIED projection: endpoints become the fully
    // qualified wire identities, verbatim.
    let qqr = bridge.query_graph(
        "MATCH (a)-[r:CALLS]->(b) WHERE a.name = 'tw_probe_caller' \
         RETURN a.qualified_name, type(r), b.qualified_name",
    );
    assert!(bridge.take_last_error().is_none());
    assert_eq!(qqr.edges.len(), 1, "{:?}", qqr.edges);
    assert!(
        qqr.edges[0].from.ends_with(".tw_probe_caller"),
        "{}",
        qqr.edges[0].from
    );
    assert!(
        qqr.edges[0].to.ends_with(".tw_probe_callee"),
        "{}",
        qqr.edges[0].to
    );
    assert_eq!(qqr.edges[0].label, "CALLS");
}

/// Preserved behavior: node-only projections keep producing nodes.
#[serial(cbm_live)]
#[test]
fn live_node_only_projection_still_returns_nodes() {
    if !cbm_binary_exists() {
        eprintln!("Skipping — CBM not installed");
        return;
    }
    let (_root, mut bridge) = fresh_fixture_bridge();

    let qr = bridge.query_graph("MATCH (f:Function) RETURN f.name LIMIT 5");
    assert!(bridge.take_last_error().is_none());
    assert!(!qr.nodes.is_empty(), "node-only projections unchanged");
    assert!(qr.edges.is_empty());
    assert!(
        qr.nodes.iter().any(|n| n.name == "tw_probe_caller"),
        "fixture functions visible: {:?}",
        qr.nodes
    );
}

/// Wide SCRAMBLED relationship projection (5 columns, type() mid-projection):
/// endpoints and properties resolve purely from the echoed column metadata.
/// Projection order rules: first non-type column => `from`, LAST non-type
/// column => `to` — here the projection asks for callee-first.
#[serial(cbm_live)]
#[test]
fn live_wide_scrambled_projection_maps_columns_by_shape() {
    if !cbm_binary_exists() {
        eprintln!("Skipping — CBM not installed");
        return;
    }
    let (_root, mut bridge) = fresh_fixture_bridge();

    let qr = bridge.query_graph(
        "MATCH (a)-[r:CALLS]->(b) WHERE a.name = 'tw_probe_caller' \
         RETURN b.name, b.qualified_name, type(r), a.name",
    );
    assert!(bridge.take_last_error().is_none(), "query must succeed");
    assert_eq!(qr.edges.len(), 1, "{:?}", qr.edges);
    let edge = &qr.edges[0];
    assert_eq!(edge.from, "tw_probe_callee", "first non-type column");
    assert_eq!(edge.to, "tw_probe_caller", "LAST non-type column");
    assert_eq!(edge.label, "CALLS", "type column becomes the label");
    let qn = edge
        .properties
        .get("b.qualified_name")
        .and_then(serde_json::Value::as_str)
        .expect("middle projected column becomes a property");
    assert!(
        qn.ends_with(".tw_probe_callee"),
        "property value preserved as projected: {qn}"
    );
}

/// The two fabrication guards, live: an ALIASED type() projection (marker
/// erased by CBM) and a numeric triple must both stay node-shaped — never
/// edges.
#[serial(cbm_live)]
#[test]
fn live_non_relationship_projections_never_become_edges() {
    if !cbm_binary_exists() {
        eprintln!("Skipping — CBM not installed");
        return;
    }
    let (_root, mut bridge) = fresh_fixture_bridge();

    // Aliased type(): CBM echoes only `caller`/`rel_kind`/`callee`.
    let aliased = bridge.query_graph(
        "MATCH (a)-[r:CALLS]->(b) WHERE a.name = 'tw_probe_caller' \
         RETURN a.name AS caller, type(r) AS rel_kind, b.name AS callee",
    );
    assert!(bridge.take_last_error().is_none());
    assert!(
        aliased.edges.is_empty(),
        "aliased type() is intentionally undetectable: {:?}",
        aliased.edges
    );
    assert!(!aliased.nodes.is_empty(), "falls back to nodes");

    // Numeric triple — the exact shape the arity rule fabricated
    // a `"10"`-labelled edge from.
    let nums = bridge.query_graph("MATCH (f:Function) RETURN f.name, f.in_degree, f.out_degree");
    assert!(bridge.take_last_error().is_none());
    assert!(
        nums.edges.is_empty(),
        "numeric triples must never become edges: {:?}",
        nums.edges
    );
    assert!(
        nums.nodes.iter().any(|n| n.name == "tw_probe_callee"),
        "node mapping preserved: {:?}",
        nums.nodes
    );
}
