// src/tests/spring_meta/properties_tests.rs
// Tests for Spring Boot properties file extraction.

use crate::spring_meta::properties::extract_properties_shape;

#[test]
fn test_properties_file_extraction() {
    let content = r#"# Server configuration
server.port=8080
server.servlet.context-path=/api

# Database configuration
spring.datasource.url=jdbc:mysql://localhost:3306/mydb
spring.datasource.username=user
spring.datasource.password=pass

# Active profiles
spring.profiles.active=dev,local
"#;
    let shape = extract_properties_shape(content);
    assert!(shape.keys.contains(&"server.port".to_string()));
    assert!(shape.keys.contains(&"spring.datasource.url".to_string()));
    assert!(shape.profiles.contains(&"dev".to_string()));
    assert!(shape.profiles.contains(&"local".to_string()));
    assert_eq!(shape.count, 6);
}

#[test]
fn test_yaml_file_extraction() {
    let content = r#"server:
  port: 8080
  servlet:
    context-path: /api

spring:
  datasource:
    url: jdbc:mysql://localhost:3306/mydb
    username: user
  profiles:
    active: prod
"#;
    let shape = extract_properties_shape(content);
    assert!(shape.keys.contains(&"server".to_string()));
    assert!(shape.keys.contains(&"spring".to_string()));
    assert!(shape.profiles.contains(&"prod".to_string()));
}

#[test]
fn test_empty_properties() {
    let content = "";
    let shape = extract_properties_shape(content);
    assert!(shape.keys.is_empty());
    assert!(shape.profiles.is_empty());
    assert_eq!(shape.count, 0);
}

#[test]
fn test_properties_with_comments() {
    let content = r#"# This is a comment
server.port=8080
# Another comment
spring.profiles.active=test
"#;
    let shape = extract_properties_shape(content);
    assert!(shape.keys.contains(&"server.port".to_string()));
    assert!(shape.profiles.contains(&"test".to_string()));
}

#[test]
fn test_properties_no_profiles() {
    let content = r#"server.port=8080
app.name=MyApp
"#;
    let shape = extract_properties_shape(content);
    assert_eq!(shape.profiles.len(), 0);
    assert_eq!(shape.count, 2);
}

#[test]
fn test_yaml_with_multiple_profiles() {
    let content = r#"spring:
  profiles:
    active: dev
    include: test,local
"#;
    let shape = extract_properties_shape(content);
    assert!(shape.profiles.contains(&"dev".to_string()));
    assert!(shape.profiles.contains(&"test".to_string()));
    assert!(shape.profiles.contains(&"local".to_string()));
}

#[test]
fn test_properties_marker_line() {
    let content = r#"server.port=8080
spring.datasource.url=jdbc:mysql://localhost:3306/mydb
"#;
    let shape = extract_properties_shape(content);
    let marker = shape.to_marker_line();
    assert!(marker.starts_with("Φpropf:"));
    assert!(marker.contains("server.port"));
    assert!(marker.contains("spring.datasource.url"));
}

#[test]
fn test_properties_marker_line_with_profiles() {
    let content = r#"server.port=8080
spring.profiles.active=dev,prod
"#;
    let shape = extract_properties_shape(content);
    let marker = shape.to_marker_line();
    assert!(marker.contains("profiles=[dev,prod]"));
    assert!(marker.contains("count=2"));
}

#[test]
fn test_empty_marker_line() {
    let content = "";
    let shape = extract_properties_shape(content);
    let marker = shape.to_marker_line();
    assert_eq!(marker, "Φpropf:empty");
}
