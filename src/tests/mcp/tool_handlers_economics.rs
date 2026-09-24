// Sibling test module for a `#[path]`-loaded test file that exceeded the
// 615-line active-file size ceiling.
//
// Declared from the parent test file with `#[path = "<file>.rs"] mod <name>;`
// -- the same nested-`#[path]` idiom already used by `src/tests/cbm/e2e.rs` --
// so this module is a DESCENDANT of that test module and `use super::*`
// inherits its entire scope (imports, helpers, fixtures). Nothing needed to be
// widened or re-imported.
//
// Pure relocation: the tests below are byte-for-byte the previously inlined
// implementations.

use super::*;
use crate::mcp::tool_handlers::core::ContentKind;

// The SCHEMA-v5 presentation is selected only when the local counter proves it
// cheaper than the exact raw document. Raw passthrough is the mandatory
// economic fallback.

fn count_tokens(text: &str) -> usize {
    let kind = crate::tokenizer::TokenizerKind::default();
    let tok = crate::tokenizer::create_tokenizer(kind).unwrap();
    tok.count_tokens(text)
}

// ── Integration tests with real files ──────────────────────────────

fn create_multi_method_fixture(dir: &tempfile::TempDir, name: &str, mc: usize) -> String {
    let path = dir.path().join(name);
    let mut s = String::new();
    s.push_str("use std::collections::HashMap;\nuse std::sync::Arc;\n");
    s.push_str("use std::sync::Mutex;\nuse std::time::Instant;\n");
    s.push_str("use std::path::PathBuf;\nuse std::fs::File;\n");
    s.push_str("use std::io::{self, BufRead, BufReader, Write};\n");
    s.push_str("use std::fmt::Debug;\n\n");
    s.push_str("pub struct Service {\n    name: String,\n    count: u64,\n    active: bool,\n    data: HashMap<String, Vec<u8>>,\n}\n\n");
    s.push_str("impl Service {\n");
    s.push_str("    pub fn new(name: String) -> Self {\n");
    s.push_str("        Self { name, count: 0, active: true, data: HashMap::new() }\n    }\n\n");
    for i in 0..mc {
        s.push_str(&format!("    pub fn method_{i}(&self) -> &str {{\n"));
        s.push_str("        &self.name\n    }\n\n");
    }
    s.push_str("    pub fn process(&mut self) -> io::Result<()> {\n");
    s.push_str("        self.count += 1;\n");
    s.push_str("        if !self.active { return Ok(()); }\n");
    s.push_str("        let _ = self.data.insert(\"key\".into(), vec![1, 2, 3]);\n");
    s.push_str("        Ok(())\n    }\n}\n");
    std::fs::write(&path, &s).unwrap();
    path.to_string_lossy().into_owned()
}

fn resp_kind(resp: &serde_json::Value) -> String {
    resp["result"]["_meta"]["content_kind"]
        .as_str()
        .unwrap_or("missing")
        .to_string()
}

fn resp_text(resp: &serde_json::Value) -> String {
    resp["result"]["content"][0]["text"]
        .as_str()
        .unwrap_or("")
        .to_string()
}

#[test]
fn small_edit_uses_byte_exact_raw_when_a1_is_not_cheaper() {
    let dir = tempfile::TempDir::new().unwrap();
    let root_str = dir.path().to_string_lossy().into_owned();
    let path = dir.path().join("tiny.rs");
    std::fs::write(&path, "fn small() { 42 }").unwrap();
    let mut config = crate::tests::test_config();
    config.additional_roots.push(root_str);
    let state = crate::mcp::McpState::new(config);
    let id = serde_json::json!(1);
    let params = serde_json::json!({ "arguments": { "filePath": path.to_string_lossy().into_owned(), "fidelity": "edit" } });
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&id, "provide_code_context", &params, &state);
    let resp = crate::protocol::captured_responses()
        .pop()
        .expect("handler must send response");
    let kind = resp_kind(&resp);
    assert_eq!(kind, ContentKind::RawPassthrough.as_str());
    assert_eq!(resp_text(&resp), "fn small() { 42 }");
}

#[test]
fn edit_mode_never_returns_a_payload_more_expensive_than_raw() {
    let dir = tempfile::TempDir::new().unwrap();
    let root_str = dir.path().to_string_lossy().into_owned();
    let path_str = create_multi_method_fixture(&dir, "svc.rs", 35);
    let source = std::fs::read_to_string(&path_str).unwrap();
    let raw_tokens = count_tokens(&source);
    assert!(
        (800..=2000).contains(&raw_tokens),
        "fixture: ~800-2000 raw tokens, got: {raw_tokens}"
    );
    let mut config = crate::tests::test_config();
    config.additional_roots.push(root_str);
    let state = crate::mcp::McpState::new(config);
    let id = serde_json::json!(1);
    let params = serde_json::json!({ "arguments": { "filePath": path_str, "fidelity": "edit" } });
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&id, "provide_code_context", &params, &state);
    let resp = crate::protocol::captured_responses()
        .pop()
        .expect("handler must send response");
    let text = resp_text(&resp);
    let comp_tokens = count_tokens(&text);
    assert!(comp_tokens <= raw_tokens);
}

