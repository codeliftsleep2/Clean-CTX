// src/edit/locate.rs
//
// Unit relocation for the `apply_edit` write path.
//
// Plan Risk Analysis #1: name-only lookup can resolve to the wrong unit
// when a file was restructured (methods reordered, same-named method in
// another class, overloads). Mitigation: units are keyed on qualified
// name + a structural fingerprint (containing class + ordered parameter
// types), and bare-name resolution is only accepted when it is unique.

use std::collections::HashMap;

use streaming_iterator::StreamingIterator;
use tree_sitter::{Language, Parser, Query, QueryCursor};

use crate::ir::opcodes::CoreOp;

/// One splice-addressable structural unit derived from compiled IR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitRecord {
    /// Qualified name ("ClassName.method"). For top-level functions with
    /// no enclosing class this equals the bare method name.
    pub qualified_name: String,
    /// IR alias id of the method ("M3") — also accepted as a lookup key.
    pub method_id: String,
    /// Bare unit name as written in source.
    pub name: String,
    /// Enclosing class original name, when the unit lives inside one.
    pub class_name: Option<String>,
    /// Absolute byte span `[start_byte, end_byte)` of the body slice in
    /// the source this table was built from. Only units WITH spans are
    /// tracked here — span-less bodies are `apply_edit`-ineligible.
    pub start_byte: u64,
    pub end_byte: u64,
    /// Byte-exact current text of the body slice.
    pub text: String,
    /// Full declaration span used only by whole-unit operations such as
    /// delete. Body-based operations continue to use `start_byte..end_byte`.
    pub declaration_span: Option<(u64, u64)>,
    /// Structured declaration start for insertion. None when leading comment
    /// ownership cannot be established; never substitutes the body position.
    pub insert_before_byte: Option<u64>,
    /// Truthful reason whole-declaration authority is absent.
    pub declaration_blocker: Option<String>,
    /// Structural fingerprint: containing class + ordered parameter types.
    /// Two units may share a bare name; they must not share a fingerprint
    /// within one class unless they are true overloads (both remain valid,
    /// distinct targets).
    pub fingerprint: String,
}

/// Errors from unit resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocateError {
    /// No unit matched the requested target.
    NotFound(String),
    /// Bare-name match hit several units; caller must qualify.
    Ambiguous {
        target: String,
        candidates: Vec<String>,
    },
    /// Structured syntax proves the declaration exists, but the edit table
    /// has no authoritative mutation boundary for its declaration kind.
    Uneditable { target: String, target_kind: String },
}

impl std::fmt::Display for LocateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LocateError::NotFound(target) => {
                write!(f, "unit not found: {}", target)
            }
            LocateError::Ambiguous { target, candidates } => {
                write!(
                    f,
                    "unit `{}` is ambiguous ({} candidates: {})",
                    target,
                    candidates.len(),
                    candidates.join(", ")
                )
            }
            LocateError::Uneditable {
                target,
                target_kind,
            } => write!(f, "unit `{target}` is not editable: {target_kind}"),
        }
    }
}

impl std::error::Error for LocateError {}

/// Per-method metadata collected during table construction (pass 1),
/// before span attachment (pass 2).
#[derive(Default)]
struct PendingMeta {
    class_id: String,
    name: String,
    param_types: Vec<String>,
}

