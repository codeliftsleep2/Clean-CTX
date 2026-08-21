// src/spring_meta/graph.rs
//
// Spring Boot cross-file dependency graph — Tier 3 of the Meta-Layer.
//
// Builds a graph of Spring Boot controllers, services, repositories,
// and configurations across all files in a workspace, then resolves:
//
// - **DI injection**: `@Autowired private UserService userService` →
//   `UserService@α12` (file alias of the `@Service` class)
// - **REST endpoint mapping**: `@RestController` + `@GetMapping` →
//   endpoint paths and HTTP methods
// - **Layer dependency order**: Controllers → Services → Repositories
//
// # Lifecycle
//
// The graph is built **once per `compress_workspace` call**, in the
// post-compression pass (after all files have been compressed and
// bundled). It is purely in-memory and is discarded after the
// workspace manifest is emitted.

use std::collections::{BTreeSet, HashMap};

use crate::compression::graph_utils::{find_cycles, has_cycle, transitive_dependencies};

/// The kind of Spring Boot class that can be registered in the graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClassKind {
    Controller,
    Service,
    Repository,
    Configuration,
}

/// Metadata for a single Spring Boot class registered in the graph.
#[derive(Debug, Clone)]
pub struct ClassEntry {
    /// The bare class name (e.g. `"UserController"`).
    pub class_name: String,
    /// The file alias (e.g. `"α12"`) from `PathDictionary`.
    pub file_alias: String,
    /// The kind of Spring Boot class.
    pub kind: ClassKind,
    /// REST endpoint mappings (HTTP method + path).
    pub endpoints: Vec<crate::spring_meta::markers::RequestMappingMapping>,
    /// The names of types injected via `@Autowired`.
    /// These are bare class names (e.g. `"UserService"`), resolved
    /// to `Type@αN` by `resolve_inject_type`.
    pub injects: Vec<String>,
}

/// Mutable builder for a `SpringGraph`.
///
/// This is the only type that exposes `register_class`. The
/// [`SpringGraphBuilder::build`] method consumes `self` and returns
/// a fully-resolved [`SpringGraph`], making it a type error to
/// register a class after resolution.
#[derive(Debug, Default, Clone)]
pub struct SpringGraphBuilder {
    /// className → ClassEntry (all registered Spring classes).
    classes: HashMap<String, ClassEntry>,
    /// endpoint path → className (for REST endpoint lookup).
    endpoints: HashMap<String, String>,
}

impl SpringGraphBuilder {
    /// Create a new empty graph builder.
    pub fn new() -> Self {
        Self {
            classes: HashMap::new(),
            endpoints: HashMap::new(),
        }
    }

    /// Register a class in the graph. If the class name already exists
    /// (possible with duplicate class names in different files), the
    /// last registration wins.
    pub fn register_class(
        &mut self,
        class_name: &str,
        file_alias: &str,
        kind: ClassKind,
        endpoints: &[crate::spring_meta::markers::RequestMappingMapping],
        injects: &[String],
    ) {
        let entry = ClassEntry {
            class_name: class_name.to_string(),
            file_alias: file_alias.to_string(),
            kind,
            endpoints: endpoints.to_vec(),
            injects: injects.to_vec(),
        };

        // Register the class.
        self.classes.insert(class_name.to_string(), entry);

        // Register endpoints for controller.
        if let ClassKind::Controller = kind {
            for endpoint in endpoints {
                self.endpoints
                    .insert(endpoint.path.clone(), class_name.to_string());
            }
        }
    }

    /// Consume the builder, resolve all cross-file edges, and return
    /// the immutable [`SpringGraph`].
    pub fn build(self) -> SpringGraph {
        // Build injected_by reverse edges.
        let mut injected_by: HashMap<String, BTreeSet<String>> = HashMap::new();
        for entry in self.classes.values() {
            for injected_type_name in &entry.injects {
                injected_by
                    .entry(injected_type_name.clone())
                    .or_default()
                    .insert(entry.class_name.clone());
            }
        }

        SpringGraph {
            classes: self.classes,
            endpoints: self.endpoints,
            injected_by,
        }
    }
}

