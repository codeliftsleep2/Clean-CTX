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
    assert_eq!(
        extract_method_sig(raw, Fidelity::High),
        "public async getUserById(id: string): Promise<User>"
    );
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
    assert!(
        !out.contains("IActionResult"),
        "return type should not appear in name position: {}",
        out
    );
    assert!(
        !out.contains("HttpGet"),
        "attribute should be stripped: {}",
        out
    );
}

#[test]
fn high_fidelity_strips_csharp_attributes() {
    let raw = "[HttpGet]\npublic IActionResult Get()";
    assert_eq!(
        extract_method_sig(raw, Fidelity::High),
        "public IActionResult Get()"
    );
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
    assert!(
        !out.contains("Dictionary"),
        "tuple contents leaked into params: {}",
        out
    );
    assert!(
        !out.contains("Exact"),
        "tuple contents leaked into params: {}",
        out
    );
    assert!(
        !out.contains("Task<"),
        "return type leaked into name: {}",
        out
    );
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

// ── Name-anchored parameter-group selection (base-initializer audit) ──
//
// The method's own parameter list is the depth-0 group anchored to the
// DECLARED NAME — not the last depth-0 group. A constructor initializer
// (`: base(...)` / `: this(...)`) is a call site, not a parameter list;
// its parens must never be selected as the parameter group.

#[test]
fn find_method_params_selects_name_anchored_group_over_base_initializer() {
    let sig = "Greeter(string prefix) : base(prefix)";
    let (open, close) = find_method_params(sig).expect("should find params");
    assert_eq!(
        &sig[open..=close],
        "(string prefix)",
        "base-initializer call site must not be selected as the parameter group"
    );
}

#[test]
fn find_method_params_selects_name_anchored_group_over_this_initializer() {
    let sig = "Greeter(string prefix) : this(prefix)";
    let (open, close) = find_method_params(sig).expect("should find params");
    assert_eq!(
        &sig[open..=close],
        "(string prefix)",
        "this-initializer call site must not be selected as the parameter group"
    );
}

/// Parens inside quoted literals must not break the group scan: a default
/// value whose string literal contains an unbalanced `(` must not defeat
/// the enclosing parameter group (today the scan returns None and every
/// consumer falls back to legacy tokenization).
#[test]
fn find_method_params_literal_parens_do_not_break_group() {
    let sig = "void M(string s = \"a (\", int n)";
    let (open, close) = find_method_params(sig).expect("should find params");
    assert_eq!(&sig[open..=close], "(string s = \"a (\", int n)");
}

/// Control (passes before and after the fix): a generic method's parameter
/// list is anchored AFTER the generic argument list — the preceding
/// non-whitespace character is `>`.
#[test]
fn find_method_params_generic_method_anchored_after_generic_close() {
    let sig = "public static T M<T>(T x)";
    let (open, close) = find_method_params(sig).expect("should find params");
    assert_eq!(&sig[open..=close], "(T x)");
}

/// A `new()` generic-constraint group can never be the parameter list —
/// the real parameter group always precedes it in the declaration.
#[test]
fn find_method_params_constraint_new_group_never_selected() {
    let sig = "void M(int id) where T : new()";
    let (open, close) = find_method_params(sig).expect("should find params");
    assert_eq!(
        &sig[open..=close],
        "(int id)",
        "constraint group must not be selected as the parameter list"
    );
}

// ── Multi-line C# signatures (F-03 diff audit) ────────────────────

/// Regression: multi-line C# signatures (parameter list spanning multiple
/// lines) must produce a correct compact signature. Previously
/// `extract_method_sig` only took the first line, producing unbalanced-paren
/// garbage like `ValidateRow(()`. F-03 diff audit.
#[test]
fn low_fidelity_multi_line_csharp_signature() {
    let raw = "private void ValidateRow(\n    DataRow data,\n    string extra)\n{\n    // body\n}";
    let out = extract_method_sig(raw, Fidelity::Low);
    assert_eq!(out, "ValidateRow(data,extra)");
}

#[test]
fn medium_fidelity_multi_line_csharp_signature() {
    let raw = "private void ValidateRow(\n    DataRow data,\n    string extra)\n{\n    // body\n}";
    let out = extract_method_sig(raw, Fidelity::Medium);
    assert!(out.starts_with("ValidateRow("), "got: {}", out);
    assert!(out.contains("DataRow data"), "got: {}", out);
    assert!(out.contains("string extra"), "got: {}", out);
}

/// Multi-line C# signature with a tuple return type — the method's own
/// parameter list (LAST depth-0 group) must be found, not the tuple.
#[test]
fn medium_fidelity_multi_line_csharp_tuple_return() {
    let raw = "Task<(Dictionary<string, Guid> Exact, Dictionary<string, Guid> IgnoreCase)> GetOrgUnitDlc(\n    int id,\n    string name)\n{\n    // body\n}";
    let out = extract_method_sig(raw, Fidelity::Medium);
    assert!(out.starts_with("GetOrgUnitDlc("), "got: {}", out);
    assert!(out.contains("int id"), "got: {}", out);
    assert!(out.contains("string name"), "got: {}", out);
    assert!(
        !out.contains("Dictionary"),
        "tuple contents leaked into params: {}",
        out
    );
}

// ── Non-CBM Tool Audit 2026-08-25, finding #2 ────────────────────────
//
// `compress_workspace` at Low fidelity rendered a real-world C#
// signature with an `internal` modifier and a named-tuple return type as
// `internal static async Task(scope)` — the method identifier vanished.
// Mechanism (traced): with a named tuple return type whose last element is
// lowercase (`... requestId)>`), `is_csharp_return_type(tokens[len-2])`
// returns false, so the name-first fallback split the WHOLE signature at
// the first `<`, yielding the type prefix `internal static async Task` as
// the "name".

#[test]
fn low_fidelity_named_tuple_return_type_keeps_method_name() {
    let raw = concat!(
        "internal static async Task<(Container section, Term term, UserAccount account, Guid requestId)> ",
        "CreateRecordWithDefaults(IServiceProvider scope)\n",
        "{\n    return await BuildAsync();\n}"
    );
    assert_eq!(
        extract_method_sig(raw, Fidelity::Low),
        "CreateRecordWithDefaults(scope)",
        "low fidelity must retain the method identifier"
    );
}

#[test]
fn low_fidelity_unnamed_lowercase_tuple_tail_keeps_method_name() {
    // Same failure class without any leading modifiers.
    let raw = concat!(
        "Task<(Container section, Term term)> LoadValidReferenceData(int id)\n",
        "{\n    return default;\n}"
    );
    assert_eq!(
        extract_method_sig(raw, Fidelity::Low),
        "LoadValidReferenceData(id)"
    );
}

#[test]
fn compound_signature_medium_high_unchanged_by_low_fix() {
    // Pins current non-Low output for the audited signature shape so the
    // Low fix cannot leak into other fidelity tiers.
    let sig = concat!(
        "internal static async Task<(Container section, Term term)> ",
        "CreateRecordWithDefaults(IServiceProvider scope)"
    );
    let raw = format!("{sig}\n{{\n}}\n");

    // Medium keeps modifiers/async/return type; collapses ", " → ",".
    assert_eq!(
        extract_method_sig(&raw, Fidelity::Medium),
        "internal static async Task<(Container section,Term term)> \
         CreateRecordWithDefaults(IServiceProvider scope)"
    );
    // High keeps the full signature verbatim (before `{`).
    assert_eq!(extract_method_sig(&raw, Fidelity::High), sig);
}

// ── Expression-bodied member regression (gitdiff RED→GREEN audit) ────
//
// An expression-bodied member (`M() => expr;`) has NO body brace, so the
// first `{` in the raw capture belongs to something else — typically an
// interpolated-string hole. The signature span must therefore ALSO end at
// the first depth-0 `=>` outside string/char literals; otherwise the `=>`
// and body/literal fragments bleed onto the rendered diff signature line
// (see `gitdiff_interpolation_does_not_bleed_into_signature_line`).

#[test]
fn medium_fidelity_expression_bodied_stops_at_arrow() {
    let raw = "public string Display() => $\"Value: {Value}\";";
    let out = extract_method_sig(raw, Fidelity::Medium);
    assert_eq!(out, "Display()", "got: {}", out);
    assert!(
        !out.contains("=>"),
        "expression body leaked onto the signature: {}",
        out
    );
    assert!(
        !out.contains("Value"),
        "interpolation fragment leaked onto the signature: {}",
        out
    );
}

/// Control: a normal brace-bodied member must keep taking the legacy
/// first-`{` path byte-for-byte, even when its body contains the same
/// interpolated string — body content never reaches the signature.
#[test]
fn medium_fidelity_brace_bodied_control_unaffected() {
    let raw = concat!(
        "public void Write(string value)\n",
        "{\n",
        "    Console.WriteLine($\"Value: {value}\");\n",
        "}\n"
    );
    let out = extract_method_sig(raw, Fidelity::Medium);
    assert_eq!(out, "Write(string value)", "got: {}", out);
    assert!(
        !out.contains("=>"),
        "body leaked onto the signature: {}",
        out
    );
    assert!(
        !out.contains("$\"Value") && !out.contains("{value}"),
        "interpolation fragment leaked onto the signature: {}",
        out
    );
}
// ── Constructor base-initializer interpolation (issue ff2a29a) ──────

/// Regression (ff2a29a): a brace-bodied constructor whose
/// base-initializer argument is an INTERPOLATED string must have its
/// signature span end at the TRUE body `{`. The legacy literal-unaware
/// `stripped.find('{')` matched an interpolation HOLE inside
/// `: base($"Unexpected value: {value}, ...")`, truncating the header
/// mid-literal — High rendered a dangling unterminated
/// `$"Unexpected value:` fragment and every tier lost the remainder of
/// the initializer, corrupting the rendered diff label.
#[test]
fn high_fidelity_base_initializer_interpolation_keeps_full_header() {
    let raw = concat!(
        "public ExampleException(string value, object context)\n",
        "        : base($\"Unexpected value: {value}, context: {context}\")\n",
        "    {\n",
        "        Value = value;\n",
        "        Context = context;\n",
        "    }\n"
    );

    // HIGH: byte-exact header span INCLUDING the base-initializer,
    // terminating at the real body brace.
    let expected_sig = concat!(
        "public ExampleException(string value, object context)\n",
        "        : base($\"Unexpected value: {value}, context: {context}\")"
    );
    assert_eq!(
        extract_method_sig(raw, Fidelity::High),
        expected_sig,
        "header must extend past the interpolated initializer holes to the true body brace"
    );

    // MEDIUM (revised by the base-initializer label fix): the initializer
    // clause is call-site metadata, not signature — the compacted label is
    // the bare declaration. (ff2a29a pinned hole survival because the
    // truncation bug destroyed the initializer outright; the label tier
    // now drops the clause entirely, while High keeps the byte-exact
    // header above.)
    let medium = extract_method_sig(raw, Fidelity::Medium);
    assert_eq!(
        medium, "ExampleException(string value,object context)",
        "medium label must be the bare declaration without the initializer clause: {medium}"
    );
}

// ── Base-initializer compaction (Low/Medium label tiers) ─────────────

/// Low fidelity must identify the constructor by its DECLARED name. The
/// LAST-group locator picked the initializer's own parens, so the label
/// became `base(prefix)` — the method's identity was destroyed.
#[test]
fn low_fidelity_base_initializer_keeps_constructor_name() {
    let raw = concat!(
        "public Greeter(string prefix)\n",
        "        : base(prefix)\n",
        "    {\n",
        "        Initialize(prefix);\n",
        "    }\n"
    );
    assert_eq!(
        extract_method_sig(raw, Fidelity::Low),
        "Greeter(prefix)",
        "constructor label must carry the declared name, not the initializer call site"
    );
}

/// Medium fidelity keeps types but drops the initializer clause — the
/// `: base(...)` tail must not render onto the label where it reads as a
/// return annotation.
#[test]
fn medium_fidelity_base_initializer_drops_initializer_clause() {
    let raw = concat!(
        "public Greeter(string prefix)\n",
        "        : base(prefix)\n",
        "    {\n",
        "        Initialize(prefix);\n",
        "    }\n"
    );
    assert_eq!(
        extract_method_sig(raw, Fidelity::Medium),
        "Greeter(string prefix)",
        "medium label must not render the initializer clause"
    );
}