/// A resolved unit table for one file version.
#[derive(Debug, Clone, Default)]
pub struct UnitTable {
    units: Vec<UnitRecord>,
    by_qualified: HashMap<String, Vec<usize>>,
    by_signature: HashMap<String, Vec<usize>>,
    by_method_id: HashMap<String, usize>,
    uneditable: Vec<UneditableRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct UneditableRecord {
    selector: String,
    name: String,
    kind: String,
}

impl UnitTable {
    /// Build a unit table from compiled IR instructions.
    ///
    /// Only `CoreOp::Body` ops carrying spans produce entries; span-less
    /// bodies (legacy wire state) are skipped by design so the write path
    /// can never splice against unknown offsets.
    pub fn from_instructions(instructions: &[CoreOp]) -> Self {
        let mut classes: HashMap<String, String> = HashMap::new();
        let mut order: Vec<PendingMeta> = Vec::new();
        let mut index_of_mid: HashMap<String, usize> = HashMap::new();
        let mut bodies: HashMap<String, (String, u64, u64)> = HashMap::new();
        // Spanned-body method ids in first-encounter order. HashMap
        // iteration is randomized, so materialization must walk this Vec
        // to keep unit ordering — and therefore ambiguity-candidate
        // listings — deterministic across runs.
        let mut body_order: Vec<String> = Vec::new();

        // Pass 1: collect structure in instruction order (preserves the
        // compiler's emission ordering).
        for op in instructions {
            match op {
                CoreOp::DefClass(id, name) => {
                    classes.entry(id.clone()).or_insert_with(|| name.clone());
                }
                CoreOp::DefMethod(cid, mid, name) => {
                    index_of_mid.insert(mid.clone(), order.len());
                    order.push(PendingMeta {
                        class_id: cid.clone(),
                        name: name.clone(),
                        param_types: Vec::new(),
                    });
                }
                CoreOp::Param(mid, _, ty, _) => {
                    if let Some(&idx) = index_of_mid.get(mid) {
                        order[idx].param_types.push(ty.clone());
                    }
                }
                CoreOp::Body(mid, text, Some(start), Some(end)) => {
                    if !bodies.contains_key(mid) {
                        body_order.push(mid.clone());
                    }
                    bodies.insert(mid.clone(), (text.clone(), *start, *end));
                }
                _ => {}
            }
        }
        Self::materialize(classes, order, &index_of_mid, &body_order, &bodies)
    }

    /// Pass 2: materialize records for methods that have spanned bodies,
    /// in `body_order` (deterministic instruction order).
    fn materialize(
        classes: HashMap<String, String>,
        order: Vec<PendingMeta>,
        index_of_mid: &HashMap<String, usize>,
        body_order: &[String],
        bodies: &HashMap<String, (String, u64, u64)>,
    ) -> Self {
        let mut table = Self::default();
        for mid in body_order {
            let Some((text, start, end)) = bodies.get(mid) else {
                continue;
            };
            let Some(&idx) = index_of_mid.get(mid) else {
                continue;
            };
            let pending = &order[idx];
            let class_name = classes.get(&pending.class_id).cloned();
            let qualified_name = match &class_name {
                Some(c) => format!("{}.{}", c, pending.name),
                None => pending.name.clone(),
            };
            let fingerprint = format!(
                "{}({})",
                class_name.as_deref().unwrap_or(""),
                pending.param_types.join(",")
            );
            let record = UnitRecord {
                qualified_name: qualified_name.clone(),
                method_id: mid.clone(),
                name: pending.name.clone(),
                class_name,
                start_byte: *start,
                end_byte: *end,
                text: text.clone(),
                declaration_span: None,
                insert_before_byte: None,
                declaration_blocker: Some("declaration_boundary_unavailable".to_string()),
                fingerprint,
            };
            let slot = table.units.len();
            table
                .by_qualified
                .entry(qualified_name.clone())
                .or_default()
                .push(slot);
            table
                .by_signature
                .entry(format!(
                    "{}({})",
                    qualified_name,
                    pending.param_types.join(",")
                ))
                .or_default()
                .push(slot);
            let base_name = method_base_name(&pending.name);
            if base_name != pending.name {
                let base_qualified = match &record.class_name {
                    Some(owner) => format!("{owner}.{base_name}"),
                    None => base_name.to_string(),
                };
                table
                    .by_qualified
                    .entry(base_qualified.clone())
                    .or_default()
                    .push(slot);
                table
                    .by_signature
                    .entry(format!(
                        "{}({})",
                        base_qualified,
                        pending.param_types.join(",")
                    ))
                    .or_default()
                    .push(slot);
            }
            table.by_method_id.insert(record.method_id.clone(), slot);
            table.units.push(record);
        }
        table
    }

    /// Build a table whose body records are also mapped back to their full
    /// declaration captures. The existing language query is the authority for
    /// declaration boundaries; contiguous leading metadata siblings are owned
    /// by that declaration as well.
    pub fn from_instructions_with_declarations(
        instructions: &[CoreOp],
        source: &str,
        language: Language,
        query: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut table = Self::from_instructions(instructions);
        let mut parser = Parser::new();
        parser.set_language(&language)?;
        let tree = parser.parse(source, None).ok_or("AST generation failed")?;
        let compiled_query = Query::new(&language, query)?;
        let capture_names = compiled_query.capture_names();
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&compiled_query, tree.root_node(), source.as_bytes());
        let mut declarations = Vec::new();
        let mut syntax_declarations = Vec::new();

