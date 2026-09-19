use super::*;

// ── CTOR orphan regression (M6 bug) ─────────────────────────────
//
// Regression test for the confirmed failure shape that caused E003:
//   FLAGS(Mx, ["CTOR"])   ← additive CodePatternRecognizer
//   DEF_M(Cx, Mx, constructor)
//   PARAM(Mx, ...)
//   RET(Mx, ...)
//   FLAGS(Mx, ["PRIVATE"])  ← language-layer flag (e.g. TypeScriptLayer)
//
// The consumptive CompressingPatternRecognizer must consume ALL trailing
// Flags ops referencing the same method_id, not just the CTOR flag.

#[test]
fn ctor_consumes_all_trailing_flags_preventing_orphan() {
    // Exact failure shape: FLAGS(CTOR) + DEF_M + PARAM + RET + FLAGS(PRIVATE)
    let ops = vec![
        flags("M6", &["CTOR"]),
        defmethod("C2", "M6", "constructor"),
        param("M6", "P1", "$s", "authService"),
        ret("M6", "$v"),
        modifiers("M6", &[DeclarationModifier::Private]),
    ];
    let rec = CompressingPatternRecognizer::new();
    let (pats, stats) = rec.compress(&ops);

    // Should produce exactly 1 PatternOp (the CTOR)
    assert_eq!(pats.len(), 1, "should produce exactly one CTOR pattern");
    let ctor = &pats[0];
    assert!(matches!(ctor, PatternOp::Constructor { .. }));

    // All 5 source ops should be consumed (no orphaned flags)
    assert_eq!(stats.source_ops, 5, "all 5 source ops must be consumed");
    assert_eq!(stats.output_ops, 1, "exactly 1 pattern op emitted");

    if let PatternOp::Constructor {
        class_id,
        method_id,
        deps,
    } = ctor
    {
        assert_eq!(class_id, "C2");
        assert_eq!(method_id, "M6");
        assert!(deps.is_empty(), "no injects in this test");
    }
}

#[test]
fn ctor_consumes_multiple_trailing_flags() {
    // Multiple language-layer flags after the constructor body
    let ops = vec![
        flags("M6", &["CTOR"]),
        defmethod("C2", "M6", "constructor"),
        param("M6", "P1", "$s", "service"),
        ret("M6", "$v"),
        modifiers(
            "M6",
            &[DeclarationModifier::Private, DeclarationModifier::Static],
        ),
    ];
    let rec = CompressingPatternRecognizer::new();
    let (pats, stats) = rec.compress(&ops);

    assert_eq!(pats.len(), 1);
    assert_eq!(stats.source_ops, 5, "all 5 source ops consumed");
    assert!(matches!(&pats[0], PatternOp::Constructor { .. }));
}

#[test]
fn ctor_consumes_all_trailing_flags_through_merged_pipeline() {
    // Test that the full pipeline (recognize -> compress_merged -> CoreOp::Pattern)
    // leaves no orphaned FLAGS in the output.
    // The FLAGS(CTOR) and FLAGS(PRIVATE) both appear AFTER the DEF_M + PARAM + RET
    // in the production instruction stream, as emitted by the additive recognizer
    // and language layer respectively.
    let ops = vec![
        defclass("C2", "TestComponent"),
        defmethod("C2", "M6", "constructor"),
        param("M6", "P1", "$s", "authService"),
        ret("M6", "$v"),
        flags("M6", &["CTOR"]),
        modifiers("M6", &[DeclarationModifier::Private]),
        defmethod("C2", "M7", "ngOnInit"),
        ret("M7", "$v"),
    ];
    let rec = CompressingPatternRecognizer::new();
    let result = rec.recognize(&ops);

    // Verify no FLAGS referencing M6 remain orphaned
    let orphaned_m6_flags: Vec<&CoreOp> = result
        .iter()
        .filter(|op| matches!(op, CoreOp::Flags(mid, _) if mid == "M6"))
        .collect();
    assert!(
        orphaned_m6_flags.is_empty(),
        "no FLAGS(M6, ...) should remain orphaned in the output"
    );

    // Verify the constructor is represented as a PAT(CTOR, ...)
    let has_ctor_pat = result.iter().any(|op| {
        matches!(op, CoreOp::Pattern(name, args) if name == "CTOR" && args.len() >= 2 && args[0] == "C2" && args[1] == "M6")
    });
    assert!(
        has_ctor_pat,
        "constructor should be represented as PAT(CTOR, ...)"
    );

    // Verify the stream still has DefClass and the second method
    let has_defclass = result.iter().any(|op| matches!(op, CoreOp::DefClass(_, _)));
    let has_ngoninit = result
        .iter()
        .any(|op| matches!(op, CoreOp::DefMethod(_, _, name) if name == "ngOnInit"));
    assert!(has_defclass, "DefClass should pass through");
    assert!(has_ngoninit, "ngOnInit should pass through");
}

