use super::*;

#[test]
fn low_fidelity_strips_all_modifiers() {
    let sig = "public async getUserById(id: string): Promise<User>";
    assert_eq!(extract_method_sig(sig, Fidelity::Low), "getUserById(id)");
}

#[test]
fn medium_fidelity_keeps_async_and_types() {
    let sig = "public async getUserById(id: string): Promise<User>";
    let out = extract_method_sig(sig, Fidelity::Medium);
    assert!(out.starts_with("async getUserById("));
    assert!(out.contains("id:string"));
    assert!(out.contains("Promise<User>"));
    assert!(!out.contains("public "));
}

#[test]
fn high_fidelity_preserves_everything() {
    let sig = "public async getUserById(id: string): Promise<User>";
    assert_eq!(extract_method_sig(sig, Fidelity::High), sig);
}

#[test]
fn edit_fidelity_carries_verbatim_body() {
    let raw = "public async getUserById(id: string): Promise<User> {\n  return this.users.find(u => u.id === id)!;\n}";
    assert_eq!(extract_method_sig(raw, Fidelity::Edit), raw);
}

#[test]
fn verbatim_fidelity_carries_verbatim_body() {
    let raw = "public async getUserById(id: string): Promise<User> {\n  return this.users.find(u => u.id === id)!;\n}";
    assert_eq!(extract_method_sig(raw, Fidelity::Verbatim), raw);
}

#[test]
fn high_fidelity_still_strips_body() {
    let raw = "public async getUserById(id: string): Promise<User> {\n  return this.users.find(u => u.id === id)!;\n}";
    assert_eq!(extract_method_sig(raw, Fidelity::High), "public async getUserById(id: string): Promise<User>");
}

// ── C# attribute handling ─────────────────────────────────────────

#[test]
fn low_fidelity_strips_csharp_attributes() {
    let raw = "[HttpGet]\npublic IActionResult Get()";
    assert_eq!(extract_method_sig(raw, Fidelity::Low), "Get()");
}

#[test]
fn medium_fidelity_strips_csharp_attributes() {
    let raw = "[HttpGet(\"{id}\")]\npublic IActionResult GetById(int id)";
    let out = extract_method_sig(raw, Fidelity::Medium);
    // C# return-type-first is normalized to name-first; params keep their
    // C# form ("int id").
    assert!(out.starts_with("GetById("), "got: {}", out);
    assert!(out.contains("int id"), "got: {}", out);
    assert!(!out.contains("IActionResult"), "return type should not appear in name position: {}", out);
    assert!(!out.contains("HttpGet"), "attribute should be stripped: {}", out);
}

#[test]
fn high_fidelity_strips_csharp_attributes() {
    let raw = "[HttpGet]\npublic IActionResult Get()";
    assert_eq!(extract_method_sig(raw, Fidelity::High), "public IActionResult Get()");
}

// ── C# tuple return type (legacy pipeline) ────────────────────────

/// Low fidelity: a C# tuple return type must NOT be mis-tokenized as the
/// parameter list. The method name must be `GetOrgUnitDlc`, not `Task<`.
#[test]
fn low_fidelity_csharp_tuple_return_type() {
    let sig = "Task<(Dictionary<string, Guid> Exact, Dictionary<string, Guid> IgnoreCase)> GetOrgUnitDlc(int id)";
    let out = extract_method_sig(sig, Fidelity::Low);
    assert_eq!(out, "GetOrgUnitDlc(id)");
}

/// Medium fidelity: a C# tuple return type must normalize to name-first
/// with the method's own params, not the tuple contents.
#[test]
fn medium_fidelity_csharp_tuple_return_type() {
    let sig = "Task<(Dictionary<string, Guid> Exact, Dictionary<string, Guid> IgnoreCase)> GetOrgUnitDlc(int id)";
    let out = extract_method_sig(sig, Fidelity::Medium);
    assert!(out.starts_with("GetOrgUnitDlc("), "got: {}", out);
    assert!(out.contains("int id"), "got: {}", out);
    assert!(!out.contains("Dictionary"), "tuple contents leaked into params: {}", out);
    assert!(!out.contains("Exact"), "tuple contents leaked into params: {}", out);
    assert!(!out.contains("Task<"), "return type leaked into name: {}", out);
}

/// `find_method_params` returns the LAST balanced depth-0 paren group.
#[test]
fn find_method_params_skips_tuple_return() {
    let sig = "Task<(A, B)> GetOrgUnitDlc(int id)";
    let (open, close) = find_method_params(sig).expect("should find params");
    assert_eq!(&sig[open..=close], "(int id)");
}

/// `find_method_params` returns None for unbalanced parens (defensive).
#[test]
fn find_method_params_unbalanced_returns_none() {
    assert!(find_method_params("foo((bar").is_none());
}