        while let Some(query_match) = matches.next() {
            for capture in query_match.captures {
                let capture_name = capture_names[capture.index as usize];
                if matches!(
                    capture_name,
                    "class.root"
                        | "interface.root"
                        | "record.root"
                        | "struct.root"
                        | "enum.root"
                        | "method.root"
                        | "constructor.root"
                        | "func.root"
                        | "field.root"
                        | "type.root"
                ) && let Some((selector, name)) = declaration_selector(capture.node, source)
                {
                    syntax_declarations.push((
                        capture.node.start_byte() as u64,
                        capture.node.end_byte() as u64,
                        selector,
                        name,
                        capture.node.kind().to_string(),
                    ));
                }
                if matches!(
                    capture_name,
                    "method.root" | "constructor.root" | "func.root" | "arrow.root"
                ) {
                    let end = capture.node.end_byte();
                    let start = declaration_start_with_metadata(capture.node, source);
                    let insertion_start = (!has_unowned_leading_comment(capture.node, source))
                        .then_some(start as u64);
                    declarations.push((start as u64, end as u64, insertion_start));
                }
            }
        }
        collect_test_callbacks(tree.root_node(), source, &mut syntax_declarations);
        syntax_declarations.sort();
        syntax_declarations.dedup();

        for unit in &mut table.units {
            let declaration = declarations
                .iter()
                .copied()
                .filter(|(start, end, _)| *start <= unit.start_byte && *end == unit.end_byte)
                .min_by_key(|(start, end, _)| end - start);
            unit.declaration_span = declaration
                .and_then(|(start, end, insertion_start)| insertion_start.map(|_| (start, end)));
            unit.insert_before_byte = declaration.and_then(|(_, _, start)| start);
            unit.declaration_blocker = match declaration {
                Some((_, _, Some(_))) => None,
                Some((_, _, None)) => {
                    Some("declaration_with_unproven_leading_documentation".to_string())
                }
                None => Some("declaration_boundary_unavailable".to_string()),
            };
        }
        for (start, end, selector, name, kind) in syntax_declarations {
            // A body-backed declaration remains an editable candidate even
            // when a whole-declaration operation lacks comment ownership.
            // Resolution selects the unit; the requested operation then
            // reports its specific missing boundary as uneditable.
            let editable = table
                .units
                .iter()
                .any(|unit| unit.end_byte == end && unit.start_byte >= start);
            if !editable {
                table.uneditable.push(UneditableRecord {
                    selector,
                    name,
                    kind,
                });
            }
        }
        Ok(table)
    }

    pub fn len(&self) -> usize {
        self.units.len()
    }

