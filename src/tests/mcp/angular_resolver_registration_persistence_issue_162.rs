//! ANG-R2 complete admission, SQLite/replay and producer compatibility.
use super::angular_bootstrap_inline_controls_tests::call;
use super::angular_resolver_registration_support::{acquire_session, fixture, keys, query};
use crate::mcp::McpState;
use serde_json::{Value, json};

#[test]
fn issue_162_resolver_registration_remaining_additional_root_refresh() {
    let _serial = crate::protocol::handler_response_serial();
    let (primary, mut config) = fixture("direct");
    let (additional, _) = fixture("direct");
    config
        .additional_roots
        .push(additional.path().to_string_lossy().into_owned());
    let state = McpState::new(config);
    let before = query(&state, primary.path(), "forward");
    assert_eq!(before["count"], 4, "{before:#}");
    let selected = call(
        &state,
        primary.path(),
        "workspace_query",
        json!({
            "type":"resolver_registrations", "direction":"forward", "route_path":"dashboard",
            "withinPath":additional.path()
        }),
    );
    assert_eq!(selected["count"], 2);
    assert!(
        selected["registrations"]
            .as_array()
            .unwrap()
            .iter()
            .all(|row| row["evidence"]["source"]
                .as_str()
                .unwrap()
                .starts_with(&*additional.path().to_string_lossy()))
    );
    let file = additional.path().join("one.ts");
    let source = std::fs::read_to_string(&file).unwrap();
    std::fs::write(&file, source.replace("secondary:", "replaced:")).unwrap();
    call(
        &state,
        primary.path(),
        "refresh_workspace",
        json!({"scope":"configured_roots"}),
    );
    assert!(
        state
            .workspace_index_read()
            .semantic_projection_owners()
            .is_empty()
    );
    let after = query(&state, primary.path(), "forward");
    assert_eq!(
        keys(&after),
        ["primary", "primary", "replaced", "secondary"]
    );
    assert_eq!(
        query(&state, primary.path(), "reverse")["registrations"],
        after["registrations"]
    );
}

#[test]
fn issue_162_resolver_registration_remaining_edit_and_delta_transport() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = fixture("direct");
    let p = root.path();
    let file = p.join("one.ts");
    let routing = std::fs::read_to_string(&file).unwrap();
    let initial = format!(
        "{routing}\nexport class Worker {{ run():number {{return 1;}} }}\n{}",
        "// stable-padding-0123456789abcdef\n".repeat(1000)
    );
    std::fs::write(&file, &initial).unwrap();
    let state = McpState::new(config.clone());
    call(
        &state,
        p,
        "provide_code_context",
        json!({"filePath":file,"fidelity":"edit"}),
    );
    let alias = state.alias_for_path(&file.to_string_lossy()).unwrap();
    let before = state
        .semantic_snapshot(&alias)
        .unwrap()
        .authority
        .resolver_registrations
        .unwrap();
    call(
        &state,
        p,
        "apply_edit",
        json!({"filePath":file,"operations":[{
            "type":"replace_body", "target":"Worker.run", "expectedOldText":"{return 1;}", "newText":"{return 2;}"
        }]}),
    );
    let edited = state
        .semantic_snapshot(&alias)
        .unwrap()
        .authority
        .resolver_registrations
        .unwrap();
    assert_eq!(edited.records.len(), 2);
    assert_ne!(
        edited.records[0].evidence.source_hash,
        before.records[0].evidence.source_hash
    );
    assert_eq!(
        edited
            .records
            .iter()
            .map(|r| r.key.as_str())
            .collect::<Vec<_>>(),
        ["primary", "secondary"]
    );
    let target = std::fs::read_to_string(&file)
        .unwrap()
        .replace("secondary:", "renamed:")
        .replace("return 2;", "return 3;");
    std::fs::write(&file, &target).unwrap();
    state.invalidate_source_cache(&file.to_string_lossy());
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(
        &json!(16203),
        "delta_code_context",
        &json!({"arguments":{
            "filePath":file,"workspaceRoot":p,"fidelity":"edit"
        }}),
        &state,
    );
    let generated = crate::protocol::captured_responses().pop().unwrap();
    assert!(generated.get("error").is_none(), "{generated:#}");
    assert_eq!(generated["result"]["strategy"], "delta", "{generated:#}");
    call(
        &state,
        p,
        "apply_delta",
        json!({
            "delta":generated["result"]["delta"],"currentVersion":generated["result"]["from_version"]
        }),
    );
    // Edit fidelity owns retained semantic data, not certified index coverage.
    // Inspect the mutation snapshot before query hydration can reconstruct it.
    let applied = state
        .semantic_snapshot(&alias)
        .unwrap()
        .authority
        .resolver_registrations
        .unwrap();
    assert_eq!(
        applied
            .records
            .iter()
            .map(|r| r.key.as_str())
            .collect::<Vec<_>>(),
        ["primary", "renamed"]
    );
    assert!(
        applied
            .records
            .iter()
            .all(|record| record.evidence.source_hash
                == crate::cache::content_hash(target.as_bytes()))
    );
    drop(state);
    let restarted = McpState::new(config);
    call(&restarted, p, "restore_context", json!({"filePath":file}));
    let restored_alias = restarted.alias_for_path(&file.to_string_lossy()).unwrap();
    assert_eq!(
        restarted
            .semantic_snapshot(&restored_alias)
            .unwrap()
            .authority
            .resolver_registrations,
        Some(applied)
    );
    assert_eq!(
        keys(&query(&restarted, p, "forward")),
        ["primary", "renamed"]
    );
}

