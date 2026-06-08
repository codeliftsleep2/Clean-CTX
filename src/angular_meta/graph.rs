// src/angular_meta/graph.rs
//
// Angular cross-file dependency graph — Tier 3 of the Meta-Layer.
//
// Builds a graph of Angular services, components, directives, and
// modules across all files in a workspace, then resolves:
//
// - **DI injection**: `constructor(private userSvc: UserService)` →
//   `UserService@α12` (file alias of the `@Injectable` class)
// - **Selector linkage**: `<app-user-card>` →
//   `UserCard@α9` (file alias of the `@Component({selector: 'app-user-card'})`)
// - **Dependency order**: services first, then components, then modules
//
// # Lifecycle
//
// The graph is built **once per `compress_workspace` call**, in the
// post-compression pass (after all files have been compressed and
// bundled). It is purely in-memory and is discarded after the
// workspace manifest is emitted.
//
// # Non-goals
//
// - Hot-reload: the graph is rebuilt per workspace call
// - Persistence: no disk cache
// - Non-DI dependencies (utility imports): out of scope

use std::collections::{BTreeSet, HashMap};

/// The kind of Angular class that can be registered in the graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClassKind {
    Component,
    Service,
    Directive,
    Pipe,
    Module,
}

/// Metadata for a single Angular class registered in the graph.
#[derive(Debug, Clone)]
pub struct ClassEntry {
    /// The bare class name (e.g. `"UserCardComponent"`).
    pub class_name: String,
    /// The file alias (e.g. `"α12"`) from `PathDictionary`.
    pub file_alias: String,
    /// The kind of Angular class.
    pub kind: ClassKind,
    /// The CSS selector, if this is a `@Component` or `@Directive`.
    pub selector: Option<String>,
    /// The names of types injected via constructor DI.
    /// These are bare class names (e.g. `"UserService"`), resolved
    /// to `Type@αN` by `resolve_inject_type`.
    pub injects: Vec<String>,
    /// The pipe name, if this is a `@Pipe`.
    pub pipe_name: Option<String>,
}

/// The cross-file Angular dependency graph.
///
/// # Graph building order
///
/// 1. **Register** classes: call `register_class` for each Angular
///    class found during the per-file compression pass.
/// 2. **Build edges**: call `resolve_all()` after all files have been
///    registered. This resolves DI types to file aliases and builds
///    the reverse `injected_by` edges.
/// 3. **Query**: use `resolve_inject_type`, `resolve_selector`,
///    `format_graph_line` to emit the enriched markers.
///
/// # Thread safety
///
/// `AngularGraph` is wrapped in `Arc<Mutex<…>>` in `McpState` so it
/// can be shared across the single-threaded MCP dispatch chain.
#[derive(Debug, Clone)]
pub struct AngularGraph {
    /// className → ClassEntry (all registered Angular classes).
    classes: HashMap<String, ClassEntry>,
    /// selector → className (for component/directive selector lookup).
    selectors: HashMap<String, String>,
    /// className → set of classNames that inject this class.
    /// Built by `resolve_all()`.
    injected_by: HashMap<String, BTreeSet<String>>,
    /// Whether graph has been resolved.
    resolved: bool,
}

impl Default for AngularGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl AngularGraph {
    /// Create a new empty graph.
    pub fn new() -> Self {
        Self {
            classes: HashMap::new(),
            selectors: HashMap::new(),
            injected_by: HashMap::new(),
            resolved: false,
        }
    }

    /// Register a class in the graph. If the class name already exists
    /// (possible with duplicate class names in different files), the
    /// last registration wins and a warning is logged via `eprintln!`
    /// (F-ANG-17). Two classes with the same name in different files
    /// usually indicates a workspace misconfiguration — callers should
    /// rename one of the classes.
    pub fn register_class(
        &mut self,
        class_name: &str,
        file_alias: &str,
        kind: ClassKind,
        selector: Option<&str>,
        injects: &[String],
        pipe_name: Option<&str>,
    ) {
        if let Some(prev) = self.classes.get(class_name) {
            eprintln!(
                "[clean-ctx] WARN: AngularGraph: duplicate class name '{}' (prev alias={}, new alias={}); last-write-wins",
                class_name, prev.file_alias, file_alias
            );
        }

        let entry = ClassEntry {
            class_name: class_name.to_string(),
            file_alias: file_alias.to_string(),
            kind,
            selector: selector.map(|s| s.to_string()),
            injects: injects.to_vec(),
            pipe_name: pipe_name.map(|s| s.to_string()),
        };

        // Register the class.
        self.classes.insert(class_name.to_string(), entry);

        // Register selector for component/directive.
        if let Some(sel) = selector {
            self.selectors
                .insert(sel.to_string(), class_name.to_string());
        }

        self.resolved = false;
    }

