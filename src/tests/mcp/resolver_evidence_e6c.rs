//! Registered public-boundary assertions; stdio receipts are a separate gate.
use super::*;
use crate::ir::typescript_resolution::e1_metrics;
use serde_json::json;
fn write(root: &std::path::Path, name: &str, text: &str) {
    let p = root.join(name);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, text).unwrap();
}
const APP: &str = "import type {Routes} from '@angular/router';export class UserResolver{};export const routes:Routes=[{path:'users',resolve:{user:UserResolver}}];";
fn budget() -> Value {
    json!({"elapsed_ms":10000,"source_bytes":2097152,"support_bytes":2097152,"retained_bytes":67108864,"active_expensive_tasks":2,"traversal_units":20000,"traversal_edges":4000,"validation_bytes":8388608,"validation_checks":20000,"response_bytes":1048576,"finalization":{"elapsed_ms":2000,"validation_bytes":4194304,"validation_checks":10000,"response_bytes":1048576}})
}
fn fixture() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "app.ts", APP);
    write(
        root.path(),
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","types":["node"]},"files":["app.ts"]}"#,
    );
    write(
        root.path(),
        "node_modules/@angular/router/package.json",
        r#"{"name":"@angular/router","version":"20.0.0","exports":{".":{"types":"./index.d.ts"}}}"#,
    );
    write(
        root.path(),
        "node_modules/@angular/router/index.d.ts",
        "export type Routes=unknown[];",
    );
    write(
        root.path(),
        "node_modules/@types/node/package.json",
        r#"{"name":"@types/node","version":"0.0.0-synthetic","types":"index.d.ts"}"#,
    );
    write(
        root.path(),
        "node_modules/@types/node/index.d.ts",
        "declare class Malformed {",
    );
    root
}
fn state() -> McpState {
    let mut state = McpState::new(crate::tests::test_config());
    state.resolver_evidence = State::configured(budget::Budget::parse(&budget()).unwrap());
    state
}
fn args(root: &std::path::Path) -> Value {
    json!({"workspaceRoot":root,"type":"resolver_registrations","contract_version":2,"direction":"forward","selector_basis":"route_literal","route_path":"users","source_files":["app.ts"],"budget":budget()})
}
fn call(state: &McpState, args: Value) -> Value {
    call_id(state, args, 71)
}
fn call_id(state: &McpState, args: Value, id: u64) -> Value {
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(
        &json!(id),
        "workspace_query",
        &json!({"arguments":args}),
        state,
    );
    crate::protocol::captured_responses()
        .pop()
        .expect("registered response")
}
fn dto(response: &Value) -> &Value {
    assert!(response.get("error").is_none(), "{response}");
    &response["result"]["structuredContent"]
}
#[test]
fn e6c_registered_forward_reverse_and_literal_absence_without_program_acquisition() {
    let root = fixture();
    let state = state();
    e1_metrics::reset();
    let result = call(&state, args(root.path()));
    let v = dto(&result);
    assert_eq!(v["observations"][0]["route_path"], "users");
    assert_eq!(
        v["interpretations"][0]["classification"],
        "strongly_supported"
    );
    assert_eq!(v["canonical_registrations"], json!([]));
    assert_eq!(
        serde_json::from_str::<Value>(result["result"]["content"][0]["text"].as_str().unwrap())
            .unwrap(),
        *v
    );
    let io = e1_metrics::snapshot();
    assert_eq!(
        (
            io.acquisition_results,
            io.automatic_type_calls,
            io.library_calls
        ),
        (0, 0, 0)
    );
    assert!(!io.source_reads.keys().any(|p| p.contains("@types")));
    assert_eq!(state.workspace_index_read().file_count(), 0);
    assert_eq!(state.workspace_index_read().edge_count(), 0);
    let mut reverse = args(root.path());
    reverse["direction"] = json!("reverse");
    reverse["selector_basis"] = json!("written_reference");
    reverse["resolver_name"] = json!("UserResolver");
    reverse.as_object_mut().unwrap().remove("route_path");
    assert_eq!(
        dto(&call(&state, reverse))["matching_records"]["returned"],
        1
    );
    let mut absent = args(root.path());
    absent["route_path"] = json!("missing");
    let result = call(&state, absent);
    assert_eq!(
        dto(&result)["search_conclusion"]["state"],
        "absent_within_closed_predicate"
    );
    assert_eq!(dto(&result)["coverage"]["angular"]["state"], "open");
}
#[test]
fn e6c_invalid_requests_and_mixed_batches_refuse_before_acquisition() {
    let root = fixture();
    let state = state();
    for (key, value) in [
        ("contract_version", json!(3)),
        ("mode", json!("exhaustive")),
        ("canonical_policy", json!("admitted_current")),
        ("budget", json!({})),
        ("source_files", json!([])),
        ("selector_basis", json!("canonical_identity")),
    ] {
        let mut a = args(root.path());
        a[key] = value;
        e1_metrics::reset();
        assert_eq!(call(&state, a)["error"]["code"], -32602);
        let io = e1_metrics::snapshot();
        assert_eq!(
            (io.bytes_read, io.parses, io.acquisition_results),
            (0, 0, 0)
        );
    }
    let a = args(root.path());
    let mut item = a.clone();
    for k in ["workspaceRoot", "budget"] {
        item.as_object_mut().unwrap().remove(k);
    }
    item["id"] = json!("v2");
    e1_metrics::reset();
    assert_eq!(
        call(
            &state,
            json!({"workspaceRoot":root.path(),"budget":budget(),"queries":[item,{"id":"old","type":"has_cycle"}]})
        )["error"]["code"],
        -32602
    );
    assert_eq!(e1_metrics::snapshot().parses, 0);
}
#[test]
fn e6c_operator_caps_and_default_advertisement_are_explicit() {
    let root = fixture();
    let state = state();
    let mut a = args(root.path());
    a["budget"]["source_bytes"] = json!(4194304);
    let response = call(&state, a);
    let v = dto(&response);
    assert_eq!(v["execution"]["requested_limits"]["source_bytes"], 4194304);
    assert_eq!(v["execution"]["effective_limits"]["source_bytes"], 2097152);
    assert_eq!(
        v["execution"]["effective_limits"]["active_expensive_tasks"],
        1
    );
    let tools = crate::mcp::tools::tool_list_for_state(&state);
    let tool = tools
        .iter()
        .find(|t| t["name"] == "workspace_query")
        .unwrap();
    assert_eq!(
        tool["resolverEvidenceCapabilities"]["contract_versions"],
        json!([1, 2])
    );
    assert!(
        tool["description"]
            .as_str()
            .unwrap()
            .contains("written_reference")
    );
    let mut disabled = state;
    disabled.resolver_evidence.ceiling = Ok(None);
    assert!(call(&disabled, args(root.path()))["error"].is_object());
    let tools = crate::mcp::tools::tool_list_for_state(&disabled);
    assert!(
        tools
            .iter()
            .find(|t| t["name"] == "workspace_query")
            .unwrap()
            .get("resolverEvidenceCapabilities")
            .is_none()
    );
}
#[test]
fn e6c_omitted_and_explicit_v1_are_identical_and_batch_uses_shared_allowance() {
    let root = fixture();
    let state = state();
    let v1 = json!({"type":"has_cycle","workspaceRoot":root.path()});
    let old = call(&state, v1.clone());
    let mut explicit = v1;
    explicit["contract_version"] = json!(1);
    assert_eq!(old, call(&state, explicit));
    let mut a = args(root.path());
    for k in ["workspaceRoot", "budget"] {
        a.as_object_mut().unwrap().remove(k);
    }
    a["id"] = json!("a");
    let mut b = a.clone();
    b["id"] = json!("b");
    let response = call(
        &state,
        json!({"workspaceRoot":root.path(),"budget":budget(),"queries":[a,b]}),
    );
    let v = dto(&response);
    assert_eq!(v["results"][0]["id"], "a");
    assert_eq!(v["results"][1]["id"], "b");
    assert_eq!(v["results"][0]["status"], "ok");
    assert_eq!(v["results"][1]["status"], "ok");
    assert_eq!(
        v["execution"]["used"]["source_bytes"],
        (APP.len() as u64 + 1) * 2
    );
    for item in v["results"].as_array().unwrap() {
        assert!(
            item["result"]["execution"]["used"]["response_bytes"]
                .as_u64()
                .unwrap()
                <= v["execution"]["used"]["response_bytes"].as_u64().unwrap()
        );
    }
    assert_eq!(
        serde_json::from_str::<Value>(response["result"]["content"][0]["text"].as_str().unwrap())
            .unwrap(),
        *v
    );
}
#[test]
fn e6c_relevant_grammar_failure_and_unknown_preserve_qualifications() {
    let root = fixture();
    let state = state();
    write(
        root.path(),
        "node_modules/@angular/router/index.d.ts",
        "export interface Routes {",
    );
    let response = call(&state, args(root.path()));
    let v = dto(&response);
    assert_eq!(v["matching_records"]["returned"], 1);
    assert_eq!(v["interpretations"][0]["classification"], "partial");
    write(
        root.path(),
        "app.ts",
        "const routes=[{path:'users',resolve:{...other}}];",
    );
    let response = call(&state, args(root.path()));
    assert_eq!(
        dto(&response)["search_conclusion"]["state"],
        "not_observed_unknown"
    );
}

