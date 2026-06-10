// src/tests/ir/symbol_table.rs
//
// Unit tests for the GlobalSymbolTable, SymbolEntry, and SymbolKind types.
// Tests cover registration, lookup, unregistration, versioning, file tracking,
// and change-detection queries.

use crate::ir::symbol_table::{GlobalSymbolTable, SymbolKind};

// ── SymbolKind Tests ──────────────────────────────────────────────────

#[test]
fn test_symbol_kind_prefixes() {
    assert_eq!(SymbolKind::Class.prefix(), "C");
    assert_eq!(SymbolKind::Method.prefix(), "M");
    assert_eq!(SymbolKind::Field.prefix(), "F");
    assert_eq!(SymbolKind::Interface.prefix(), "I");
    assert_eq!(SymbolKind::Param.prefix(), "P");
    assert_eq!(SymbolKind::Import.prefix(), "IM");
    assert_eq!(SymbolKind::Type.prefix(), "T");
}

#[test]
fn test_symbol_kind_equality() {
    assert_eq!(SymbolKind::Class, SymbolKind::Class);
    assert_ne!(SymbolKind::Class, SymbolKind::Method);
}

// ── New / Default Tests ──────────────────────────────────────────────

#[test]
fn test_new_table_is_empty() {
    let table = GlobalSymbolTable::new();
    assert!(table.is_empty());
    assert_eq!(table.len(), 0);
    assert_eq!(table.version(), 0);
}

#[test]
fn test_default_table_is_empty() {
    let table = GlobalSymbolTable::default();
    assert!(table.is_empty());
    assert_eq!(table.len(), 0);
    assert_eq!(table.version(), 0);
}

// ── Version Tests ────────────────────────────────────────────────────

#[test]
fn test_bump_version_increments() {
    let mut table = GlobalSymbolTable::new();
    assert_eq!(table.version(), 0);
    assert_eq!(table.bump_version(), 1);
    assert_eq!(table.version(), 1);
    assert_eq!(table.bump_version(), 2);
    assert_eq!(table.version(), 2);
}

#[test]
fn test_set_version_explicit() {
    let mut table = GlobalSymbolTable::new();
    table.set_version(42);
    assert_eq!(table.version(), 42);
    table.bump_version();
    assert_eq!(table.version(), 43);
}

// ── Next Alias Tests ─────────────────────────────────────────────────

#[test]
fn test_next_alias_generates_correct_prefixes() {
    let mut table = GlobalSymbolTable::new();

    assert_eq!(table.next_alias(SymbolKind::Class), "C1");
    assert_eq!(table.next_alias(SymbolKind::Method), "M1");
    assert_eq!(table.next_alias(SymbolKind::Field), "F1");
    assert_eq!(table.next_alias(SymbolKind::Interface), "I1");
    assert_eq!(table.next_alias(SymbolKind::Param), "P1");
    assert_eq!(table.next_alias(SymbolKind::Import), "IM1");
    assert_eq!(table.next_alias(SymbolKind::Type), "T1");
}

#[test]
fn test_next_alias_increments_per_kind() {
    let mut table = GlobalSymbolTable::new();

    // Class aliases
    assert_eq!(table.next_alias(SymbolKind::Class), "C1");
    assert_eq!(table.next_alias(SymbolKind::Class), "C2");
    assert_eq!(table.next_alias(SymbolKind::Class), "C3");

    // Method aliases (independent counter)
    assert_eq!(table.next_alias(SymbolKind::Method), "M1");
    assert_eq!(table.next_alias(SymbolKind::Method), "M2");

    // Field aliases
    assert_eq!(table.next_alias(SymbolKind::Field), "F1");
}

// ── Registration Tests ───────────────────────────────────────────────

#[test]
fn test_register_single_symbol() {
    let mut table = GlobalSymbolTable::new();
    table.register("C1".into(), "SampleService".into(), SymbolKind::Class, "file1.ts");

    assert_eq!(table.len(), 1);
    assert!(!table.is_empty());

    let entry = table.get("C1").unwrap();
    assert_eq!(entry.alias, "C1");
    assert_eq!(entry.original, "SampleService");
    assert_eq!(entry.kind, SymbolKind::Class);
    assert_eq!(entry.file_id, "file1.ts");
    assert_eq!(entry.version_first, 0);
    assert_eq!(entry.version_last, 0);
}

