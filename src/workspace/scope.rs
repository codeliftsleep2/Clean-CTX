// src/workspace/scope.rs
//
// WorkspaceScope — the EFFECTIVE query scope of one occurrence-returning
// workspace query: workspace authorization, optionally narrowed by provenance.
//
// Two layers, composed in this order:
//
//   WSC-004 authorization    the caller's `workspaceRoot` plus that workspace's
//                            configured `additional_roots`;
//   `withinPath` narrowing   an optional file/directory subtree INSIDE the
//                            authorized set (the `workspace_query` argument).
//
//   effective scope = WorkspaceScope ∩ withinPath
//
// `withinPath` may only ever NARROW an authorized workspace. It is resolved and
// authorized BEFORE it is used, so a path outside `workspaceRoot` +
// `additional_roots` is rejected instead of being promoted into a new
// authorization root.
//
// Why this exists (query scoping, never semantic identity):
//   `EntityKey` stays `(domain, entity_type, name)` — Model C is untouched and a
//   scope never participates in identity. A session legitimately accumulates the
//   evidence of every repository it has compiled, so `WorkspaceIndex` must keep
//   all of it. A query issued FOR a workspace, however, must answer with that
//   workspace's occurrences only. The live defect was cross-repository bleed: a
//   `reverse_edges OrderBy` query returned real `Calls` facts authored by an
//   unrelated repository that merely happened to be indexed in the same session.
//
//   The boundary is therefore occurrence PROVENANCE. Every stored edge
//   occurrence carries the canonical file that asserted it
//   (`StoredEdge::asserting_file`), and a scope admits an occurrence when that
//   file lies inside the workspace roots taking part in the query. For a call
//   fact the asserting file is the CALLER's file — the callee may be external
//   and have no local declaration, and the fact is still valid.
//
// Root identity and containment:
//   * Roots are canonicalized with the same `canonical_identity_key` helper that
//     produces production asserting-file keys, so both sides of the comparison
//     share one canonical form (including Windows `\\?\` prefix stripping).
//   * Containment is PATH-COMPONENT aware, never a raw string prefix:
//     `C:\repo` admits `C:\repo\src\a.cs` but NOT `C:\repo-old\src\a.cs`.
//     Comparison is case-insensitive on Windows to match the filesystem.
//   * The scope covers the primary `workspaceRoot` PLUS the configured
//     `additional_roots` belonging to that workspace, so evidence authored in a
//     project-relative additional root is still admitted. The session holds ONE
//     flat root configuration — there is no per-primary association between a
//     queried root and the additional roots — so every configured additional root
//     participates in every scoped query of that session, while a repository that
//     is NOT configured never participates in one.
//
// No explicit workspace root ⇒ no scope ⇒ no filtering: the caller declared no
// workspace, so there is no root set to constrain the answer to. That keeps the
// behaviour of every root-less query exactly as it was. `withinPath` therefore
// REQUIRES an explicit `workspaceRoot`: with no declared workspace there is no
// authorized set to narrow, and an unrooted narrowing argument would silently
// become a new authorization root.
//
// `withinPath` resolution (the final rule):
//   * An ABSOLUTE path is used as declared; a RELATIVE path is resolved against
//     the declared `workspaceRoot` — exactly the rule `resolve_file_path`
//     (`src/mcp/tool_helpers.rs`) already applies to every other path argument,
//     so there is no second path-resolution system.
//   * The resolved path is canonicalized ONCE per query through
//     `canonical_identity_key`, the helper that also produces root and
//     asserting-file keys, so equivalent spellings of one path select the same
//     occurrences.
//   * An existing regular FILE narrows to exactly that file (component
//     equality). An existing directory, or a path that cannot be observed on
//     disk, narrows to the SUBTREE rooted at that path (component ancestry); a
//     source file that has since been deleted is therefore still selectable by
//     its own path, exactly as its provenance stays valid.
//   * Authorization is the SAME component-ancestry rule the roots already use,
//     so `feature` never admits `feature-old` and an unauthorized path is
//     rejected without consulting the index at all.
//   * Hydration is deliberately unaffected: candidate discovery remains
//     authorized by the WSC-004 roots alone, and the narrowing is applied to the
//     query answers (the initial run and the post-hydration rerun alike).
//
// Scope is a VIEW, not a mutation: nothing is removed from `WorkspaceIndex`, and
// the graph may legitimately hold `A: Process --Calls--> OrderBy` and
// `B: SortData --Calls--> OrderBy` at once — each answers its own scoped query.