#[test]
fn e6c_public_source_support_mutation_and_cancellation_are_qualified() {
    use crate::ir::source_observations::bounded::test_hooks;
    for mutation in ["source", "support", "config", "cancel", "finalization"] {
        let root = fixture();
        let state = std::sync::Arc::new(state());
        let held = state.clone();
        let path = root.path().to_path_buf();
        test_hooks::set_serving(move || match mutation {
            "source" => write(&path, "app.ts", "export const changed=1;"),
            "support" => write(
                &path,
                "node_modules/@angular/router/index.d.ts",
                "export type Routes=never;",
            ),
            "config" => write(
                &path,
                "tsconfig.json",
                r#"{"compilerOptions":{"types":[],"moduleResolution":"node"}}"#,
            ),
            "cancel" => held.resolver_evidence.cancel(&json!(71)),
            _ => {
                let r = held.resolver_evidence.request(&json!(71));
                let remaining =
                    r.limits().total.validation_checks - r.snapshot().used.validation_checks;
                let _ = r
                    .reserve_nested(
                        Cost {
                            validation_checks: remaining,
                            ..Cost::default()
                        },
                        true,
                    )
                    .unwrap();
            }
        });
        let response = call(&state, args(root.path()));
        let v = dto(&response);
        if matches!(mutation, "source" | "finalization") {
            assert_eq!(v["matching_records"]["returned"], 0);
            assert_eq!(v["search_conclusion"]["state"], "not_observed_unknown");
        } else {
            assert_eq!(v["matching_records"]["returned"], 1);
            assert_eq!(v["interpretations"][0]["classification"], "partial");
        }
        if mutation == "cancel" {
            assert_eq!(v["execution"]["state"], "cancelled");
        }
        assert_eq!(
            serde_json::from_str::<Value>(
                response["result"]["content"][0]["text"].as_str().unwrap()
            )
            .unwrap(),
            *v
        );
    }
}

