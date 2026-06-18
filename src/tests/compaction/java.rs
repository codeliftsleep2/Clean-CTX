use super::*;

// ── Java type name extraction tests ───────────────────────────────────────

#[test]
fn java_extract_type_name_interface_basic() {
    assert_eq!(
        extract_java_type_name("public interface UserRepository", "interface.root"),
        "UserRepository"
    );
}

#[test]
fn java_extract_type_name_interface_with_extends() {
    assert_eq!(
        extract_java_type_name(
            "public interface UserRepository extends JpaRepository<User, Long>",
            "interface.root"
        ),
        "UserRepository:JpaRepository"
    );
}

#[test]
fn java_extract_type_name_enum() {
    assert_eq!(
        extract_java_type_name("public enum Status { ACTIVE, INACTION }", "enum.root"),
        "Status"
    );
}

#[test]
fn java_extract_type_name_record() {
    assert_eq!(
        extract_java_type_name("public record UserDto(String name, int age)", "record.root"),
        "UserDto"
    );
}

#[test]
fn java_extract_type_name_strips_modifiers() {
    assert_eq!(
        extract_java_type_name("public abstract interface Service", "interface.root"),
        "Service"
    );
}

// ── Java constructor signature tests ──────────────────────────────────────

#[test]
fn java_constructor_sig_low_fidelity() {
    assert_eq!(
        extract_java_constructor_sig("public UserService(UserRepository repo, AuthService auth)", Fidelity::Low),
        "UserService(UserRepository,AuthService)"
    );
}

#[test]
fn java_constructor_sig_medium_fidelity() {
    assert_eq!(
        extract_java_constructor_sig("public UserService(UserRepository repo, AuthService auth)", Fidelity::Medium),
        "UserService(UserRepository,AuthService)"
    );
}

#[test]
fn java_constructor_sig_high_fidelity() {
    assert_eq!(
        extract_java_constructor_sig("public UserService(UserRepository repo, AuthService auth)", Fidelity::High),
        "public UserService(UserRepository repo, AuthService auth)"
    );
}

#[test]
fn java_constructor_sig_no_params() {
    assert_eq!(
        extract_java_constructor_sig("public DefaultService()", Fidelity::Low),
        "DefaultService()"
    );
}

// ── Java package declaration tests ────────────────────────────────────────

#[test]
fn java_package_low_fidelity() {
    assert_eq!(
        compact_java_package("package com.example.userservice;", Fidelity::Low),
        "com.example.userservice"
    );
}

#[test]
fn java_package_medium_fidelity() {
    assert_eq!(
        compact_java_package("package com.example.userservice;", Fidelity::Medium),
        "package com.example.userservice"
    );
}

#[test]
fn java_package_high_fidelity() {
    assert_eq!(
        compact_java_package("package com.example.userservice;", Fidelity::High),
        "package com.example.userservice"
    );
}

// ── Java type entry formatting tests ──────────────────────────────────────

#[test]
fn java_format_type_entry_interface_low() {
    assert_eq!(
        format_java_type_entry("UserRepository", "interface.root", &[], Fidelity::Low),
        "UserRepository"
    );
}

#[test]
fn java_format_type_entry_interface_medium() {
    assert_eq!(
        format_java_type_entry("UserRepository", "interface.root", &[], Fidelity::Medium),
        "interface UserRepository"
    );
}

#[test]
fn java_format_type_entry_enum_low() {
    assert_eq!(
        format_java_type_entry("Status", "enum.root", &["ACTIVE".to_string(), "INACTIVE".to_string()], Fidelity::Low),
        "Status{ACTIVE;INACTIVE}"
    );
}

#[test]
fn java_format_type_entry_record_high() {
    assert_eq!(
        format_java_type_entry("UserDto", "record.root", &["String name".to_string(), "int age".to_string()], Fidelity::High),
        "record UserDto {\n  String name\n  int age"
    );
}
