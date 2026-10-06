use super::*;

#[test]
fn malformed_and_oversized_inputs_return_no_replacement() {
    let malformed = process_hook_bytes(b"{", 100);
    assert!(malformed.response.is_none());
    assert_eq!(
        malformed.facts.pass_through_reason,
        Some(PassThroughReason::InvalidEnvelope)
    );
    let oversized = process_hook_bytes(b"{}", 1);
    assert!(oversized.response.is_none());
    assert_eq!(
        oversized.facts.pass_through_reason,
        Some(PassThroughReason::InvalidEnvelope)
    );
}

#[test]
fn wire_output_separates_protocol_response_from_facts() {
    let input = br#"{"hook_event_name":"PostToolUse","tool_name":"Bash","tool_input":{"command":"echo ok"},"tool_response":{"stdout":"password=hunter2","stderr":"","interrupted":false,"isImage":false}}"#;
    let output = render_hook_io(input, 4096).unwrap();
    let stdout: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let stderr: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(stdout["hookSpecificOutput"]["hookEventName"], "PostToolUse");
    assert!(stdout.to_string().contains("[REDACTED]"));
    assert!(!stderr.to_string().contains("hunter2"));
    assert!(stderr.get("outcome").is_some());
}
