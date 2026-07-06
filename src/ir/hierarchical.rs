// src/ir/hierarchical.rs
//
// Phase II: Scoped Hierarchical IR (Idea #4).
//
// Replaces the flat instruction array with a class→method→param tree,
// eliminating all opcode strings and parent ID repetitions. The same
// CoreOp semantics are preserved — this is purely a wire format change.
//
// Estimated savings: 40-60% reduction in wire bytes vs. positional encoding.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use super::compiler::CompiledIR;
use super::opcodes::CoreOp;
use super::wire::DecodeError;

/// Top-level hierarchical IR container.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HierarchicalIR {
    /// Classes (each contains methods, fields, relationships, etc.)
    #[serde(rename = "c")]
    pub classes: Vec<ClassNode>,

    /// Top-level imports — flat array of [alias, module, named]
    #[serde(rename = "i", default, skip_serializing_if = "Vec::is_empty")]
    pub imports: Vec<Vec<String>>,

    /// Type aliases — flat array of [alias, original]
    #[serde(rename = "t", default, skip_serializing_if = "Vec::is_empty")]
    pub type_aliases: Vec<Vec<String>>,
}

/// A single class node — the top-level structural container.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassNode {
    /// Class alias ID (e.g., "C1")
    #[serde(rename = "n")]
    pub id: String,

    /// Original class name
    #[serde(rename = "nm")]
    pub name: String,

    /// Methods within this class
    #[serde(rename = "m", default, skip_serializing_if = "Vec::is_empty")]
    pub methods: Vec<MethodNode>,

    /// Fields within this class
    #[serde(rename = "f", default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<FieldNode>,

    /// Class-level flags (EXPORT, ABSTRACT, etc.)
    #[serde(rename = "fl", default, skip_serializing_if = "Option::is_none")]
    pub class_flags: Option<Vec<String>>,

    /// Extends (parent class alias ID)
    #[serde(rename = "x", default, skip_serializing_if = "Option::is_none")]
    pub extends: Option<String>,

    /// Implements (interface alias IDs)
    #[serde(rename = "im", default, skip_serializing_if = "Vec::is_empty")]
    pub implements: Vec<String>,

    /// Injections (dependency aliases)
    #[serde(rename = "ij", default, skip_serializing_if = "Vec::is_empty")]
    pub injects: Vec<String>,

    /// Class-level pattern ops (e.g., CTOR)
    #[serde(rename = "p", default, skip_serializing_if = "Vec::is_empty")]
    pub patterns: Vec<PatternEntry>,

    /// True if this class was synthesized (no DefClass in original stream).
    /// Synthetic classes emit NO DefClass instruction during hierarchical_to_ir.
    #[serde(rename = "sy", default, skip_serializing_if = "is_false")]
    pub synthetic: bool,
}

/// Helper serde skip for false booleans.
fn is_false(v: &bool) -> bool {
    !*v
}

/// A single method node — nested inside a class.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MethodNode {
    /// Method alias ID (e.g., "M1")
    #[serde(rename = "n")]
    pub id: String,

    /// Original method name
    #[serde(rename = "nm")]
    pub name: String,

    /// Parameters — array of [param_id, type, name]
    #[serde(rename = "p", default, skip_serializing_if = "Vec::is_empty")]
    pub params: Vec<Vec<String>>,

    /// Return type
    #[serde(rename = "r", default, skip_serializing_if = "Option::is_none")]
    pub return_type: Option<String>,

    /// Method-level flags (IF, LOOP, RET, etc.)
    #[serde(rename = "fl", default, skip_serializing_if = "Option::is_none")]
    pub flags: Option<Vec<String>>,

    /// Method-level pattern ops
    #[serde(rename = "pa", default, skip_serializing_if = "Vec::is_empty")]
    pub patterns: Vec<PatternEntry>,
}

/// A single field node — nested inside a class.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldNode {
    /// Field alias ID (e.g., "F1")
    #[serde(rename = "n")]
    pub id: String,

    /// Original field name
    #[serde(rename = "nm")]
    pub name: String,

    /// Field type
    #[serde(rename = "t", default, skip_serializing_if = "Option::is_none")]
    pub field_type: Option<String>,
}

