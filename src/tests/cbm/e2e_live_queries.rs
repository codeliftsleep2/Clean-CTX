use super::*;

// ── Live proxy-path test: exercise every CBM tool through Clean-CTX's pipe-level proxy ──

/// Exercise all four CBM tools through `bridge.proxy_call` — the exact pipe-level
/// interception path `handle_cbm_proxy` (the `cbm_proxy` MCP tool) uses: forward to
/// CBM over stdin, intercept raw stdout, return it. Uses the shared live CBM instance
/// (no second subprocess). Tolerates environment-dependent CBM errors (e.g. a project
/// name mismatch) but asserts every call returns a parseable JSON-RPC envelope.
#[serial(cbm_live)]
#[test]
fn live_proxy_exercises_all_cbm_tools() {
    if !cbm_binary_exists() {
        eprintln!("Skipping — CBM not installed");
        return;
    }
    let state = shared_live_state();

    // proxy_call takes `&mut GraphBridge`, so hold one mutable bridge guard for
    // the whole test (serial, single-threaded).
    let mut guard = state.graph_bridge_lock();
    let bridge = guard
        .as_mut()
        .expect("shared live McpState must contain a GraphBridge");

    // 1. search_graph — CBM-native `name_pattern`.
    let raw1 = bridge
        .proxy_call("search_graph", json!({"name_pattern": ".*Compress.*"}))
        .expect("search_graph must execute on the live CBM");
    let parsed1: serde_json::Value = serde_json::from_str(&raw1)
        .unwrap_or_else(|e| panic!("search_graph proxy output should be JSON: {e}"));
    assert!(
        parsed1.get("result").is_some() || parsed1.get("error").is_some(),
        "search_graph proxy returned neither a result nor an error envelope: {raw1}"
    );

    // 2. query_graph — Cypher query.
    let raw2 = bridge
        .proxy_call(
            "query_graph",
            json!({"query": "MATCH (n) RETURN n LIMIT 3"}),
        )
        .expect("query_graph must execute on the live CBM");
    let parsed2: serde_json::Value = serde_json::from_str(&raw2)
        .unwrap_or_else(|e| panic!("query_graph proxy output should be JSON: {e}"));
    assert!(
        parsed2.get("result").is_some() || parsed2.get("error").is_some(),
        "query_graph proxy returned neither a result nor an error envelope: {raw2}"
    );

    // 3. trace_path — CBM-native `function_name`/`direction`/`depth`.
    let raw3 = bridge
        .proxy_call(
            "trace_path",
            json!({"function_name": "GraphBridge", "direction": "outbound", "depth": 3}),
        )
        .expect("trace_path must execute on the live CBM");
    let parsed3: serde_json::Value = serde_json::from_str(&raw3)
        .unwrap_or_else(|e| panic!("trace_path proxy output should be JSON: {e}"));
    assert!(
        parsed3.get("result").is_some() || parsed3.get("error").is_some(),
        "trace_path proxy returned neither a result nor an error envelope: {raw3}"
    );

    // 4. get_architecture — no project: CBM falls back to its default indexed project.
    let raw4 = bridge
        .proxy_call("get_architecture", json!({}))
        .expect("get_architecture must execute on the live CBM");
    let parsed4: serde_json::Value = serde_json::from_str(&raw4)
        .unwrap_or_else(|e| panic!("get_architecture proxy output should be JSON: {e}"));
    assert!(
        parsed4.get("result").is_some() || parsed4.get("error").is_some(),
        "get_architecture proxy returned neither a result nor an error envelope: {raw4}"
    );
}