#[test]
fn e6c_shared_batch_exhaustion_withholding_and_minimum_failure() {
    let root = fixture();
    let state = state();
    let mut a = args(root.path());
    a["budget"]["response_bytes"] = json!(100000);
    a["budget"]["finalization"]["response_bytes"] = json!(100000);
    let response = call(&state, a);
    let v = dto(&response);
    assert_eq!(v["matching_records"]["established"], 1);
    assert_eq!(v["matching_records"]["returned"], 0);
    assert_eq!(v["search_conclusion"]["state"], "observed_not_returned");
    let mut item = args(root.path());
    for k in ["workspaceRoot", "budget"] {
        item.as_object_mut().unwrap().remove(k);
    }
    item["id"] = json!("a");
    let mut second = item.clone();
    second["id"] = json!("b");
    let mut allowance = budget();
    allowance["source_bytes"] = json!(APP.len() + 1);
    let response = call(
        &state,
        json!({"workspaceRoot":root.path(),"budget":allowance,"queries":[item,second]}),
    );
    let v = dto(&response);
    assert_eq!(v["results"][0]["result"]["matching_records"]["returned"], 1);
    assert_eq!(
        v["results"][1]["result"]["search_conclusion"]["state"],
        "not_observed_unknown"
    );
    assert_eq!(v["execution"]["used"]["source_bytes"], APP.len() + 1);
    let mut small = args(root.path());
    small["budget"]["response_bytes"] = json!(1);
    small["budget"]["finalization"]["response_bytes"] = json!(1);
    e1_metrics::reset();
    assert_eq!(call(&state, small)["error"]["code"], -32004);
    assert_eq!(e1_metrics::snapshot().parses, 0);
}