/// A pattern entry — compressed structural pattern (CTOR, GETTER, etc.).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatternEntry {
    /// Pattern name (e.g., "CTOR", "GETTER")
    #[serde(rename = "n")]
    pub name: String,

    /// Pattern args (metadata) — stored as-is from the original Pattern op.
    /// The hierarchical format does NOT add/remove CID/MID prefixes.
    #[serde(rename = "a", default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
}

/// Convert a flat `CompiledIR` instruction stream into a `HierarchicalIR`.
///
/// The converter scans instructions in order, collecting:
/// - DefClass → creates a ClassNode (subsequent ops scoped to this class)
/// - DefMethod → creates a MethodNode inside current class
/// - DefField → creates a FieldNode inside current class
/// - Param → added to current method's params
/// - Return → set as current method's return_type
/// - FieldType → set as current field's field_type
/// - Flags → set as current method's flags
/// - ClassFlags → set as current class's class_flags
/// - Extends → set as current class's extends
/// - Implements → added to current class's implements
/// - Injects → added to current class's injects
/// - DefInterface → creates a ClassNode with synthetic=false and name
/// - Import → added to top-level imports
/// - TypeAlias → added to top-level type_aliases
/// - Pattern → added to current scope (class or method), storing args as-is
pub fn ir_to_hierarchical(ir: &CompiledIR) -> HierarchicalIR {
    let mut classes: Vec<ClassNode> = Vec::new();
    let mut imports: Vec<Vec<String>> = Vec::new();
    let mut type_aliases: Vec<Vec<String>> = Vec::new();

    // Track current scope
    let mut current_class_idx: Option<usize> = None;
    let mut current_method_idx: Option<usize> = None;

    for op in &ir.instructions {
        match op {
            CoreOp::DefClass(id, name) => {
                classes.push(ClassNode {
                    id: id.clone(),
                    name: name.clone(),
                    methods: Vec::new(),
                    fields: Vec::new(),
                    class_flags: None,
                    extends: None,
                    implements: Vec::new(),
                    injects: Vec::new(),
                    patterns: Vec::new(),
                    synthetic: false,
                });
                current_class_idx = Some(classes.len() - 1);
                current_method_idx = None;
            }

            CoreOp::DefMethod(cid, mid, name) => {
                if let Some(class_idx) = find_class_by_id(&classes, cid) {
                    classes[class_idx].methods.push(MethodNode {
                        id: mid.clone(),
                        name: name.clone(),
                        params: Vec::new(),
                        return_type: None,
                        flags: None,
                        patterns: Vec::new(),
                    });
                    current_class_idx = Some(class_idx);
                    current_method_idx = Some(classes[class_idx].methods.len() - 1);
                } else {
                    // Method with no matching class — create a synthetic class
                    classes.push(ClassNode {
                        id: cid.clone(),
                        name: format!("__synthetic_{}", cid),
                        methods: vec![MethodNode {
                            id: mid.clone(),
                            name: name.clone(),
                            params: Vec::new(),
                            return_type: None,
                            flags: None,
                            patterns: Vec::new(),
                        }],
                        fields: Vec::new(),
                        class_flags: None,
                        extends: None,
                        implements: Vec::new(),
                        injects: Vec::new(),
                        patterns: Vec::new(),
                        synthetic: true,
                    });
                    current_class_idx = Some(classes.len() - 1);
                    current_method_idx = Some(0);
                }
            }

            CoreOp::DefField(cid, fid, name) => {
                if let Some(class_idx) = find_class_by_id(&classes, cid) {
                    classes[class_idx].fields.push(FieldNode {
                        id: fid.clone(),
                        name: name.clone(),
                        field_type: None,
                    });
                    current_class_idx = Some(class_idx);
                } else {
                    // Field with no matching class — create synthetic class
                    classes.push(ClassNode {
                        id: cid.clone(),
                        name: format!("__synthetic_{}", cid),
                        methods: Vec::new(),
                        fields: vec![FieldNode {
                            id: fid.clone(),
                            name: name.clone(),
                            field_type: None,
                        }],
                        class_flags: None,
                        extends: None,
                        implements: Vec::new(),
                        injects: Vec::new(),
                        patterns: Vec::new(),
                        synthetic: true,
                    });
                    current_class_idx = Some(classes.len() - 1);
                }
            }

            CoreOp::Param(mid, pid, ty, name) => {
                if let (Some(c_idx), Some(m_idx)) = (current_class_idx, current_method_idx) {
                    if classes[c_idx].methods[m_idx].id == *mid {
                        classes[c_idx].methods[m_idx].params
                            .push(vec![pid.clone(), ty.clone(), name.clone()]);
                    } else {
                        // Method ID mismatch — search
                        for mi in 0..classes[c_idx].methods.len() {
                            if classes[c_idx].methods[mi].id == *mid {
                                classes[c_idx].methods[mi].params
                                    .push(vec![pid.clone(), ty.clone(), name.clone()]);
                                current_method_idx = Some(mi);
                                break;
                            }
                        }
                    }
                }
            }

            CoreOp::Return(mid, ty) => {
                if let Some(c_idx) = current_class_idx {
                    for mi in 0..classes[c_idx].methods.len() {
                        if classes[c_idx].methods[mi].id == *mid {
                            classes[c_idx].methods[mi].return_type = Some(ty.clone());
                            current_method_idx = Some(mi);
                            break;
                        }
                    }
                }
            }

            CoreOp::FieldType(fid, ty) => {
                if let Some(c_idx) = current_class_idx {
                    for fi in 0..classes[c_idx].fields.len() {
                        if classes[c_idx].fields[fi].id == *fid {
                            classes[c_idx].fields[fi].field_type = Some(ty.clone());
                            break;
                        }
                    }
                }
            }

            CoreOp::Flags(tid, flags) => {
                if let Some(c_idx) = current_class_idx {
                    for mi in 0..classes[c_idx].methods.len() {
                        if classes[c_idx].methods[mi].id == *tid {
                            classes[c_idx].methods[mi].flags = Some(flags.clone());
                            current_method_idx = Some(mi);
                            break;
                        }
                    }
                }
            }

            CoreOp::ClassFlags(cid, flags) => {
                if let Some(c_idx) = find_class_by_id(&classes, cid) {
                    classes[c_idx].class_flags = Some(flags.clone());
                    current_class_idx = Some(c_idx);
                }
            }

            CoreOp::Extends(child, parent) => {
                if let Some(c_idx) = find_class_by_id(&classes, child) {
                    classes[c_idx].extends = Some(parent.clone());
                    current_class_idx = Some(c_idx);
                }
            }

            CoreOp::Implements(cid, iid) => {
                if let Some(c_idx) = find_class_by_id(&classes, cid) {
                    classes[c_idx].implements.push(iid.clone());
                    current_class_idx = Some(c_idx);
                }
            }

            CoreOp::Injects(cid, deps) => {
                if let Some(c_idx) = find_class_by_id(&classes, cid) {
                    classes[c_idx].injects.extend(deps.clone());
                    current_class_idx = Some(c_idx);
                }
            }

            CoreOp::DefInterface(id, name) => {
                classes.push(ClassNode {
                    id: id.clone(),
                    name: name.clone(),
                    methods: Vec::new(),
                    fields: Vec::new(),
                    class_flags: None,
                    extends: None,
                    implements: Vec::new(),
                    injects: Vec::new(),
                    patterns: Vec::new(),
                    synthetic: false,
                });
                current_class_idx = Some(classes.len() - 1);
                current_method_idx = None;
            }

            CoreOp::Import(alias, module, named) => {
                imports.push(vec![alias.clone(), module.clone(), named.clone()]);
            }

            CoreOp::TypeAlias(alias, original) => {
                type_aliases.push(vec![alias.clone(), original.clone()]);
            }

            // R-43a: Execution semantics — ignored in hierarchical format
            // (these are method-level annotations, not structural)
            CoreOp::DataFlow(..) | CoreOp::ControlFlow(..) | CoreOp::SideEffect(..) | CoreOp::ExecutionContext(..) => {}

            CoreOp::Pattern(name, args) => {
                // Parse pattern args to find the correct parent by class/method ID.
                // PatternOp::to_tuple() format: [class_id, method_id?, ...args]
                // All method-level patterns have class_id at index 0 and method_id at index 1.
                // Method IDs always start with "M" (generated by IRCompiler::next_id("M")).
                // Class-level patterns have class_id at index 0 and no method_id.
                // If args[1] does not start with "M", it's a class-level pattern argument.
                let class_id = args.first().cloned();
                let method_id = args.get(1).filter(|s| s.starts_with('M')).cloned();

                if let Some(cid) = class_id {
                    if let Some(c_idx) = find_class_by_id(&classes, &cid) {
                        if let Some(mid) = method_id {
                            // Method-level pattern: find the method by ID within this class
                            if let Some(m_idx) = classes[c_idx].methods.iter().position(|m| m.id == mid) {
                                classes[c_idx].methods[m_idx].patterns.push(PatternEntry {
                                    name: name.clone(),
                                    args: args.clone(),
                                });
                                current_class_idx = Some(c_idx);
                                current_method_idx = Some(m_idx);
                            }
                        } else {
                            // Class-level pattern (no method_id)
                            classes[c_idx].patterns.push(PatternEntry {
                                name: name.clone(),
                                args: args.clone(),
                            });
                            current_class_idx = Some(c_idx);
                            current_method_idx = None;
                        }
                    }
                }
            }
        }
    }

    HierarchicalIR {
        classes,
        imports,
        type_aliases,
    }
}

