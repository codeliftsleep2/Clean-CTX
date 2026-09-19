use super::*;

// ── Two-pass: Override with leading additive flags + trailing language flags ──

#[test]
fn override_two_pass_no_orphan_e003() {
    use crate::ir::layers::patterns::CodePatternRecognizer;
    use crate::ir::validator::DefaultValidator;
    use crate::ir::validator::IRValidator;

    // Input: DEF_M + FLAGS(OVERRIDE) + MOD_M(EXPORT)
    let input = vec![
        defclass("C2", "TestComponent"),
        defmethod("C2", "M6", "toString"),
        flags("M6", &["OVERRIDE"]),
        modifiers("M6", &[DeclarationModifier::Export]),
        defmethod("C2", "M7", "ngOnInit"),
        ret("M7", "$v"),
    ];

    // Pass 1: additive CodePatternRecognizer
    let additive = CodePatternRecognizer::new();
    let after_additive = additive.recognize(&input);

    // Pass 2: consumptive CompressingPatternRecognizer
    let consumptive = CompressingPatternRecognizer::new();
    let after_consumptive = consumptive.recognize(&after_additive);

    // Verify no FLAGS referencing M6 remain orphaned
    let orphaned_m6_flags: Vec<&CoreOp> = after_consumptive
        .iter()
        .filter(
            |op| matches!(op, CoreOp::Flags(mid, _) | CoreOp::PatternFacts(mid, _) if mid == "M6"),
        )
        .collect();
    assert!(
        orphaned_m6_flags.is_empty(),
        "no FLAGS(M6, ...) should remain for Override after two-pass pipeline"
    );

    // Verify override is compressed to PAT(OVERRIDE, ...)
    let has_override_pat = after_consumptive.iter().any(|op| {
        matches!(op, CoreOp::Pattern(name, args) if name == "OVERRIDE" && args.len() >= 2 && args[0] == "C2" && args[1] == "M6")
    });
    assert!(has_override_pat, "override should be PAT(OVERRIDE, ...)");

    // Pass 3: validation — must produce no E003 errors
    let validator = DefaultValidator::new();
    let ir = crate::ir::compiler::CompiledIR {
        file_id: "test".into(),
        instructions: after_consumptive.clone(),
        version: 1,
    };
    let errors = validator.validate(&ir);
    let e003_errors: Vec<_> = errors.iter().filter(|e| e.code == "E003").collect();
    assert!(
        e003_errors.is_empty(),
        "no E003 errors for Override; got: {:?}",
        e003_errors
    );
}

// ── Two-pass: Getter/Setter ──
//
// NOTE: The additive CodePatternRecognizer's accessor pattern consumes the
// DEF_M (consumed=1) and replaces it with FLAGS(GETTER/SETTER, ...). After
// the additive pass, no DEF_M remains for the consumptive recognizer to match.
// The FLAGS op itself references the consumed method_id, which the validator
// flags as E003 — this is a pre-existing issue in the additive recognizer.
// These patterns are tested by the single-pass compress() tests above.

// ── Two-pass: Rust pub fn new() with EXPORT flag ──
//
// RustLayer emits FLAGS(Mx, ["export"]) after DEF_M + PARAM + RET.
// The additive recognizer emits FLAGS(Mx, ["CTOR"]) before DEF_M.
// Both must be consumed by the centralized wrapper.

