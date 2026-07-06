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
// # Typestate (Track B, F-ANG-05)
//
// `AngularGraph` is the **resolved** form — once you have one, the
// `injected_by` reverse edges are guaranteed to be in sync with the
// class registrations. There is no public way to mutate it.
//
// To *build* a graph, use [`AngularGraphBuilder`]. The builder owns
// the mutable `classes` / `selectors` maps and exposes `register_class`.
// Calling [`AngularGraphBuilder::build`] consumes the builder by value,
// runs the `resolve_all` pass internally, and returns the immutable
// `AngularGraph`. After that point the `register_class` method is
// simply not available — registering after resolution is a type error.
//
// # Non-goals
//
// - Hot-reload: the graph is rebuilt per workspace call
// - Persistence: no disk cache
// - Non-DI dependencies (utility imports): out of scope

use std::collections::{BTreeSet, HashMap};

use crate::compression::graph_utils::{find_cycles, has_cycle, transitive_dependencies};

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

/// Mutable builder for an `AngularGraph`.
///
/// This is the only type that exposes `register_class`. The
/// [`AngularGraphBuilder::build`] method consumes `self` and returns
/// a fully-resolved [`AngularGraph`], making it a type error to
/// register a class after resolution (Track B, F-ANG-05).
///
/// # Example
///
/// ```ignore
/// use crate::angular_meta::graph::{AngularGraphBuilder, ClassKind};
///
/// let mut builder = AngularGraphBuilder::new();
/// builder.register_class("UserService", "α1", ClassKind::Service, None, &[], None);
/// let graph = builder.build(); // moves self
/// // builder is now gone; graph.is_resolved() == true
/// ```
#[derive(Debug, Default, Clone)]
pub struct AngularGraphBuilder {
    /// className → ClassEntry (all registered Angular classes).
    classes: HashMap<String, ClassEntry>,
    /// selector → className (for component/directive selector lookup).
    selectors: HashMap<String, String>,
    /// F-FINAL-06: Non-fatal warnings collected during
    /// `register_class` (currently: duplicate class name).
    /// Propagated to the `AngularGraph` by `build()`.
    pub(crate) warnings: Vec<String>,
}

impl AngularGraphBuilder {
    /// Create a new empty graph builder.
    pub fn new() -> Self {
        Self {
            classes: HashMap::new(),
            selectors: HashMap::new(),
            warnings: Vec::new(),
        }
    }

    /// Register a class in the graph. If the class name already exists
    /// (possible with duplicate class names in different files), the
    /// last registration wins and a warning is recorded (F-ANG-17).
    /// Two classes with the same name in different files usually
    /// indicates a workspace misconfiguration — callers should rename
    /// one of the classes.
    ///
    /// F-FINAL-06: the warning is now collected into
    /// `self.warnings` (a `Vec<String>`) instead of being printed to
    /// stderr via `eprintln!`. The MCP workspace pass drains these
    /// warnings into `WorkspaceResult.warnings` so the JSON-RPC
    /// `_warnings` field surfaces them.
    ///
    /// This method is only available on the builder. Once the builder
    /// is consumed by [`build`](Self::build), the resulting
    /// `AngularGraph` has no `register_class` method — registering
    /// after resolution is a type error (Track B, F-ANG-05).
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
            self.warnings.push(format!(
                "AngularGraph: duplicate class name '{}' (prev alias={}, new alias={}); last-write-wins",
                class_name, prev.file_alias, file_alias
            ));
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
    }

    /// Consume the builder, resolve all cross-file edges, and return
    /// the immutable [`AngularGraph`].
    ///
    /// This takes `self` by value, so calling `register_class` after
    /// `build` is a compile error — the builder is gone.
    pub fn build(self) -> AngularGraph {
        let mut injected_by: HashMap<String, BTreeSet<String>> = HashMap::new();

        for entry in self.classes.values() {
            for injected_type_name in &entry.injects {
                injected_by
                    .entry(injected_type_name.clone())
                    .or_default()
                    .insert(entry.class_name.clone());
            }
        }

        AngularGraph {
            classes: self.classes,
            selectors: self.selectors,
            injected_by,
            // F-FINAL-06: propagate the builder's warnings into the
            // resolved graph so the workspace pass can drain them
            // into `WorkspaceResult.warnings` (and ultimately the
            // JSON-RPC `_warnings` field).
            warnings: self.warnings,
            // Builder-side invariant: every `AngularGraph` produced by
            // `build()` is resolved. The flag is kept (privately) so
            // query methods can assert the invariant without forcing
            // them to handle an unreachable "unresolved" branch.
            resolved: true,
        }
    }
}

