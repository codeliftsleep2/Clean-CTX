//! Agent MaxHeadRoom: protect replay's later dictionary observation, no candidate mode.
use crate::mcp::McpState;
use crate::mcp::content_economics::{EconomicContent, SelectedRepresentation};
use serde_json::{Value, json};
use std::cell::RefCell;
type Hook = Box<dyn FnOnce(&McpState, &str, &str, &EconomicContent)>;
thread_local! { static HOOK: RefCell<Option<Hook>> = const {RefCell::new(None)}; }
struct Reset;
impl Drop for Reset {
    fn drop(&mut self) {
        drop(HOOK.with(|h| h.borrow_mut().take()));
    }
}
pub(super) fn after_economics(
    state: &McpState,
    path: &str,
    alias: &str,
    economic: &EconomicContent,
) {
    let hook = HOOK.with(|h| h.borrow_mut().take());
    if let Some(hook) = hook {
        hook(state, path, alias, economic);
    }
}
fn call(state: &McpState, tool: &str, args: Value) -> Value {
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(&json!(1), tool, &json!({"arguments":args}), state);
    crate::protocol::captured_responses().pop().unwrap()
}
#[test]
fn af002_replay_footer_observes_dictionary_after_economics() {
    let _serial = crate::protocol::handler_response_serial();
    for action in ["unrelated", "remove", "delete"] {
        let root = tempfile::tempdir().unwrap();
        let file = root.path().join("history.ts");
        let source = format!(
            "// {}\nexport class Historical {{ method(): string {{return 'old';}} }}\n",
            "共有".repeat(4096)
        );
        std::fs::write(&file, &source).unwrap();
        let mut cfg = crate::tests::test_config();
        cfg.cbm.enabled = false;
        cfg.proxy.port = 0;
        cfg.persistence.enabled = true;
        cfg.persistence.auto_save = true;
        cfg.persistence.db_path = root
            .path()
            .join("history.db")
            .to_string_lossy()
            .into_owned();
        cfg.additional_roots
            .push(root.path().to_string_lossy().into_owned());
        let state = McpState::new(cfg);
        *state.graph_bridge_lock() = None;
        let args = json!({"filePath":file,"workspaceRoot":root.path(),"fidelity":"high","tokenizer":"cl100k","targetSequence":0});
        assert!(
            call(&state, "compress_code_context", args.clone())
                .get("error")
                .is_none()
        );
        let before = call(&state, "replay_history", args.clone());
        assert!(before.get("error").is_none(), "{before}");
        let alias = state.alias_for_path(&file.to_string_lossy()).unwrap();
        let canonical_before = {
            let ctx = state.ir_context_read();
            json!({"tuples":ctx.get_ir(&alias),
            "version":ctx.file_version(&alias),"hash":ctx.get_source_hash(&alias)})
        };
        let expected = std::rc::Rc::new(RefCell::new(None::<String>));
        let capture = expected.clone();
        let _reset = Reset;
        HOOK.with(|h| {
            *h.borrow_mut() = Some(Box::new(move |state, path, alias, economic| {
                assert_eq!(economic.selected, SelectedRepresentation::Candidate);
                let owner = state.path_for_alias(alias).unwrap();
                let mapping = format!("\n  {alias} = {owner}");
                let text = if action == "unrelated" {
                    economic.text.clone()
                } else {
                    economic
                        .text
                        .strip_suffix(&mapping)
                        .expect("Exact requested alias footer")
                        .to_owned()
                };
                match action {
                    "unrelated" => {
                        state.get_or_create_alias(
                            std::path::Path::new(path)
                                .parent()
                                .unwrap()
                                .join("other.ts")
                                .to_string_lossy()
                                .into_owned(),
                        );
                    }
                    "remove" => {
                        assert!(state.dict_lock().remove_path_alias(alias, path));
                    }
                    "delete" => {
                        let deleted = call(state, "delete_context", json!({"filePath":path}));
                        assert!(deleted.get("error").is_none(), "{deleted}");
                    }
                    _ => unreachable!(),
                }
                *capture.borrow_mut() = Some(text);
            }))
        });
        let response = call(&state, "replay_history", args);
        let text = expected.borrow().clone().expect("Observation hook reached");
        let mut expected_response = before;
        expected_response["result"]["content"][0]["text"] = json!(text);
        assert_eq!(
            response, expected_response,
            "Later dictionary observation changed: {action}"
        );
        assert_eq!(state.llm_text_cache_lock().get(&alias), Some(&text));
        let canonical_after = {
            let ctx = state.ir_context_read();
            json!({"tuples":ctx.get_ir(&alias),
            "version":ctx.file_version(&alias),"hash":ctx.get_source_hash(&alias)})
        };
        assert_eq!(canonical_before, canonical_after);
        // This protects rendering's existing observation boundary only. It does
        // not grant replay-after-delete a new lifecycle/authority guarantee.
    }
}