#[test]
fn test_register_creates_reverse_index() {
    let mut table = GlobalSymbolTable::new();
    table.register("M1".into(), "processData".into(), SymbolKind::Method, "file1.ts");

    let by_original = table.get_by_original("processData").unwrap();
    assert_eq!(by_original.alias, "M1");

    let alias = table.alias_for("processData").unwrap();
    assert_eq!(alias, "M1");
}

#[test]
fn test_register_adds_to_file_members() {
    let mut table = GlobalSymbolTable::new();
    table.register("C1".into(), "ServiceA".into(), SymbolKind::Class, "alpha.ts");
    table.register("C2".into(), "ServiceB".into(), SymbolKind::Class, "alpha.ts");
    table.register("M1".into(), "helper".into(), SymbolKind::Method, "beta.ts");

    let alpha_symbols = table.get_file_symbols("alpha.ts");
    assert_eq!(alpha_symbols.len(), 2);
    assert!(alpha_symbols.iter().any(|s| s.alias == "C1"));
    assert!(alpha_symbols.iter().any(|s| s.alias == "C2"));

    let beta_symbols = table.get_file_symbols("beta.ts");
    assert_eq!(beta_symbols.len(), 1);
    assert_eq!(beta_symbols[0].alias, "M1");
}

#[test]
fn test_register_overwrite_existing_alias() {
    let mut table = GlobalSymbolTable::new();
    table.register("C1".into(), "OriginalName".into(), SymbolKind::Class, "file1.ts");
    assert_eq!(table.len(), 1);
    assert!(table.contains_original("OriginalName"));

    // Re-register with same alias but different name
    table.register("C1".into(), "RenamedName".into(), SymbolKind::Class, "file1.ts");

    assert_eq!(table.len(), 1);
    assert!(!table.contains_original("OriginalName")); // old reverse entry removed
    assert!(table.contains_original("RenamedName"));   // new reverse entry present

    let entry = table.get("C1").unwrap();
    assert_eq!(entry.original, "RenamedName");
}

// ── Unregistration Tests ─────────────────────────────────────────────

#[test]
fn test_unregister_removes_from_all_indexes() {
    let mut table = GlobalSymbolTable::new();
    table.register("C1".into(), "Service".into(), SymbolKind::Class, "main.ts");
    table.register("M1".into(), "doStuff".into(), SymbolKind::Method, "main.ts");

    assert_eq!(table.len(), 2);

    let removed = table.unregister("C1");
    assert!(removed.is_some());
    assert_eq!(removed.unwrap().alias, "C1");

    // Should no longer be findable
    assert_eq!(table.len(), 1);
    assert!(table.get("C1").is_none());
    assert!(table.get_by_original("Service").is_none());

    // Other symbols unaffected
    assert!(table.get("M1").is_some());
    assert!(table.get_by_original("doStuff").is_some());

    // File members updated
    let file_syms = table.get_file_symbols("main.ts");
    assert_eq!(file_syms.len(), 1);
    assert_eq!(file_syms[0].alias, "M1");
}

#[test]
fn test_unregister_nonexistent_returns_none() {
    let mut table = GlobalSymbolTable::new();
    let result = table.unregister("NONEXISTENT");
    assert!(result.is_none());
    assert_eq!(table.len(), 0);
}

// ── Touch Tests ──────────────────────────────────────────────────────

#[test]
fn test_touch_updates_version_last() {
    let mut table = GlobalSymbolTable::new();
    table.bump_version(); // version: 1
    table.bump_version(); // version: 2
    table.register("C1".into(), "Service".into(), SymbolKind::Class, "main.ts");

    // version_last should be 2 (current version at registration time)
    assert_eq!(table.get("C1").unwrap().version_last, 2);

    table.bump_version(); // version: 3
    table.touch("C1");

    assert_eq!(table.get("C1").unwrap().version_last, 3);
    // version_first should remain unchanged
    assert_eq!(table.get("C1").unwrap().version_first, 2);
}

#[test]
fn test_touch_nonexistent_is_noop() {
    let mut table = GlobalSymbolTable::new();
    table.touch("GHOST"); // should not panic
}

// ── Query Tests ──────────────────────────────────────────────────────

#[test]
fn test_get_file_symbols_empty_for_unknown_file() {
    let table = GlobalSymbolTable::new();
    let symbols = table.get_file_symbols("nonexistent.ts");
    assert!(symbols.is_empty());
}

#[test]
fn test_get_changed_since_empty_when_no_changes() {
    let table = GlobalSymbolTable::new();
    let changed = table.get_changed_since(0);
    assert!(changed.is_empty());
}