#[test]
fn structural_fidelities_keep_the_correctness_baseline() {
    let dir = tempfile::TempDir::new().unwrap();
    let root_str = dir.path().to_string_lossy().into_owned();
    let path_str = create_multi_method_fixture(&dir, "s.rs", 15);
    let source = std::fs::read_to_string(&path_str).unwrap();
    let raw_tokens = count_tokens(&source);
    let mut config = crate::tests::test_config();
    config.additional_roots.push(root_str);
    let state = crate::mcp::McpState::new(config);
    let id = serde_json::json!(1);
    for &fidelity in &["low", "medium", "high"] {
        let params = serde_json::json!({ "arguments": { "filePath": path_str.clone(), "fidelity": fidelity } });
        crate::protocol::captured_responses().clear();
        dispatch_tools_call(&id, "provide_code_context", &params, &state);
        let resp = crate::protocol::captured_responses()
            .pop()
            .expect("handler must send resp");
        let text = resp_text(&resp);
        assert!(count_tokens(&text) <= raw_tokens, "{fidelity}");
        assert!(
            text.starts_with("// SCHEMA v5") || text == source,
            "{fidelity}"
        );
    }
}

#[test]
fn large_files_never_exceed_raw_even_when_a1_does_not_win() {
    let dir = tempfile::TempDir::new().unwrap();
    let root_str = dir.path().to_string_lossy().into_owned();
    let path = dir.path().join("large.rs");
    let mut source = String::new();
    source.push_str("use std::collections::HashMap;\nuse std::sync::Arc;\n\n");
    source.push_str("pub struct LargeService {\n    data: HashMap<String, Vec<u8>>,\n    name: String,\n    count: u64,\n}\n\n");
    source.push_str("impl LargeService {\n");
    for i in 0..80 {
        source.push_str(&format!(
            "    pub fn method_{i}(&self, key: &str) -> Option<&Vec<u8>> {{\n"
        ));
        source.push_str("        self.data.get(key)\n    }\n\n");
    }
    source.push_str("}\n");
    std::fs::write(&path, &source).unwrap();
    let path_str = path.to_string_lossy().into_owned();
    let source = std::fs::read_to_string(&path_str).unwrap();
    let raw_tokens = count_tokens(&source);
    assert!(
        raw_tokens > 1500,
        "fixture: >1500 raw tokens, got: {raw_tokens}"
    );
    let mut config = crate::tests::test_config();
    config.additional_roots.push(root_str);
    let state = crate::mcp::McpState::new(config);
    let id = serde_json::json!(1);
    for &fidelity in &["low", "medium", "high"] {
        let params = serde_json::json!({ "arguments": { "filePath": path_str.clone(), "fidelity": fidelity } });
        crate::protocol::captured_responses().clear();
        dispatch_tools_call(&id, "provide_code_context", &params, &state);
        let resp = crate::protocol::captured_responses()
            .pop()
            .expect("handler must send resp");
        let text = resp_text(&resp);
        assert!(count_tokens(&text) <= raw_tokens, "{fidelity}");
        assert!(text.starts_with("// SCHEMA v5") || text == source);
        assert!(raw_tokens > 0);
    }
}

#[test]
fn intent_edit_selects_control_full_edit_mode() {
    let dir = tempfile::TempDir::new().unwrap();
    let root_str = dir.path().to_string_lossy().into_owned();
    let path_str = create_multi_method_fixture(&dir, "intent.rs", 20);
    let source = std::fs::read_to_string(&path_str).unwrap();
    let raw_tokens = count_tokens(&source);
    let mut config = crate::tests::test_config();
    config.additional_roots.push(root_str);
    let state = crate::mcp::McpState::new(config);
    let id = serde_json::json!(1);
    let params = serde_json::json!({ "arguments": { "filePath": path_str, "intent": "edit" } });
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&id, "provide_code_context", &params, &state);
    let resp = crate::protocol::captured_responses()
        .pop()
        .expect("handler must send resp");
    let text = resp_text(&resp);
    assert!(count_tokens(&text) <= raw_tokens);
    assert!(text.starts_with("// SCHEMA v5") || text == source);
}