/// The cross-file Spring Boot dependency graph (resolved form).
///
/// # How to obtain one
///
/// `SpringGraph` can only be constructed via [`SpringGraphBuilder::build`].
///
/// # Querying
///
/// Once you have a `SpringGraph`, use:
/// - [`resolve_inject_type`](Self::resolve_inject_type) — bare class
///   name → `"ClassName@αN"`
/// - [`resolve_endpoint`](Self::resolve_endpoint) — path string
///   → `"ClassName@αN"`
/// - [`format_graph_line`](Self::format_graph_line) / [`format_graph_footer`](Self::format_graph_footer)
///   — formatted `Φgraph:` / `§ΦGRAPH` output
/// - [`class_names_by_kind`](Self::class_names_by_kind),
///   [`get_class`](Self::get_class), [`all_classes`](Self::all_classes)
///   — direct accessors
#[derive(Debug, Clone)]
pub struct SpringGraph {
    /// className → ClassEntry (all registered Spring classes).
    classes: HashMap<String, ClassEntry>,
    /// endpoint path → className (for REST endpoint lookup).
    endpoints: HashMap<String, String>,
    /// className → set of classNames that inject this class.
    /// Built by [`SpringGraphBuilder::build`].
    injected_by: HashMap<String, BTreeSet<String>>,
}

impl SpringGraph {
    /// Resolve an injected type name to its file-aliased form.
    ///
    /// Given a bare class name like `"UserService"`, returns
    /// `Some("UserService@α12")` if the class is registered in the
    /// graph. Returns `None` if the type is not found (unresolved or
    /// external dependency).
    pub fn resolve_inject_type(&self, type_name: &str) -> Option<String> {
        self.classes
            .get(type_name)
            .map(|entry| format!("{}@{}", entry.class_name, entry.file_alias))
    }

