//! ANG-R2 public discovery, query contracts and bounded-work controls.
use super::angular_bootstrap_inline_controls_tests::call;
use super::angular_resolver_registration_support::{acquire_session, fixture, keys, query, raw};
use crate::mcp::McpState;
use serde_json::{Value, json};
use std::path::Path;

#[test]
fn issue_162_resolver_registration_cold_alias_barrel_namespace_local() {
    let _serial = crate::protocol::handler_response_serial();
    for form in ["alias", "barrel", "namespace", "local"] {
        for direction in ["forward", "reverse"] {
            // A new fixture, state and never-created durable DB for EACH direction.
            let (root, config) = fixture(form);
            assert!(!Path::new(&config.persistence.db_path).exists());
            if form == "barrel" {
                assert!(
                    !std::fs::read_to_string(root.path().join("one.ts"))
                        .unwrap()
                        .contains("Gate")
                );
            }
            let state = McpState::new(config);
            let answer = query(&state, root.path(), direction);
            assert_eq!(
                keys(&answer),
                ["primary", "secondary"],
                "{form} {direction}: {answer:#}"
            );
            assert_eq!(answer["count"], 2);
            for record in answer["registrations"].as_array().unwrap() {
                assert_eq!(record["resolver"]["name"], "Gate");
                assert_eq!(record["route_path"], "dashboard");
                let file = record["evidence"]["source"].as_str().unwrap();
                let source = std::fs::read(file).unwrap();
                assert_eq!(
                    record["evidence"]["source_hash"],
                    crate::cache::content_hash(&source)
                );
                let span = record["evidence"]["key"].as_array().unwrap();
                assert_eq!(
                    &source[span[0].as_u64().unwrap() as usize..span[1].as_u64().unwrap() as usize],
                    record["key"].as_str().unwrap().as_bytes()
                );
            }
        }
    }
}

#[test]
fn issue_162_resolver_registration_namesakes_occurrences_and_asserting_scope() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = fixture("direct");
    let p = root.path();
    let source = "import type {Routes} from '@angular/router'; import {Gate as Left} from './guard-a'; import {Gate as Right} from './guard-b'; export const routes:Routes=[{path:'dashboard',resolve:{primary:Left,secondary:Left}},{path:'dashboard',resolve:{primary:Right}}];";
    std::fs::write(p.join("one.ts"), source).unwrap();
    let state = McpState::new(config);
    let answer = query(&state, p, "reverse");
    assert_eq!(answer["count"], 3);
    let records = answer["registrations"].as_array().unwrap();
    assert_ne!(records[0]["route"], records[2]["route"]);
    assert_eq!(records[0]["resolver"], records[1]["resolver"]);
    assert_ne!(records[0]["resolver"], records[2]["resolver"]);
    assert_eq!(
        records[0]["resolver"]["name"],
        records[2]["resolver"]["name"]
    );
    for (within, expected) in [(p.join("one.ts"), 3), (p.join("guard-a.ts"), 0)] {
        let scoped = call(
            &state,
            p,
            "workspace_query",
            json!({"type":"resolver_registrations","direction":"reverse","resolver_name":"Gate","withinPath":within}),
        );
        assert_eq!(scoped["count"], expected);
    }
    let membership = call(
        &state,
        p,
        "workspace_query",
        json!({"type":"forward_edges","domain":"angular","entity_type":"Route","name":"dashboard"}),
    );
    assert_eq!(
        membership["edges"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|edge| edge["relation"] == "ResolvedBy")
            .count(),
        2
    );
}