#[test]
fn issue_162_resolver_registration_replay_adopts_exact_keys_before_query() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = fixture("direct");
    let p = root.path();
    let file = p.join("one.ts");
    let producer = McpState::new(config.clone());
    acquire_session(&producer, p);
    call(&producer, p, "save_context", json!({"filePath":file}));
    let owner = producer.semantic_owner_path(&file.to_string_lossy());
    let expected = producer
        .workspace_index_read()
        .resolver_registrations(&owner)
        .unwrap()
        .clone();
    assert_eq!(expected.records.len(), 2);
    drop(producer);

    let replayed = McpState::new(config);
    assert!(
        replayed
            .workspace_index_read()
            .resolver_registrations(&owner)
            .is_none()
    );
    call(
        &replayed,
        p,
        "replay_history",
        json!({"filePath":file,"targetSequence":0}),
    );
    // Inspect actual replay adoption before query hydration can conceal a lost
    // collection by compiling source again.
    assert_eq!(
        replayed
            .workspace_index_read()
            .resolver_registrations(&owner),
        Some(&expected)
    );
    assert!(
        replayed
            .workspace_index_read()
            .has_resolver_registration_admission(&owner)
    );
    assert_eq!(
        keys(&query(&replayed, p, "forward")),
        ["primary", "secondary"]
    );
    assert_eq!(
        keys(&query(&replayed, p, "reverse")),
        ["primary", "secondary"]
    );
}

#[test]
fn issue_162_resolver_registration_complete_admission_rejects_missing_extra_crossed() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = fixture("direct");
    let state = McpState::new(config);
    let _ = query(&state, root.path(), "forward");
    acquire_session(&state, root.path());
    let alias = state
        .alias_for_path(&root.path().join("one.ts").to_string_lossy())
        .unwrap();
    let snapshot = state.semantic_snapshot(&alias).unwrap();
    let authority = &snapshot.authority;
    crate::angular_meta::route_resolvers::replay_metrics::reset();
    assert!(authority.admits_semantic_projection(&snapshot.edges, true));
    assert_eq!(
        crate::angular_meta::route_resolvers::replay_metrics::counts(),
        (1, 2)
    );
    for mode in [
        "missing",
        "scope_false",
        "scope_unknown",
        "empty",
        "version",
        "extra",
        "removed",
        "key",
        "source",
        "span",
        "route",
        "target",
    ] {
        let mut forged = authority.clone();
        if mode == "missing" {
            forged.resolver_registrations = None;
        } else {
            let collection = forged.resolver_registrations.as_mut().unwrap();
            match mode {
                "scope_false" => forged.resolver_registration_scope = Some(false),
                "scope_unknown" => forged.resolver_registration_scope = None,
                "empty" => collection.records.clear(),
                "version" => collection.format_version = 999,
                "extra" => collection.records.push(collection.records[0].clone()),
                "removed" => {
                    collection.records.pop();
                }
                "key" => collection.records[0].key = "borrowed".into(),
                "source" => collection.records[0].evidence.source_hash = "0".repeat(64),
                "span" => collection.records[0].evidence.pair = collection.records[1].evidence.pair,
                "route" => collection.records[0].route.end += 1,
                "target" => collection.records[0].resolver.name = "borrowed".into(),
                _ => unreachable!(),
            }
        }
        assert!(
            !forged.admits_semantic_projection(&snapshot.edges, true),
            "{mode}"
        );
    }
    let no_membership: Vec<_> = snapshot
        .edges
        .iter()
        .filter(|edge| edge.relation != crate::layers::meta::semantic::SemanticRelation::ResolvedBy)
        .cloned()
        .collect();
    assert!(!authority.admits_semantic_projection(&no_membership, true));
    assert!(!authority.admits_semantic_projection(&snapshot.edges, false));
    assert!(
        authority.admits_relation_evidence(&snapshot.edges),
        "legacy subset contract survives"
    );
    // A produced empty result still differs from an absent collection: the
    // current producer identity, not candidate-record presence, establishes need.
    std::fs::write(
        root.path().join("one.ts"),
        "import type {Routes} from '@angular/router'; export const routes:Routes=[];",
    )
    .unwrap();
    call(&state, root.path(), "refresh_workspace", json!({}));
    assert_eq!(query(&state, root.path(), "forward")["count"], 0);
    acquire_session(&state, root.path());
    let alias = state
        .alias_for_path(&root.path().join("one.ts").to_string_lossy())
        .unwrap();
    let empty = state.semantic_snapshot(&alias).unwrap();
    assert_eq!(empty.authority.resolver_registration_scope, Some(true));
    assert!(
        empty
            .authority
            .resolver_registrations
            .as_ref()
            .unwrap()
            .records
            .is_empty()
    );
    let mut absent = empty.authority.clone();
    absent.resolver_registrations = None;
    assert!(!absent.admits_semantic_projection(&empty.edges, true));
}

