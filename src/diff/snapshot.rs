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
    /// Normalized method body text (whitespace-collapsed). `None` when the
    /// body could not be extracted (e.g. abstract/interface methods, or
    /// test fixtures that don't set it).
    ///
    /// This field exists so the diff comparator can detect **body-only**
    /// changes (logic fixes) that leave the signature and markers
    /// untouched. Previously `diff_snapshots` compared only `sig` and
    /// `markers`, so a method whose body changed was reported as
    /// `Unchanged` — a false negative for `diff_commits`.
    pub body: Option<String>,
}