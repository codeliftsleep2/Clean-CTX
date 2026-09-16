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

// ── Token-economics regression tests (2026-08-31) ───────────────────

// ── Token-economics regression tests (2026-08-31) ───────────────────
// Verifies post-compression economic invariant: candidate_tokens <= raw_tokens
// for all fidelity levels. See token_economics.rs for the two-stage gate.

use crate::mcp::tool_handlers::core::maybe_economics_fallback;

fn count_tokens(text: &str) -> usize {
    let kind = crate::tokenizer::TokenizerKind::default();
    let tok = crate::tokenizer::create_tokenizer(kind).unwrap();
    tok.count_tokens(text)
}

// ── maybe_economics_fallback unit tests ────────────────────────────

#[test]
fn economics_candidate_worse_than_raw_triggers_fallback() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = serde_json::json!(1);
    let source = "hello world";
    let result = maybe_economics_fallback(
        &id,
        source,
        100,
        200,
        &state,
        "/test/file.ts",
        false,
        crate::compression::Fidelity::Edit,
        "test",
    );
    assert!(result, "should fall back when candidate > raw");
}

#[test]
fn economics_candidate_cheaper_than_raw_does_not_trigger_fallback() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = serde_json::json!(1);
    let source = "hello world";
    let result = maybe_economics_fallback(
        &id,
        source,
        200,
        100,
        &state,
        "/test/file.ts",
        false,
        crate::compression::Fidelity::Edit,
        "test",
    );
    assert!(!result, "should NOT fall back when candidate < raw");
}

#[test]
fn economics_candidate_equal_to_raw_does_not_trigger_fallback() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = serde_json::json!(1);
    let source = "hello world";
    let result = maybe_economics_fallback(
        &id,
        source,
        100,
        100,
        &state,
        "/test/file.ts",
        false,
        crate::compression::Fidelity::Edit,
        "test",
    );
    assert!(!result, "should NOT fall back when candidate == raw");
}

#[test]
fn economics_fallback_works_for_all_fidelity_levels() {
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let id = serde_json::json!(1);
    let source = "hello world";
    for fidelity in &[
        crate::compression::Fidelity::Low,
        crate::compression::Fidelity::Medium,
        crate::compression::Fidelity::High,
        crate::compression::Fidelity::Edit,
    ] {
        let result = maybe_economics_fallback(
            &id,
            source,
            100,
            200,
            &state,
            "/test/file.ts",
            false,
            *fidelity,
            "test",
        );
        assert!(result, "{fidelity:?}: must fall back when candidate > raw");
        let result = maybe_economics_fallback(
            &id,
            source,
            200,
            100,
            &state,
            "/test/file.ts",
            false,
            *fidelity,
            "test",
        );
        assert!(
            !result,
            "{fidelity:?}: must NOT fall back when candidate < raw"
        );
    }
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
fn economics_small_file_edit_uses_raw_passthrough() {
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
    assert_eq!(
        kind, "raw_passthrough",
        "small Edit -> raw_passthrough, got: {kind}"
    );
}

#[test]
fn economics_edit_multi_method_candidate_vs_raw() {
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
    let kind = resp_kind(&resp);
    let text = resp_text(&resp);
    let comp_tokens = count_tokens(&text);
    if kind == "raw_passthrough" {
        assert_eq!(text, source, "raw_passthrough must return verbatim source");
    } else {
        assert!(
            comp_tokens <= raw_tokens,
            "invariant at Edit: raw={raw_tokens}, candidate={comp_tokens}"
        );
    }
}

#[test]
fn economics_structural_fidelities_obey_invariant() {
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
        let kind = resp_kind(&resp);
        let text = resp_text(&resp);
        let comp_tokens = count_tokens(&text);
        if kind == "raw_passthrough" {
            assert_eq!(text, source, "raw_passthrough at {fidelity}");
        } else {
            assert!(
                comp_tokens <= raw_tokens,
                "invariant at {fidelity}: raw={raw_tokens}, candidate={comp_tokens}"
            );
        }
    }
}

#[test]
fn economics_positive_compression_still_selected() {
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
        let kind = resp_kind(&resp);
        let text = resp_text(&resp);
        let comp_tokens = count_tokens(&text);
        assert_ne!(
            kind, "raw_passthrough",
            "{fidelity}: large file must compress"
        );
        assert!(
            comp_tokens <= raw_tokens,
            "invariant at {fidelity}: raw={raw_tokens}, candidate={comp_tokens}"
        );
        assert!(
            raw_tokens > comp_tokens,
            "{fidelity}: saves tokens, raw={raw_tokens}, candidate={comp_tokens}"
        );
    }
}

#[test]
fn economics_intent_edit_obeys_invariant() {
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
    let kind = resp_kind(&resp);
    let text = resp_text(&resp);
    let comp_tokens = count_tokens(&text);
    if kind == "raw_passthrough" {
        assert_eq!(text, source, "raw_passthrough must return verbatim source");
    } else {
        assert!(
            comp_tokens <= raw_tokens,
            "invariant for intent=edit: raw={raw_tokens}, candidate={comp_tokens}"
        );
    }
}
