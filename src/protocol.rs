// src/protocol.rs
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    pub method: String,
    pub params: Option<serde_json::Value>,
}

pub fn send_response(val: &serde_json::Value) {
    use std::io::{self, Write};
    let mut stdout = io::stdout().lock();
    if let Ok(payload) = serde_json::to_string(val) {
        let _ = writeln!(stdout, "{}", payload);
        let _ = stdout.flush();
    }
}