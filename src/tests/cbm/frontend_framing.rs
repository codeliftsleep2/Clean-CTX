use super::{is_response, read_frame};
use std::io::{BufReader, Cursor};

#[test]
fn cbm_a0_matching_response_and_rpc_error_are_frames() {
    for frame in [
        r#"{"jsonrpc":"2.0","id":7,"result":{}}"#,
        r#"{"jsonrpc":"2.0","id":7,"error":{"code":-32603,"message":"failed"}}"#,
    ] {
        assert!(is_response(frame, 7).unwrap());
    }
}

#[test]
fn cbm_a0_newline_frame_keeps_escaped_tree_content() {
    let wire = b"{\"jsonrpc\":\"2.0\",\"id\":7,\"result\":{\"text\":\"a\\nb\"}}\n";
    let mut reader = Cursor::new(wire);
    assert_eq!(read_frame(&mut reader, 128).unwrap().unwrap().as_bytes(), wire);
    assert!(read_frame(&mut reader, 128).unwrap().is_none());
}
