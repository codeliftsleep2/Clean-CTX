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
    by_qualified: HashMap<String, usize>,
    by_method_id: HashMap<String, usize>,
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
                fingerprint,
            };
            let slot = table.units.len();
            table.by_qualified.insert(qualified_name, slot);
            table.by_method_id.insert(record.method_id.clone(), slot);
            table.units.push(record);
        }
        table
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
    /// 1. Qualified name (`"UserService.processOrder"`)
    /// 2. Method alias id (`"M3"`)
    /// 3. Bare name (`"processOrder"`) — only when unambiguous.
    pub fn resolve(&self, target: &str) -> Result<&UnitRecord, LocateError> {
        if let Some(&idx) = self.by_qualified.get(target) {
            return Ok(&self.units[idx]);
        }
        if let Some(&idx) = self.by_method_id.get(target) {
            return Ok(&self.units[idx]);
        }
        let matches: Vec<&UnitRecord> = self.units.iter().filter(|u| u.name == target).collect();
        match matches.len() {
            0 => Err(LocateError::NotFound(target.to_string())),
            1 => Ok(matches[0]),
            _ => Err(LocateError::Ambiguous {
                target: target.to_string(),
                candidates: matches
                    .iter()
                    .map(|u| format!("{} [{}]", u.qualified_name, u.fingerprint))
                    .collect(),
            }),
        }
    }
}
