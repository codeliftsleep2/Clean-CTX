// src/tests/mcp/server.rs
//
// Tests for the bounded JSON-RPC read loop (FAANG audit F-02). The
// `read_request_line` helper is the only piece of the read loop that
// is unit-testable in isolation; the higher-level `run()` would require
// piping stdin/stdout through a real subprocess.

use super::*;
use std::io::Cursor;

#[test]
fn read_normal_line() {
    let input = b"{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/list\"}\n";
    let mut handle = Cursor::new(&input[..]);

    let result = read_request_line(&mut handle).expect("should produce a line");
    let line = result.expect("line should be Ok");

    assert_eq!(line, "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/list\"}");
}

#[test]
fn read_multiple_lines() {
    let input = b"line1\nline2\nline3\n";
    let mut handle = Cursor::new(&input[..]);

    let a = read_request_line(&mut handle).unwrap().unwrap();
    let b = read_request_line(&mut handle).unwrap().unwrap();
    let c = read_request_line(&mut handle).unwrap().unwrap();
    let d = read_request_line(&mut handle);

    assert_eq!(a, "line1");
    assert_eq!(b, "line2");
    assert_eq!(c, "line3");
    assert!(d.is_none(), "expected EOF after three lines");
}

#[test]
fn read_oversize_request_returns_err() {
    // Build a line that exceeds MAX_LINE_BYTES by a comfortable margin
    // (16 MiB cap, so use 20 MiB). We use a `Cursor` over a `Vec<u8>`,
    // which means we're not actually allocating 20 MiB on the heap in
    // the test process — the test itself uses ~20 MiB of memory, but
    // it's reclaimed as soon as the test returns.
    let mut payload = vec![b'x'; 20 * 1024 * 1024];
    payload.push(b'\n');
    let mut handle = Cursor::new(payload);

    let result = read_request_line(&mut handle).expect("expected a result (not EOF)");
    assert!(
        result.is_err(),
        "a 20 MiB line should be flagged as oversize"
    );
}

#[test]
fn read_request_at_exact_cap_is_ok() {
    // A line of exactly `MAX_LINE_BYTES - 1` (i.e. the body is one byte
    // short of the cap, plus the trailing newline) should be accepted.
    // The cap is on the *received* byte count, so the body must be
    // `MAX_LINE_BYTES - 1` (excluding the newline) to land at the cap.
    let body_size = MAX_LINE_BYTES - 1;
    let mut payload = vec![b'y'; body_size];
    payload.push(b'\n');
    let mut handle = Cursor::new(payload);

    let result = read_request_line(&mut handle).expect("expected a result (not EOF)");
    let line = result.expect("a line at the cap should be accepted");

    assert_eq!(line.len(), body_size, "line should be returned with newline stripped");
}

#[test]
fn read_recovers_after_oversize() {
    // Send an oversize request followed by a normal request. The
    // `drain_line` recovery logic must ensure the second request is
    // parsed independently of the first.
    let mut payload = vec![b'X'; 20 * 1024 * 1024];
    payload.push(b'\n');
    payload.extend_from_slice(b"{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"ping\"}\n");

    let mut handle = Cursor::new(payload);

    let first = read_request_line(&mut handle).unwrap();
    assert!(first.is_err(), "first line should be rejected as oversize");

    let second = read_request_line(&mut handle).unwrap().unwrap();
    assert_eq!(
        second,
        "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"ping\"}"
    );
}

#[test]
fn empty_input_returns_none() {
    let mut handle = Cursor::new(Vec::<u8>::new());
    let result = read_request_line(&mut handle);
    assert!(result.is_none(), "empty stdin should produce None (clean EOF)");
}

#[test]
fn find_project_root_returns_valid_path() {
    let root = super::find_project_root();
    // The project root should exist and contain Cargo.toml
    assert!(root.exists(), "project root should exist: {}", root.display());
    assert!(
        root.join("Cargo.toml").exists() || root.join(".clean-ctx.json").exists(),
        "project root should contain Cargo.toml or .clean-ctx.json: {}",
        root.display()
    );
}

#[test]
fn find_project_root_is_stable() {
    // Calling twice should return the same path (OnceLock)
    let root1 = super::find_project_root();
    let root2 = super::find_project_root();
    assert_eq!(root1, root2, "find_project_root should return the same path on repeated calls");
}
