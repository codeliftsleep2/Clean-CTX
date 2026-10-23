use crate::compression::Fidelity;
use crate::ir::typescript_resolution::Resolution;
use crate::mcp::state::durable_semantics::DurableSemanticSnapshot;
use serde_json::json;

#[test]
fn issue_162_retained_authority_survives_artifact_decomposition_into_durable_payload() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    std::fs::write(
        root.path().join("tsconfig.json"),
        r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","moduleDetection":"legacy","noLib":true,"types":[]}}"#,
    )
    .unwrap();
    let leaf = "export class Leaf {}";
    std::fs::write(root.path().join("leaf.ts"), leaf).unwrap();
    let source = "import { Leaf as Local } from './leaf'; export const selected = Local;";
    let file = root.path().join("app.ts");
    std::fs::write(&file, source).unwrap();
    let file = file.to_str().unwrap();
    let database = root.path().join("retained-authority.db");
    let mut config = crate::tests::test_config();
    config.persistence.enabled = true;
    config.persistence.db_path = database.to_string_lossy().into_owned();
    config.additional_roots.push(root.path().to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    let artifact = crate::mcp::tool_helpers::compile_source_ir_candidate(
        file,
        source,
        Fidelity::High,
        &state,
        root.path().to_str(),
    )
    .expect("real compilation artifact");

    let authority = artifact.typescript_authority();
    assert!(authority.disposition.is_none(), "{:?}", authority.disposition);
    assert!(authority.capture_current);
    let start = source.rfind("Local").unwrap();
    assert!(authority.reference_sites.iter().any(|(from, to, result)| {
        *from == start
            && *to == start + "Local".len()
            && matches!(result, Resolution::Resolved(id)
                if id.name == "Leaf"
                    && id.source.ends_with("leaf.ts")
                    && id.source_hash == crate::cache::content_hash(leaf.as_bytes())
                    && id.start < id.end)
    }), "nonempty exact aliased endpoint control: {:?}", authority.reference_sites);
    assert!(!authority.witnesses.is_empty());
    assert!(authority.witnesses.iter().all(|witness| witness.current()));
    let expected = serde_json::to_value(authority).unwrap();

    // The control capture is never injected into the production persistence path.
    drop(artifact);
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(
        &json!(162),
        "compress_code_context",
        &json!({"arguments": {
            "filePath": file,
            "workspaceRoot": root.path(),
            "fidelity": "high"
        }}),
        &state,
    );
    let response = crate::protocol::captured_responses().pop().expect("MCP response");
    assert!(response.get("error").is_none(), "{response}");
    state.flush_persistence();
    let connection = rusqlite::Connection::open(&database).unwrap();
    let persisted: String = connection.query_row(
        "SELECT edges_json FROM semantic_edge_snapshots WHERE file_path = ?1",
        [file],
        |row| row.get(0),
    ).expect("production persisted semantic snapshot");
    let encoded: serde_json::Value = serde_json::from_str(&persisted).unwrap();
    assert_eq!(
        encoded.get("typescript_authority"),
        Some(&expected),
        "typed authority must survive decomposition and the existing durable payload; absence is not a complete inventory"
    );
    let restored: DurableSemanticSnapshot = serde_json::from_value(encoded).unwrap();
    assert_eq!(
        serde_json::to_value(restored).unwrap().get("typescript_authority"),
        Some(&expected),
        "restoration must preserve all typed authority evidence"
    );
}