// ── Two-pass pipeline regression (production ordering) ──────────
//
// The additive CodePatternRecognizer runs FIRST, emitting FLAGS(Mx, ["CTOR"])
// BEFORE DEF_M. The consumptive CompressingPatternRecognizer runs SECOND
// and must handle the leading FLAGS op before DEF_M.
//
// This test reproduces the exact production ordering:
//   CodePatternRecognizer → CompressingPatternRecognizer → validator

#[test]
fn ctor_two_pass_pipeline_no_orphan_e003() {
    use crate::ir::layers::patterns::CodePatternRecognizer;
    use crate::ir::validator::DefaultValidator;
    use crate::ir::validator::IRValidator;

    // Input: DEF_M(M6, constructor) + PARAM + RET + FLAGS(PRIVATE) — exactly
    // what CoreIRPass + TypeScriptLayer produce before pattern recognition.
    let input = vec![
        defclass("C2", "TestComponent"),
        defmethod("C2", "M6", "constructor"),
        param("M6", "P1", "$s", "authService"),
        ret("M6", "$v"),
        modifiers("M6", &[DeclarationModifier::Private]),
        defmethod("C2", "M7", "ngOnInit"),
        ret("M7", "$v"),
    ];

    // Pass 1: additive CodePatternRecognizer — emits FLAGS(CTOR) before DEF_M
    let additive = CodePatternRecognizer::new();
    let after_additive = additive.recognize(&input);

    // Verify that additive recognizer prepended FLAGS(CTOR) before DEF_M(M6)
    let ctor_flag_pos = after_additive.iter().position(|op| {
        matches!(op, CoreOp::Flags(mid, flags) if mid == "M6" && flags.contains(&"CTOR".to_string()))
    });
    let def_m_pos = after_additive
        .iter()
        .position(|op| matches!(op, CoreOp::DefMethod(_, mid, _) if mid == "M6"));
    assert!(
        ctor_flag_pos < def_m_pos,
        "FLAGS(CTOR) must appear before DEF_M(M6) after additive pass"
    );

    // Pass 2: consumptive CompressingPatternRecognizer
    let consumptive = CompressingPatternRecognizer::new();
    let after_consumptive = consumptive.recognize(&after_additive);

    // Verify no FLAGS referencing M6 remain orphaned
    let orphaned_m6_flags: Vec<&CoreOp> = after_consumptive
        .iter()
        .filter(|op| matches!(op, CoreOp::Flags(mid, _) if mid == "M6"))
        .collect();
    assert!(
        orphaned_m6_flags.is_empty(),
        "no FLAGS(M6, ...) should remain after two-pass pipeline"
    );

    // Verify constructor is compressed to PAT(CTOR, ["C2", "M6", ...])
    let has_ctor_pat = after_consumptive.iter().any(|op| {
        matches!(op, CoreOp::Pattern(name, args) if name == "CTOR" && args.len() >= 2 && args[0] == "C2" && args[1] == "M6")
    });
    assert!(
        has_ctor_pat,
        "constructor should be PAT(CTOR, ...) after two-pass pipeline"
    );

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
        "no E003 errors should remain after two-pass pipeline; got: {:?}",
        e003_errors
    );

    // Verify passthrough ops are preserved
    let has_defclass = after_consumptive
        .iter()
        .any(|op| matches!(op, CoreOp::DefClass(_, _)));
    let has_ngoninit = after_consumptive
        .iter()
        .any(|op| matches!(op, CoreOp::DefMethod(_, _, name) if name == "ngOnInit"));
    assert!(has_defclass, "DefClass should pass through");
    assert!(has_ngoninit, "ngOnInit should pass through");
}

