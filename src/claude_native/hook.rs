use super::{
    facts::{ClaudeNativeFacts, PassThroughReason},
    pipeline,
};
use serde_json::Value;

#[derive(Debug)]
pub struct HookProcessResult {
    pub response: Option<Value>,
    pub facts: ClaudeNativeFacts,
}

#[derive(Debug)]
pub struct HookIoOutput {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

pub fn process_hook_value(event: &Value) -> HookProcessResult {
    let (response, facts) = pipeline::process(event);
    HookProcessResult { response, facts }
}

pub fn process_hook_bytes(input: &[u8], max_bytes: usize) -> HookProcessResult {
    if input.len() > max_bytes {
        return HookProcessResult {
            response: None,
            facts: ClaudeNativeFacts::passed(PassThroughReason::InvalidEnvelope, 0),
        };
    }
    match serde_json::from_slice::<Value>(input) {
        Ok(value) => process_hook_value(&value),
        Err(_) => HookProcessResult {
            response: None,
            facts: ClaudeNativeFacts::passed(PassThroughReason::InvalidEnvelope, 0),
        },
    }
}

pub fn render_hook_io(input: &[u8], max_bytes: usize) -> Result<HookIoOutput, serde_json::Error> {
    let result = process_hook_bytes(input, max_bytes);
    let mut stderr = serde_json::to_vec(&result.facts)?;
    stderr.push(b'\n');
    let stdout = match result.response {
        Some(response) => {
            let mut bytes = serde_json::to_vec(&response)?;
            bytes.push(b'\n');
            bytes
        }
        None => Vec::new(),
    };
    Ok(HookIoOutput { stdout, stderr })
}

#[cfg(test)]
#[path = "../tests/claude_native/hook.rs"]
mod tests;
