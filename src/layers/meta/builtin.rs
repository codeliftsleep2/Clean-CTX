// src/layers/meta/builtin.rs
//
// Always-on fallback meta layer that indexes ordinary type declarations into
// the WorkspaceIndex for every compiled file.
//
// Root cause (2026-09): the WorkspaceIndex is contractually a general
// workspace entity index (src/workspace/index.rs: "Framework-agnostic
// cross-file semantic index"), but it only received entities when a framework
// meta layer (Angular/.NET/Spring) emitted semantic edges. For plain files
// the pipeline was:
//
//   source → CoreIRPass captures → MetaLayerPass → collect_semantic_edges()
//     → no applicable framework layer → semantic_edges = []
//     → WorkspaceIndex.add_edges([]) → no entities
//
// This layer consumes the SAME capture pairs MetaLayerPass already builds
// (capture name + class-source span). It adds NO tree-sitter parsing and NO
// second declaration-discovery mechanism — it is a projection of the existing
// compiler captures into the semantic-edge model.
//
// Self-referential `Defines(entity, entity)` representation (audited 2026-09):
//   - The self-`Defines` shape is an entity-registration carrier. The index
//     write boundary normalizes it (`WorkspaceIndex::add_edges`): the entity
//     is registered once with file provenance in `entities`, `name_index`,
//     and `file_map`, and the record never enters the relationship graph
//     (`edge_set` / `file_edges` / `forward` / `reverse`).
//   - `transitive_dependencies` filters through `DEPENDENCY_RELATIONS`, which
//     does NOT include `Defines` — unchanged by the carrier shape.
//   - `has_cycle` therefore stays false for ordinary compiled files. Real
//     relationships are unaffected: non-self `Defines(A, B)` edges remain
//     graph edges, and a real dependency self-loop (`Injects(A, A)`) is still
//     detected as a cycle (`workspace::index::tests::has_cycle_self_loop`).

use crate::compression::Fidelity;
use crate::config::CleanCtxConfig;
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};
use crate::layers::meta::{MetaLayer, MetaLayerOutput};
use std::path::Path;

/// Fallback meta layer that indexes ordinary type declarations (classes,
/// interfaces, structs, enums, traits, records) as `builtin` entities.
///
/// Registration order contract: registered LAST in `LayerRegistry` so
/// framework layers run first. Because the `builtin` domain is disjoint from
/// `angular` / `dotnet` / `spring` / `ngrx`, entity and edge identities never
/// collide with framework output and ordering never affects correctness.
pub struct BuiltinMetaLayer;

