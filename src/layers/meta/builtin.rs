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
        _source: &str,
        class_captures: &[(String, String)],
        _fidelity: Fidelity,
        _config: Option<&CleanCtxConfig>,
    ) -> Vec<SemanticEdge> {
        let mut edges = Vec::new();
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
                object: entity,
                layer: "builtin",
            });
        }
        edges
    }
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