#[test]
fn test_get_changed_since_returns_correct_subset() {
    let mut table = GlobalSymbolTable::new();

    // Register at version 0
    table.register("C1".into(), "ClassA".into(), SymbolKind::Class, "a.ts");
    table.register("C2".into(), "ClassB".into(), SymbolKind::Class, "a.ts");

    let changed_v0 = table.get_changed_since(0);
    assert!(changed_v0.is_empty()); // none have version_last > 0

    table.bump_version(); // version: 1

    // Touch C1 at version 1
    table.touch("C1");

    let changed_v0_now = table.get_changed_since(0);
    assert_eq!(changed_v0_now.len(), 1);
    assert_eq!(changed_v0_now[0].alias, "C1");

    let changed_v1 = table.get_changed_since(1);
    assert!(changed_v1.is_empty()); // C1 version_last == 1, not > 1
}

#[test]
fn test_get_changed_at_or_after() {
    let mut table = GlobalSymbolTable::new();
    table.register("C1".into(), "A".into(), SymbolKind::Class, "f.ts");
    table.bump_version();
    table.register("C2".into(), "B".into(), SymbolKind::Class, "f.ts");
    table.bump_version();
    table.register("C3".into(), "C".into(), SymbolKind::Class, "f.ts");

    // C1: v0, C2: v1, C3: v2
    let changed = table.get_changed_at_or_after(1);
    assert_eq!(changed.len(), 2);
    assert!(changed.iter().any(|e| e.alias == "C2"));
    assert!(changed.iter().any(|e| e.alias == "C3"));
}

#[test]
fn test_all_symbols_returns_everything() {
    let mut table = GlobalSymbolTable::new();
    table.register("C1".into(), "A".into(), SymbolKind::Class, "f1.ts");
    table.register("M1".into(), "foo".into(), SymbolKind::Method, "f1.ts");
    table.register("F1".into(), "bar".into(), SymbolKind::Field, "f2.ts");

    assert_eq!(table.all_symbols().len(), 3);
}

#[test]
fn test_file_count_and_ids() {
    let mut table = GlobalSymbolTable::new();
    assert_eq!(table.file_count(), 0);

    table.register("C1".into(), "A".into(), SymbolKind::Class, "alpha.ts");
    table.register("M1".into(), "foo".into(), SymbolKind::Method, "alpha.ts");
    table.register("C2".into(), "B".into(), SymbolKind::Interface, "beta.ts");

    assert_eq!(table.file_count(), 2);
    let ids = table.file_ids();
    assert!(ids.contains(&"alpha.ts"));
    assert!(ids.contains(&"beta.ts"));
    assert_eq!(ids.len(), 2);
}

// ── Contains Tests ───────────────────────────────────────────────────

#[test]
fn test_contains_checks_existence() {
    let mut table = GlobalSymbolTable::new();
    table.register("C1".into(), "Service".into(), SymbolKind::Class, "x.ts");

    assert!(table.contains("C1"));
    assert!(!table.contains("C2"));
    assert!(table.contains_original("Service"));
    assert!(!table.contains_original("Missing"));
}

// ── Clear Tests ──────────────────────────────────────────────────────

#[test]
fn test_clear_resets_everything() {
    let mut table = GlobalSymbolTable::new();
    table.register("C1".into(), "Svc".into(), SymbolKind::Class, "main.ts");
    table.register("M1".into(), "do".into(), SymbolKind::Method, "main.ts");
    table.bump_version();
    table.bump_version();

    assert_eq!(table.len(), 2);
    assert_eq!(table.file_count(), 1);
    assert_eq!(table.version(), 2);

    table.clear();

    assert!(table.is_empty());
    assert_eq!(table.len(), 0);
    assert_eq!(table.file_count(), 0);
    assert_eq!(table.version(), 0);
    assert!(!table.contains("C1"));
    assert!(!table.contains_original("Svc"));
}

// ── Integration: 10 Symbols Across 3 Files ──────────────────────────

