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

#[test]
fn cbm_a1_wrong_or_missing_response_identity_is_rejected() {
    for id in ["8", "null", "\"7\"", "7.0", "true", "-1"] {
        let frame = format!(r#"{{"jsonrpc":"2.0","id":{id},"result":{{}}}}"#);
        assert!(is_response(&frame, 7).is_err(), "accepted identity {id}");
    }
    assert!(is_response(r#"{"jsonrpc":"2.0","result":{}}"#, 7).is_err());
}

#[test]
fn cbm_a1_notification_is_not_pending_completion() {
    assert!(!is_response(r#"{"jsonrpc":"2.0","method":"notifications/message","params":{}}"#, 7).unwrap());
}

#[test]
fn cbm_a1_startup_error_is_not_a_correlated_reply() {
    assert!(is_response(r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32001,"message":"startup failed"}}"#, 7).is_err());
}

#[test]
fn cbm_a1_envelope_and_error_shape_are_validated() {
    for frame in [
        "[]", "null", "7",
        r#"{"jsonrpc":"1.0","id":7,"result":{}}"#,
        r#"{"id":7,"result":{}}"#,
        r#"{"jsonrpc":"2.0","id":7}"#,
        r#"{"jsonrpc":"2.0","id":7,"result":{},"error":{"code":1,"message":"x"}}"#,
        r#"{"jsonrpc":"2.0","id":7,"error":null}"#,
        r#"{"jsonrpc":"2.0","id":7,"error":{"code":"1","message":"x"}}"#,
        r#"{"jsonrpc":"2.0","id":7,"error":{"code":1}}"#,
        r#"{"jsonrpc":"2.0","id":7,"method":"tools/call","result":{}}"#,
    ] {
        assert!(is_response(frame, 7).is_err(), "accepted envelope {frame}");
    }
}

#[test]
fn cbm_a1_frame_limit_is_checked_before_whole_line_consumption() {
    let mut reader = BufReader::with_capacity(8, Cursor::new(vec![b'x'; 4096]));
    assert!(read_frame(&mut reader, 64).is_err());
    assert!(reader.into_inner().position() <= 72, "unbounded line read before cap");
}

#[test]
fn cbm_a1_unterminated_frame_at_eof_is_rejected() {
    let mut reader = Cursor::new(b"{\"jsonrpc\":\"2.0\",\"id\":7,\"result\":{}}");
    assert!(read_frame(&mut reader, 128).is_err());
}

#[test]
fn cbm_a1_boundary_and_utf8_frames() {
    let mut exact = Cursor::new(b"123\n");
    assert_eq!(read_frame(&mut exact, 4).unwrap().unwrap(), "123\n");
    let mut over = Cursor::new(b"1234\n");
    assert!(read_frame(&mut over, 4).is_err());
    let mut invalid = Cursor::new([0xff, b'\n']);
    assert!(read_frame(&mut invalid, 8).is_err());
}
