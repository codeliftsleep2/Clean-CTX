use super::*;
use clap::Parser;

#[test]
fn claude_post_tool_use_subcommand_is_registered() {
    let parsed = Cli::try_parse_from(["clean-ctx", "claude-hook", "post-tool-use"]).unwrap();
    assert!(matches!(
        parsed,
        Cli::ClaudeHook {
            event: ClaudeHookEvent::PostToolUse
        }
    ));
}