use std::path::{Path, PathBuf};

/// The effective query scope of one occurrence-returning query: the authorized
/// workspace root set, optionally narrowed to a provenance subtree.
#[derive(Debug, Clone)]
pub struct WorkspaceScope {
    /// Canonicalized roots, each pre-split into path components.
    roots: Vec<Vec<String>>,
    /// The optional second layer (`withinPath`), already authorized against
    /// `roots` and pre-split into path components. `None` ⇒ authorization only.
    narrowing: Option<WithinPath>,
}

/// Provenance narrowing inside an already-authorized workspace (`withinPath`).
#[derive(Debug, Clone)]
struct WithinPath {
    /// The canonicalized narrowing path, pre-split into path components — the
    /// form admission is decided in.
    parts: Vec<String>,
    /// The same path as the canonical string, kept for the rejection message:
    /// re-joining `parts` would render a Windows root as `C:/\/Users/...`,
    /// because the `RootDir` component's string form is a lone backslash.
    path: String,
    /// The path was observed to be an existing regular FILE, so ONLY occurrences
    /// asserted by exactly that file are admitted.
    exact_file: bool,
}

impl WorkspaceScope {
    /// Build the authorization-only scope for a query issued at `primary`, or
    /// `None` when no usable workspace root was supplied (⇒ unscoped, unfiltered
    /// query).
    ///
    /// `additional_roots` are the configured roots belonging to that workspace.
    /// A root is used as declared; canonicalization is tolerant (a root that
    /// cannot be resolved keeps its declared spelling, exactly like
    /// `canonical_identity_key`), so an offline or not-yet-created root degrades
    /// to a literal comparison instead of failing the query.
    pub(crate) fn new(primary: Option<&str>, additional_roots: &[String]) -> Option<Self> {
        let primary = primary.map(str::trim).filter(|root| !root.is_empty())?;
        Some(Self {
            roots: root_set(primary, additional_roots),
            narrowing: None,
        })
    }

    /// Build the COMPOSED scope of one query: the authorized workspace root set
    /// (`primary` + configured `additional_roots`) intersected with the optional
    /// `within_path` narrowing.
    ///
    /// `Err(message)` — the query is refused before the index is consulted at
    /// all — when `within_path` was supplied with no explicit workspace root
    /// (there is no authorized set to narrow) or when it resolves outside every
    /// authorized root (it is not a narrowing of this workspace but a request for
    /// another one). Both are refusals rather than implicit authorizations:
    /// without them `withinPath` would be a second, caller-chosen root.
    pub(crate) fn narrowed(
        primary: Option<&str>,
        additional_roots: &[String],
        within_path: &str,
    ) -> Result<Self, String> {
        let Some(primary) = primary.map(str::trim).filter(|root| !root.is_empty()) else {
            return Err(
                "'withinPath' requires an explicit 'workspaceRoot': there is no declared \
                 workspace scope for it to narrow."
                    .to_string(),
            );
        };
        let roots = root_set(primary, additional_roots);
        let narrowing = WithinPath::resolve(within_path, primary);
        if !roots.iter().any(|root| is_within(&narrowing.parts, root)) {
            return Err(format!(
                "'withinPath' resolves outside the active workspace scope: {} \
                 (workspaceRoot: {}; additional roots: {})",
                narrowing.display(),
                canonical_root(primary),
                if additional_roots.is_empty() {
                    "none".to_string()
                } else {
                    additional_roots.join(", ")
                }
            ));
        }
        Ok(Self {
            roots,
            narrowing: Some(narrowing),
        })
    }

    /// Is occurrence provenance `asserting_file` inside the EFFECTIVE scope —
    /// admitted by the authorized root set AND, when present, by the `withinPath`
    /// narrowing?
    ///
    /// This is the single rule every occurrence-bearing surface uses: entity
    /// occurrences through `EntityRef.file`, edge occurrences through
    /// `StoredEdge::asserting_file`, and traversal adjacency. Composing both
    /// layers here is what keeps `withinPath` from being re-implemented as a
    /// per-handler path check.
    ///
    /// `asserting_file` is compared as stored. Production keys are produced by
    /// `canonical_identity_key` (see `tool_handlers::core`, `edit`, `hydration`),
    /// i.e. in exactly the canonical form the roots and the narrowing path are
    /// canonicalized into. The file is deliberately NOT canonicalized here: that
    /// would cost a filesystem hit per occurrence, and provenance must stay valid
    /// after a file is gone.
    pub(crate) fn admits(&self, asserting_file: &str) -> bool {
        let file = path_components(Path::new(asserting_file));
        let authorized =
            self.roots.is_empty() || self.roots.iter().any(|root| is_within(&file, root));
        authorized
            && self
                .narrowing
                .as_ref()
                .is_none_or(|narrowing| narrowing.admits(&file))
    }

