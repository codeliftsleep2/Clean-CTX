// src/tests/spring_meta/detect_tests.rs
// Tests for Spring Boot detection heuristic.

use crate::spring_meta::detect::is_spring_file;

#[test]
fn test_rest_controller_detection() {
    let source = r#"
import org.springframework.web.bind.annotation.RestController;
import org.springframework.web.bind.annotation.GetMapping;

@RestController
@RequestMapping("/api")
public class UserController {
    @GetMapping("/users")
    public List<User> getUsers() {
        return userService.findAll();
    }
}
"#;
    assert!(is_spring_file(source));
}

#[test]
fn test_service_detection() {
    let source = r#"
import org.springframework.stereotype.Service;

@Service
public class UserService {
    public List<User> findAll() {
        return userRepository.findAll();
    }
}
"#;
    assert!(is_spring_file(source));
}

#[test]
fn test_repository_detection() {
    let source = r#"
import org.springframework.stereotype.Repository;

@Repository
public class UserRepository {
    // Repository methods
}
"#;
    assert!(is_spring_file(source));
}

#[test]
fn test_configuration_detection() {
    let source = r#"
import org.springframework.context.annotation.Configuration;

@Configuration
public class AppConfig {
    // Configuration methods
}
"#;
    assert!(is_spring_file(source));
}

#[test]
fn test_spring_boot_application_detection() {
    let source = r#"
import org.springframework.boot.autoconfigure.SpringBootApplication;

@SpringBootApplication
public class Application {
    public static void main(String[] args) {
        SpringApplication.run(Application.class, args);
    }
}
"#;
    assert!(is_spring_file(source));
}

#[test]
fn test_weak_signal_with_spring_import() {
    let source = r#"
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

@Component
public class MyComponent {
    @Autowired
    private UserService userService;
}
"#;
    assert!(is_spring_file(source));
}

#[test]
fn test_non_spring_file() {
    let source = r#"
public class PlainJavaClass {
    private String name;
    
    public String getName() {
        return name;
    }
}
"#;
    assert!(!is_spring_file(source));
}

#[test]
fn test_plain_java_with_autowired_only() {
    // @Autowired alone without Spring import should not trigger
    let source = r#"
public class MyClass {
    // This is not a Spring file
    private String data;
}
"#;
    assert!(!is_spring_file(source));
}

#[test]
fn test_request_mapping_detection() {
    let source = r#"
import org.springframework.web.bind.annotation.RequestMapping;

@RequestMapping("/api")
public class ApiController {
    // Controller methods
}
"#;
    assert!(is_spring_file(source));
}

#[test]
fn test_all_mapping_annotations() {
    for annotation in &["@GetMapping", "@PostMapping", "@PutMapping", "@DeleteMapping", "@PatchMapping"] {
        let source = format!(r#"
import org.springframework.web.bind.annotation.RestController;
import org.springframework.web.bind.annotation.{0};

@RestController
public class TestController {{
    {0}("/test")
    public void test() {{}}
}}
"#, annotation);
        assert!(is_spring_file(&source), "Failed to detect {}", annotation);
    }
}