// src/tests/spring_meta/markers_tests.rs
// Tests for Spring Boot marker construction and expansion.

use crate::spring_meta::markers::{
    build_autowired_line, build_bean_line, build_configuration_line,
    build_configuration_properties_line, build_controller_line, build_repository_line, build_rest_controller_line, build_service_line, build_value_line,
    expand_phi_in_line, PhiLineKind, RequestMappingMapping,
};

#[test]
fn test_rest_controller_line() {
    let mappings = vec![
        RequestMappingMapping {
            method: Some("GET".to_string()),
            path: "/api/users".to_string(),
        },
        RequestMappingMapping {
            method: Some("POST".to_string()),
            path: "/api/users".to_string(),
        },
    ];
    let line = build_rest_controller_line("UserController", &mappings);
    assert_eq!(line, "Φrest:UserController map=[GET /api/users,POST /api/users]");
}

#[test]
fn test_rest_controller_line_no_mappings() {
    let line = build_rest_controller_line("UserController", &[]);
    assert_eq!(line, "Φrest:UserController");
}

#[test]
fn test_service_line() {
    let line = build_service_line("UserService");
    assert_eq!(line, "Φsvc:UserService");
}

#[test]
fn test_repository_line() {
    let line = build_repository_line("UserRepository");
    assert_eq!(line, "Φrepo:UserRepository");
}

#[test]
fn test_controller_line() {
    let mappings = vec![RequestMappingMapping {
        method: None,
        path: "/api".to_string(),
    }];
    let line = build_controller_line("ApiController", &mappings);
    assert_eq!(line, "Φctrl:ApiController map=[/api]");
}

#[test]
fn test_configuration_line() {
    let line = build_configuration_line("AppConfig");
    assert_eq!(line, "Φconf:AppConfig");
}

#[test]
fn test_autowired_line() {
    let line = build_autowired_line("userService");
    assert_eq!(line, "Φaut:userService");
}

#[test]
fn test_value_line() {
    let line = build_value_line("serverUrl");
    assert_eq!(line, "Φval:serverUrl");
}

#[test]
fn test_bean_line() {
    let line = build_bean_line("userService");
    assert_eq!(line, "Φbean:userService");
}

#[test]
fn test_configuration_properties_line() {
    let line = build_configuration_properties_line("AppProperties");
    assert_eq!(line, "Φprop:AppProperties");
}

#[test]
fn test_expand_phi_in_line() {
    let input = "Φrest:UserController map=[GET /api/users] Φsvc:UserService";
    let expected = "@RestController UserController map=[GET /api/users] @Service UserService";
    assert_eq!(expand_phi_in_line(input), expected);
}

#[test]
fn test_expand_phi_in_line_partial() {
    // Unknown markers should pass through
    let input = "Φrest:Controller Φunknown:test";
    let expected = "@RestController Controller Φunknown:test";
    assert_eq!(expand_phi_in_line(input), expected);
}

#[test]
fn test_phi_line_kind_from_token() {
    assert_eq!(PhiLineKind::from_token("Φrest"), Some(PhiLineKind::RestController));
    assert_eq!(PhiLineKind::from_token("Φsvc"), Some(PhiLineKind::Service));
    assert_eq!(PhiLineKind::from_token("Φrepo"), Some(PhiLineKind::Repository));
    assert_eq!(PhiLineKind::from_token("Φconf"), Some(PhiLineKind::Configuration));
    assert_eq!(PhiLineKind::from_token("Φmap"), Some(PhiLineKind::RequestMapping));
    assert_eq!(PhiLineKind::from_token("Φaut"), Some(PhiLineKind::Autowired));
    assert_eq!(PhiLineKind::from_token("Φval"), Some(PhiLineKind::Value));
    assert_eq!(PhiLineKind::from_token("Φbean"), Some(PhiLineKind::Bean));
    assert_eq!(PhiLineKind::from_token("Φprop"), Some(PhiLineKind::ConfigurationProperties));
    assert_eq!(PhiLineKind::from_token("unknown"), None);
}

#[test]
fn test_request_mapping_mapping_to_string() {
    let mapping = RequestMappingMapping {
        method: Some("GET".to_string()),
        path: "/api/users".to_string(),
    };
    assert_eq!(mapping.to_string(), "GET /api/users");

    let mapping_no_method = RequestMappingMapping {
        method: None,
        path: "/api".to_string(),
    };
    assert_eq!(mapping_no_method.to_string(), "/api");
}