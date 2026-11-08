//! Independent actual production reconstruction budget; public semantics precede RED.
use super::*;
#[test]
fn af002_binary_reverse_public_budget() {
    let _serial = crate::protocol::handler_response_serial();
    let records = rows();
    check_public(&records);
    let entries: usize = records.iter().map(|(_, w)| w.reverse_entries).sum();
    let capacity: usize = records.iter().map(|(_, w)| w.reverse_capacity).sum();
    let queries: usize = records.iter().map(|(_, w)| w.operand_lookups).sum();
    let intern: usize = records.iter().map(|(_, w)| w.intern_calls).sum();
    assert!(
        queries > 0 && intern > 0,
        "necessary StringTable and query work must execute"
    );
    assert_eq!(
        [entries, capacity],
        [0, 0],
        "Binary encoder redundant reverse-map budget"
    );
}
