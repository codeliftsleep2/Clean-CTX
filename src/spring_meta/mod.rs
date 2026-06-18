// src/spring_meta/mod.rs
//
// Spring Boot Meta-Layer — Tier 1 (Annotations) + Tier 2 (Layer Bundling).
//
// The Meta-Layer is **purely additive**: it never modifies the existing
// Java compaction output. It only appends a `Φ` block below the existing
// compacted class entry. Existing users see no change; Spring Boot users
// get enriched output with `@RestController` / `@Service` / `@Repository`
// / `@RequestMapping` context.
//
// # Module structure
//
// - `detect`     : Spring Boot detection heuristic
// - `annotations`: `@RestController` / `@Service` / `@Repository` /
//                  `@Controller` / `@Configuration` / `@RequestMapping` extractor
// - `markers`    : `Φ` marker construction & expansion
// - `bundler`    : layer resolver (Controller → Service → Repository)
// - `properties` : application.properties / application.yml extractor
// - `footer`     : `§ΦMAP` workspace footer formatter
// - `graph`      : cross-file dependency graph (DI, REST endpoints)
// - `graph_state`: McpState integration
// - (this file)  : Public surface, `MetaBlock` struct, `run_meta_layer`

pub(crate) mod annotations;
pub(crate) mod detect;
pub(crate) mod markers;
pub mod bundler;
pub mod properties;
pub mod footer;
pub mod graph;
pub mod graph_state;

use crate::compression::Fidelity;

/// The Meta-Layer output for a single Java file.
///
/// `None` means "not a Spring Boot file" — the caller should not emit any
/// Φ block at all (zero overhead, byte-identical to non-Spring output).
///
/// `Some(block)` means "Spring Boot file" — the caller should append the
/// Φ block lines below the existing compacted class entry.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MetaBlock {
    /// One `Φ` line per Spring-bearing class, in document order.
    /// Each line is the fully-formatted marker line (e.g. `Φrest:UserController map=[GET /api/users]`).
    /// Already newline-separated; caller is responsible for the
    /// surrounding `// --- Φ Spring Boot Meta ---` header.
    pub lines: Vec<String>,
}

impl MetaBlock {
    /// Returns `true` if there are no Φ lines to emit (i.e. the
    /// caller should skip the entire `// --- Φ Spring Boot Meta ---` block).
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// Render the full Φ block, including the header. Returns an
    /// empty string when the block is empty (so callers can `+=`
    /// blindly).
    pub fn render(&self) -> String {
        if self.is_empty() {
            return String::new();
        }
        let mut s = String::new();
        s.push_str("// --- Φ Spring Boot Meta ---\n");
        for line in &self.lines {
            s.push_str(line);
            s.push('\n');
        }
        s
    }
}

/// Run the Spring Boot Meta-Layer pass on a single Java file's raw source
/// text. Returns `None` when the file is not a Spring Boot file (no
/// Spring annotations present), and `Some(MetaBlock)` when at least one
/// class carries a recognised Spring annotation.
///
/// The function is deliberately **string-based** for Phase 1 — it
/// does not re-parse the AST. The Java capture pipeline has already
/// given us `class.root` / `interface.root` / `record.root` captures;
/// we walk the raw text of each capture looking for annotations that
/// immediately precede it. This is the same strategy used by
/// `angular_meta::decorators` and is sufficient for the Phase 1
/// deliverable (no new dependencies, no AST changes).
///
/// # Arguments
///
/// - `source_code`    : the full source text of the file being compressed
/// - `class_captures` : the slice texts of each class/interface/record capture
///   (in document order, already sorted by `run_capture_pipeline`)
/// - `fidelity`       : fidelity level (controls verbosity):
///   - `Fidelity::Low`    → emit only class-level summaries (`@RestController`,
///     `@Service`, `@Repository`, `@Controller`). No field-level `@Autowired`,
///     no `@RequestMapping` details.
///   - `Fidelity::Medium` → add `@RequestMapping` method mappings; skip
///     field-level `@Autowired`.
///   - `Fidelity::High`   → emit everything including field-level `@Autowired`
///     and `@Value` / `@ConfigurationProperties` markers.
pub fn run_meta_layer(
    source_code: &str,
    class_captures: &[String],
    fidelity: Fidelity,
) -> Option<MetaBlock> {
    // Tier 0 (detection): is this a Spring Boot file at all?
    if !detect::is_spring_file(source_code) {
        return None;
    }

    // Tier 1 (extraction): walk each class capture and emit Φ lines.
    let mut block = MetaBlock::default();
    for raw_class in class_captures {
        if let Some(result) = annotations::extract_annotations(raw_class, fidelity) {
            block.lines.extend(result.lines);
        }
    }

    if block.is_empty() {
        // Spring Boot file but no Spring annotations on any class. Be
        // conservative — do not emit a Φ block header.
        return None;
    }

    Some(block)
}

#[cfg(test)]
#[path = "../tests/spring_meta/mod.rs"]
mod tests;