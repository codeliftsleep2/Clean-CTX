package com.example.demo;

import org.springframework.boot.SpringApplication;
import org.springframework.boot.autoconfigure.SpringBootApplication;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.context.annotation.Primary;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.boot.context.properties.ConfigurationProperties;
import org.springframework.stereotype.Service;
import org.springframework.stereotype.Repository;
import org.springframework.stereotype.Controller;
import org.springframework.web.bind.annotation.RestController;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.PutMapping;
import org.springframework.web.bind.annotation.DeleteMapping;
import org.springframework.web.bind.annotation.PatchMapping;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.ResponseBody;
import java.io.Serializable;
import java.util.List;
import java.util.Map;

/**
 * Comprehensive Java/Spring Boot test fixture.
 *
 * Exercises every supported feature of:
 *   - src/ir/layers/java.rs  (Java language layer)
 *   - src/spring_meta/       (Spring Boot meta-layer)
 *
 * Coverage inline before each section.
 */
// ════════════════════════════════════════════════════════════════════════
// Java Language Layer: class.root — EXPORT + ABSTRACT + extends + implements
// Expected IR: DEF_C BaseEntity, EXT→Serializable, FLAGS_C EXPORT+ABSTRACT
// Methods: constructor (EXPORT), getId (EXPORT+RET), setId (EXPORT)
// ════════════════════════════════════════════════════════════════════════
public abstract class BaseEntity implements Serializable {

    private Long id;

    public BaseEntity() {
    }

    public Long getId() {
        return id;
    }

    public void setId(Long id) {
        this.id = id;
    }
}

// ════════════════════════════════════════════════════════════════════════
// Java Language Layer: interface.root — extends (generics stripped)
// Expected IR: DEF_C MyRepository, EXT→JpaRepository
// ════════════════════════════════════════════════════════════════════════
interface MyRepository extends JpaRepository<BaseEntity, Long> {
}

// ════════════════════════════════════════════════════════════════════════
// Java Language Layer: enum.root
// Expected IR: DEF_C + FLAGS_C EXPORT
// ════════════════════════════════════════════════════════════════════════
public enum Status {
    ACTIVE, INACTIVE, PENDING
}

// ════════════════════════════════════════════════════════════════════════
// Java Language Layer: record.root
// Expected IR: DEF_C CreateUserRequest
// ════════════════════════════════════════════════════════════════════════
public record CreateUserRequest(String email, String name) {
}

// ════════════════════════════════════════════════════════════════════════
// Java Language Layer: method.root — PRIVATE, PROTECTED, STATIC, ABSTRACT
// Expected IR:
//   doInternal  → FLAGS PRIVATE
//   doHelper    → FLAGS PROTECTED
//   doUtility   → FLAGS STATIC+EXPORT
//   doTemplate  → FLAGS ABSTRACT+EXPORT
// ════════════════════════════════════════════════════════════════════════
abstract class HelperBase {

    private void doInternal() {
    }

    protected void doHelper() {
    }

    public static void doUtility() {
    }

// ════════════════════════════════════════════════════════════════════════
// Spring Meta-Layer: @Controller (Φctrl:)
// Low:    Φctrl:ViewController
// Med:    Φctrl:ViewController map=[GET /view]
// High:   same as Medium
// ════════════════════════════════════════════════════════════════════════
@Controller
@RequestMapping("/view")
public class ViewController {
    @GetMapping
    @ResponseBody
    public String renderView() { return "view"; }
}

// ════════════════════════════════════════════════════════════════════════
// Spring Meta-Layer: @RestController (Φrest:) + all HTTP verb mappings
// Also: @Autowired field, @Value field, @RequestBody, @PathVariable
// Low:    Φrest:UserController map=[/api/users], Φmap:...
// Med:    +GET /{id}, POST, PUT /{id}, DELETE /{id}
// High:   +Φaut:userService, Φval:defaultPageSize
// ════════════════════════════════════════════════════════════════════════
@RestController
@RequestMapping("/api/users")
public class UserController {

    @Autowired
    private UserService userService;

    @Value("${app.default-page-size:20}")
    private int defaultPageSize;

    public UserController() {}

    @GetMapping
    public List<UserDto> getAllUsers() { return userService.findAll(); }

    @GetMapping("/{id}")
    public UserDto getUserById(@PathVariable Long id) { return userService.findById(id); }

    @PostMapping
    public UserDto createUser(@RequestBody CreateUserRequest request) { return userService.create(request); }

    @PutMapping("/{id}")
    public UserDto updateUser(@PathVariable Long id, @RequestBody UserDto request) { return userService.update(id, request); }