impl BuiltinMetaLayer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for BuiltinMetaLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl MetaLayer for BuiltinMetaLayer {
    fn name(&self) -> &'static str {
        "builtin"
    }

    /// Always applicable: this is the fallback catcher for every file the
    /// framework layers do not claim. Produces no Phi markers (`enrich` →
    /// `None`), only semantic edges.
    fn is_applicable(&self, _source: &str, _path: &Path, _config: Option<&CleanCtxConfig>) -> bool {
        true
    }

    fn enrich(
        &self,
        _source: &str,
        _class_captures: &[String],
        _fidelity: Fidelity,
        _config: Option<&CleanCtxConfig>,
    ) -> Option<MetaLayerOutput> {
        None
    }

    fn extract_semantic_edges_paired(
        &self,
        source: &str,
        class_captures: &[(String, String)],
        _fidelity: Fidelity,
        _config: Option<&CleanCtxConfig>,
    ) -> Vec<SemanticEdge> {
        let mut edges = Vec::new();
        // Java language attribution for the semantic projection. The builtin
        // layer is language-agnostic (no language parameter reaches it); the
        // `implements` shape is Java language knowledge, so it is only
        // projected when the source text is attributed to Java (see
        // `is_java_source`). The emitted relation itself stays generic.
        let is_java = is_java_source(source);
        for (capture_name, raw_class) in class_captures {
            let entity_type = match capture_name.as_str() {
                "class.root" => "Class",
                "interface.root" => "Interface",
                "struct.root" => "Struct",
                "enum.root" => "Enum",
                "trait.root" => "Trait",
                "record.root" => "Record",
                _ => continue,
            };
            let name = declaration_name(capture_name, raw_class);
            if name.is_empty() {
                continue;
            }
            let entity = EntityRef::new("builtin", entity_type, name);
            edges.push(SemanticEdge {
                relation: SemanticRelation::Defines,
                subject: entity.clone(),
                object: entity.clone(),
                layer: "builtin",
            });

            // Generic `Extends` from `extends` keyword. Authoritative across
            // Java and TypeScript (the target is always class-like). C# `:`
            // syntax is NOT emitted — its flat base list cannot distinguish
            // an optional base class from interfaces without inference.
            for base_name in parse_extends_bases(raw_class) {
                edges.push(SemanticEdge {
                    relation: SemanticRelation::Extends,
                    subject: entity.clone(),
                    object: EntityRef::new("builtin", "Class", &base_name),
                    layer: "builtin",
                });
            }

            // Phase 30-A: `Implements` projection.
            // builtin/Class/<implementation> → Implements → builtin/Interface/<interface>
            // for each authoritative Java `implements` clause. Java language
            // knowledge (`is_java_source`) decides the keyword means
            // `Implements`; the relation remains the generic `Implements`.
            // This NEVER creates a `Binds` edge — the DI registration fact
            // (`Implements ≠ Binds`) stays an independent concern.
            if is_java {
                for iface_name in parse_implements_bases(raw_class) {
                    edges.push(SemanticEdge {
                        relation: SemanticRelation::Implements,
                        subject: entity.clone(),
                        object: EntityRef::new("builtin", "Interface", &iface_name),
                        layer: "builtin",
                    });
                }
            }
        }
        edges
    }
}

/// Parse base class names from the `extends` keyword in a class declaration.
///
/// Returns the list of base type names that follow `extends`. This is
/// authoritative for `Extends` across Java and TypeScript — the target of
/// `extends` is always class-like. Does NOT handle C# `:` syntax (its flat
/// base list cannot distinguish an optional base class from interfaces).
fn parse_extends_bases(raw_class: &str) -> Vec<String> {
    let declaration_root = strip_leading_annotations(raw_class);
    let decl = declaration_root.lines().next().unwrap_or(declaration_root);
    let decl = decl.split('{').next().unwrap_or(decl).trim();
    crate::compaction::class::extract_base_types(decl, "extends")
}

/// Parse interface names from the `implements` keyword in a class
/// declaration.
///
/// Mirror of [`Self::parse_extends_bases`] for the Java `implements` clause,
/// reusing the same authoritative shared extractor
/// (`compaction::class::extract_base_types`), which splits top-level commas,
/// strips generic arguments at the first `<`, preserves qualified names, and
/// returns nothing when the keyword is absent (fail closed).
///
/// Callers MUST gate this on Java language attribution
/// ([`Self::is_java_source`]): `implements` is a Java (and TypeScript)
/// keyword, and this layer only projects it when the source is attributed
/// to Java.
fn parse_implements_bases(raw_class: &str) -> Vec<String> {
    let declaration_root = strip_leading_annotations(raw_class);
    let decl = declaration_root.lines().next().unwrap_or(declaration_root);
    let decl = decl.split('{').next().unwrap_or(decl).trim();
    crate::compaction::class::extract_base_types(decl, "implements")
}

/// Establish Java language attribution for a source file.
///
/// The BuiltinMetaLayer receives no language parameter, so Java language
/// knowledge must be derived from the source text. Within the supported
/// language set (TypeScript, C#, Rust, Java) the `package` declaration and
/// the semicolon-terminated `import a.b.C;` statement are Java-only markers
/// (TypeScript uses `import ... from '...'`, C# uses `using`, Rust uses
/// `use`). Comment lines are skipped so commented-out fragments do not
/// attribute a file as Java.
///
/// Fails closed: a source with neither marker is NOT attributed as Java,
/// preserving the language-agnostic behavior and the historical deferral.
fn is_java_source(source: &str) -> bool {
    let is_comment_line = |t: &str| {
        t.starts_with("//") || t.starts_with("/*") || t.starts_with('*') || t.starts_with('#')
    };
    source.lines().any(|line| {
        let t = line.trim_start();
        if is_comment_line(t) {
            return false;
        }
        (t.starts_with("package ") && t.contains(';'))
            || ((t.starts_with("import ") || t.starts_with("import static "))
                && t.contains(';')
                && !t.contains(" from "))
    })
}

