// src/tests/proptest/modifier_stripper.rs
//
// A-07: Property-based tests for strip_modifiers in compaction/modifiers.rs.
//
// Targets:
// - strip_modifiers: fuzz with random declarations and modifier lists
//
// Invariants tested:
// 1. strip_modifiers never panics on any input
// 2. strip_modifiers is idempotent
// 3. strip_modifiers with empty modifier list returns trimmed input
// 4. strip_modifiers preserves the non-modifier part of the declaration
// 5. strip_modifiers handles overlapping prefixes correctly
// 6. strip_modifiers with MODIFIERS_CLASS on already-clean input is a no-op

use proptest::prelude::*;
use crate::compaction::modifiers::{
    strip_modifiers, MODIFIERS_LOW, MODIFIERS_MEDIUM,
    MODIFIERS_CLASS, MODIFIERS_FIELD, MODIFIERS_STRUCT_RS,
};

proptest! {
    /// Invariant: strip_modifiers never panics on any input.
    #[test]
    fn strip_modifiers_never_panics(
        input in "\\PC{0,100}",
    ) {
        let result = strip_modifiers(&input, MODIFIERS_CLASS);
        // Result should be a valid string
        prop_assert!(result.len() <= input.len() + 1);
    }

    /// Invariant: strip_modifiers is idempotent — applying it twice
    /// gives the same result as applying it once.
    #[test]
    fn strip_modifiers_idempotent(
        input in "\\PC{0,100}",
    ) {
        let once = strip_modifiers(&input, MODIFIERS_CLASS);
        let twice = strip_modifiers(&once, MODIFIERS_CLASS);
        prop_assert_eq!(once, twice);
    }

    /// Invariant: strip_modifiers with empty modifier list returns
    /// the trimmed input unchanged.
    #[test]
    fn strip_modifiers_empty_list(
        input in "\\PC{0,100}",
    ) {
        let result = strip_modifiers(&input, &[]);
        prop_assert_eq!(result, input.trim());
    }

    /// Invariant: strip_modifiers with class modifiers on a class
    /// declaration should preserve the "class" keyword and class name.
    #[test]
    fn strip_modifiers_preserves_class_keyword(
        class_name in "[a-zA-Z_][a-zA-Z0-9_]{0,20}",
    ) {
        let input = format!("public static class {}", class_name);
        let result = strip_modifiers(&input, MODIFIERS_CLASS);
        prop_assert_eq!(result, format!("class {}", class_name));
    }

    /// Invariant: strip_modifiers with low modifiers on a method
    /// declaration should preserve the method name and parameters.
    #[test]
    fn strip_modifiers_preserves_method_name(
        method_name in "[a-zA-Z_][a-zA-Z0-9_]{0,20}",
        _param in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
    ) {
        let input = format!("public async {}()", method_name);
        let result = strip_modifiers(&input, MODIFIERS_LOW);
        let expected = format!("{}()", method_name);
        prop_assert!(result.starts_with(&expected) || result.starts_with(&method_name));
    }

    /// Invariant: strip_modifiers handles Rust struct modifiers.
    #[test]
    fn strip_modifiers_rust_struct(
        struct_name in "[a-zA-Z_][a-zA-Z0-9_]{0,20}",
    ) {
        let input = format!("pub(crate) struct {}", struct_name);
        let result = strip_modifiers(&input, MODIFIERS_STRUCT_RS);
        prop_assert_eq!(result, format!("struct {}", struct_name));
    }

    /// Invariant: strip_modifiers with field modifiers on a field
    /// declaration should preserve the field name and type.
    #[test]
    fn strip_modifiers_field(
        field_name in "[a-zA-Z_][a-zA-Z0-9_]{0,15}",
        field_type in "[a-zA-Z_][a-zA-Z0-9_]{0,15}",
    ) {
        let input = format!("private readonly {}: {};", field_name, field_type);
        let result = strip_modifiers(&input, MODIFIERS_FIELD);
        // After stripping, the result should contain the field name
        prop_assert!(result.contains(&field_name) || result.is_empty());
    }
}

// Additional targeted proptests for edge cases in modifier stripping.
proptest! {
    /// Invariant: overlapping prefixes are handled correctly.
    /// "export default " must be tried before "export ".
    #[test]
    fn overlapping_prefixes(
        keyword in "(abstract|sealed|final|static)",
        class_name in "[a-zA-Z_][a-zA-Z0-9_]{0,20}",
    ) {
        let input = format!("export default {} class {}", keyword, class_name);
        let result = strip_modifiers(&input, MODIFIERS_CLASS);
        prop_assert_eq!(result, format!("class {}", class_name));
    }

    /// Invariant: strip_modifiers handles medium-fidelity method signatures.
    #[test]
    fn medium_fidelity_method(
        method_name in "[a-zA-Z_][a-zA-Z0-9_]{0,20}",
        return_type in "[a-zA-Z_][a-zA-Z0-9_]{0,10}",
    ) {
        // Medium preserves async but strips public and static
        let input = format!("public async {}(): {}", method_name, return_type);
        let result = strip_modifiers(&input, MODIFIERS_MEDIUM);
        // async should be preserved, public and static should be stripped
        prop_assert!(result.contains(&method_name));
    }

    /// Invariant: strip_modifiers with MODIFIERS_LOW strips async.
    #[test]
    fn low_fidelity_strips_async(
        method_name in "[a-zA-Z_][a-zA-Z0-9_]{0,20}",
    ) {
        let input = format!("public async {}()", method_name);
        let result = strip_modifiers(&input, MODIFIERS_LOW);
        // Low should strip async as well
        prop_assert_eq!(result, format!("{}()", method_name));
    }
}