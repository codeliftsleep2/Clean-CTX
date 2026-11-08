use super::write_varint;
pub(crate) fn missing_operand() {
    let str_to_idx: std::collections::HashMap<&str, u64> = std::collections::HashMap::new();
    let encode_operand = |buf: &mut Vec<u8>, s: &str| {
        let idx = str_to_idx
            .get(s)
            .copied()
            .expect("canonical binary operand missing from string table");
        write_varint(buf, idx);
    };

    encode_operand(&mut Vec::new(), "absent");
}

fn reused_missing_operand() {
    let table = crate::ir::string_table::StringTable::new();
    let encode_operand = |buf: &mut Vec<u8>, s: &str| {
        let idx = table
            .index_of(s)
            .map(|i| i as u64)
            .expect("canonical binary operand missing from string table");
        write_varint(buf, idx);
    };
    encode_operand(&mut Vec::new(), "absent");
}

#[test]
fn afperf_binary_reverse_accessor_law() {
    use crate::ir::string_table::StringTable;
    let mut table = StringTable::new();
    let mut first = Vec::new();
    for value in ["", "共有", "a", "共有", "", "λ", "a"] {
        let index = first.iter().position(|s| s == &value).unwrap_or_else(|| {
            first.push(value);
            first.len() - 1
        });
        assert_eq!(table.intern(value), index);
        for (index, key) in first.iter().enumerate() {
            assert_eq!(table.index_of(key), Some(index));
        }
    }
    let before = table.to_json();
    assert_eq!(table.index_of("absent"), None);
    assert_eq!(table.to_json(), before, "lookup must be read-only");
    for restored in [table.clone(), StringTable::from_json(&before).unwrap()] {
        for (index, key) in first.iter().enumerate() {
            assert_eq!(restored.index_of(key), Some(index));
        }
    }
    let panic = std::panic::catch_unwind(reused_missing_operand).unwrap_err();
    let message = panic
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| panic.downcast_ref::<String>().map(String::as_str))
        .unwrap();
    assert_eq!(
        message,
        "canonical binary operand missing from string table"
    );
}