/// Extract the bare declaration name using the existing class-name extraction
/// infrastructure (`src/compaction/class.rs`). No new parsing logic.
fn declaration_name(capture_name: &str, raw_class: &str) -> String {
    // C-22 class spans are decorator/annotation-inclusive by design. The
    // shared class-name extractors assume the declaration header is
    // reachable from byte 0, so trim any leading `@Decorator(...)` /
    // `@Annotation` group (TypeScript decorators, Java annotations) before
    // delegating. C# `[Attribute]` prefixes are handled inside the shared
    // extractor itself.
    let declaration_root = strip_leading_annotations(raw_class);
    let extracted = match capture_name {
        // Only Rust emits `trait.root`; its traits/structs/enums go through
        // the Rust-aware extractor (`pub`-prefix stripping).
        "trait.root" => crate::compaction::class::extract_rust_struct_name(declaration_root),
        // Every other supported type root shares the class-like declaration
        // shape (modifiers + keyword + optional base list).
        _ => crate::compaction::class::extract_class_name(declaration_root),
    };
    // Collapse to the bare entity identity name:
    //   - "Foo:Base" / "Foo:Base,IFoo" (extract_class_name appends a TS
    //     extends/implements list) → "Foo"
    //   - "Foo<T>" (extract_rust_struct_name preserves generics) → "Foo"
    extracted
        .split(':')
        .next()
        .unwrap_or(&extracted)
        .split('<')
        .next()
        .unwrap_or("")
        .trim()
        .to_string()
}

/// Strip leading `@Decorator(...)` / `@Annotation` groups from a class span.
///
/// The builtin layer receives decorator/annotation-inclusive source spans by
/// design (invariant C-22: `class_source_from_capture`). TypeScript
/// decorators (`@Component({...})`) and Java annotations (`@RestController`)
/// precede the declaration with `@`-prefixed groups that the shared
/// class-name extractors do not understand (their attribute stripping covers
/// C# `[...]` only). This trims every leading `@` group — with or without a
/// balanced `(...)` argument list — so the remaining text starts at the
/// declaration header (`export class ...`, `public class ...`).
///
/// Generic by construction: no framework vocabulary is consulted.
fn strip_leading_annotations(text: &str) -> &str {
    let mut rest = text.trim_start();
    loop {
        if !rest.starts_with('@') {
            return rest;
        }
        // Advance past the annotation name (identifier, optionally dotted).
        let bytes = rest.as_bytes();
        let mut name_end = 1;
        while name_end < bytes.len()
            && (bytes[name_end] == b'.'
                || bytes[name_end].is_ascii_alphanumeric()
                || bytes[name_end] == b'_'
                || bytes[name_end] == b'$')
        {
            name_end += 1;
        }
        // Optional balanced argument group: `@Name(...)`.
        if name_end < bytes.len() && bytes[name_end] == b'(' {
            let group = &rest[name_end..];
            match crate::meta_util::find_matching_brace(group, '(') {
                Some(close) => {
                    rest = rest[name_end + close + 1..].trim_start();
                    continue;
                }
                // Unbalanced argument group — leave the text unchanged
                // (defensive; the shared extractor fails safe too).
                None => return rest,
            }
        }
        // Bare annotation (`@Name`) — advance past the name so stacked
        // annotations are all consumed.
        rest = rest[name_end..].trim_start();
    }
}

#[cfg(test)]
#[path = "../../tests/layers/meta/builtin.rs"]
mod tests;
