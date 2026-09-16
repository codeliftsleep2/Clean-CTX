// src/workspace/scope.rs
//
// WorkspaceScope — the active workspace root set that constrains
// occurrence-returning workspace queries.
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
// behaviour of every root-less query exactly as it was.
//
// Scope is a VIEW, not a mutation: nothing is removed from `WorkspaceIndex`, and
// the graph may legitimately hold `A: Process --Calls--> OrderBy` and
// `B: SortData --Calls--> OrderBy` at once — each answers its own scoped query.

use std::path::Path;

/// The active workspace root set of one query.
#[derive(Debug, Clone)]
pub struct WorkspaceScope {
    /// Canonicalized roots, each pre-split into path components.
    roots: Vec<Vec<String>>,
}

impl WorkspaceScope {
    /// Build the scope for a query issued at `primary`, or `None` when no usable
    /// workspace root was supplied (⇒ unscoped, unfiltered query).
    ///
    /// `additional_roots` are the configured roots belonging to that workspace.
    /// A root is used as declared; canonicalization is tolerant (a root that
    /// cannot be resolved keeps its declared spelling, exactly like
    /// `canonical_identity_key`), so an offline or not-yet-created root degrades
    /// to a literal comparison instead of failing the query.
    pub(crate) fn new(primary: Option<&str>, additional_roots: &[String]) -> Option<Self> {
        let primary = primary.map(str::trim).filter(|root| !root.is_empty())?;
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
        Some(Self { roots })
    }

    /// Is occurrence provenance `asserting_file` inside the active workspace?
    ///
    /// `asserting_file` is compared as stored. Production keys are produced by
    /// `canonical_identity_key` (see `tool_handlers::core`, `edit`, `hydration`),
    /// i.e. in exactly the canonical form the roots are canonicalized into. The
    /// file is deliberately NOT canonicalized here: that would cost a filesystem
    /// hit per occurrence, and provenance must stay valid after a file is gone.
    pub(crate) fn admits(&self, asserting_file: &str) -> bool {
        if self.roots.is_empty() {
            return true;
        }
        let file = path_components(Path::new(asserting_file));
        self.roots.iter().any(|root| is_within(&file, root))
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