    /// Does this scope carry a `withinPath` narrowing?
    ///
    /// The one surface whose file path is already validated elsewhere
    /// (`entities_in_file`'s explicit `file_path`) applies the extra check only
    /// when there is a second layer to apply, so its accepted set is unchanged
    /// whenever no narrowing is present.
    pub(crate) fn has_narrowing(&self) -> bool {
        self.narrowing.is_some()
    }
}

impl WithinPath {
    /// Resolve and canonicalize one `withinPath` argument — once per query. The
    /// resolution rule is documented in the module header.
    fn resolve(raw: &str, declared_root: &str) -> Self {
        let resolved = resolve_against_root(raw.trim(), declared_root);
        // File-vs-directory is taken from the filesystem where it is observable.
        // A path that cannot be observed (not yet created, or deleted since) is
        // treated as a SUBTREE boundary, which for a file path still admits
        // exactly the occurrences whose provenance equals it.
        let exact_file = std::fs::metadata(&resolved).is_ok_and(|metadata| metadata.is_file());
        let path = canonical_root(&resolved.to_string_lossy());
        let parts = path_components(Path::new(&path));
        Self {
            parts,
            path,
            exact_file,
        }
    }

    /// Is this occurrence's path admitted by the narrowing?
    fn admits(&self, file: &[String]) -> bool {
        if self.exact_file {
            file.len() == self.parts.len()
                && self
                    .parts
                    .iter()
                    .zip(file.iter())
                    .all(|(path_part, file_part)| component_eq(path_part, file_part))
        } else {
            is_within(file, &self.parts)
        }
    }

    /// The resolved narrowing path, for the rejection message only.
    fn display(&self) -> &str {
        &self.path
    }
}

/// The authorized root set of one workspace: the primary root plus the configured
/// additional roots, each canonicalized and pre-split into components
/// (deduplicated).
fn root_set(primary: &str, additional_roots: &[String]) -> Vec<Vec<String>> {
    let mut roots: Vec<Vec<String>> = Vec::new();
    for root in std::iter::once(primary).chain(
        additional_roots
            .iter()
            .map(String::as_str)
            .filter(|root| !root.trim().is_empty()),
    ) {
        let parts = path_components(Path::new(&canonical_root(root)));
        if !parts.is_empty() && !roots.contains(&parts) {
            roots.push(parts);
        }
    }
    roots
}

/// Resolve a path argument against the declared workspace root, mirroring
/// `resolve_file_path` (`src/mcp/tool_helpers.rs`): an absolute path is used as
/// declared, a relative path is joined to the root, and a relative root is
/// itself joined to the process CWD — the same fallback that function uses.
fn resolve_against_root(path: &str, declared_root: &str) -> PathBuf {
    let path = Path::new(path);
    if path.is_absolute() {
        return path.to_path_buf();
    }
    let root = Path::new(declared_root);
    if root.is_absolute() {
        root.join(path)
    } else {
        std::env::current_dir()
            .unwrap_or_default()
            .join(root)
            .join(path)
    }
}

/// Canonical root identity — the shared helper, so roots and asserting-file keys
/// share one form (Windows verbatim prefixes stripped, unresolvable paths kept
/// verbatim).
fn canonical_root(root: &str) -> String {
    crate::dictionary::path::canonical_identity_key(root)
}

fn path_components(path: &Path) -> Vec<String> {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect()
}

/// Component-wise ancestry: `file` is inside `root` when `root` is a component
/// prefix of `file`. A raw string prefix check would wrongly admit
/// `C:\repo-old` for root `C:\repo`.
fn is_within(file: &[String], root: &[String]) -> bool {
    file.len() >= root.len()
        && root
            .iter()
            .zip(file.iter())
            .all(|(root_part, file_part)| component_eq(root_part, file_part))
}

#[cfg(windows)]
fn component_eq(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b)
}

#[cfg(not(windows))]
fn component_eq(a: &str, b: &str) -> bool {
    a == b
}
