//! Claude Code native lifecycle adapter.

mod bash;
mod command;
mod facts;
mod hook;
mod pipeline;

/// Explicit policy for one Claude `PostToolUse` transformation.
///
/// The persisted-output option is intentionally opt-in because Claude creates
/// the referenced file before invoking the hook. Omitting the pointer from a
/// replacement may reduce model-visible reachability, but cannot redact or
/// remove the already-persisted raw output.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PostToolUseOptions {
    pub drop_persisted_output_pointer: bool,
}

pub use facts::{ClaudeNativeFacts, PassThroughReason, ProcessingOutcome};
pub use hook::{
    HookIoOutput, HookProcessResult, process_hook_bytes, process_hook_bytes_with_options,
    process_hook_value, process_hook_value_with_options, render_hook_io,
    render_hook_io_with_options,
};
