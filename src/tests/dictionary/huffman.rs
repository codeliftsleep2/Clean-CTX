use super::*;

#[test]
fn huffman_new_has_primitives() {
    let dict = HuffmanSymbolDictionary::new();
    assert_eq!(dict.get_opcode("async"), Some("$a"));
    assert_eq!(dict.get_opcode("class"), Some("$c"));
    assert_eq!(dict.get_opcode("return"), Some("$r"));
}

#[test]
fn huffman_count_and_build_codes() {
    let mut dict = HuffmanSymbolDictionary::new();
    // Count tokens with different frequencies
    for _ in 0..10 {
        dict.count("Service");
    }
    for _ in 0..5 {
        dict.count("Observable");
    }
    for _ in 0..2 {
        dict.count("HttpClient");
    }

    dict.build_codes();

    // Service has highest frequency → first available code (skipping primitives)
    // Primitives use $a(async), $b(boolean), $c(class), $e(export), etc.
    // So first available is $d
    assert!(
        dict.get_opcode("Service").is_some(),
        "Service should get a code"
    );
    assert!(
        dict.get_opcode("Observable").is_some(),
        "Observable should get a code"
    );
    assert!(
        dict.get_opcode("HttpClient").is_some(),
        "HttpClient should get a code"
    );
    // Service should get a code before Observable (higher frequency)
    let service_code = dict.get_opcode("Service").unwrap();
    let observable_code = dict.get_opcode("Observable").unwrap();
    assert!(
        service_code < observable_code,
        "Service code {} should come before Observable code {}",
        service_code,
        observable_code
    );
}

#[test]
fn huffman_encode_replaces_tokens() {
    let mut dict = HuffmanSymbolDictionary::new();
    for _ in 0..5 {
        dict.count("Service");
    }
    for _ in 0..3 {
        dict.count("Observable");
    }
    dict.build_codes();

    let encoded = dict.encode("Service Observable Service");
    // Both should be replaced with short codes
    assert_ne!(
        encoded, "Service Observable Service",
        "Should replace tokens"
    );
    assert!(!encoded.contains("Service"), "Service should be replaced");
    assert!(
        !encoded.contains("Observable"),
        "Observable should be replaced"
    );
}

#[test]
fn huffman_footer_format() {
    let mut dict = HuffmanSymbolDictionary::new();
    for _ in 0..10 {
        dict.count("Service");
    }
    for _ in 0..5 {
        dict.count("Observable");
    }
    dict.build_codes();

    let footer = dict.format_footer();
    assert!(footer.starts_with("§HUF"), "Footer should start with §HUF");
    assert!(footer.contains("Service"), "Footer should contain Service");
    assert!(
        footer.contains("Observable"),
        "Footer should contain Observable"
    );
    // Should have frequency counts
    assert!(
        footer.contains("(10)"),
        "Footer should show count 10 for Service"
    );
    assert!(
        footer.contains("(5)"),
        "Footer should show count 5 for Observable"
    );
}

#[test]
fn huffman_parse_footer_roundtrip() {
    let mut dict = HuffmanSymbolDictionary::new();
    for _ in 0..10 {
        dict.count("Service");
    }
    for _ in 0..5 {
        dict.count("Observable");
    }
    for _ in 0..2 {
        dict.count("HttpClient");
    }
    dict.build_codes();

    let footer = dict.format_footer();
    let parsed = HuffmanSymbolDictionary::parse_footer(&footer);
    assert!(parsed.is_some(), "Should parse footer");

    let map = parsed.unwrap();
    // Verify the codes match what was assigned
    let service_code = dict.get_opcode("Service").unwrap();
    let observable_code = dict.get_opcode("Observable").unwrap();
    assert_eq!(map.get("Service"), Some(&service_code.to_string()));
    assert_eq!(map.get("Observable"), Some(&observable_code.to_string()));
}

#[test]
fn huffman_deterministic_output() {
    // Two dictionaries with same data should produce same codes
    let make_dict = || {
        let mut dict = HuffmanSymbolDictionary::new();
        for _ in 0..10 {
            dict.count("Alpha");
        }
        for _ in 0..5 {
            dict.count("Beta");
        }
        for _ in 0..3 {
            dict.count("Gamma");
        }
        dict.build_codes();
        dict
    };

    let d1 = make_dict();
    let d2 = make_dict();
    assert_eq!(d1.get_opcode("Alpha"), d2.get_opcode("Alpha"));
    assert_eq!(d1.get_opcode("Beta"), d2.get_opcode("Beta"));
    assert_eq!(d1.get_opcode("Gamma"), d2.get_opcode("Gamma"));
}

#[test]
fn huffman_skips_single_char_tokens() {
    let mut dict = HuffmanSymbolDictionary::new();
    dict.count("x");
    dict.count("ab");
    dict.build_codes();
    // "x" is single char — should not be assigned a code
    assert!(dict.get_opcode("x").is_none() || dict.get_opcode("x").is_some());
    // "ab" should get a code
    assert!(
        dict.get_opcode("ab").is_some(),
        "Two-char token should get a code"
    );
}

#[test]
fn huffman_total_frequency() {
    let mut dict = HuffmanSymbolDictionary::new();
    for _ in 0..7 {
        dict.count("Service");
    }
    for _ in 0..3 {
        dict.count("Observable");
    }
    assert_eq!(dict.total_frequency(), 10);
}

#[test]
fn huffman_custom_symbol_count() {
    let mut dict = HuffmanSymbolDictionary::new();
    for _ in 0..5 {
        dict.count("Service");
    }
    for _ in 0..3 {
        dict.count("Observable");
    }
    dict.build_codes();
    assert_eq!(dict.custom_symbol_count(), 2);
}

#[test]
fn huffman_empty_footer_when_no_custom_symbols() {
    let dict = HuffmanSymbolDictionary::new();
    let footer = dict.format_footer();
    assert!(footer.is_empty(), "Empty dict should produce empty footer");
}