    /// Resolve all cross-file edges. Must be called after all classes
    /// have been registered (after the per-file compression pass).
    ///
    /// This builds the `injected_by` reverse map:
    /// if class A injects class B, then `injected_by[B].insert(A)`.
    pub fn resolve_all(&mut self) {
        self.injected_by.clear();

        for entry in self.classes.values() {
            for injected_type_name in &entry.injects {
                self.injected_by
                    .entry(injected_type_name.clone())
                    .or_default()
                    .insert(entry.class_name.clone());
            }
        }

        self.resolved = true;
    }

    /// Check whether the graph has been resolved.
    pub fn is_resolved(&self) -> bool {
        self.resolved
    }

    /// Get the number of registered classes.
    pub fn len(&self) -> usize {
        self.classes.len()
    }

    /// Check if the graph is empty (no Angular classes registered).
    pub fn is_empty(&self) -> bool {
        self.classes.is_empty()
    }

    /// Resolve an injected type name to its file-aliased form.
    ///
    /// Given a bare class name like `"UserService"`, returns
    /// `Some("UserService@α12")` if the class is registered in the
    /// graph. Returns `None` if the type is not found (unresolved or
    /// external dependency).
    pub fn resolve_inject_type(&self, type_name: &str) -> Option<String> {
        if !self.resolved {
            return None;
        }
        self.classes.get(type_name).map(|entry| {
            format!("{}@{}", entry.class_name, entry.file_alias)
        })
    }

    /// Resolve a custom-element tag name to its component file-aliased
    /// form.
    ///
    /// Given a selector like `"app-user-card"`, returns
    /// `Some("UserCardComponent@α9")` if a component with that
    /// selector is registered. Returns `None` if the selector is not
    /// found.
    pub fn resolve_selector(&self, selector: &str) -> Option<String> {
        if !self.resolved {
            return None;
        }
        self.selectors
            .get(selector)
            .and_then(|class_name| self.classes.get(class_name))
            .map(|entry| format!("{}@{}", entry.class_name, entry.file_alias))
    }

    /// Build the `Φgraph:<ClassName> → injects=[…] ← injected-by=[…]`
    /// marker line for a given class name.
    ///
    /// Returns `None` if the class is not registered in the graph.
    pub fn format_graph_line(&self, class_name: &str) -> Option<String> {
        let entry = self.classes.get(class_name)?;

        let injects_str = if entry.injects.is_empty() {
            String::from("[]")
        } else {
            let resolved: Vec<String> = entry
                .injects
                .iter()
                .map(|t| {
                    self.resolve_inject_type(t)
                        .unwrap_or_else(|| format!("{}?", t))
                })
                .collect();
            format!("[{}]", resolved.join(","))
        };

        let injected_by_str = if let Some(injectors) = self.injected_by.get(class_name) {
            if injectors.is_empty() {
                String::from("[]")
            } else {
                let resolved: Vec<String> = injectors
                    .iter()
                    .map(|c| {
                        self.resolve_inject_type(c)
                            .unwrap_or_else(|| format!("{}?", c))
                    })
                    .collect();
                format!("[{}]", resolved.join(","))
            }
        } else {
            String::from("[]")
        };

        Some(format!(
            "Φgraph:{} → injects={} ← injected-by={}",
            class_name, injects_str, injected_by_str
        ))
    }

    /// Get all registered class names, optionally filtered by kind.
    pub fn class_names_by_kind(&self, kind: ClassKind) -> Vec<String> {
        self.classes
            .values()
            .filter(|e| e.kind == kind)
            .map(|e| e.class_name.clone())
            .collect()
    }

    /// Get a class entry by name.
    pub fn get_class(&self, class_name: &str) -> Option<&ClassEntry> {
        self.classes.get(class_name)
    }

    /// Iterate over all registered classes in insertion order
    /// (deterministic by BTreeMap key for stability).
    pub fn all_classes(&self) -> Vec<&ClassEntry> {
        let mut sorted: Vec<&ClassEntry> = self.classes.values().collect();
        sorted.sort_by(|a, b| a.class_name.cmp(&b.class_name));
        sorted
    }