#[test]
fn e6c_implicit_automatic_types_inventory_exclusions_and_context_competitors() {
    let root = fixture();
    let mut state = state();
    write(
        root.path(),
        "tsconfig.json",
        r#"{"compilerOptions":{"moduleResolution":"bundler","module":"esnext"}}"#,
    );
    for n in 0..4 {
        write(
            root.path(),
            &format!("feature{n}/unrelated.ts"),
            "export const unrelated=1;",
        );
    }
    write(root.path(), "skip-me/routes.ts", APP);
    state.config.exclude_patterns.push("skip-me".into());
    let mut a = args(root.path());
    a.as_object_mut().unwrap().remove("source_files");
    a["route_path"] = json!("missing");
    e1_metrics::reset();
    let response = call(&state, a);
    let v = dto(&response);
    assert_eq!(
        v["examined_scope"]["enumerated"].as_array().unwrap().len(),
        5
    );
    assert_eq!(
        v["search_conclusion"]["state"],
        "absent_within_closed_predicate"
    );
    let io = e1_metrics::snapshot();
    assert_eq!(
        (
            io.acquisition_results,
            io.automatic_type_calls,
            io.library_calls,
            io.dependency_parses
        ),
        (0, 0, 0, 0)
    );
    assert!(
        v["execution"]["inventory_work"]["directory_enumerations"]
            .as_u64()
            .unwrap()
            > 0
    );
    write(
        root.path(),
        "app.ts",
        &format!(
            "{APP} declare module '@angular/router' {{export interface Route {{conflict:string;}}}}"
        ),
    );
    let response = call(&state, args(root.path()));
    assert_eq!(
        dto(&response)["interpretations"][0]["classification"],
        "partial"
    );
}

#[test]
fn e6c_concurrent_public_calls_are_request_isolated_and_cancel_slots_retire() {
    let root = fixture();
    let state = state();
    let results = std::thread::scope(|scope| {
        let a = scope.spawn(|| call(&state, args(root.path())));
        let b = scope.spawn(|| call_id(&state, args(root.path()), 72));
        vec![a.join().unwrap(), b.join().unwrap()]
    });
    for r in results {
        assert_eq!(dto(&r)["matching_records"]["returned"], 1);
        assert_eq!(dto(&r)["execution"]["used"]["source_bytes"], APP.len() + 1);
    }
    let request = crate::protocol::JsonRpcRequest {
        jsonrpc: "2.0".into(),
        id: Some(json!(71)),
        method: "tools/call".into(),
        params: Some(json!({"name":"workspace_query","arguments":args(root.path())})),
    };
    state.resolver_evidence.queue(&request).unwrap();
    state.resolver_evidence.cancel(&json!(71));
    let response = call(&state, args(root.path()));
    assert_eq!(dto(&response)["execution"]["state"], "cancelled");
    state.resolver_evidence.queue(&request).unwrap();
    state.resolver_evidence.retire(&json!(71));
}

