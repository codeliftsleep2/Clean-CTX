use super::*;

#[test]
fn build_global_symbol_table_basic() {
    let bodies = vec![
        (
            "file1.ts".to_string(),
            "Service Observable HttpClient".to_string(),
        ),
        ("file2.ts".to_string(), "Service HttpClient".to_string()),
    ];
    let table = build_global_symbol_table(&bodies);
    // Service (3 occurrences across files) should be top-ranked
    assert_eq!(table.custom_symbol_count(), 3);
    assert!(table.get_opcode("Service").is_some());
    assert!(table.get_opcode("Observable").is_some());
    assert!(table.get_opcode("HttpClient").is_some());
}

#[test]
fn build_global_symbol_table_empty_files() {
    let bodies: Vec<(String, String)> = vec![];
    let table = build_global_symbol_table(&bodies);
    assert_eq!(table.custom_symbol_count(), 0);
}

#[test]
fn encode_with_global_symbols_at_low() {
    let bodies = vec![
        (
            "file1.ts".to_string(),
            "Service Observable HttpClient".to_string(),
        ),
        ("file2.ts".to_string(), "Service HttpClient".to_string()),
    ];
    let mut table = build_global_symbol_table(&bodies);
    let (encoded, footer) =
        encode_with_global_symbols(&mut table, "Service Observable HttpClient", Fidelity::Low);
    // All three tokens should be encoded with global opcodes
    assert!(!encoded.contains("Service"));
    assert!(!encoded.contains("Observable"));
    assert!(!encoded.contains("HttpClient"));
    // Each token should have been replaced by its opcode
    let svc = table.get_opcode("Service").unwrap();
    let http = table.get_opcode("HttpClient").unwrap();
    let obs = table.get_opcode("Observable").unwrap();
    assert!(encoded.contains(svc));
    assert!(encoded.contains(http));
    assert!(encoded.contains(obs));
    assert!(footer.contains("§GSYM"));
}

#[test]
fn encode_with_global_symbols_skips_at_medium() {
    let bodies = vec![("file1.ts".to_string(), "Service Observable".to_string())];
    let mut table = build_global_symbol_table(&bodies);
    let (encoded, footer) =
        encode_with_global_symbols(&mut table, "Service Observable", Fidelity::Medium);
    // At medium fidelity, should return unchanged
    assert_eq!(encoded, "Service Observable");
    assert!(footer.is_empty());
}

#[test]
fn encode_with_global_symbols_multiple_files() {
    let bodies = vec![
        (
            "file1.ts".to_string(),
            "Service Observable HttpClient".to_string(),
        ),
        ("file2.ts".to_string(), "Service HttpClient".to_string()),
    ];
    let mut table = build_global_symbol_table(&bodies);

    // Encode file 1
    table.begin_file();
    let _ = table.encode_body("Service Observable HttpClient");
    let refs1 = table.format_file_refs();

    // Encode file 2
    table.begin_file();
    let _ = table.encode_body("Service HttpClient");
    let refs2 = table.format_file_refs();

    // Both should reference the global dictionary
    assert!(refs1.contains("§GSYM"));
    assert!(refs2.contains("§GSYM"));

    // File 1 uses all 3 symbols, file 2 uses only 2
    let ids1: Vec<&str> = refs1
        .trim()
        .split(' ')
        .next_back()
        .unwrap()
        .split(',')
        .collect();
    let ids2: Vec<&str> = refs2
        .trim()
        .split(' ')
        .next_back()
        .unwrap()
        .split(',')
        .collect();
    assert!(ids1.len() > ids2.len());
}

#[test]
fn encode_with_global_symbols_deterministic() {
    let bodies = vec![(
        "file1.ts".to_string(),
        "Service Observable HttpClient".to_string(),
    )];
    let mut table1 = build_global_symbol_table(&bodies);
    let mut table2 = build_global_symbol_table(&bodies);

    let (enc1, _) = encode_with_global_symbols(&mut table1, "Service Observable", Fidelity::Low);
    let (enc2, _) = encode_with_global_symbols(&mut table2, "Service Observable", Fidelity::Low);
    assert_eq!(enc1, enc2);
}
