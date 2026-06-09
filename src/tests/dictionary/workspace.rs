use super::*;

#[test]
fn global_symbol_table_new_has_primitives() {
    let table = GlobalSymbolTable::new();
    // Should have primitive opcodes loaded
    assert!(table.get_opcode("async").is_some());
    assert!(table.get_opcode("class").is_some());
}

#[test]
fn global_symbol_table_count_tokens() {
    let mut table = GlobalSymbolTable::new();
    table.count_tokens("Service Service Observable HttpClient");
    table.count_tokens("Service HttpClient");
    // Service appears 3 times, HttpClient 2 times, Observable 1 time
    assert_eq!(table.total_frequency(), 6);
}

#[test]
fn global_symbol_table_build_codes_assigns_by_frequency() {
    let mut table = GlobalSymbolTable::new();
    table.count_tokens("Service Service Observable HttpClient");
    table.count_tokens("Service HttpClient");
    table.build_codes();
    // Service (3) should get the first available code ($d, since
    // $a=async, $b=boolean, $c=class are taken by primitives).
    // HttpClient (2) gets the next, Observable (1) the one after.
    let svc = table.get_opcode("Service").expect("Service should have a code");
    let http = table.get_opcode("HttpClient").expect("HttpClient should have a code");
    let obs = table.get_opcode("Observable").expect("Observable should have a code");
    // Service should have a lower (earlier) code than HttpClient
    assert!(svc < http, "Service ({}) should rank before HttpClient ({})", svc, http);
    // HttpClient should have a lower code than Observable
    assert!(http < obs, "HttpClient ({}) should rank before Observable ({})", http, obs);
}

#[test]
fn global_symbol_table_encode_body() {
    let mut table = GlobalSymbolTable::new();
    table.count_tokens("Service Service Observable HttpClient");
    table.count_tokens("Service HttpClient");
    table.build_codes();
    table.begin_file();
    let encoded = table.encode_body("Service Observable HttpClient");
    // All three tokens should be replaced by global opcodes
    assert!(!encoded.contains("Service"), "Service should be encoded");
    assert!(!encoded.contains("Observable"), "Observable should be encoded");
    assert!(!encoded.contains("HttpClient"), "HttpClient should be encoded");
    // At least 3 distinct global opcodes should appear
    let svc = table.get_opcode("Service").unwrap();
    let http = table.get_opcode("HttpClient").unwrap();
    let obs = table.get_opcode("Observable").unwrap();
    assert!(encoded.contains(svc));
    assert!(encoded.contains(http));
    assert!(encoded.contains(obs));
}

#[test]
fn global_symbol_table_format_global_footer() {
    let mut table = GlobalSymbolTable::new();
    table.count_tokens("Service Service Observable HttpClient");
    table.count_tokens("Service HttpClient");
    table.build_codes();
    let footer = table.format_global_footer();
    assert!(footer.contains("§GSYM"));
    assert!(footer.contains("§/GSYM"));
    // Footer should contain all three custom symbols (order depends on
    // which codes are available after skipping primitives)
    let svc_code = table.get_opcode("Service").unwrap();
    let http_code = table.get_opcode("HttpClient").unwrap();
    let obs_code = table.get_opcode("Observable").unwrap();
    assert!(footer.contains(&format!("{} = Service", svc_code)));
    assert!(footer.contains(&format!("{} = HttpClient", http_code)));
    assert!(footer.contains(&format!("{} = Observable", obs_code)));
}

#[test]
fn global_symbol_table_format_file_refs() {
    let mut table = GlobalSymbolTable::new();
    table.count_tokens("Service Service Observable HttpClient");
    table.count_tokens("Service HttpClient");
    table.build_codes();
    table.begin_file();
    let _ = table.encode_body("Service HttpClient");
    let refs = table.format_file_refs();
    assert!(refs.contains("§GSYM"));
    // Should contain indices for Service (0) and HttpClient (1)
    assert!(refs.contains("0"));
    assert!(refs.contains("1"));
}

#[test]
fn global_symbol_table_deterministic_order() {
    let mut table1 = GlobalSymbolTable::new();
    table1.count_tokens("Service Service Observable HttpClient");
    table1.count_tokens("Service HttpClient");
    table1.build_codes();

    let mut table2 = GlobalSymbolTable::new();
    table2.count_tokens("Service Service Observable HttpClient");
    table2.count_tokens("Service HttpClient");
    table2.build_codes();

    assert_eq!(table1.format_global_footer(), table2.format_global_footer());
}

#[test]
fn global_symbol_table_empty_body() {
    let mut table = GlobalSymbolTable::new();
    table.count_tokens("");
    table.build_codes();
    table.begin_file();
    let encoded = table.encode_body("");
    assert_eq!(encoded, "");
}

#[test]
fn global_symbol_table_no_custom_symbols() {
    let mut table = GlobalSymbolTable::new();
    table.build_codes();
    let footer = table.format_global_footer();
    assert!(footer.is_empty());
}

#[test]
fn global_symbol_table_custom_symbol_count() {
    let mut table = GlobalSymbolTable::new();
    table.count_tokens("Service Service Observable HttpClient");
    table.count_tokens("Service HttpClient");
    table.build_codes();
    assert_eq!(table.custom_symbol_count(), 3);
}

#[test]
fn global_symbol_table_is_primitive() {
    let table = GlobalSymbolTable::new();
    assert!(table.is_primitive("async"));
    assert!(table.is_primitive("class"));
    assert!(!table.is_primitive("Service"));
}

#[test]
fn global_symbol_table_file_refs_empty_after_begin() {
    let mut table = GlobalSymbolTable::new();
    table.count_tokens("Service Service");
    table.build_codes();
    table.begin_file();
    // No encoding done yet — refs should be empty
    let refs = table.format_file_refs();
    assert!(refs.is_empty());
}

#[test]
fn global_symbol_table_build_codes_idempotent() {
    let mut table = GlobalSymbolTable::new();
    table.count_tokens("Service Service Observable");
    table.build_codes();
    let footer1 = table.format_global_footer();
    // Calling build_codes again should not change anything
    table.build_codes();
    let footer2 = table.format_global_footer();
    assert_eq!(footer1, footer2);
}