    /// Resolve a REST endpoint path to its controller file-aliased
    /// form.
    ///
    /// Given a path like `"/api/users"`, returns
    /// `Some("UserController@α9")` if a controller with that endpoint
    /// is registered. Returns `None` if the path is not found.
    pub fn resolve_endpoint(&self, path: &str) -> Option<String> {
        self.endpoints
            .get(path)
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

    /// Iterate over all registered classes in insertion order.
    pub fn all_classes(&self) -> Vec<&ClassEntry> {
        self.classes.values().collect()
    }

    /// Format the full `§ΦGRAPH` footer section for the workspace
    /// manifest. Returns an empty string if the graph has no entries.
    pub fn format_graph_footer(&self) -> String {
        if self.is_empty() {
            return String::new();
        }

        let mut s = String::from("\n§ΦGRAPH\n");
        s.push_str("// Cross-file Spring Boot dependency graph\n");

        for entry in self.all_classes() {
            let kind_str = match entry.kind {
                ClassKind::Controller => "ctrl",
                ClassKind::Service => "svc",
                ClassKind::Repository => "repo",
                ClassKind::Configuration => "conf",
            };
            s.push_str(&format!(
                "  {} {}@{}\n",
                kind_str, entry.class_name, entry.file_alias
            ));

            // Endpoints (controller only).
            if !entry.endpoints.is_empty() {
                s.push_str("    endpoints=[");
                let endpoint_strs: Vec<String> = entry
                    .endpoints
                    .iter()
                    .map(|e| {
                        if let Some(ref m) = e.method {
                            format!("{} {}", m, e.path)
                        } else {
                            e.path.clone()
                        }
                    })
                    .collect();
                s.push_str(&endpoint_strs.join(","));
                s.push_str("]\n");
            }

            // Dependency kind marker.
            let dep_marker = match entry.kind {
                ClassKind::Controller => "Φctrl:",
                ClassKind::Service => "Φsvc:",
                ClassKind::Repository => "Φrepo:",
                ClassKind::Configuration => "Φconf:",
            };
            s.push_str(&format!("    {}injects=[", dep_marker));
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
        }

        s
    }

    /// Check if the graph contains a cycle using DFS.
    ///
    /// Returns `true` if at least one cycle is detected.
    /// Uses three-color DFS (white/gray/black) for O(V+E) performance.
    pub fn has_cycle(&self) -> bool {
        let class_names: Vec<String> = self.classes.keys().cloned().collect();
        let name_to_idx: HashMap<&str, usize> = class_names
            .iter()
            .enumerate()
            .map(|(i, name)| (name.as_str(), i))
            .collect();

        let node_count = class_names.len();
        if node_count == 0 {
            return false;
        }

        let adj_fn = |i: usize| {
            let name = class_names.get(i).map(|s| s.as_str()).unwrap_or("");
            if let Some(entry) = self.classes.get(name) {
                entry
                    .injects
                    .iter()
                    .filter_map(|injected| name_to_idx.get(injected.as_str()))
                    .copied()
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        has_cycle(node_count, adj_fn)
    }

    /// Find all cycles in the graph using DFS.
    ///
    /// Returns a list of cycles, each represented as a path of node IDs.
    /// If no cycles exist, returns an empty `Vec`.
    pub fn find_cycles(&self) -> Vec<Vec<String>> {
        let class_names: Vec<String> = self.classes.keys().cloned().collect();
        let name_to_idx: HashMap<&str, usize> = class_names
            .iter()
            .enumerate()
            .map(|(i, name)| (name.as_str(), i))
            .collect();

        let node_count = class_names.len();
        if node_count == 0 {
            return Vec::new();
        }

        let adj_fn = |i: usize| {
            let name = class_names.get(i).map(|s| s.as_str()).unwrap_or("");
            if let Some(entry) = self.classes.get(name) {
                entry
                    .injects
                    .iter()
                    .filter_map(|injected| name_to_idx.get(injected.as_str()))
                    .copied()
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        let label_fn = |i: usize| {
            class_names
                .get(i)
                .cloned()
                .unwrap_or_else(|| format!("unknown_{}", i))
        };

        find_cycles(node_count, adj_fn, label_fn)
    }

    /// Compute transitive dependencies for a node up to a given depth.
    ///
    /// Returns all reachable node IDs by following inject edges outward from `class_name`.
    /// - `depth=1` → direct dependencies only
    /// - `depth=2` → dependencies of dependencies
    /// - `depth=0` or negative → all transitive dependencies (BFS to completion)
    pub fn transitive_dependencies(&self, class_name: &str, depth: i32) -> Vec<String> {
        if !self.classes.contains_key(class_name) {
            return Vec::new();
        }

        let class_names: Vec<String> = self.classes.keys().cloned().collect();
        let name_to_idx: HashMap<&str, usize> = class_names
            .iter()
            .enumerate()
            .map(|(i, name)| (name.as_str(), i))
            .collect();

        let start_idx = name_to_idx[class_name];
        let node_count = class_names.len();

        let adj_fn = |i: usize| {
            let name = class_names.get(i).map(|s| s.as_str()).unwrap_or("");
            if let Some(entry) = self.classes.get(name) {
                entry
                    .injects
                    .iter()
                    .filter_map(|injected| name_to_idx.get(injected.as_str()))
                    .copied()
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        let indices = transitive_dependencies(start_idx, depth, node_count, adj_fn);
        indices
            .into_iter()
            .filter_map(|i| class_names.get(i))
            .cloned()
            .collect()
    }

    /// Check if the graph is empty (no Spring classes registered).
    pub fn is_empty(&self) -> bool {
        self.classes.is_empty()
    }
}

#[cfg(test)]
#[path = "../tests/spring_meta/graph.rs"]
mod tests;

/// Helper type for building a batch of graph entries during the
/// per-file compression pass.
#[derive(Debug, Default)]
pub struct GraphCollector {
    /// List of (class_name, file_alias, kind, endpoints, injects)
    pub entries: Vec<GraphEntry>,
}

/// A single graph entry collected during compression.
#[derive(Debug, Clone)]
pub struct GraphEntry {
    pub class_name: String,
    pub file_alias: String,
    pub kind: ClassKind,
    pub endpoints: Vec<crate::spring_meta::markers::RequestMappingMapping>,
    pub injects: Vec<String>,
}

impl GraphCollector {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Register a class for later graph building.
    pub fn push(
        &mut self,
        class_name: &str,
        file_alias: &str,
        kind: ClassKind,
        endpoints: &[crate::spring_meta::markers::RequestMappingMapping],
        injects: &[String],
    ) {
        self.entries.push(GraphEntry {
            class_name: class_name.to_string(),
            file_alias: file_alias.to_string(),
            kind,
            endpoints: endpoints.to_vec(),
            injects: injects.to_vec(),
        });
    }

    /// Flush all collected entries into a `SpringGraph`.
    pub fn build_graph(&self) -> SpringGraph {
        let mut builder = SpringGraphBuilder::new();
        for entry in &self.entries {
            builder.register_class(
                &entry.class_name,
                &entry.file_alias,
                entry.kind,
                &entry.endpoints,
                &entry.injects,
            );
        }
        builder.build()
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