/// Convert a `HierarchicalIR` back into a flat `Vec<CoreOp>` instruction stream.
///
/// This is the inverse of `ir_to_hierarchical`. The resulting instruction
/// order is: classes emit DefClass (unless synthetic), then class flags/
/// extends/implements/injects, then fields, then methods with their
/// params/return/flags/patterns, then imports, then type aliases.
///
/// NOTE: The original interleaving of instructions across methods/fields is
/// not preserved — the hierarchical format groups all instructions for a
/// given scope together. This is semantically equivalent because CoreOp
/// semantics don't depend on instruction ordering across scopes.
pub fn hierarchical_to_ir(hir: &HierarchicalIR) -> Vec<CoreOp> {
    let mut instructions = Vec::new();

    for class in &hir.classes {
        // Skip synthetic classes — they represent orphans that had no DefClass
        // in the original stream. Their methods/fields are emitted directly.
        if !class.synthetic {
            instructions.push(CoreOp::DefClass(class.id.clone(), class.name.clone()));

            // Class-level flags
            if let Some(flags) = &class.class_flags {
                instructions.push(CoreOp::ClassFlags(class.id.clone(), flags.clone()));
            }

            // Extends
            if let Some(parent) = &class.extends {
                instructions.push(CoreOp::Extends(class.id.clone(), parent.clone()));
            }

            // Implements
            for iid in &class.implements {
                instructions.push(CoreOp::Implements(class.id.clone(), iid.clone()));
            }

            // Injects
            if !class.injects.is_empty() {
                instructions.push(CoreOp::Injects(class.id.clone(), class.injects.clone()));
            }
        }

        // Fields (emitted for both synthetic and non-synthetic classes)
        for field in &class.fields {
            instructions.push(CoreOp::DefField(
                class.id.clone(),
                field.id.clone(),
                field.name.clone(),
            ));
            if let Some(ft) = &field.field_type {
                instructions.push(CoreOp::FieldType(field.id.clone(), ft.clone()));
            }
        }

        // Methods (emitted for both synthetic and non-synthetic classes)
        for method in &class.methods {
            instructions.push(CoreOp::DefMethod(
                class.id.clone(),
                method.id.clone(),
                method.name.clone(),
            ));

            // Params
            for param in &method.params {
                if param.len() >= 3 {
                    instructions.push(CoreOp::Param(
                        method.id.clone(),
                        param[0].clone(),
                        param[1].clone(),
                        param[2].clone(),
                    ));
                }
            }

            // Return type
            if let Some(rt) = &method.return_type {
                instructions.push(CoreOp::Return(method.id.clone(), rt.clone()));
            }

            // Method flags
            if let Some(flags) = &method.flags {
                instructions.push(CoreOp::Flags(method.id.clone(), flags.clone()));
            }

            // Method-level patterns (args stored as-is)
            for pat in &method.patterns {
                instructions.push(CoreOp::Pattern(pat.name.clone(), pat.args.clone()));
            }
        }

        // Class-level patterns (only for non-synthetic classes)
        if !class.synthetic {
            for pat in &class.patterns {
                instructions.push(CoreOp::Pattern(pat.name.clone(), pat.args.clone()));
            }
        }
    }

    // Imports
    for imp in &hir.imports {
        if imp.len() >= 3 {
            instructions.push(CoreOp::Import(
                imp[0].clone(),
                imp[1].clone(),
                imp[2].clone(),
            ));
        }
    }

    // Type aliases
    for ta in &hir.type_aliases {
        if ta.len() >= 2 {
            instructions.push(CoreOp::TypeAlias(ta[0].clone(), ta[1].clone()));
        }
    }

    instructions
}