#[test]
fn rust_pub_new_two_pass_no_orphan_e003() {
    use crate::ir::layers::patterns::CodePatternRecognizer;
    use crate::ir::validator::DefaultValidator;
    use crate::ir::validator::IRValidator;

    // Input simulating Rust `pub fn new()`: DEF_M + PARAM + RET + FLAGS(export)
    let input = vec![
        defclass("C2", "Service"),
        defmethod("C2", "M6", "new"),
        param("M6", "P1", "$s", "config"),
        ret("M6", "$v"),
        flags("M6", &["export"]),
        defmethod("C2", "M7", "process"),
        ret("M7", "$v"),
    ];

    // Pass 1: additive CodePatternRecognizer
    let additive = CodePatternRecognizer::new();
    let after_additive = additive.recognize(&input);

    // Verify FLAGS(CTOR) appears before DEF_M(M6)
    let ctor_flag_pos = after_additive.iter().position(|op| {
        matches!(op, CoreOp::PatternFacts(mid, facts) if mid == "M6" && facts.contains(&PatternFact::Constructor))
    });
    let def_m_pos = after_additive
        .iter()
        .position(|op| matches!(op, CoreOp::DefMethod(_, mid, _) if mid == "M6"));
    assert!(
        ctor_flag_pos < def_m_pos,
        "FLAGS(CTOR) must appear before DEF_M(M6) for Rust pub fn new()"
    );

    // Pass 2: consumptive CompressingPatternRecognizer
    let consumptive = CompressingPatternRecognizer::new();
    let after_consumptive = consumptive.recognize(&after_additive);

    // Verify no FLAGS referencing M6 remain orphaned
    let orphaned_m6_flags: Vec<&CoreOp> = after_consumptive
        .iter()
        .filter(
            |op| matches!(op, CoreOp::Flags(mid, _) | CoreOp::PatternFacts(mid, _) if mid == "M6"),
        )
        .collect();
    assert!(
        orphaned_m6_flags.is_empty(),
        "no FLAGS(M6, ...) should remain for Rust pub fn new()"
    );

    // Verify constructor is compressed to PAT(CTOR, ...)
    let has_ctor_pat = after_consumptive.iter().any(|op| {
        matches!(op, CoreOp::Pattern(name, args) if name == "CTOR" && args.len() >= 2 && args[0] == "C2" && args[1] == "M6")
    });
    assert!(has_ctor_pat, "Rust pub fn new() should be PAT(CTOR, ...)");

    // Pass 3: validation — must produce no E003 errors
    let validator = DefaultValidator::new();
    let ir = crate::ir::compiler::CompiledIR {
        file_id: "test".into(),
        instructions: after_consumptive.clone(),
        version: 1,
    };
    let errors = validator.validate(&ir);
    let e003_errors: Vec<_> = errors.iter().filter(|e| e.code == "E003").collect();
    assert!(
        e003_errors.is_empty(),
        "no E003 errors for Rust pub fn new(); got: {:?}",
        e003_errors
    );
}

// ── Two-pass: Java/C# metadata flags ──
//
// JavaLayer/CSharpLayer emit FLAGS(Mx, ["PUBLIC", "STATIC"]) after the method body.
// Combined with additive FLAGS(CTOR), this exercises the full invariant.

#[test]
fn java_csharp_metadata_two_pass_no_orphan_e003() {
    use crate::ir::layers::patterns::CodePatternRecognizer;
    use crate::ir::validator::DefaultValidator;
    use crate::ir::validator::IRValidator;

    // Input simulating Java/C# constructor with PUBLIC STATIC flags
    let input = vec![
        defclass("C2", "MyClass"),
        defmethod("C2", "M6", "constructor"),
        param("M6", "P1", "$s", "config"),
        ret("M6", "$v"),
        modifiers(
            "M6",
            &[DeclarationModifier::Export, DeclarationModifier::Static],
        ),
        defmethod("C2", "M7", "doWork"),
        ret("M7", "$v"),
    ];

    // Pass 1: additive CodePatternRecognizer
    let additive = CodePatternRecognizer::new();
    let after_additive = additive.recognize(&input);

    // Pass 2: consumptive CompressingPatternRecognizer
    let consumptive = CompressingPatternRecognizer::new();
    let after_consumptive = consumptive.recognize(&after_additive);

    // Verify no FLAGS referencing M6 remain orphaned
    let orphaned_m6_flags: Vec<&CoreOp> = after_consumptive
        .iter()
        .filter(
            |op| matches!(op, CoreOp::Flags(mid, _) | CoreOp::PatternFacts(mid, _) if mid == "M6"),
        )
        .collect();
    assert!(
        orphaned_m6_flags.is_empty(),
        "no FLAGS(M6, ...) should remain for Java/C# metadata"
    );

    // Verify constructor is compressed to PAT(CTOR, ...)
    let has_ctor_pat = after_consumptive.iter().any(|op| {
        matches!(op, CoreOp::Pattern(name, args) if name == "CTOR" && args.len() >= 2 && args[0] == "C2" && args[1] == "M6")
    });
    assert!(has_ctor_pat, "Java/C# constructor should be PAT(CTOR, ...)");

    // Pass 3: validation — must produce no E003 errors
    let validator = DefaultValidator::new();
    let ir = crate::ir::compiler::CompiledIR {
        file_id: "test".into(),
        instructions: after_consumptive.clone(),
        version: 1,
    };
    let errors = validator.validate(&ir);
    let e003_errors: Vec<_> = errors.iter().filter(|e| e.code == "E003").collect();
    assert!(
        e003_errors.is_empty(),
        "no E003 errors for Java/C# metadata; got: {:?}",
        e003_errors
    );
}