/// The cross-file Angular dependency graph (resolved form).
///
/// # How to obtain one
///
/// `AngularGraph` can only be constructed via [`AngularGraphBuilder::build`].
/// Direct construction is `pub(crate)`-only — callers in the `mcp`
/// crate go through [`GraphCollector::build_graph`], which itself
/// uses the builder internally.
///
/// # Querying
///
/// Once you have an `AngularGraph`, use:
/// - [`resolve_inject_type`](Self::resolve_inject_type) — bare class
///   name → `"ClassName@αN"`
/// - [`resolve_selector`](Self::resolve_selector) — selector string
///   → `"ClassName@αN"`
/// - [`format_graph_line`](Self::format_graph_line) / [`format_graph_footer`](Self::format_graph_footer)
///   — formatted `Φgraph:` / `§ΦGRAPH` output
/// - [`class_names_by_kind`](Self::class_names_by_kind),
///   [`get_class`](Self::get_class), [`all_classes`](Self::all_classes)
///   — direct accessors
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
    /// Built by [`AngularGraphBuilder::build`].
    injected_by: HashMap<String, BTreeSet<String>>,
    /// F-FINAL-06: Non-fatal warnings propagated from the builder
    /// (e.g. duplicate class names). Drained by callers via
    /// [`take_warnings`] when surfacing via the JSON-RPC
    /// `_warnings` field.
    pub(crate) warnings: Vec<String>,
    /// Whether graph has been resolved. Always `true` for graphs
    /// produced by [`AngularGraphBuilder::build`] (the public
    /// construction path). Kept for `is_resolved` symmetry and to
    /// allow `pub(crate)` direct construction in tests.
    resolved: bool,
}

impl AngularGraph {
    /// Check whether the graph has been resolved.
    ///
    /// For graphs produced by [`AngularGraphBuilder::build`] (the
    /// public construction path) this is always `true`. Kept for
    /// symmetry and for `pub(crate)` direct construction paths.
    pub fn is_resolved(&self) -> bool {
        self.resolved
    }

    /// F-FINAL-06: Drain the warnings collected during graph
    /// construction (e.g. duplicate class names). Returns a `Vec`
    /// that the caller embeds in the JSON-RPC `_warnings` field.
    /// Idempotent — a second call returns an empty `Vec`.
    pub fn take_warnings(&mut self) -> Vec<String> {
        std::mem::take(&mut self.warnings)
    }

    /// F-FINAL-06: Borrow the warnings (read-only).
    pub fn warnings(&self) -> &[String] {
        &self.warnings
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

    /// Iterate over all registered classes in insertion order.
    pub fn all_classes(&self) -> Vec<&ClassEntry> {
        self.classes.values().collect()
    }

    /// Check if the graph contains a cycle using DFS.
    ///
    /// Returns `true` if at least one cycle is detected.
    /// Uses three-color DFS (white/gray/black) for O(V+E) performance.
    pub fn has_cycle(&self) -> bool {
        let class_names: Vec<String> = self.classes.keys().cloned().collect();
        let name_to_idx: HashMap<&str, usize> = class_names.iter().enumerate()
            .map(|(i, name)| (name.as_str(), i))
            .collect();

        let node_count = class_names.len();
        if node_count == 0 {
            return false;
        }

        let adj_fn = |i: usize| {
            let name = class_names.get(i).map(|s| s.as_str()).unwrap_or("");
            if let Some(entry) = self.classes.get(name) {
                entry.injects.iter()
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
        let name_to_idx: HashMap<&str, usize> = class_names.iter().enumerate()
            .map(|(i, name)| (name.as_str(), i))
            .collect();

        let node_count = class_names.len();
        if node_count == 0 {
            return Vec::new();
        }

        let adj_fn = |i: usize| {
            let name = class_names.get(i).map(|s| s.as_str()).unwrap_or("");
            if let Some(entry) = self.classes.get(name) {
                entry.injects.iter()
                    .filter_map(|injected| name_to_idx.get(injected.as_str()))
                    .copied()
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        let label_fn = |i: usize| {
            class_names.get(i)
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
        let name_to_idx: HashMap<&str, usize> = class_names.iter().enumerate()
            .map(|(i, name)| (name.as_str(), i))
            .collect();

        let start_idx = name_to_idx[class_name];
        let node_count = class_names.len();

        let adj_fn = |i: usize| {
            let name = class_names.get(i).map(|s| s.as_str()).unwrap_or("");
            if let Some(entry) = self.classes.get(name) {
                entry.injects.iter()
                    .filter_map(|injected| name_to_idx.get(injected.as_str()))
                    .copied()
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        let indices = transitive_dependencies(start_idx, depth, node_count, adj_fn);
        indices.into_iter()
            .filter_map(|i| class_names.get(i))
            .cloned()
            .collect()
    }

    /// Format the full `§ΦGRAPH` footer section for the workspace
    /// manifest. Returns an empty string if the graph has no entries.
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
    ///
    /// Internally this drives an [`AngularGraphBuilder`] (Track B,
    /// F-ANG-05): the builder's `register_class` accepts the entries
    /// one at a time, then `build` consumes the builder and runs the
    /// `resolve_all` pass. The result is an immutable, fully-resolved
    /// `AngularGraph` with no public `register_class` method.
    pub fn build_graph(&self) -> AngularGraph {
        let mut builder = AngularGraphBuilder::new();
        for entry in &self.entries {
            builder.register_class(
                &entry.class_name,
                &entry.file_alias,
                entry.kind,
                entry.selector.as_deref(),
                &entry.injects,
                entry.pipe_name.as_deref(),
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

#[cfg(test)]
#[path = "../tests/angular_meta/graph.rs"]
mod tests;

#[cfg(test)]
#[path = "../tests/angular_meta/di_resolution.rs"]
mod di_tests;

#[cfg(test)]
#[path = "../tests/angular_meta/selector_linkage.rs"]
mod selector_tests;