// ── Two-pass: Empty CTOR with leading additive flags + trailing language flags ──
//
// The additive CodePatternRecognizer emits FLAGS(Mx, ["CTOR"]) before DEF_M.
// The consumptive CompressingPatternRecognizer must consume both the leading
// CTOR flag and trailing language-layer flags (e.g. PRIVATE from TypeScript).

#[test]
fn empty_ctor_two_pass_no_orphan_e003() {
    use crate::ir::layers::patterns::CodePatternRecognizer;
    use crate::ir::validator::DefaultValidator;
    use crate::ir::validator::IRValidator;

    // Input: DEF_M(M6, constructor) + RET — empty constructor with trailing PRIVATE flag
    let input = vec![
        defclass("C2", "TestComponent"),
        defmethod("C2", "M6", "constructor"),
        ret("M6", "$v"),
        modifiers("M6", &[DeclarationModifier::Private]),
        defmethod("C2", "M7", "ngOnInit"),
        ret("M7", "$v"),
    ];

    // Pass 1: additive CodePatternRecognizer
    let additive = CodePatternRecognizer::new();
    let after_additive = additive.recognize(&input);

    // Verify FLAGS(CTOR) appears before DEF_M(M6)
    let ctor_flag_pos = after_additive.iter().position(|op| {
        matches!(op, CoreOp::Flags(mid, flags) if mid == "M6" && flags.contains(&"CTOR".to_string()))
    });
    let def_m_pos = after_additive
        .iter()
        .position(|op| matches!(op, CoreOp::DefMethod(_, mid, _) if mid == "M6"));
    assert!(
        ctor_flag_pos < def_m_pos,
        "FLAGS(CTOR) must appear before DEF_M(M6) after additive pass"
    );

    // Pass 2: consumptive CompressingPatternRecognizer
    let consumptive = CompressingPatternRecognizer::new();
    let after_consumptive = consumptive.recognize(&after_additive);

    // Verify no FLAGS referencing M6 remain orphaned
    let orphaned_m6_flags: Vec<&CoreOp> = after_consumptive
        .iter()
        .filter(|op| matches!(op, CoreOp::Flags(mid, _) if mid == "M6"))
        .collect();
    assert!(
        orphaned_m6_flags.is_empty(),
        "no FLAGS(M6, ...) should remain for empty CTOR after two-pass pipeline"
    );

    // Verify empty constructor is compressed to PAT(EMPTY_CTOR, ...)
    let has_empty_ctor_pat = after_consumptive.iter().any(|op| {
        matches!(op, CoreOp::Pattern(name, args) if name == "EMPTY_CTOR" && args.len() >= 2 && args[0] == "C2" && args[1] == "M6")
    });
    assert!(
        has_empty_ctor_pat,
        "empty constructor should be PAT(EMPTY_CTOR, ...)"
    );

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
        "no E003 errors for empty CTOR; got: {:?}",
        e003_errors
    );
}

// ── Two-pass: Observable with leading additive flags + trailing language flags ──

