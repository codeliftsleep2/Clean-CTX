// src/spring_meta/detect.rs
//
// Spring Boot detection heuristic.
//
// The Meta-Layer must run **only** on Spring Boot files. Plain Java
// files should pay **zero** overhead — no Φ markers, no extra parse,
// no newlines.
//
// # Strategy
//
// We do not re-parse the AST. We scan the raw source for the
// annotation names that are unique to Spring Boot:
//
//   @RestController, @Controller, @Service, @Repository,
//   @Configuration, @RequestMapping, @GetMapping, @PostMapping, etc.
//
// A single occurrence of `@RestController(` or `@SpringBootApplication(`
// is a strong enough signal.
//
// The detection is O(n) over the source length and never allocates
// more than a single `BTreeSet` of the matches it found.

/// Spring-specific annotation names that we treat as a strong signal
/// of a Spring Boot file. A single match anywhere in the source is enough
/// to consider the file Spring.
const STRONG_ANNOTATIONS: &[&str] = &[
    "@RestController",
    "@Controller",
    "@Service",
    "@Repository",
    "@Configuration",
    "@SpringBootApplication",
    "@EnableAutoConfiguration",
    "@ComponentScan",
    "@RequestMapping",
    "@GetMapping",
    "@PostMapping",
    "@PutMapping",
    "@DeleteMapping",
    "@PatchMapping",
];

/// Spring-specific annotation names that are NOT unique to Spring
/// (e.g. `@Autowired` is also used in plain Spring, `@Value` can be
/// used in non-Spring contexts). These count as weak signals and must
/// be paired with a strong signal to trigger Meta-Layer output.
const WEAK_ANNOTATIONS: &[&str] = &[
    "@Autowired",
    "@Value(",
    "@ConfigurationProperties",
    "@Bean",
    "@Primary",
    "@Qualifier",
];

/// Detects the presence of the Spring `org.springframework` import.
/// Almost every Spring Boot file imports something from
/// `org.springframework`.
const SPRING_IMPORT: &str = "org.springframework";

/// Decide whether the given source code is from a Spring Boot file.
///
/// A file is "Spring Boot" iff:
/// 1. It contains at least one **strong** annotation (`@RestController`,
///    `@Service`, `@Repository`, `@Configuration`, `@RequestMapping`, etc.), OR
/// 2. It imports from `org.springframework` AND has at least one
///    weak annotation (`@Autowired`, `@Value`, `@Bean`, etc.).
///
/// Plain `@Autowired` alone (no strong signal, no Spring import) returns
/// `false` — that annotation is also used in plain Spring Framework
/// contexts, and a false positive would inject meaningless `Φ` markers
/// into non-Spring output.
pub fn is_spring_file(source: &str) -> bool {
    // Tier 1: any strong annotation?
    for anno in STRONG_ANNOTATIONS {
        if source.contains(anno) {
            return true;
        }
    }

    // Tier 2: Spring import + weak annotation pair?
    if source.contains(SPRING_IMPORT) {
        for weak in WEAK_ANNOTATIONS {
            if source.contains(weak) {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
#[path = "../tests/spring_meta/detect_tests.rs"]
mod tests;
