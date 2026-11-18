//! A4 fake frontend failures; normal launch/admission still execute production code.
use serde_json::{Value, json};
use std::io::{self, Write};
use std::path::Path;

pub(super) fn fixture_mode(mode: &str, log: &Path) -> String {
    let ordinal = std::fs::read_to_string(log.with_extension("launches"))
        .unwrap_or_default()
        .lines()
        .count();
    if ordinal > 0 {
        match mode {
            "a4-eof-once" | "a4-timeout-once" => return "normal".into(),
            "a4-timeout-then-eof" => return "a4-eof-always".into(),
            "a4-replacement-refusal" => return "a3-startup-refusal-exit".into(),
            "a4-replacement-missing" => return "a3-missing-query_graph".into(),
            _ => {}
        }
    }
    mode.into()
}
pub(super) fn fixture_tool(mode: &str, request: &Value) -> bool {
    match mode {
        "a4-eof-once" | "a4-eof-always" | "a4-replacement-refusal" | "a4-replacement-missing" => {
            std::process::exit(0)
        }
        "a4-timeout-once" | "a4-timeout-then-eof" => {
            std::thread::sleep(std::time::Duration::from_secs(30));
            true
        }
        "a4-rpc" | "a4-rpc-then-eof" | "a4-refusal" | "a4-poison" => {
            if mode == "a4-rpc-then-eof" {
                let log =
                    std::path::PathBuf::from(std::env::var_os("CLEAN_CTX_A2_FIXTURE_LOG").unwrap());
                let count = std::fs::read_to_string(log.with_extension("requests"))
                    .unwrap()
                    .lines()
                    .filter(|line| {
                        serde_json::from_str::<Value>(line).unwrap()["method"] == "tools/call"
                    })
                    .count();
                if count > 1 {
                    std::process::exit(0);
                }
            }
            let response = match mode {
                "a4-refusal" => json!({"jsonrpc":"2.0", "id":null,
                    "error":{"code":-32001,"message":"account refusal during call"}}),
                "a4-poison" => json!({"jsonrpc":"2.0", "id":999,"result":{}}),
                _ => json!({"jsonrpc":"2.0", "id":request["id"],
                    "error":{"code":-32603,"message":"uncertain accepted operation"}}),
            };
            println!("{response}");
            io::stdout().flush().unwrap();
            true
        }
        _ => false,
    }
}
