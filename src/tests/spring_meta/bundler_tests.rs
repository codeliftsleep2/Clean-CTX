// src/tests/spring_meta/bundler_tests.rs
// Tests for Spring Boot layer bundler.

use crate::spring_meta::bundler::{SpringLayer, resolve_bundle, resolve_layer};

#[test]
fn test_resolve_layer_controller() {
    let source = r#"@RestController
public class UserController {}"#;
    assert_eq!(resolve_layer(source), SpringLayer::Controller);
}

#[test]
fn test_resolve_layer_service() {
    let source = r#"@Service
public class UserService {}"#;
    assert_eq!(resolve_layer(source), SpringLayer::Service);
}

#[test]
fn test_resolve_layer_repository() {
    let source = r#"@Repository
public class UserRepository {}"#;
    assert_eq!(resolve_layer(source), SpringLayer::Repository);
}

#[test]
fn test_resolve_layer_configuration() {
    let source = r#"@Configuration
public class AppConfig {}"#;
    assert_eq!(resolve_layer(source), SpringLayer::Configuration);
}

#[test]
fn test_resolve_layer_spring_boot_application() {
    let source = r#"@SpringBootApplication
public class Application {}"#;
    assert_eq!(resolve_layer(source), SpringLayer::Configuration);
}

#[test]
fn test_resolve_layer_unknown() {
    let source = r#"public class PlainClass {}"#;
    assert_eq!(resolve_layer(source), SpringLayer::Unknown);
}

#[test]
fn test_resolve_bundle() {
    let source = r#"@RestController
public class UserController {}"#;
    let path = std::path::Path::new("/src/UserController.java");
    let bundle = resolve_bundle(path, source);
    assert!(bundle.is_some());
    let bundle = bundle.unwrap();
    assert_eq!(bundle.name, "UserController");
    assert_eq!(bundle.layer, SpringLayer::Controller);
}

#[test]
fn test_resolve_bundle_non_java() {
    let source = r#"@RestController
public class UserController {}"#;
    let path = std::path::Path::new("/src/UserController.ts");
    let bundle = resolve_bundle(path, source);
    assert!(bundle.is_none());
}

#[test]
fn test_spring_layer_priority() {
    assert_eq!(SpringLayer::Controller.priority(), 1);
    assert_eq!(SpringLayer::Service.priority(), 2);
    assert_eq!(SpringLayer::Repository.priority(), 3);
    assert_eq!(SpringLayer::Configuration.priority(), 4);
    assert_eq!(SpringLayer::Unknown.priority(), 5);
}
