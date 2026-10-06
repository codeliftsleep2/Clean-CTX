//! Claude Code native lifecycle adapter.

mod bash;
mod facts;
mod hook;
mod pipeline;

pub use facts::{ClaudeNativeFacts, PassThroughReason, ProcessingOutcome};
pub use hook::{
    HookIoOutput, HookProcessResult, process_hook_bytes, process_hook_value, render_hook_io,
};