    pub fn is_empty(&self) -> bool {
        self.units.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &UnitRecord> {
        self.units.iter()
    }

    /// Resolve a target to exactly one unit.
    ///
    /// Accepted key forms, in priority order:
    /// 1. Session-local method alias id (`"M3"`)
    /// 2. Signature-qualified name (`"UserService.processOrder(string)"`)
    /// 3. Qualified name (`"UserService.processOrder"`) when unique
    /// 4. Bare name (`"processOrder"`) when unique.
    pub fn resolve(&self, target: &str) -> Result<&UnitRecord, LocateError> {
        if let Some(&idx) = self.by_method_id.get(target) {
            return Ok(&self.units[idx]);
        }
        if let Some(indices) = self.by_signature.get(target) {
            return self.resolve_indices(target, indices);
        }
        let editable: Vec<usize> = self.by_qualified.get(target).cloned().unwrap_or_else(|| {
            self.units
                .iter()
                .enumerate()
                .filter_map(|(index, unit)| {
                    (unit.name == target || method_base_name(&unit.name) == target).then_some(index)
                })
                .collect()
        });
        let uneditable: Vec<&UneditableRecord> = self
            .uneditable
            .iter()
            .filter(|record| record.selector == target || record.name == target)
            .collect();
        match (editable.as_slice(), uneditable.as_slice()) {
            ([index], []) => Ok(&self.units[*index]),
            ([], [record]) => Err(LocateError::Uneditable {
                target: target.to_string(),
                target_kind: record.kind.clone(),
            }),
            ([], []) => Err(LocateError::NotFound(target.to_string())),
            _ => Err(LocateError::Ambiguous {
                target: target.to_string(),
                candidates: editable
                    .iter()
                    .map(|index| self.units[*index].selector())
                    .chain(uneditable.iter().map(|record| record.selector.clone()))
                    .collect(),
            }),
        }
    }

    fn resolve_indices(&self, target: &str, indices: &[usize]) -> Result<&UnitRecord, LocateError> {
        match indices {
            [index] => Ok(&self.units[*index]),
            _ => Err(LocateError::Ambiguous {
                target: target.to_string(),
                candidates: indices
                    .iter()
                    .map(|index| self.units[*index].selector())
                    .collect(),
            }),
        }
    }
}
impl UnitRecord {
    fn selector(&self) -> String {
        let parameters = self
            .fingerprint
            .strip_prefix(self.class_name.as_deref().unwrap_or(""))
            .unwrap_or(&self.fingerprint);
        format!("{}{}", self.qualified_name, parameters)
    }
}

fn declaration_selector(node: tree_sitter::Node<'_>, source: &str) -> Option<(String, String)> {
    let name = match node.kind() {
        "operator_declaration" => "operator".to_string(),
        "conversion_operator_declaration" => "conversion_operator".to_string(),
        _ => node
            .child_by_field_name("name")
            .or_else(|| descendant_name(node))?
            .utf8_text(source.as_bytes())
            .ok()?
            .trim()
            .to_string(),
    };
    if name.is_empty() {
        return None;
    }
    let mut ancestor = node.parent();
    while let Some(candidate) = ancestor {
        if matches!(
            candidate.kind(),
            "class_declaration"
                | "abstract_class_declaration"
                | "interface_declaration"
                | "record_declaration"
                | "struct_declaration"
        ) && let Some(owner_node) = candidate.child_by_field_name("name")
            && let Ok(owner) = owner_node.utf8_text(source.as_bytes())
        {
            return Some((format!("{}.{}", owner.trim(), name), name));
        }
        ancestor = candidate.parent();
    }
    Some((name.clone(), name))
}

fn method_base_name(name: &str) -> &str {
    name.find('<')
        .filter(|_| name.ends_with('>'))
        .map_or(name, |index| &name[..index])
}

fn descendant_name<'tree>(node: tree_sitter::Node<'tree>) -> Option<tree_sitter::Node<'tree>> {
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        if let Some(name) = child.child_by_field_name("name") {
            return Some(name);
        }
        if let Some(name) = descendant_name(child) {
            return Some(name);
        }
    }
    None
}

fn collect_test_callbacks(
    node: tree_sitter::Node<'_>,
    source: &str,
    declarations: &mut Vec<(u64, u64, String, String, String)>,
) {
    if node.kind() == "call_expression"
        && let Some(function) = node.child_by_field_name("function")
        && let Ok(name) = function.utf8_text(source.as_bytes())
        && matches!(name, "describe" | "it" | "test")
    {
        declarations.push((
            node.start_byte() as u64,
            node.end_byte() as u64,
            name.to_string(),
            name.to_string(),
            "spec_test_callback".to_string(),
        ));
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        collect_test_callbacks(child, source, declarations);
    }
}

fn declaration_start_with_metadata(mut node: tree_sitter::Node<'_>, source: &str) -> usize {
    let mut start = node.start_byte();
    while let Some(previous) = node.prev_named_sibling() {
        if !is_declaration_metadata(previous.kind())
            || !source[previous.end_byte()..start].trim().is_empty()
        {
            break;
        }
        start = previous.start_byte();
        node = previous;
    }
    start
}

fn is_declaration_metadata(kind: &str) -> bool {
    matches!(
        kind,
        "decorator"
            | "attribute"
            | "attribute_item"
            | "attribute_list"
            | "annotation"
            | "marker_annotation"
            | "normal_annotation"
            | "single_element_annotation"
    )
}

/// A sibling comment is not a structured declaration-owned child. Refuse
/// whole-declaration edits when it borders the metadata group rather than
/// guessing whether it is documentation. Body replacement remains available.
fn has_unowned_leading_comment(mut node: tree_sitter::Node<'_>, source: &str) -> bool {
    while let Some(previous) = node.prev_named_sibling() {
        if !source[previous.end_byte()..node.start_byte()]
            .trim()
            .is_empty()
        {
            return false;
        }
        if matches!(
            previous.kind(),
            "comment" | "line_comment" | "block_comment"
        ) {
            return true;
        }
        if !is_declaration_metadata(previous.kind()) {
            return false;
        }
        node = previous;
    }
    false
}