#[test]
fn observable_two_pass_no_orphan_e003() {
    use crate::ir::layers::patterns::CodePatternRecognizer;
    use crate::ir::validator::DefaultValidator;
    use crate::ir::validator::IRValidator;

    // Input: DEF_M + RET($P) + MOD_M(ASYNC) + MOD_M(PRIVATE)
    let input = vec![
        defclass("C2", "TestComponent"),
        defmethod("C2", "M6", "fetchData"),
        ret("M6", "$P"),
        modifiers("M6", &[DeclarationModifier::Async]),
        modifiers("M6", &[DeclarationModifier::Private]),
        defmethod("C2", "M7", "ngOnInit"),
        ret("M7", "$v"),
    ];

    // Pass 1: additive CodePatternRecognizer — emits FLAGS(OBSERVABLE) before DEF_M
    let additive = CodePatternRecognizer::new();
    let after_additive = additive.recognize(&input);

    // Verify FLAGS(OBSERVABLE) appears before DEF_M(M6)
    let obs_flag_pos = after_additive.iter().position(|op| {
        matches!(op, CoreOp::Flags(mid, flags) if mid == "M6" && flags.contains(&"OBSERVABLE".to_string()))
    });
    let def_m_pos = after_additive
        .iter()
        .position(|op| matches!(op, CoreOp::DefMethod(_, mid, _) if mid == "M6"));
    assert!(
        obs_flag_pos < def_m_pos,
        "FLAGS(OBSERVABLE) must appear before DEF_M(M6) after additive pass"
    );

    // Pass 2: consumptive CompressingPatternRecognizer
    let consumptive = CompressingPatternRecognizer::new();
    let after_consumptive = consumptive.recognize(&after_additive);

    // Verify no FLAGS referencing M6 remain orphaned
    let orphaned_m6_flags: Vec<&CoreOp> = after_consumptive
        .iter()
        .filter(|op| matches!(op, CoreOp::Flags(mid, _) if mid == "M6"))
        .collect();
    assert!(
        orphaned_m6_flags.is_empty(),
        "no FLAGS(M6, ...) should remain for Observable after two-pass pipeline"
    );
    let preserved_modifiers = after_consumptive
        .iter()
        .filter(|op| matches!(op, CoreOp::MethodModifiers(mid, _) if mid == "M6"))
        .count();
    assert_eq!(
        preserved_modifiers, 2,
        "both authoritative modifier occurrences survive"
    );

    // Verify observable is compressed to PAT(OBSERVABLE, ...)
    let has_obs_pat = after_consumptive.iter().any(|op| {
        matches!(op, CoreOp::Pattern(name, args) if name == "OBSERVABLE" && args.len() >= 3 && args[0] == "C2" && args[1] == "M6")
    });
    assert!(has_obs_pat, "observable should be PAT(OBSERVABLE, ...)");

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
        "no E003 errors for Observable; got: {:?}",
        e003_errors
    );
}

// ── Two-pass: Promise with leading additive flags + trailing language flags ──

#[test]
fn promise_two_pass_no_orphan_e003() {
    use crate::ir::layers::patterns::CodePatternRecognizer;
    use crate::ir::validator::DefaultValidator;
    use crate::ir::validator::IRValidator;

    // Input: DEF_M + RET($P) + MOD_M(PRIVATE) — no ASYNC modifier, so it is a Promise.
    let input = vec![
        defclass("C2", "TestComponent"),
        defmethod("C2", "M6", "fetchData"),
        ret("M6", "$P"),
        modifiers("M6", &[DeclarationModifier::Private]),
        defmethod("C2", "M7", "ngOnInit"),
        ret("M7", "$v"),
    ];

    // Pass 1: additive CodePatternRecognizer — emits FLAGS(OBSERVABLE) before DEF_M
    let additive = CodePatternRecognizer::new();
    let after_additive = additive.recognize(&input);

    // Pass 2: consumptive CompressingPatternRecognizer
    let consumptive = CompressingPatternRecognizer::new();
    let after_consumptive = consumptive.recognize(&after_additive);

    // Verify no FLAGS referencing M6 remain orphaned
    let orphaned_m6_flags: Vec<&CoreOp> = after_consumptive
        .iter()
        .filter(|op| matches!(op, CoreOp::Flags(mid, _) if mid == "M6"))
        .collect();
    assert!(
        orphaned_m6_flags.is_empty(),
        "no FLAGS(M6, ...) should remain for Promise after two-pass pipeline"
    );

    // Verify promise is compressed to PAT(PROMISE, ...)
    let has_promise_pat = after_consumptive.iter().any(|op| {
        matches!(op, CoreOp::Pattern(name, args) if name == "PROMISE" && args.len() >= 3 && args[0] == "C2" && args[1] == "M6")
    });
    assert!(has_promise_pat, "promise should be PAT(PROMISE, ...)");

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
        "no E003 errors for Promise; got: {:?}",
        e003_errors
    );
}