#[test]
fn test_integration_10_symbols_3_files() {
    let mut table = GlobalSymbolTable::new();

    // File α (alpha.ts): 4 symbols
    table.register("C1".into(), "UserService".into(), SymbolKind::Class, "alpha.ts");
    table.register("M1".into(), "getUser".into(), SymbolKind::Method, "alpha.ts");
    table.register("M2".into(), "setUser".into(), SymbolKind::Method, "alpha.ts");
    table.register("F1".into(), "userName".into(), SymbolKind::Field, "alpha.ts");

    // File β (beta.ts): 3 symbols
    table.register("C2".into(), "Logger".into(), SymbolKind::Class, "beta.ts");
    table.register("M3".into(), "logInfo".into(), SymbolKind::Method, "beta.ts");
    table.register("I1".into(), "ILogger".into(), SymbolKind::Interface, "beta.ts");

    // File γ (gamma.ts): 3 symbols
    table.register("C3".into(), "AuthGuard".into(), SymbolKind::Class, "gamma.ts");
    table.register("IM1".into(), "AuthModule".into(), SymbolKind::Import, "gamma.ts");
    table.register("P1".into(), "token".into(), SymbolKind::Param, "gamma.ts");

    // Total count
    assert_eq!(table.len(), 10);
    assert_eq!(table.file_count(), 3);

    // ── Lookup by alias ──
    assert_eq!(table.get("C1").unwrap().original, "UserService");
    assert_eq!(table.get("M3").unwrap().original, "logInfo");
    assert_eq!(table.get("P1").unwrap().original, "token");
    assert!(table.get("NONEXISTENT").is_none());

    // ── Lookup by original name ──
    assert_eq!(table.get_by_original("ILogger").unwrap().alias, "I1");
    assert_eq!(table.get_by_original("AuthGuard").unwrap().alias, "C3");
    assert!(table.get_by_original("Missing").is_none());

    // ── Alias for ──
    assert_eq!(table.alias_for("getUser"), Some("M1"));
    assert_eq!(table.alias_for("AuthModule"), Some("IM1"));

    // ── File membership ──
    let alpha = table.get_file_symbols("alpha.ts");
    assert_eq!(alpha.len(), 4);
    let alpha_names: Vec<&str> = alpha.iter().map(|s| s.original.as_str()).collect();
    assert!(alpha_names.contains(&"UserService"));
    assert!(alpha_names.contains(&"getUser"));
    assert!(alpha_names.contains(&"setUser"));
    assert!(alpha_names.contains(&"userName"));

    let beta = table.get_file_symbols("beta.ts");
    assert_eq!(beta.len(), 3);

    let gamma = table.get_file_symbols("gamma.ts");
    assert_eq!(gamma.len(), 3);

    // Unknown file
    assert!(table.get_file_symbols("unknown.ts").is_empty());

    // ── Version tracking ──
    // All registered at version 0, so changed_since(0) is empty
    assert!(table.get_changed_since(0).is_empty());

    // Bump and touch one symbol
    table.bump_version();
    table.touch("C2");

    let changed = table.get_changed_since(0);
    assert_eq!(changed.len(), 1);
    assert_eq!(changed[0].alias, "C2");

    // ── Unregister one symbol ──
    let removed = table.unregister("F1");
    assert!(removed.is_some());
    assert_eq!(table.len(), 9);
    assert_eq!(table.get_file_symbols("alpha.ts").len(), 3);
    assert!(table.get("F1").is_none());
    assert!(table.get_by_original("userName").is_none());
}

// ── Edge Cases ───────────────────────────────────────────────────────

#[test]
fn test_register_unregister_register_cycle() {
    let mut table = GlobalSymbolTable::new();

    table.register("C1".into(), "Service".into(), SymbolKind::Class, "main.ts");
    assert_eq!(table.len(), 1);

    table.unregister("C1");
    assert_eq!(table.len(), 0);

    // Re-register with same alias
    table.register("C1".into(), "ServiceV2".into(), SymbolKind::Class, "main.ts");
    assert_eq!(table.len(), 1);

    let entry = table.get("C1").unwrap();
    assert_eq!(entry.original, "ServiceV2");
}

#[test]
fn test_large_alias_counters() {
    let mut table = GlobalSymbolTable::new();

    for i in 1..=100 {
        let alias = table.next_alias(SymbolKind::Method);
        assert_eq!(alias, format!("M{}", i));
    }
}

#[test]
fn test_import_and_type_aliases() {
    let mut table = GlobalSymbolTable::new();

    table.register("IM1".into(), "Observable".into(), SymbolKind::Import, "rx.ts");
    table.register("T1".into(), "UserId".into(), SymbolKind::Type, "types.ts");

    assert_eq!(table.get("IM1").unwrap().kind, SymbolKind::Import);
    assert_eq!(table.get("T1").unwrap().kind, SymbolKind::Type);

    assert_eq!(table.get_file_symbols("rx.ts").len(), 1);
    assert_eq!(table.get_file_symbols("types.ts").len(), 1);
}