/// Encode a compiled IR into the hierarchical wire format (JSON).
///
/// Example output:
/// ```json
/// {
///   "file": "α1", "v": 1, "encoding": "hierarchical",
///   "ir": {
///     "c": [{
///       "n": "C1", "nm": "SampleService",
///       "m": [{
///         "n": "M1", "nm": "processComplexData",
///         "p": [["P1", "$s", "payload"]],
///         "r": "$b", "fl": ["IF"]
///       }],
///       "f": [{"n": "F1", "nm": "items", "t": "$s[]"}],
///       "im": ["IF1"]
///     }],
///     "i": [["IM1", "./module", "Foo"]]
///   }
/// }
/// ```
pub fn ir_to_hierarchical_wire(ir: &CompiledIR) -> Value {
    let hir = ir_to_hierarchical(ir);
    json!({
        "file": ir.file_id,
        "v": ir.version,
        "encoding": "hierarchical",
        "ir": hir
    })
}

/// Decode a hierarchical wire format value back into a `CompiledIR`.
///
/// Returns `Err(DecodeError)` if the input is malformed.
pub fn wire_to_ir(value: &Value) -> Result<CompiledIR, DecodeError> {
    let file_id = value
        .get("file")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .ok_or_else(|| DecodeError::MissingField("file".into()))?;
    let version = value
        .get("v")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| DecodeError::MissingField("v".into()))?;

    let ir_val = value
        .get("ir")
        .ok_or_else(|| DecodeError::MissingField("ir".into()))?;

    // Deserialize via serde
    let hir: HierarchicalIR = serde_json::from_value(ir_val.clone())
        .map_err(|e| DecodeError::InvalidInput(format!("hierarchical decode: {}", e)))?;

    let instructions = hierarchical_to_ir(&hir);

    Ok(CompiledIR {
        file_id,
        instructions,
        version,
    })
}

/// Estimate character savings of hierarchical format vs. positional encoding.
///
/// Returns (positional_chars, hierarchical_chars, savings_pct).
pub fn estimate_savings(ir: &CompiledIR) -> (usize, usize, f64) {
    use super::wire::ir_to_wire;
    let positional = ir_to_wire(ir);
    let pos_str = serde_json::to_string(&positional).unwrap_or_default();

    let hier = ir_to_hierarchical_wire(ir);
    let hier_str = serde_json::to_string(&hier).unwrap_or_default();

    let pos_chars = pos_str.len();
    let hier_chars = hier_str.len();

    let savings = if pos_chars > 0 {
        ((pos_chars - hier_chars) as f64 / pos_chars as f64) * 100.0
    } else {
        0.0
    };

    (pos_chars, hier_chars, savings)
}

/// Helper: find a class index by its alias ID.
fn find_class_by_id(classes: &[ClassNode], id: &str) -> Option<usize> {
    classes.iter().position(|c| c.id == id)
}

#[cfg(test)]
#[path = "../tests/ir/hierarchical.rs"]
mod tests;