#[test]
fn issue_162_resolver_registration_sqlite_restart_key_target_and_retirement() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = fixture("direct");
    let p = root.path();
    let state = McpState::new(config.clone());
    let before = query(&state, p, "forward");
    assert_eq!(
        query(&state, p, "forward")["registrations"],
        before["registrations"]
    );
    acquire_session(&state, p);
    call(
        &state,
        p,
        "save_context",
        json!({"filePath":p.join("one.ts")}),
    );
    drop(state);
    let db = rusqlite::Connection::open(&config.persistence.db_path).unwrap();
    let encoded: String = db
        .query_row(
            "SELECT edges_json FROM semantic_edge_snapshots WHERE file_path=?1",
            [p.join("one.ts").to_string_lossy().into_owned()],
            |row| row.get(0),
        )
        .unwrap();
    let stored: Value = serde_json::from_str(&encoded).unwrap();
    assert_eq!(
        stored["resolver_registrations"]["records"],
        before["registrations"]
    );
    drop(db);
    let state = McpState::new(config);
    call(
        &state,
        p,
        "restore_context",
        json!({"filePath":p.join("one.ts")}),
    );
    let owner = state.semantic_owner_path(&p.join("one.ts").to_string_lossy());
    assert_eq!(
        state
            .workspace_index_read()
            .resolver_registrations(&owner)
            .unwrap()
            .records
            .len(),
        2,
        "normal SQLite restore must reach index before a registration query"
    );
    assert_eq!(
        query(&state, p, "reverse")["registrations"],
        before["registrations"]
    );
    let file = p.join("one.ts");
    let original = std::fs::read_to_string(&file).unwrap();
    std::fs::write(&file, original.replace("primary:", "fallback:")).unwrap();
    call(&state, p, "refresh_workspace", json!({}));
    let changed = query(&state, p, "forward");
    assert_eq!(keys(&changed), ["fallback", "secondary"]);
    assert_ne!(
        changed["registrations"][0]["route"],
        before["registrations"][0]["route"]
    );
    let target_hash = changed["registrations"][0]["resolver"]["source_hash"].clone();
    let target = p.join("guard-a.ts");
    let source = std::fs::read_to_string(&target).unwrap();
    std::fs::write(&target, format!("{source}\n// changed target witness\n")).unwrap();
    let current = query(&state, p, "forward");
    assert_eq!(keys(&current), ["fallback", "secondary"]);
    assert_ne!(
        current["registrations"][0]["resolver"]["source_hash"],
        target_hash
    );
    std::fs::write(&file, original.replace("primary:Gate,secondary:Gate", "")).unwrap();
    call(&state, p, "refresh_workspace", json!({}));
    assert_eq!(query(&state, p, "forward")["count"], 0);
    assert!(
        state
            .workspace_index_read()
            .resolver_registrations(&owner)
            .unwrap()
            .records
            .is_empty()
    );
    std::fs::remove_file(&file).unwrap();
    call(&state, p, "refresh_workspace", json!({}));
    assert_eq!(query(&state, p, "reverse")["count"], 0);
    assert!(
        state
            .workspace_index_read()
            .resolver_registrations(&owner)
            .is_none()
    );
    std::fs::write(&file, original).unwrap();
    call(&state, p, "refresh_workspace", json!({}));
    assert_eq!(query(&state, p, "forward")["count"], 2);
}