#[test]
fn issue_162_resolver_registration_omission_attribution_has_no_guessed_target() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = fixture("direct");
    let p = root.path();
    std::fs::write(p.join("one.ts"), "import type {Routes} from '@angular/router'; import {Gate} from './guard-a'; export const routes:Routes=[{path:'dashboard',resolve:{good:Gate,duplicate:Gate,duplicate:()=>true,unknown:missing}},{path:'elsewhere',resolve:{bad:missing}}];").unwrap();
    let state = McpState::new(config);
    let forward = query(&state, p, "forward");
    assert_eq!(keys(&forward), ["good"]);
    assert_eq!(forward["semantic_omissions"].as_array().unwrap().len(), 3);
    assert_eq!(
        forward["candidate_source_omissions"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    for omission in forward["semantic_omissions"].as_array().unwrap() {
        assert_eq!(omission["attribution"], "selected_route");
        assert_eq!(omission["route_path"], "dashboard");
    }
    let reverse = query(&state, p, "reverse");
    assert_eq!(keys(&reverse), ["good"]);
    assert_eq!(reverse["semantic_omissions"], json!([]));
    assert_eq!(
        reverse["candidate_source_omissions"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
    for omission in reverse["candidate_source_omissions"].as_array().unwrap() {
        assert_eq!(omission["attribution"], "candidate_source");
        assert!(omission.get("resolver").is_none(), "{omission:#}");
    }
    let absent = call(
        &state,
        p,
        "workspace_query",
        json!({"type":"resolver_registrations","direction":"reverse","resolver_name":"NeverRegistered"}),
    );
    assert_eq!(absent["count"], 0);
    assert_eq!(absent["semantic_omissions"], json!([]));
    assert_eq!(
        absent["candidate_source_omissions"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
    assert_eq!(absent["coverage"]["omitted_possible"], true);
}

#[test]
fn issue_162_resolver_registration_batch_text_schema_and_invalid_requests() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = fixture("direct");
    let p = root.path();
    let state = McpState::new(config);
    crate::mcp::tool_handlers::query::work_metrics::reset();
    let response = raw(
        &state,
        json!({"workspaceRoot":p,"queries":[
            {"id":"f","type":"resolver_registrations","direction":"forward","route_path":"dashboard"},
            {"id":"bad","type":"resolver_registrations","direction":"reverse","resolver_name":"Gate","route_path":"dashboard"},
            {"id":"r","type":"resolver_registrations","direction":"reverse","resolver_name":"Gate"}
        ]}),
    );
    assert_eq!(
        crate::mcp::tool_handlers::query::work_metrics::snapshot().resolver_nominations,
        1,
        "F/R items share candidate acquisition without sharing selector semantics"
    );
    let results = &response["result"]["structuredContent"]["results"];
    assert_eq!(results[0]["status"], "ok", "{response:#}");
    assert_eq!(results[1]["status"], "error");
    assert_eq!(results[2]["status"], "ok");
    assert_eq!(
        results[0]["result"]["registrations"],
        results[2]["result"]["registrations"]
    );
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    let model: Value = serde_json::from_str(text.split_once('\n').unwrap().1).unwrap();
    assert_eq!(model["results"][0]["query"]["direction"], "forward");
    assert_eq!(model["results"][0]["query"]["route_path"], "dashboard");
    assert_eq!(model["results"][2]["query"]["resolver_name"], "Gate");
    assert_eq!(
        model["results"][0]["result"]["registrations"],
        results[0]["result"]["registrations"]
    );
    for args in [
        json!({"type":"resolver_registrations","direction":"forward","route_path":"dashboard"}),
        json!({"type":"resolver_registrations","workspaceRoot":p,"direction":"invalid","route_path":"dashboard"}),
        json!({"type":"resolver_registrations","workspaceRoot":p,"direction":"forward","route_path":["dashboard"]}),
        json!({"type":"resolver_registrations","workspaceRoot":p,"direction":"reverse","resolver_name":""}),
        json!({"type":"resolver_registrations","workspaceRoot":p,"direction":"reverse","resolver_name":null}),
    ] {
        assert_eq!(raw(&state, args)["error"]["code"], -32602);
    }
    let outside = tempfile::tempdir().unwrap();
    crate::mcp::tool_handlers::query::work_metrics::reset();
    let denied = raw(
        &state,
        json!({"workspaceRoot":p,"withinPath":outside.path(),"type":"resolver_registrations","direction":"forward","route_path":"dashboard"}),
    );
    assert_eq!(denied["error"]["code"], -32602);
    assert_eq!(
        crate::mcp::tool_handlers::query::work_metrics::snapshot().resolver_nominations,
        0
    );
    let tools = crate::mcp::tools::tool_list();
    let schema = &tools
        .iter()
        .find(|tool| tool["name"] == "workspace_query")
        .unwrap()["outputSchema"];
    assert_eq!(
        schema["properties"]["registrations"]["items"]["required"],
        json!(["route", "route_path", "key", "resolver", "evidence"])
    );
    assert_eq!(
        schema["properties"]["semantic_omissions"]["items"]["properties"]["attribution"]["enum"],
        json!(["selected_route", "selected_resolver"])
    );
}

#[test]
fn issue_162_resolver_registration_query_work_scales_by_owner_and_records() {
    let _serial = crate::protocol::handler_response_serial();
    use crate::mcp::tool_handlers::query::work_metrics;
    let mut baseline_owners = None;
    let mut previous_payload_delta = 0;
    for count in [2, 16, 64] {
        let (root, config) = fixture("direct");
        let file = root.path().join("one.ts");
        let source = std::fs::read_to_string(&file).unwrap();
        let mappings = (0..count)
            .map(|i| format!("key{i}:Gate"))
            .collect::<Vec<_>>()
            .join(",");
        std::fs::write(
            file,
            source.replace("primary:Gate,secondary:Gate", &mappings),
        )
        .unwrap();
        let state = McpState::new(config);
        let cold_started = std::time::Instant::now();
        let cold = query(&state, root.path(), "forward");
        let cold_elapsed = cold_started.elapsed();
        assert_eq!(cold["count"], count);
        work_metrics::reset();
        let started = std::time::Instant::now();
        let answer = query(&state, root.path(), "reverse");
        let warm_elapsed = started.elapsed();
        let observed = work_metrics::snapshot();
        assert_eq!(answer["count"], count);
        assert_eq!(observed.resolver_nominations, 1);
        assert_eq!(observed.resolver_records_considered, count);
        assert!(observed.resolver_owner_validations > 0);
        if let Some(owners) = baseline_owners {
            assert_eq!(observed.resolver_owner_validations, owners);
        }
        baseline_owners = Some(observed.resolver_owner_validations);
        let bytes = serde_json::to_vec(&answer).unwrap().len();
        acquire_session(&state, root.path());
        call(
            &state,
            root.path(),
            "save_context",
            json!({"filePath":root.path().join("one.ts")}),
        );
        let db = rusqlite::Connection::open(&state.config.persistence.db_path).unwrap();
        let encoded: String = db
            .query_row(
                "SELECT edges_json FROM semantic_edge_snapshots WHERE file_path=?1",
                [root.path().join("one.ts").to_string_lossy().into_owned()],
                |row| row.get(0),
            )
            .unwrap();
        let mut stored: Value = serde_json::from_str(&encoded).unwrap();
        let with_collection = serde_json::to_vec(&stored).unwrap().len();
        let _ = stored
            .as_object_mut()
            .unwrap()
            .remove("resolver_registrations");
        let payload_delta = with_collection - serde_json::to_vec(&stored).unwrap().len();
        assert!(payload_delta > previous_payload_delta);
        previous_payload_delta = payload_delta;
        eprintln!(
            "ANG-R2 registrations={count} owner_validations={} nominations={} records_considered={} response_bytes={bytes} durable_collection_bytes={payload_delta} cold={cold_elapsed:?} warm={warm_elapsed:?}",
            observed.resolver_owner_validations,
            observed.resolver_nominations,
            observed.resolver_records_considered
        );
    }
}

#[test]
fn issue_162_resolver_registration_decoded_keys_empty_path_and_duplicate_refusal() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = fixture("direct");
    let p = root.path();
    let file = p.join("one.ts");
    let original = std::fs::read_to_string(&file).unwrap();
    let state = McpState::new(config);
    let before = query(&state, p, "forward");
    std::fs::write(
        &file,
        original.replace("primary:Gate", "'pri\\u006dary':Gate"),
    )
    .unwrap();
    call(&state, p, "refresh_workspace", json!({}));
    let escaped = query(&state, p, "forward");
    assert_eq!(keys(&escaped), keys(&before));
    assert_ne!(
        escaped["registrations"][0]["route"],
        before["registrations"][0]["route"]
    );
    let record = &escaped["registrations"][0];
    let bytes = std::fs::read(&file).unwrap();
    let start = record["evidence"]["key"][0].as_u64().unwrap() as usize;
    let end = record["evidence"]["key"][1].as_u64().unwrap() as usize;
    assert_eq!(&bytes[start..end], b"'pri\\u006dary'");
    std::fs::write(&file, "import type {Routes} from '@angular/router'; import {Gate} from './guard-a'; export const routes:Routes=[{path:'dashboard',resolve:{primary:Gate,'pri\\u006dary':Gate,safe:Gate}},{path:'',resolve:{emptyRoute:Gate}}];").unwrap();
    call(&state, p, "refresh_workspace", json!({}));
    assert_eq!(keys(&query(&state, p, "forward")), ["safe"]);
    let empty = call(
        &state,
        p,
        "workspace_query",
        json!({"type":"resolver_registrations","direction":"forward","route_path":""}),
    );
    assert_eq!(keys(&empty), ["emptyRoute"]);
}

#[test]
fn issue_162_resolver_registration_unsupported_forms_never_gain_weak_fallback() {
    let _serial = crate::protocol::handler_response_serial();
    for (field, count) in [
        ("resolve:{...other,safe:Gate}", 0),
        ("resolve:{[computed]:Gate}", 0),
        ("resolve:{Gate}", 0),
        ("resolve:{inline:()=>true}", 0),
        ("resolve:{call:Gate()}", 0),
        ("resolve:{a:Gate},resolve:{b:Gate}", 0),
        ("resolve:{safe:Gate,unknown:missing}", 1),
    ] {
        let (root, config) = fixture("direct");
        let p = root.path();
        std::fs::write(p.join("one.ts"), format!("import type {{Routes}} from '@angular/router'; import {{Gate}} from './guard-a'; const other={{}}; const computed='data'; export const routes:Routes=[{{path:'dashboard',{field}}}];")).unwrap();
        let state = McpState::new(config);
        assert_eq!(query(&state, p, "forward")["count"], count, "{field}");
        let membership = call(
            &state,
            p,
            "workspace_query",
            json!({"type":"forward_edges","domain":"angular","entity_type":"Route","name":"dashboard"}),
        );
        assert_eq!(
            membership["edges"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|edge| edge["relation"] == "ResolvedBy")
                .count(),
            count
        );
    }
    let (root, config) = fixture("direct");
    let p = root.path();
    let file = p.join("one.ts");
    let source = std::fs::read_to_string(&file)
        .unwrap()
        .replace("import {Gate}", "import type {Gate}");
    std::fs::write(file, source).unwrap();
    let state = McpState::new(config);
    assert_eq!(query(&state, p, "reverse")["count"], 0);
}

#[cfg(feature = "rust")]
#[test]
fn issue_162_resolver_registration_required_publication_failure_is_not_empty_success() {
    let _hydration = crate::mcp::tool_handlers::hydration::TEST_PROJECT_HYDRATION_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = fixture("direct");
    let p = root.path();
    let state = McpState::new(config);
    crate::mcp::tool_handlers::hydration::fail_test_publication_once(
        &p.join("one.ts").to_string_lossy(),
    );
    let failed = raw(
        &state,
        json!({"workspaceRoot":p,"type":"resolver_registrations","direction":"forward","route_path":"dashboard"}),
    );
    assert_eq!(failed["error"]["code"], -32603, "{failed:#}");
    assert!(
        failed["result"].is_null(),
        "required failure must not masquerade as empty success"
    );
    assert_eq!(
        query(&state, p, "forward")["count"],
        2,
        "failure is retryable, not cached success"
    );
}
