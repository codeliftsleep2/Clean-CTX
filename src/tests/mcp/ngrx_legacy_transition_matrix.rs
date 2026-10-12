use super::ngrx_legacy_transition_tests::{PRIMARY, call};
use serde_json::{Value, json};

fn query(
    state: &crate::mcp::McpState,
    root: &std::path::Path,
    kind: &str,
    entity_type: &str,
    name: &str,
) -> Value {
    call(
        state,
        "workspace_query",
        json!({"workspaceRoot":root,"type":kind,"domain":"ngrx","entity_type":entity_type,"name":name}),
    )
}

#[test]
fn ngrx_legacy_transition_public_falsifiers_and_reverse_identity() {
    let _serial = crate::protocol::handler_response_serial();
    for (operand, expected) in [
        ("SET_AUDIT", vec!["AuditAction"]),
        ("OTHER_AUDIT", vec!["OtherAction"]),
        ("UNUSED_AUDIT", vec![]),
        ("'[Audit] Set Audit'", vec![]),
        ("wrapper(SET_AUDIT)", vec![]),
        ("Actions.SET_AUDIT", vec![]),
    ] {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("actions.ts");
        let source = PRIMARY.replace("export function auditReducer", "export const OTHER_AUDIT='[Audit] Set Audit'; export const UNUSED_AUDIT='unused'; export class OtherAction implements Action {readonly type=OTHER_AUDIT;} export function auditReducer")
            .replace("case SET_AUDIT:", &format!("case {operand}:")) + &"\n// economics padding".repeat(100);
        std::fs::write(&path, source).unwrap();
        let mut config = crate::tests::test_config();
        config
            .additional_roots
            .push(root.path().to_string_lossy().into_owned());
        let state = crate::mcp::McpState::new(config);
        call(
            &state,
            "provide_code_context",
            json!({"workspaceRoot":root.path(),"filePath":path,"fidelity":"high"}),
        );
        let answer = query(
            &state,
            root.path(),
            "reverse_edges",
            "Reducer",
            "auditReducer",
        );
        let edges: Vec<_> = answer["structuredContent"]["edges"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["relation"] == "TriggersReducer")
            .collect();
        assert_eq!(
            edges
                .iter()
                .map(|e| e["subject"]["name"].as_str().unwrap())
                .collect::<Vec<_>>(),
            expected,
            "{operand}: {answer}"
        );
        for edge in edges {
            assert_eq!(edge["subject"]["domain"], "ngrx");
            assert_eq!(edge["subject"]["entity_type"], "Action");
            assert_eq!(edge["subject"]["file"], path.to_string_lossy().as_ref());
            assert_eq!(edge["object"]["domain"], "ngrx");
            assert_eq!(edge["object"]["entity_type"], "Reducer");
            assert_eq!(edge["object"]["name"], "auditReducer");
            assert_eq!(edge["object"]["file"], path.to_string_lossy().as_ref());
            let forward = query(
                &state,
                root.path(),
                "forward_edges",
                "Action",
                edge["subject"]["name"].as_str().unwrap(),
            );
            assert!(
                forward["structuredContent"]["edges"]
                    .as_array()
                    .unwrap()
                    .contains(edge)
            );
        }
        let entities = call(
            &state,
            "workspace_query",
            json!({"workspaceRoot":root.path(),"type":"entities_in_file","file_path":path}),
        );
        let actions: Vec<_> = entities["structuredContent"]["entities"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["domain"] == "ngrx" && e["entity_type"] == "Action")
            .collect();
        assert_eq!(actions.len(), 2);
        assert!(!actions.iter().any(|e| e["name"] == "UNUSED_AUDIT"));
    }
}

#[test]
fn ngrx_legacy_transition_public_duplicate_and_refresh_lifecycle() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("actions.ts");
    let source = PRIMARY.replace("export function auditReducer", "export const OTHER_AUDIT='[Audit] Set Audit'; export class OtherAction implements Action {readonly type=OTHER_AUDIT;} export function auditReducer");
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    for (body, expected) in [
        (
            "case SET_AUDIT: return state; case SET_AUDIT: return state;",
            vec!["AuditAction"],
        ),
        ("case OTHER_AUDIT: return state;", vec!["OtherAction"]),
        ("case '[Audit] Set Audit': return state;", vec![]),
        ("", vec![]),
    ] {
        let text = source.replace("case SET_AUDIT: return state;", body)
            + &"\n// economics padding".repeat(100);
        std::fs::write(&path, text).unwrap();
        call(
            &state,
            "refresh_workspace",
            json!({"workspaceRoot":root.path()}),
        );
        for fidelity in ["high", "medium", "low"] {
            call(
                &state,
                "provide_code_context",
                json!({"workspaceRoot":root.path(),"filePath":path,"fidelity":fidelity}),
            );
            let answer = query(
                &state,
                root.path(),
                "reverse_edges",
                "Reducer",
                "auditReducer",
            );
            assert_eq!(
                answer["structuredContent"]["edges"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|e| e["relation"] == "TriggersReducer")
                    .map(|e| e["subject"]["name"].as_str().unwrap())
                    .collect::<Vec<_>>(),
                expected,
                "{answer}"
            );
        }
    }
    std::fs::write(
        &path,
        source.split("export function auditReducer").next().unwrap(),
    )
    .unwrap();
    call(
        &state,
        "refresh_workspace",
        json!({"workspaceRoot":root.path()}),
    );
    call(
        &state,
        "provide_code_context",
        json!({"workspaceRoot":root.path(),"filePath":path,"fidelity":"high"}),
    );
    for name in ["AuditAction", "OtherAction"] {
        assert!(query(&state, root.path(), "forward_edges", "Action", name)["structuredContent"]["edges"].as_array().unwrap().is_empty());
    }
}

#[test]
fn ngrx_legacy_transition_public_feature_disabled_preserves_no_new_edges() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("actions.ts");
    std::fs::write(&path, format!("{PRIMARY}\n{}", "// padding\n".repeat(100))).unwrap();
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    let mut angular = crate::config::MetaLayerConfig::default();
    angular.ngrx.enabled = false;
    config.meta_layers.insert("angular".into(), angular);
    let state = crate::mcp::McpState::new(config);
    call(
        &state,
        "provide_code_context",
        json!({"workspaceRoot":root.path(),"filePath":path,"fidelity":"high"}),
    );
    assert!(
        query(
            &state,
            root.path(),
            "forward_edges",
            "Action",
            "AuditAction"
        )["structuredContent"]["edges"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn ngrx_legacy_transition_public_same_names_preserve_file_occurrences() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let paths = [root.path().join("a.ts"), root.path().join("b.ts")];
    let source = format!("{PRIMARY}\n{}", "// padding\n".repeat(100));
    for path in &paths {
        std::fs::write(path, &source).unwrap();
    }
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    for path in &paths {
        call(
            &state,
            "provide_code_context",
            json!({"workspaceRoot":root.path(),"filePath":path,"fidelity":"high"}),
        );
    }
    let answer = query(
        &state,
        root.path(),
        "forward_edges",
        "Action",
        "AuditAction",
    );
    let edges: Vec<_> = answer["structuredContent"]["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["relation"] == "TriggersReducer")
        .collect();
    assert_eq!(edges.len(), 2, "{answer}");
    for path in &paths {
        assert_eq!(
            edges
                .iter()
                .filter(|e| e["subject"]["file"] == path.to_string_lossy().as_ref()
                    && e["object"]["file"] == path.to_string_lossy().as_ref()
                    && e["subject"]["name"] == "AuditAction"
                    && e["object"]["name"] == "auditReducer")
                .count(),
            1
        );
    }
}