#[test]
fn issue_162_resolver_registration_missing_sqlite_collection_is_not_empty_success() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = fixture("direct");
    let p = root.path();
    let state = McpState::new(config.clone());
    assert_eq!(query(&state, p, "forward")["count"], 2);
    acquire_session(&state, p);
    call(
        &state,
        p,
        "save_context",
        json!({"filePath":p.join("one.ts")}),
    );
    drop(state);
    let db = rusqlite::Connection::open(&config.persistence.db_path).unwrap();
    let owner = p.join("one.ts").to_string_lossy().into_owned();
    let encoded: String = db
        .query_row(
            "SELECT edges_json FROM semantic_edge_snapshots WHERE file_path=?1",
            [&owner],
            |row| row.get(0),
        )
        .unwrap();
    let mut payload: Value = serde_json::from_str(&encoded).unwrap();
    let _ = payload
        .as_object_mut()
        .unwrap()
        .remove("resolver_registrations");
    db.execute(
        "UPDATE semantic_edge_snapshots SET edges_json=?1 WHERE file_path=?2",
        rusqlite::params![payload.to_string(), owner],
    )
    .unwrap();
    drop(db);
    let state = McpState::new(config);
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(
        &json!(16279),
        "restore_context",
        &json!({"arguments":{"workspaceRoot":p,"filePath":p.join("one.ts")}}),
        &state,
    );
    let refused = crate::protocol::captured_responses().pop().unwrap();
    assert!(
        refused.get("error").is_some() || refused["result"]["isError"] == true,
        "{refused:#}"
    );
    assert_eq!(
        query(&state, p, "forward")["count"],
        2,
        "normal reacquisition repairs missing current proof"
    );
}

#[test]
fn issue_162_resolver_registration_generation78_sqlite_payload_requires_reacquisition() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = fixture("direct");
    let p = root.path();
    let state = McpState::new(config.clone());
    assert_eq!(query(&state, p, "forward")["count"], 2);
    acquire_session(&state, p);
    call(
        &state,
        p,
        "save_context",
        json!({"filePath":p.join("one.ts")}),
    );
    drop(state);
    let db = rusqlite::Connection::open(&config.persistence.db_path).unwrap();
    let owner = p.join("one.ts").to_string_lossy().into_owned();
    let (encoded, identity): (String, String) = db.query_row(
        "SELECT edges_json,semantic_producer_identity FROM semantic_edge_snapshots WHERE file_path=?1",
        [&owner], |row| Ok((row.get(0)?, row.get(1)?))).unwrap();
    let mut payload: Value = serde_json::from_str(&encoded).unwrap();
    let _ = payload
        .as_object_mut()
        .unwrap()
        .remove("resolver_registrations");
    let mut identity: Value = serde_json::from_str(&identity).unwrap();
    assert_eq!(identity["relevant_producers"]["angular_semantic"], 79);
    identity["relevant_producers"]["angular_semantic"] = json!(78);
    // Simulated predecessor envelope using the real unchanged SQLite codec.
    db.execute("UPDATE semantic_edge_snapshots SET edges_json=?1,semantic_producer_identity=?2 WHERE file_path=?3",
        rusqlite::params![payload.to_string(), identity.to_string(), owner]).unwrap();
    drop(db);
    let state = McpState::new(config);
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(
        &json!(16279),
        "restore_context",
        &json!({"arguments":{"workspaceRoot":p,"filePath":p.join("one.ts")}}),
        &state,
    );
    let refused = crate::protocol::captured_responses().pop().unwrap();
    assert_eq!(
        refused["error"]["data"]["reason"], "semantic_producer_incompatible",
        "{refused:#}"
    );
    assert_eq!(query(&state, p, "reverse")["count"], 2);
}

#[test]
fn issue_162_resolver_registration_conservative_producer_hint_is_not_actual_scope() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = fixture("direct");
    let p = root.path();
    let file = p.join("one.ts");
    let source = "function custom(value:any){return value;} @custom export class Ordinary {}";
    std::fs::write(&file, source).unwrap();
    let state = McpState::new(config.clone());
    let identities = crate::mcp::compatibility::derive_identities(source, &file, &config).unwrap();
    assert!(
        identities
            .semantic_producers
            .relevant_producers
            .contains_key(&crate::mcp::compatibility::identity::ProducerKey::AngularSemantic)
    );
    assert_eq!(query(&state, p, "forward")["count"], 0);
    acquire_session(&state, p);
    let alias = state.alias_for_path(&file.to_string_lossy()).unwrap();
    let snapshot = state.semantic_snapshot(&alias).unwrap();
    assert_eq!(snapshot.authority.resolver_registration_scope, Some(false));
    assert!(snapshot.authority.resolver_registrations.is_none());
    call(&state, p, "save_context", json!({"filePath":file}));
    drop(state);
    let state = McpState::new(config);
    call(&state, p, "restore_context", json!({"filePath":file}));
    assert_eq!(
        query(&state, p, "reverse")["count"],
        0,
        "unrelated decorated TS must not acquire a mandatory Angular collection prerequisite"
    );
}
