use super::*;

// --- Basic tests ----------------------------------------------------------

#[test]
fn apply_scope_defaults_passthrough_for_non_low() {
    let body = "SampleService;getUser(id);isAuthenticated()";
    let result = apply_scope_defaults(body, Fidelity::Medium);
    assert_eq!(result, body, "Medium fidelity should pass through unchanged");
}

#[test]
fn apply_scope_defaults_empty_body() {
    assert_eq!(apply_scope_defaults("", Fidelity::Low), "");
}

#[test]
fn apply_scope_defaults_no_classes() {
    let body = "import x from y";
    assert_eq!(apply_scope_defaults(body, Fidelity::Low), "import x from y");
}

// --- Class with one method (no defaults needed) ---------------------------

#[test]
fn apply_scope_defaults_one_method() {
    // Single class with one method — no defaults should be emitted
    let body = "Foo;$ctor C1 M1 $s payload;$r M1 $b";
    let result = apply_scope_defaults(body, Fidelity::Low);
    assert!(!result.contains("$dft"), "Single method should not emit $dft");
}

// --- Class with two methods sharing return type ---------------------------

#[test]
fn apply_scope_defaults_two_methods_shared_return() {
    let body = "Foo;$ctor C1 M1 $s payload;$r M1 $b;$ctor C1 M2 $n data;$r M2 $b";
    let result = apply_scope_defaults(body, Fidelity::Low);
    assert!(result.contains("$dft r=$b"), "Should default shared return type $b");
}

#[test]
fn apply_scope_defaults_two_methods_different_return() {
    let body = "Foo;$ctor C1 M1 $s payload;$r M1 $b;$ctor C1 M2 $n data;$r M2 $v";
    let result = apply_scope_defaults(body, Fidelity::Low);
    assert!(!result.contains("$dft"), "Different returns should not emit $dft");
}

// --- Class with shared flags ----------------------------------------------

#[test]
fn apply_scope_defaults_shared_flags() {
    let body = "Foo;$ctor C1 M1 $s payload;$r M1 $b;FLAGS M1 IF;$ctor C1 M2 $n data;$r M2 $b;FLAGS M2 IF";
    let result = apply_scope_defaults(body, Fidelity::Low);
    assert!(result.contains("$dft"), "Shared flags should produce $dft");
    assert!(result.contains("fl=IF"), "Should default IF flags");
}

#[test]
fn apply_scope_defaults_different_flags() {
    let body = "Foo;$ctor C1 M1 $s payload;$r M1 $b;FLAGS M1 IF;$ctor C1 M2 $n data;$r M2 $b;FLAGS M2 PU";
    let result = apply_scope_defaults(body, Fidelity::Low);
    // Different flags: IF vs PU — no flags default
    assert!(result.contains("FLAGS M1 IF") || result.contains("FLAGS M2 PU"),
            "Different flags should remain explicit");
}

// --- Two classes with independent defaults --------------------------------

#[test]
fn apply_scope_defaults_two_classes_separate_defaults() {
    let body = "Foo;$ctor C1 M1 $s payload;$r M1 $b;FLAGS M1 IF;$ctor C1 M2 $n data;$r M2 $b;FLAGS M2 IF;Bar;$ctor C2 M3 $v val;$r M3 $v;FLAGS M3 PU;$ctor C2 M4 $s data;$r M4 $v;FLAGS M4 PU";
    let result = apply_scope_defaults(body, Fidelity::Low);
    // Both classes should have $dft lines
    let dft_count = result.matches("$dft").count();
    assert_eq!(dft_count, 2, "Should have two $dft lines (one per class)");
}

// --- Pattern markers preserved --------------------------------------------

#[test]
fn apply_scope_defaults_pattern_markers_preserved() {
    let body = "Foo;$ctor C1 M1 $s payload;$r M1 $b;CTOR C1 M1";
    let result = apply_scope_defaults(body, Fidelity::Low);
    assert!(result.contains("CTOR C1 M1"), "Pattern marker should be preserved");
}