#[test]
fn e6c_legacy_node_discovery_fixture_and_v2_avoidance_are_distinct() {
    for implicit in [false, true] {
        let root = fixture();
        let state = state();
        write(
            root.path(),
            "app.ts",
            &APP.replace(
                "class UserResolver{}",
                "class UserResolver{value!:NodeJS.Process;}",
            ),
        );
        let mut options = json!({"module":"esnext","moduleResolution":"bundler","noLib":true});
        if !implicit {
            options["types"] = json!(["node"]);
        }
        write(
            root.path(),
            "tsconfig.json",
            &json!({"compilerOptions":options,"files":["app.ts"]}).to_string(),
        );
        write(
            root.path(),
            "node_modules/typescript/package.json",
            r#"{"name":"typescript","version":"5.9.3"}"#,
        );
        e1_metrics::reset();
        let mut v2 = args(root.path());
        v2["route_path"] = json!("missing");
        let response = call(&state, v2);
        assert_eq!(
            dto(&response)["search_conclusion"]["state"],
            "absent_within_closed_predicate"
        );
        let io = e1_metrics::snapshot();
        assert_eq!(
            (
                io.acquisition_results,
                io.automatic_type_calls,
                io.library_calls,
                io.dependency_parses
            ),
            (0, 0, 0, 0)
        );
        let legacy = json!({"workspaceRoot":root.path(),"type":"resolver_registrations","direction":"forward","route_path":"missing"});
        e1_metrics::reset();
        let probe = crate::ir::source_observations::request_budget::telemetry::legacy_probe::start(
            root.path(),
        );
        let omitted = call(&state, legacy.clone());
        let io = probe.snapshot();
        eprintln!(
            "E6C_LEGACY_NODE implicit={implicit} counts={}",
            serde_json::to_string(&io).unwrap()
        );
        assert!(
            io.automatic > 0,
            "legacy did not reach automatic types, implicit={implicit}: {io:?}"
        );
        assert!(
            io.program > 0,
            "legacy did not attempt program acquisition: {io:?}"
        );
        let mut explicit = legacy;
        explicit["contract_version"] = json!(1);
        let cold = self::state();
        let versioned = call(&cold, explicit);
        assert_eq!(
            omitted.get("error").map(|e| &e["code"]),
            versioned.get("error").map(|e| &e["code"])
        );
        assert_eq!(omitted.get("result"), versioned.get("result"));
    }
}

#[test]
fn e6c_pending_response_capacity_is_not_reported_as_completed_work() {
    let root = fixture();
    let session = Session::new(
        vec![root.path().to_path_buf()],
        budget::Budget::parse(&budget()).unwrap().limits().unwrap(),
    )
    .unwrap();
    let request = session.ledger();
    let remaining = request.response_available();
    let credit = request.response_capacity(10000).unwrap();
    assert_eq!(request.snapshot().used.response_bytes, 0);
    assert_eq!(request.snapshot().pending_response_bytes, 10000);
    assert_eq!(request.response_available(), remaining - 10000);
    credit.commit(1000).unwrap();
    assert_eq!(request.snapshot().used.response_bytes, 1000);
    assert_eq!(request.snapshot().pending_response_bytes, 0);
    assert_eq!(request.response_available(), remaining - 1000);
    let credit = request.response_capacity(10000).unwrap();
    drop(credit);
    assert_eq!(request.snapshot().used.response_bytes, 1000);
    assert_eq!(request.snapshot().pending_response_bytes, 0);
}

#[test]
fn e6c_public_finalization_memory_is_reserved_before_optional_expansion() {
    let root = fixture();
    let state = std::sync::Arc::new(state());
    let held = state.clone();
    let competing = std::sync::Arc::new(std::sync::Mutex::new(None));
    let store = competing.clone();
    crate::ir::source_observations::bounded::test_hooks::set_serving(move || {
        let request = held.resolver_evidence.request(&json!(71));
        let snapshot = request.snapshot();
        let limits = request.limits();
        let protected = limits
            .finalization
            .retained_bytes
            .saturating_sub(snapshot.finalization_used.retained_bytes);
        let free = limits.total.retained_bytes - snapshot.used.retained_bytes - protected;
        *store.lock().unwrap() = Some(
            request
                .reserve_input(Cost {
                    retained_bytes: free,
                    ..Cost::default()
                })
                .unwrap(),
        );
    });
    let response = call(&state, args(root.path()));
    let value = dto(&response);
    assert_eq!(value["matching_records"]["returned"], 1);
    assert_eq!(value["interpretations"][0]["classification"], "partial");
    assert_eq!(value["execution"]["state"], "exhausted");
    drop(competing);
}
