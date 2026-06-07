// src/diff/snapshot.rs
//
// Data structures representing a structural snapshot of a source file.

/// Per-file structural snapshot. Two snapshots can be diffed to produce
/// an AST-level change-set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapturedStructure {
    pub imports: Vec<String>,
    pub classes: Vec<CapturedClass>,
    /// Fields that appeared outside of any class context.
    pub orphan_fields: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapturedClass {
    /// Display name (e.g. "FooService" or "FooService:BaseService,IFoo").
    pub name: String,
    pub fields: Vec<String>,
    pub methods: Vec<CapturedMethod>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapturedMethod {
    /// The compact signature text (e.g. "process(id:string):boolean").
    pub sig: String,
    /// Behavior markers attached to the method (⊕guard, ⊕loop, etc.).
    pub markers: Vec<String>,
}
