use super::*;
use clap::Parser;

#[test]
fn claude_post_tool_use_subcommand_is_registered() {
    let parsed = Cli::try_parse_from(["clean-ctx", "claude-hook", "post-tool-use"]).unwrap();
    assert!(matches!(
        parsed,
        Cli::ClaudeHook {
            event: ClaudeHookEvent::PostToolUse {
                drop_persisted_output_pointer: false
            }
        }
    ));
}

#[test]
fn persisted_output_pointer_experiment_is_explicitly_opt_in() {
    let parsed = Cli::try_parse_from([
        "clean-ctx",
        "claude-hook",
        "post-tool-use",
        "--drop-persisted-output-pointer",
    ])
    .unwrap();
    assert!(matches!(
        parsed,
        Cli::ClaudeHook {
            event: ClaudeHookEvent::PostToolUse {
                drop_persisted_output_pointer: true
            }
        }
    ));
}