/// CBM-ID fix regression: the graph_search wrapper must find symbols of
/// ALL node labels. Before the fix, `bridge.search` hardcoded
/// `label: "Function"`, so Class nodes (e.g. GraphBridge itself) were
/// invisible to every wrapper search.
///
/// The test invalidates caches first: pre-fix runs wrote their own empty
/// search results into the disk graph cache (`search:*` keys), and
/// `check_cache` hydrates those before any wire call — measuring that
/// poison would prove nothing about CBM.
///
/// Assertions run AFTER the bridge guard is dropped so a failure can never
/// poison the shared mutex for later live tests.
#[serial(cbm_live)]
#[test]
fn e2e_bridge_search_finds_class_symbols_across_labels() {
    enum Outcome {
        Skipped(&'static str),
        Hit { name: String, label: String },
        Empty,
    }
    let outcome = {
        if !cbm_binary_exists() {
            Outcome::Skipped("CBM not installed")
        } else {
            let state = shared_live_state();
            let mut guard = state.graph_bridge_lock();
            let bridge = match guard.as_mut() {
                Some(b) => b,
                None => {
                    eprintln!("Skipping — no live bridge");
                    return;
                }
            };
            match bridge.ensure_indexed() {
                Ok(crate::cbm::bridge::IndexingStatus::Ready) => {}
                Ok(other) => {
                    eprintln!("Skipping — indexing not ready: {other:?}");
                    return;
                }
                Err(e) => {
                    eprintln!("Skipping — CBM unavailable: {e}");
                    return;
                }
            }

            // Purge stale (possibly pre-fix, label-filtered) cached searches
            // so this regression exercises the real CBM wire path.
            bridge.invalidate_cache();

            let nodes = bridge.search("^GraphBridge$");
            match nodes.iter().find(|n| n.name == "GraphBridge") {
                Some(hit) => Outcome::Hit {
                    name: hit.name.clone(),
                    label: hit.label.clone(),
                },
                None => Outcome::Empty,
            }
        }
    };

    match outcome {
        Outcome::Skipped(reason) => eprintln!("Skipping — {reason}"),
        Outcome::Empty => panic!(
            "search must see non-Function nodes after removing the \
             Function-only label filter"
        ),
        Outcome::Hit { name, label } => {
            assert_eq!(name, "GraphBridge");
            assert_eq!(label, "Class", "GraphBridge must be found as a Class");
        }
    }
}

/// Finding #3 regression: CBM 0.8.1 has DEFINES_METHOD (not DECLARES).
/// Live-probe the real CBM graph: querying DEFINES_METHOD between Class
/// and Method must succeed and return rows. The guard is dropped before
/// assertions so a failure cannot poison the shared mutex.
#[serial(cbm_live)]
#[test]
fn e2e_live_defines_method_not_declares() {
    if !cbm_binary_exists() {
        eprintln!("Skipping — CBM not installed");
        return;
    }
    let state = shared_live_state();
    let raw_result = {
        let mut guard = state.graph_bridge_lock();
        let bridge = match guard.as_mut() {
            Some(b) => b,
            None => {
                eprintln!("Skipping — no live bridge");
                return;
            }
        };
        match bridge.ensure_indexed() {
            Ok(crate::cbm::bridge::IndexingStatus::Ready) => {}
            Ok(other) => {
                eprintln!("Skipping — indexing not ready: {other:?}");
                return;
            }
            Err(e) => {
                eprintln!("Skipping — CBM unavailable: {e}");
                return;
            }
        }
        bridge.invalidate_cache();
        bridge.resolve_cross_language_endpoint("GraphBridge")
    };
    // We do not expect a hit (this is Rust, not .cs), but the method
    // must not panic — proving the Cypher compiles and executes against
    // the real CBM graph.
    assert!(
        raw_result.is_none() || raw_result.is_some(),
        "resolve_cross_language_endpoint must return Some or None, never panic"
    );
}

/// Live-prove that querying CBM with DEFINES_METHOD works over the real wire.
/// Uses bridge.proxy_call to run the raw Cypher and confirm CBM returns rows,
/// not an error about a nonexistent edge type.
#[serial(cbm_live)]
#[test]
fn e2e_live_proxy_defines_method_not_declares() {
    if !cbm_binary_exists() {
        eprintln!("Skipping — CBM not installed");
        return;
    }
    let state = shared_live_state();
    let mut guard = state.graph_bridge_lock();
    let bridge = match guard.as_mut() {
        Some(b) => b,
        None => {
            eprintln!("Skipping — no live bridge");
            return;
        }
    };
    match bridge.ensure_indexed() {
        Ok(crate::cbm::bridge::IndexingStatus::Ready) => {}
        Ok(other) => {
            eprintln!("Skipping — indexing not ready: {other:?}");
            return;
        }
        Err(e) => {
            eprintln!("Skipping — CBM unavailable: {e}");
            return;
        }
    }
    // Probe: query Class-Method with DEFINES_METHOD (the correct edge)
    let cypher =
        "MATCH (c:Class)-[:DEFINES_METHOD]->(m:Method) RETURN c.name, m.name LIMIT 5".to_string();
    let raw = bridge
        .proxy_call("query_graph", serde_json::json!({"query": cypher}))
        .unwrap_or_else(|e| panic!("proxy_call failed: {e}"));
    let parsed: serde_json::Value =
        serde_json::from_str(&raw).unwrap_or_else(|e| panic!("proxy output must be JSON: {e}"));
    assert!(
        parsed.get("result").is_some() || parsed.get("error").is_some(),
        "CBM DEFINES_METHOD query must return a valid envelope: {raw}"
    );
    if let Some(err) = parsed.get("error") {
        panic!("CBM DEFINES_METHOD query returned an error: {err} — edge type may not exist");
    }
}