// ── Two-pass: Multiple methods with different flags — no cross-contamination ──
//
// Ensures the centralized wrapper never consumes another method's flags.

#[test]
fn multiple_methods_no_cross_contamination() {
    use crate::ir::layers::patterns::CodePatternRecognizer;
    use crate::ir::validator::DefaultValidator;
    use crate::ir::validator::IRValidator;

    // Two constructors with different flags — must not cross-contaminate
    let input = vec![
        defclass("C2", "Service"),
        // First constructor: M6 with PRIVATE flag
        defmethod("C2", "M6", "constructor"),
        param("M6", "P1", "$s", "config"),
        ret("M6", "$v"),
        modifiers("M6", &[DeclarationModifier::Private]),
        // Second constructor: M7 with PUBLIC flag
        defmethod("C2", "M7", "new"),
        param("M7", "P1", "$s", "data"),
        ret("M7", "$v"),
        modifiers("M7", &[DeclarationModifier::Export]),
    ];

    // Pass 1: additive CodePatternRecognizer
    let additive = CodePatternRecognizer::new();
    let after_additive = additive.recognize(&input);

    // Pass 2: consumptive CompressingPatternRecognizer
    let consumptive = CompressingPatternRecognizer::new();
    let after_consumptive = consumptive.recognize(&after_additive);

    // Verify no FLAGS referencing M6 or M7 remain orphaned
    let orphaned_m6_flags: Vec<&CoreOp> = after_consumptive
        .iter()
        .filter(
            |op| matches!(op, CoreOp::Flags(mid, _) | CoreOp::PatternFacts(mid, _) if mid == "M6"),
        )
        .collect();
    let orphaned_m7_flags: Vec<&CoreOp> = after_consumptive
        .iter()
        .filter(
            |op| matches!(op, CoreOp::Flags(mid, _) | CoreOp::PatternFacts(mid, _) if mid == "M7"),
        )
        .collect();
    assert!(
        orphaned_m6_flags.is_empty(),
        "no FLAGS(M6, ...) should remain"
    );
    assert!(
        orphaned_m7_flags.is_empty(),
        "no FLAGS(M7, ...) should remain"
    );

    // Verify both constructors are compressed
    let has_m6_ctor = after_consumptive.iter().any(|op| {
        matches!(op, CoreOp::Pattern(name, args) if name == "CTOR" && args.len() >= 2 && args[1] == "M6")
    });
    let has_m7_ctor = after_consumptive.iter().any(|op| {
        matches!(op, CoreOp::Pattern(name, args) if name == "CTOR" && args.len() >= 2 && args[1] == "M7")
    });
    assert!(has_m6_ctor, "M6 constructor should be PAT(CTOR, ...)");
    assert!(has_m7_ctor, "M7 constructor should be PAT(CTOR, ...)");

    // Pass 3: validation — must produce no E003 errors
    let validator = DefaultValidator::new();
    let ir = crate::ir::compiler::CompiledIR {
        file_id: "test".into(),
        instructions: after_consumptive.clone(),
        version: 1,
    };
    let errors = validator.validate(&ir);
    let e003_errors: Vec<_> = errors.iter().filter(|e| e.code == "E003").collect();
    assert!(
        e003_errors.is_empty(),
        "no E003 errors for multiple methods; got: {:?}",
        e003_errors
    );
}
