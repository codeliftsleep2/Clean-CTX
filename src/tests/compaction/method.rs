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