    /// Format the full `§ΦGRAPH` footer section for the workspace
    /// manifest. Returns an empty string if the graph has no entries
    /// or has not been resolved.
    pub fn format_graph_footer(&self) -> String {
        if !self.resolved || self.is_empty() {
            return String::new();
        }

        let mut s = String::from("\n§ΦGRAPH\n");
        s.push_str("// Cross-file Angular dependency graph\n");

        for entry in self.all_classes() {
            let kind_str = match entry.kind {
                ClassKind::Component => "cmp",
                ClassKind::Service => "svc",
                ClassKind::Directive => "dir",
                ClassKind::Pipe => "pipe",
                ClassKind::Module => "mod",
            };
            s.push_str(&format!(
                "  {} {}@{}\n",
                kind_str, entry.class_name, entry.file_alias
            ));

            // Dependency kind marker.
            let dep_marker = match entry.kind {
                ClassKind::Component => "Φcmp:",
                ClassKind::Service => "Φsvc:",
                ClassKind::Directive => "Φdir:",
                ClassKind::Pipe => "Φpipe:",
                ClassKind::Module => "Φmod:",
            };
            s.push_str(&format!(
                "    {}injects=[",
                dep_marker
            ));
            let resolved_injects: Vec<String> = entry
                .injects
                .iter()
                .map(|t| {
                    self.resolve_inject_type(t)
                        .unwrap_or_else(|| format!("{}?", t))
                })
                .collect();
            s.push_str(&resolved_injects.join(","));
            s.push_str("]\n");

            // Injected-by (reverse edges).
            if let Some(injectors) = self.injected_by.get(&entry.class_name) {
                if !injectors.is_empty() {
                    let resolved: Vec<String> = injectors
                        .iter()
                        .map(|c| {
                            self.resolve_inject_type(c)
                                .unwrap_or_else(|| format!("{}?", c))
                        })
                        .collect();
                    s.push_str(&format!("    ← injected-by=[{}]\n", resolved.join(",")));
                }
            }

            // Selector linkage (component/directive only).
            if let Some(sel) = &entry.selector {
                s.push_str(&format!("    selector=\"{}\"\n", sel));
            }
        }

        s
    }
}

/// Helper type for building a batch of graph entries during the
/// per-file compression pass.
///
/// The `GraphCollector` is populated during each `compress_file` call
/// and flushed into the `AngularGraph` during the post-compression
/// bundling pass.
#[derive(Debug, Default)]
pub struct GraphCollector {
    /// List of (class_name, file_alias, kind, selector, injects, pipe_name)
    pub entries: Vec<GraphEntry>,
}

/// A single graph entry collected during compression.
#[derive(Debug, Clone)]
pub struct GraphEntry {
    pub class_name: String,
    pub file_alias: String,
    pub kind: ClassKind,
    pub selector: Option<String>,
    pub injects: Vec<String>,
    pub pipe_name: Option<String>,
}

impl GraphCollector {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    /// Register a class for later graph building.
    pub fn push(
        &mut self,
        class_name: &str,
        file_alias: &str,
        kind: ClassKind,
        selector: Option<&str>,
        injects: &[String],
        pipe_name: Option<&str>,
    ) {
        self.entries.push(GraphEntry {
            class_name: class_name.to_string(),
            file_alias: file_alias.to_string(),
            kind,
            selector: selector.map(|s| s.to_string()),
            injects: injects.to_vec(),
            pipe_name: pipe_name.map(|s| s.to_string()),
        });
    }

    /// Flush all collected entries into an `AngularGraph`.
    pub fn build_graph(&self) -> AngularGraph {
        let mut graph = AngularGraph::new();
        for entry in &self.entries {
            graph.register_class(
                &entry.class_name,
                &entry.file_alias,
                entry.kind,
                entry.selector.as_deref(),
                &entry.injects,
                entry.pipe_name.as_deref(),
            );
        }
        graph.resolve_all();
        graph
    }

    /// Check if any entries have been collected.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get the number of collected entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

#[cfg(test)]
#[path = "../tests/angular_meta/graph.rs"]
mod tests;

#[cfg(test)]
#[path = "../tests/angular_meta/di_resolution.rs"]
mod di_tests;

#[cfg(test)]
#[path = "../tests/angular_meta/selector_linkage.rs"]
mod selector_tests;