    @DeleteMapping("/{id}")
    public void deleteUser(@PathVariable Long id) { userService.delete(id); }
}

// ════════════════════════════════════════════════════════════════════════
// Spring Meta-Layer: @Service (Φsvc:) + constructor @Autowired
// Low/Med: Φsvc:UserService  |  High: +Φaut:userRepository
// ════════════════════════════════════════════════════════════════════════
@Service
public class UserService {
    private final UserRepository userRepository;
    @Autowired
    public UserService(UserRepository userRepository) { this.userRepository = userRepository; }
    public List<UserDto> findAll() { return userRepository.findAll(); }
    public UserDto findById(Long id) { return userRepository.findById(id); }
// ════════════════════════════════════════════════════════════════════════
// Spring Meta-Layer: @ConfigurationProperties (Φprop:) — High only
// ════════════════════════════════════════════════════════════════════════
@ConfigurationProperties(prefix = "app")
public class AppProperties {
    private String name;
    private int version;
    public String getName() { return name; }
    public void setName(String n) { this.name = n; }
    public int getVersion() { return version; }
    public void setVersion(int v) { this.version = v; }
}

// ════════════════════════════════════════════════════════════════════════
// Spring Meta-Layer: @SpringBootApplication + @ComponentScan
// (strong detection signals — triggers Spring path, no specific Φ)
// ════════════════════════════════════════════════════════════════════════
@SpringBootApplication
@ComponentScan(basePackages = "com.example.demo")
public class DemoApplication {
    public static void main(String[] args) {
        SpringApplication.run(DemoApplication.class, args);
    }
}

// ════════════════════════════════════════════════════════════════════════
// Spring Meta-Layer: @Value fields (Φval:) on @Service — High only
// ════════════════════════════════════════════════════════════════════════
@Service
public class ConfigService {
    @Value("${app.name:DefaultApp}")
    private String appName;
    @Value("${app.timeout:30}")
    private int timeout;
    public String getAppName() { return appName; }
    public int getTimeout() { return timeout; }
}

// ════════════════════════════════════════════════════════════════════════
// Spring Meta-Layer: @Qualifier + @Autowired field (Φaut:) — High only
// ════════════════════════════════════════════════════════════════════════
@Service
public class CacheService {
    @Autowired
    @Qualifier("redisCache")
    private CacheManager cacheManager;
    public void clearCache() { cacheManager.clear(); }
}

// ════════════════════════════════════════════════════════════════════════
// Spring Meta-Layer: @PatchMapping (Φmap: PATCH) — Medium+
// ════════════════════════════════════════════════════════════════════════
@RestController
@RequestMapping("/resource")
public class PartialUpdateController {
    @PatchMapping("/{id}")
    public void partialUpdate(@PathVariable Long id, @RequestBody Map<String, Object> changes) {}
}

// ════════════════════════════════════════════════════════════════════════
// Plain Java — NO Spring annotations (no Spring markers emitted)
// ════════════════════════════════════════════════════════════════════════
class UserDto {
    private Long id;
    private String email;
    private String name;
    public UserDto() {}
    public Long getId() { return id; }
    public void setId(Long id) { this.id = id; }
    public String getEmail() { return email; }
    public void setEmail(String e) { this.email = e; }
    public String getName() { return name; }
    public void setName(String n) { this.name = n; }
}

// Helpers
class CacheManager { public void clear() {} }
interface JpaRepository<T, ID> {}
    public UserDto create(CreateUserRequest r) { return userRepository.save(r); }
    public UserDto update(Long id, UserDto r) { return userRepository.update(id, r); }
    public void delete(Long id) { userRepository.delete(id); }
}

// ════════════════════════════════════════════════════════════════════════
// Spring Meta-Layer: @Repository (Φrepo:)
// Always: Φrepo:UserRepository
// ════════════════════════════════════════════════════════════════════════
@Repository
public class UserRepository {
    public List<UserDto> findAll() { return List.of(); }
    public UserDto findById(Long id) { return null; }
    public UserDto save(CreateUserRequest r) { return null; }
    public UserDto update(Long id, UserDto r) { return null; }
    public void delete(Long id) {}
}

// ════════════════════════════════════════════════════════════════════════
// Spring Meta-Layer: @Configuration (Φconf:) + @Bean (Φbean:)
// Low:    Φconf:AppConfig  |  High: +Φbean:userService, Φbean:cacheManager
// ════════════════════════════════════════════════════════════════════════
@Configuration
public class AppConfig {
    @Primary @Bean
    public UserService userService() { return new UserService(); }
    @Bean
    public CacheManager cacheManager() { return new CacheManager(); }
}
    public abstract void doTemplate();
}